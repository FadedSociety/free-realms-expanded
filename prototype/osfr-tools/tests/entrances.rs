//! The clicks that open dungeons and battles (`data/content/entrances.json`): a dungeon's door
//! at its point of interest, a wandering battle's starter, and the scene each one runs in.
//!
//! As captured (p12.pcap, 2014-03-31) a click went 26/8, 26/9 with one "Fight" entry (event
//! 4891, icon 1345, name 3301, type 2, the encounter), 26/10, 41/120 "Your job has been
//! switched because you are entering a combat area." (text 634), then the zone load. Client
//! readers: 41/4 `0x00a9ec30` and 41/120 `0x00a9cf90` (dispatcher `0x00aa36c0` cases 4 and
//! 0x78), 41/114 `0x00aa3430`, Go 41/108 constructor `0x008b6e70`.
//!
//! The first tests read the shipped table and walk every entrance in the engine. The last two
//! walk the real entry path through the gateway (they need the private fixture, like the other
//! field-battle suites).
use osfr_tools::{
    account_store::AccountRole,
    character_store::CharacterState,
    encounter::{
        EncounterEngine, EncounterPhase, FightFact,
        director::{
            self,
            entrances::{EntranceKind, Launch, Table},
        },
    },
    gateway::{Gateway, GatewayOutcome},
    quest_engine::{CharacterEligibility, QuestEngine},
    startup_world::parse_add_npc,
};
use serde_json::{Value, json};
use std::{collections::BTreeSet, fs, path::PathBuf};

#[path = "common/content_copy.rs"]
mod content_copy;
#[path = "common/field_battle_fixture.rs"]
mod field_battle_fixture;
#[path = "support/encounter_card.rs"]
mod encounter_card;
use field_battle_fixture::{
    Fixture, active_profile, arena_spawned, assert_returned, available, battle, battles, chat,
    chat_text, content_dir, details, go, i32_at, interact, melee, move_to, npc_select,
    quest_counts, send, unframe, won_flag,
};

const BRAWLER: i32 = 43;
const BIXIE_HIVE_BATTLE: u32 = 12370;

fn table() -> Table {
    Table::load(&content_dir())
        .unwrap()
        .expect("entrances.json ships")
}

/// The shipped engine with the shipped battles, as `Gameplay::load` builds it.
fn engine_in(directory: &std::path::Path) -> EncounterEngine {
    let mut engine = EncounterEngine::load(directory, None).unwrap_or_else(|e| panic!("{e}"));
    engine
        .load_field_battles(&directory.join("field_battles.json"))
        .unwrap_or_else(|e| panic!("{e}"));
    engine
}

fn engine() -> EncounterEngine {
    engine_in(&content_dir())
}

fn at(position: [f32; 3]) -> CharacterState {
    CharacterState {
        guid: 77,
        zone_id: 0,
        position,
        rotation: [0.0, 0.0, 1.0],
    }
}

/// 26/8: the player clicks an actor.
fn click(guid: u64) -> Vec<u8> {
    let mut b = vec![26, 0, 8, 0];
    b.extend(guid.to_le_bytes());
    b.push(0);
    b
}

fn row<'a>(table: &'a Table, id: &str) -> &'a osfr_tools::encounter::director::entrances::Entrance {
    (table.entrances.iter().find(|row| row.id == id)).unwrap_or_else(|| panic!("no entrance {id}"))
}

/// The data test: every proven entrance has its actor or its reason, and every door its scene.
#[test]
fn every_proven_entrance_has_an_actor_and_every_door_a_scene_row() {
    let (table, engine) = (table(), engine());
    let spawners = engine.entrance_spawners();
    let doors = (table.entrances.iter())
        .filter(|row| row.kind == EntranceKind::Door)
        .count();
    assert_eq!(
        (table.entrances.len(), doors, table.scenes.len()),
        (56, 35, 46),
        "35 dungeon doors and 21 battle starters in 46 scenes (rc-0136: a Bone Bog snake and five \
         Legacy spawner huts, two of them in scenes of their own; rc-0137 holds the Troll \
         Summoner starter, where battle 96's own marker now stands; L1192 places the Hooligan \
         Brawling Club hut for fight 14017)"
    );
    for row in &table.entrances {
        let actor = spawners.iter().find(|actor| actor.0 == row.guid);
        let (_, name, name_id, model_id, position, _) =
            actor.unwrap_or_else(|| panic!("{} has no actor in the world", row.id));
        assert_eq!(
            (name.as_str(), *name_id, *model_id, *position),
            (row.name.as_str(), row.name_id, row.model_id, row.at),
            "{}",
            row.id
        );
        assert!(engine.entrance(row.guid).is_some(), "{}", row.id);
        assert_eq!(
            engine.entrance_pose(row.guid),
            Some((row.heading, row.scale)),
            "{} faces the way its point of interest does",
            row.id
        );
        let scene = row.scene.as_deref().and_then(|name| table.scene(name));
        if row.kind == EntranceKind::Door {
            let scene = scene.unwrap_or_else(|| panic!("the door {} has no scene row", row.id));
            assert!(
                scene.zone_id != 0 && !scene.sky.is_empty() && scene.geometry_id != 0,
                "{}: its scene row is a whole zone row",
                row.id
            );
            assert_eq!(row.roam, 0.0, "{}: a door stands still", row.id);
        }
        assert!(
            (row.source.iter())
                .any(|s| s["evidence"].as_str().is_some_and(|e| e.starts_with("(R)"))),
            "{} marks what is reconstructed",
            row.id
        );
    }
    // The evidence proves 73 entrances: each is placed, or held with its reason (rc-0136 placed
    // Snakes in a Maze at a Bone Bog snake, (R), proved the nine Legacy spawner huts the fork's
    // Battles.json names as starters, five open and four held, and holds the second Ice Troll
    // Scout hut). The other 46 are parked, each with its reason.
    let proven_held = (table.held.iter())
        .filter(|held| held.reason.starts_with("proven, held: "))
        .count();
    let parked = (table.held.iter())
        .filter(|held| held.reason.starts_with("parked: "))
        .count();
    assert_eq!((table.entrances.len() + proven_held, parked), (73, 46));
    assert_eq!(table.held.len(), proven_held + parked);
    let placed: BTreeSet<&str> = table.entrances.iter().map(|row| row.id.as_str()).collect();
    for held in &table.held {
        assert!(held.reason.len() > 20, "{} says why it is held", held.id);
        assert!(
            !placed.contains(held.id.as_str()),
            "{} is held, not placed",
            held.id
        );
    }
    // No source puts the Bixie Queen at the hive door: the entrance there is the door.
    let hive = row(&table, "bixie_hive");
    assert_eq!(
        (hive.name.as_str(), hive.at, hive.scene.as_deref()),
        (
            "Bixie Hive",
            [-1445.68, -38.34, 49.89],
            Some("sg_bixie_hive")
        )
    );
    assert!(!table.entrances.iter().any(|row| row.name.contains("Queen")));
    assert_eq!(table.launch, Launch::Offer, "the proven waiting card ships");
}

/// Every open entrance: the click opens the start card, Go's own steps begin the fight, and
/// the zone is the entrance's scene. Every closed one refuses and can begin nothing.
#[test]
fn an_open_entrance_zones_into_its_scene_and_a_closed_one_never_starts() {
    let (table, engine) = (table(), engine());
    let (mut open, mut closed) = (Vec::new(), 0);
    for row in &table.entrances {
        let mut engine = engine.clone();
        let (encounter, is_open) = engine.entrance(row.guid).unwrap();
        let out = engine
            .handle(&click(row.guid), 77, row.at, BRAWLER, 1_000)
            .unwrap();
        if !is_open {
            closed += 1;
            // 41/4: the header, then the client's own text 139541.
            let refusal = [
                vec![41, 0, 4, 0],
                (encounter as i32).to_le_bytes().to_vec(),
                out.replies[0][8..12].to_vec(),
                table.closed_text_id.to_le_bytes().to_vec(),
            ]
            .concat();
            assert_eq!(out.replies, [refusal], "{} answers with 41/4 alone", row.id);
            assert_eq!(engine.take_pending_battle(row.at), None, "{}", row.id);
            assert!(
                engine.begin(encounter, at(row.at), BRAWLER).is_err(),
                "{}",
                row.id
            );
            assert!(!engine.active(), "{} never starts an empty fight", row.id);
            continue;
        }
        open.push(row.id.as_str());
        let subs: Vec<u8> = out.replies.iter().map(|packet| packet[2]).collect();
        assert_eq!(
            subs,
            [114, 107],
            "{}: the waiting card, then zone-is-ready",
            row.id
        );
        assert_eq!(i32_at(&out.replies[0], 4), encounter as i32, "{}", row.id);
        // Out of reach the same click opens nothing.
        let far = [row.at[0] + 40.0, row.at[1], row.at[2]];
        assert!(
            (engine
                .clone()
                .handle(&click(row.guid), 77, far, BRAWLER, 1_000))
            .unwrap()
            .replies
            .is_empty()
        );
        assert_eq!(
            engine.take_pending_battle(row.at),
            Some(encounter),
            "{}",
            row.id
        );
        engine.set_player_job(BRAWLER, 20);
        let zone = (engine.begin(encounter, at(row.at), BRAWLER).unwrap().zoning).unwrap();
        if row.enter_scene {
            let scene = table.scene(row.scene.as_deref().unwrap()).unwrap();
            assert_eq!(
                (
                    zone.name.as_str(),
                    zone.zone_id,
                    zone.sky.as_str(),
                    zone.position
                ),
                (
                    scene.scene.as_str(),
                    scene.zone_id,
                    scene.sky.as_str(),
                    scene.arrive
                ),
                "{} zones into its own scene",
                row.id
            );
        }
        let ready = engine.on_zone_ready(BRAWLER, 2_000).unwrap();
        // A door that runs its dungeon's own fight still counts as the battle it ran before.
        let fight = row.counts_as.or(row.fight).unwrap();
        assert!(
            ready.facts.contains(&FightFact::Entered { fight }),
            "{} counts as fight {fight}: {:?}",
            row.id,
            ready.facts
        );
        let active = engine.state().active.clone().unwrap();
        assert_eq!(active.phase, EncounterPhase::Fighting, "{}", row.id);
        assert!(
            !active.enemies.is_empty(),
            "{} has enemies to fight",
            row.id
        );
        if row.enter_scene {
            // The battle written for the clearing stands around the scene's stage.
            let stage = table.scene(row.scene.as_deref().unwrap()).unwrap().stage;
            for enemy in &active.enemies {
                let far = (enemy.position[0] - stage[0]).hypot(enemy.position[2] - stage[2]);
                assert!(far <= 40.0, "{}: an enemy {far} m from the stage", row.id);
                assert!((enemy.position[1] - stage[1]).abs() <= 3.0, "{}", row.id);
            }
        }
    }
    open.sort_unstable();
    assert_eq!(
        open,
        // Lead 2026-11-11: the combat quests' fight files (fights/14xxx.json) open 21 more.
        // Fix F5: the dungeon guides' fights built from the fork battles open five more.
        // rc-0136: five Legacy spawner huts open their combat-quest fights.
        [
            "arachnia_s_lair",
            "bandit_hideout",
            "bixie_hive",
            "briarheart_caverns",
            "cracked_claw_caverns",
            "cray_marauders",
            "cursed_graveyard",
            "danger_peaks",
            "darvon_s_descent",
            "deep_mines",
            "explosive_robgoblins",
            "floren_forest",
            "forest_troll_fort",
            "frostfang_caverns",
            "frostfang_growler",
            "frostfang_snarlers",
            "grexan_s_camp",
            "hooligan_brawling_club",
            "hooligan_bullies",
            "hot_springs_haven",
            "howling_hills",
            "ice_troll_scout",
            "moldy_shamblers",
            "mugwort_s_hollow",
            "petty_yetis",
            "robgoblin_adept_trouble",
            "robgoblin_geomancer",
            "robgoblin_treasure_trove",
            "robgoblin_troublemakers",
            "shady_smugglers",
            "sheep_watch",
            "snakes_in_a_maze",
            "snowy_canyon",
            "sweetwater_climb",
            "tanglewood_fort",
            "tavern_cellar",
            "tormented_spirits",
            "vulture_watch"
        ],
        "the entrances whose fight exists today; a new fight file opens its door by itself"
    );
    assert_eq!(closed, table.entrances.len() - open.len());
    // The battle markers of field_battles.json are untouched by their doors.
    let file = battles();
    let base = battle(&file, BIXIE_HIVE_BATTLE);
    assert_eq!(
        engine.battle_marker(BIXIE_HIVE_BATTLE).unwrap().1,
        field_battle_fixture::position(&base["spawner"]["position"])
    );
}

/// A door stands still while a starter may walk, and a closed entrance shows no combat marker.
#[test]
fn doors_stand_still_and_only_open_entrances_carry_the_marker() {
    let (table, mut engine) = (table(), engine());
    let marked: BTreeSet<u64> = engine.battle_markers().iter().map(|m| m.0).collect();
    for row in &table.entrances {
        let open = engine.entrance(row.guid).unwrap().1;
        assert_eq!(marked.contains(&row.guid), open, "{}", row.id);
    }
    // A player in sight of the hive door for a minute: the door never moves.
    let hive = row(&table, "bixie_hive");
    let stand = [hive.at[0] + 20.0, hive.at[1], hive.at[2]];
    let mut moved = Vec::new();
    for second in 1..=60_u64 {
        let out = engine.tick(stand, BRAWLER, second * 1_000).unwrap();
        moved.extend(
            out.replies
                .into_iter()
                .filter(|packet| packet.len() >= 12 && packet[4..12] == hive.guid.to_le_bytes()),
        );
    }
    assert!(
        moved.is_empty(),
        "the door was sent {} packets",
        moved.len()
    );
    assert_eq!(engine.take_pending_battle(stand), None);
    let out = engine.handle(&click(hive.guid), 77, hive.at, BRAWLER, 61_000);
    assert_eq!(
        out.unwrap().replies.len(),
        2,
        "and still opens where it stands"
    );

    // A developer reload of the battle file places every entrance again, once. A row an
    // entrance runs or counts as stands only as that entrance (rc-0137: the Troll Summoner
    // stood twice).
    let (battles, taken) = (field_battle_fixture::battles(), field_battle_fixture::entrance_rows());
    let own = (battles["battles"].as_array().unwrap().iter())
        .filter(|b| !taken.contains(&b["id"].as_u64().unwrap()))
        .count();
    assert_eq!(
        engine.battle_spawners().len(),
        own,
        "the file's own markers stay apart"
    );
    assert_eq!(engine.entrance_spawners().len(), table.entrances.len());
    engine
        .reload_field_battles(&content_dir().join("field_battles.json"))
        .unwrap();
    assert_eq!(engine.battle_spawners().len(), own);
    assert_eq!(engine.entrance_spawners().len(), table.entrances.len());
    assert_eq!(engine.entrance(hive.guid), Some((hive.encounter_id, true)));
    let growler = row(&table, "frostfang_growler");
    assert_eq!(
        engine.entrance(growler.guid),
        Some((12365, true)),
        "a fight file's arena row outlives the reload"
    );
}

fn scratch(name: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("entrances-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("fights")).unwrap();
    for file in [
        "encounters.json",
        "encounter-template.bin",
        "field_battles.json",
        "enemies.json",
        "fights/12365.json",
        // The Bixie Hive door's own fight (fix F5): its door opens only with it.
        "fights/13003.json",
    ] {
        fs::copy(content_dir().join(file), dir.join(file)).unwrap();
    }
    dir
}

fn shipped_json() -> Value {
    serde_json::from_slice(&fs::read(content_dir().join("entrances.json")).unwrap()).unwrap()
}

/// The straight launch (`launch: direct`): the click answers the captured one-entry menu, and
/// the client's own pick of it is the Go.
#[test]
fn the_straight_launch_offers_the_captured_fight_menu() {
    let dir = scratch("direct");
    let mut file = shipped_json();
    file["launch"] = json!("direct");
    fs::write(dir.join("entrances.json"), file.to_string()).unwrap();
    let (table, mut engine) = (table(), engine_in(&dir));
    let hive = row(&table, "bixie_hive");
    let (encounter, _) = engine.entrance(hive.guid).unwrap();
    let pick = npc_select(hive.guid, 4891);
    assert!(!engine.entrance_picked(&pick), "no menu is open yet");
    let out = engine
        .handle(&click(hive.guid), 77, hive.at, BRAWLER, 1_000)
        .unwrap();
    // 26/9: guid, auto-select, one entry of seven ints, the name, two bytes.
    let mut menu = vec![26, 0, 9, 0];
    menu.extend(hive.guid.to_le_bytes());
    menu.push(1);
    menu.extend(1_i32.to_le_bytes());
    for v in [4891, 1345, 3301, 2, encounter as i32, 0, 0] {
        menu.extend(v.to_le_bytes());
    }
    menu.extend((hive.name.len() as u32).to_le_bytes());
    menu.extend(hive.name.bytes());
    menu.extend([1, 0]);
    assert_eq!(out.replies, [menu]);
    assert!(
        engine.entrance_picked(&pick),
        "the pick of the open menu is a Go"
    );
    assert!(
        !engine.entrance_picked(&npc_select(hive.guid, 103)),
        "another choice is not"
    );
    assert!(!engine.entrance_picked(&npc_select(hive.guid + 1, 4891)));
    assert_eq!(engine.take_pending_battle(hive.at), Some(encounter));
    assert!(!engine.entrance_picked(&pick), "Go consumed the menu");
    // The shipped waiting form never treats a 26/10 as Go.
    let mut shipped = engine_in(&content_dir());
    shipped
        .handle(&click(hive.guid), 77, hive.at, BRAWLER, 1_000)
        .unwrap();
    assert!(!shipped.entrance_picked(&pick));
    let _ = fs::remove_dir_all(&dir);
}

/// The loader's refusals, a table with no file, and a fight file that lands later.
#[test]
fn the_table_is_checked_and_a_new_fight_file_opens_its_door() {
    let dir = scratch("checks");
    let engine = engine_in(&dir);
    assert!(engine.entrances().is_none(), "no file: no entrances");
    assert!(engine.entrance_spawners().is_empty());
    assert_eq!(
        engine.battle_spawners().len(),
        11,
        "only the file's own markers"
    );

    let refusal = |edit: &dyn Fn(&mut Value)| {
        let mut file = shipped_json();
        edit(&mut file);
        fs::write(dir.join("entrances.json"), file.to_string()).unwrap();
        let mut engine = EncounterEngine::load(&dir, None).unwrap();
        let error = engine.load_field_battles(&dir.join("field_battles.json"));
        error
            .expect_err("the loader accepted a broken table")
            .to_string()
    };
    let cases: [(&dyn Fn(&mut Value), &str); 6] = [
        (
            &|f| f["entrances"][0]["scene"] = json!("no_such_scene"),
            "its scene has no row",
        ),
        (
            &|f| f["entrances"][1]["guid"] = f["entrances"][0]["guid"].clone(),
            "not a valid, unique actor",
        ),
        (
            &|f| f["entrances"][0]["guid"] = json!(100001237000_u64),
            "its guid is taken",
        ),
        (
            &|f| f["entrances"][0]["encounter_id"] = json!(12370),
            "its encounter id is taken",
        ),
        (
            &|f| f["scenes"][0]["sky"] = json!("sky with spaces.xml"),
            "not a valid, unique zone row",
        ),
        (&|f| f["closed_text_id"] = json!(0), "must be positive"),
    ];
    for (edit, why) in cases {
        let error = refusal(edit);
        assert!(
            error.starts_with("entrances.json: ") && error.contains(why),
            "{error}"
        );
    }

    // A dungeon fight file under the id a closed door names: the door opens, the card and the
    // level are the fight's, and the scene row gives the arena its sky.
    let table = table();
    let lair = row(&table, "arachnia_s_lair");
    let (fight, scene) = (lair.fight.unwrap(), table.scene("bw_spider_lair").unwrap());
    let mut file: Value =
        serde_json::from_slice(&fs::read(dir.join("fights/12365.json")).unwrap()).unwrap();
    file["id"] = json!(fight);
    file["kind"] = json!("dungeon");
    file["place"] = json!({"zone": scene.zone_id, "arena": "bw_spider_lair",
                           "arrive": scene.arrive, "exit_door": scene.exit_door});
    file["entry"] = json!({"battle_row": false, "min_combat_level": 3});
    fs::write(dir.join(format!("fights/{fight}.json")), file.to_string()).unwrap();
    fs::write(dir.join("entrances.json"), shipped_json().to_string()).unwrap();
    let mut engine = engine_in(&dir);
    assert_eq!(engine.entrance(lair.guid), Some((fight, true)));
    let out = engine
        .handle(&click(lair.guid), 77, lair.at, BRAWLER, 1_000)
        .unwrap();
    assert_eq!(i32_at(&out.replies[0], 4), fight as i32);
    assert_eq!(engine.take_pending_battle(lair.at), Some(fight));
    let low = engine.begin(fight, at(lair.at), BRAWLER).unwrap_err();
    assert!(low.to_string().contains("combat level 3 required"), "{low}");
    engine.set_player_job(BRAWLER, 5);
    let zone = (engine.begin(fight, at(lair.at), BRAWLER).unwrap().zoning).unwrap();
    assert_eq!(
        (zone.name.as_str(), zone.sky.as_str(), zone.geometry_id),
        ("bw_spider_lair", scene.sky.as_str(), scene.geometry_id)
    );
    let _ = fs::remove_dir_all(&dir);
}

// ---------------------------------------------------------------------------
// The real entry path, through the gateway.
// ---------------------------------------------------------------------------

fn packets<'a>(replies: &'a [Vec<u8>], head: [u8; 4]) -> Vec<&'a [u8]> {
    (replies.iter().map(|packet| unframe(packet)))
        .filter(|packet| packet.starts_with(&head))
        .collect()
}

const ROYAL_HONEY: u32 = 3_957_507;

/// The private fixture with the repository's fight files and kill rewards beside its content:
/// the engine reads `fights/` and `field_kill_rewards.json` (its quest drops) from the session's
/// content directory (production: `data/content`), while the doors come from the shipped
/// `entrances.json`, so a door that runs a fight file needs both.
fn fixture_with_fights() -> Fixture {
    let mut fixture = Fixture::new();
    let content = fixture.root.join("content");
    content_copy::copy_tree(&fixture.content, &content);
    content_copy::copy_tree(&content_dir().join("fights"), &content.join("fights"));
    let rewards = "field_kill_rewards.json";
    fs::copy(content_dir().join(rewards), content.join(rewards)).unwrap();
    fixture.content = content;
    fixture
}

/// A world entry at `position` whose save holds "Royal Honey" (Yuri Petaldancer's "Recover some
/// Royal Honey from Fauzz") accepted, through the production bind a reconnect uses.
fn session_with_honey(fixture: &Fixture, position: [f32; 3], guid: u64) -> (Gateway, GatewayOutcome) {
    let seed = fixture.gateway(position, guid);
    let mut engine = QuestEngine::from_split_content(
        &content_dir(),
        guid,
        CharacterEligibility {
            player_level: 20,
            member: true,
            nonmember_quest_limit: 30,
            member_quest_limit: 30,
            ..Default::default()
        },
        None,
    )
    .unwrap();
    assert!(engine.offer(ROYAL_HONEY, 9_500_000_544).changed);
    assert!(engine.accept(ROYAL_HONEY).changed);
    let mut snapshot = seed.snapshot();
    snapshot.gameplay.as_mut().unwrap().npc_quests = Some(engine.state().clone());
    let mut gateway = Gateway::load_gameplay(
        &fixture.reference,
        seed.state().clone(),
        &fixture.content,
        snapshot.gameplay,
    )
    .unwrap();
    gateway.set_dev_tools(true);
    gateway.set_role(AccountRole::Admin);
    fixture.login(&mut gateway);
    send(&mut gateway, &13_u16.to_le_bytes(), 100);
    let loaded = send(&mut gateway, &10_u16.to_le_bytes(), 200);
    assert_eq!(quest_counts(&gateway, ROYAL_HONEY), [0, 0]);
    (gateway, loaded)
}

/// Walk in 8 m steps with the server's tick between them, as a client would.
fn walk_to(gateway: &mut Gateway, to: [f32; 3], now: &mut u64) -> Vec<Vec<u8>> {
    let (guid, from) = (gateway.state().guid, gateway.state().position);
    let far = (to[0] - from[0]).hypot(to[2] - from[2]);
    let steps = (far / 8.0).ceil().max(1.0) as usize;
    let mut replies = Vec::new();
    for step in 1..=steps {
        let t = step as f32 / steps as f32;
        *now += 400;
        let p = [0, 1, 2].map(|a| from[a] + (to[a] - from[a]) * t);
        replies.extend(send(gateway, &move_to(guid, p), *now).replies);
        replies.extend(gateway.tick_at(*now).unwrap().replies);
    }
    replies
}

/// The run's enemies that stand now (alive and called in): guid and spot.
fn standing(gateway: &Gateway) -> Vec<(u64, [f32; 3])> {
    let state = gateway.snapshot().gameplay.unwrap().encounter.unwrap();
    (state.active.unwrap().enemies.iter())
        .filter(|e| e.health > 0 && !e.dormant)
        .map(|e| (e.guid, e.position))
        .collect()
}

/// A 15/4 line said by `speaker` with client text `text`.
fn said(replies: &[Vec<u8>], speaker: u64, text: i32) -> bool {
    packets(replies, [15, 0, 4, 0])
        .iter()
        .any(|p| p.len() >= 16 && p[4..12] == speaker.to_le_bytes() && p[12..16] == text.to_le_bytes())
}

/// The Bixie Hive door, end to end: drawn at its point of interest, the click opens the hive's
/// card, Go switches the job with the live notice, zones into `sg_bixie_hive`, the workers go
/// free, the Queen's waves and then Drone Fauzz fall to real attacks, Royal Honey counts only
/// then, the exit gives the results and the close returns the player to the door.
#[test]
fn the_bixie_hive_door_runs_the_whole_route() {
    if !available() {
        eprintln!("skipped: the private fixture is missing");
        return;
    }
    let table = table();
    let hive = row(&table, "bixie_hive");
    let scene = table.scene("sg_bixie_hive").unwrap();
    let fixture = fixture_with_fights();
    let (mut gateway, loaded) = session_with_honey(&fixture, hive.at, 2901);

    // The door is in the world: the invisible cube over the hive's own door prop, facing the
    // way the point of interest does.
    let add = (packets(&loaded.replies, [35, 0, 2, 0]).into_iter())
        .find(|packet| packet[4..12] == hive.guid.to_le_bytes())
        .expect("the door is drawn at world entry");
    let spawn = parse_add_npc(add).unwrap();
    assert_eq!(
        (spawn.model_id, spawn.name_string_id, spawn.scale),
        (hive.model_id, hive.name_id, hive.scale)
    );
    assert_eq!(spawn.position[..3], hive.at);
    assert!((spawn.rotation[0] - hive.heading.sin()).abs() < 1e-6);
    assert!((spawn.rotation[2] - hive.heading.cos()).abs() < 1e-6);

    let job_before = active_profile(&gateway);
    assert_ne!(job_before, BRAWLER, "the default job is not a combat job");
    let screen = send(&mut gateway, &interact(hive.guid), 400);
    let card = details(&screen).expect("the click opens 41/114").to_vec();
    let file = battles();
    let base = battle(&file, BIXIE_HIVE_BATTLE);
    let mut shown = base.clone();
    shown["id"] = json!(hive.encounter_id);
    field_battle_fixture::assert_start_screen(&card, &shown);
    // rc-0137: the native client dropped the two-goal card (657 bytes, rows 21 bytes short), so
    // Go did nothing. Read as the client reads it, the card ends on its last byte and opens the
    // start screen with both goals: the main one, then the Bonus one with its four workers.
    let read = encounter_card::read_card(&card);
    assert!(!read.launch, "the start screen, not a refresh");
    let goals: Vec<_> = (read.goals.iter()).map(|g| (g.key, g.category, g.counters)).collect();
    assert_eq!(goals, [(16791, 0, [0, 0, 1, 0]), (430337, 4, [0, 0, 4, 0])]);

    // Go, as the client sent it (trace 20261112T020333Z: id, instance, then 1 and 0): the job is
    // switched, the live notice says so, and the zone is the hive.
    let entered = send(&mut gateway, &client_go(&card), 500);
    assert!(entered.began_zoning, "{}", chat_text(&entered));
    assert_eq!(gateway.state().zone_id, scene.zone_id);
    let notice = packets(&entered.replies, [41, 0, 120, 0]);
    assert_eq!(notice.len(), 1, "one 41/120");
    assert_eq!(i32_at(notice[0], 12), 634, "text 634");
    assert_eq!(notice[0][16..24], 2901_u64.to_le_bytes(), "for this player");
    assert_eq!(notice[0].len(), 28, "and an empty name");
    assert_ne!(
        active_profile(&gateway),
        job_before,
        "the entry switched the job"
    );
    // 31: the zone load, its first field the 13-letter scene name.
    let zone = packets(&entered.replies, [31, 0, 13, 0]);
    assert!(
        zone.iter().any(|packet| packet[6..19] == *b"sg_bixie_hive"),
        "the zone load names the hive"
    );

    // Fix F5: the door runs the hive's own script (fights/13003.json), not the clearing battle.
    // 18 guards in the tunnels, the Queen and four workers stand; Drone Fauzz does not.
    let fight = (director::load_dir(&content_dir()).unwrap().into_iter())
        .find(|f| Some(f.id) == hive.fight)
        .expect("the hive's fight ships");
    let ready = arena_spawned(&mut gateway, 600);
    let refresh = packets(&ready.replies, [41, 0, 114, 0]);
    assert!(encounter_card::read_card(refresh[0]).launch, "the zone-ready card refreshes the game");
    let key = |name: &str| fight.roster.iter().position(|s| s.key == name);
    let guid = |name: &str| fight.guid(key(name));
    let spot = |name: &str| fight.roster[key(name).unwrap()].spot;
    let spawned: Vec<u64> = (packets(&ready.replies, [35, 0, 2, 0]).iter())
        .map(|p| u64::from_le_bytes(p[4..12].try_into().unwrap()))
        .collect();
    assert!(spawned.contains(&guid("speaker")) && !spawned.contains(&guid("boss")));
    let mut now = 1_000;
    // The route, not the fight: since the owner's x2 health and x1.5 hits (rc-0137) a fresh
    // level-1 Brawler is knocked out in the hive's waves, so it walks the route in god mode.
    send(&mut gateway, &chat("realm god"), now);

    // The bonus: each worker goes free with a talk (26/8) and says its thanks.
    for n in 1..=4 {
        let worker = format!("captive_{n}");
        let at = spot(&worker);
        walk_to(&mut gateway, [at[0] + 1.0, at[1], at[2]], &mut now);
        now += 300;
        let out = send(&mut gateway, &interact(guid(&worker)), now);
        assert!(said(&out.replies, guid(&worker), 130474), "worker {n} thanks the player");
        assert!(panel_twins(&out.replies) > 0, "worker {n}: the bonus count on both lists");
    }
    // The Queen's call brings three waves; the quest that needs the hive waits.
    let mut heard = walk_to(&mut gateway, spot("speaker"), &mut now);
    let guards: Vec<u64> = (0..fight.roster.len())
        .filter(|&i| fight.roster[i].tags.iter().any(|t| t == "guards"))
        .map(|i| fight.guid(Some(i)))
        .collect();
    // Twice the swings since the owner doubled enemy health (rc-0137).
    for _ in 0..1_200 {
        let targets: Vec<(u64, [f32; 3])> = (standing(&gateway).into_iter())
            .filter(|(g, _)| !guards.contains(g))
            .collect();
        if targets.iter().any(|(g, _)| *g == guid("boss")) {
            break;
        }
        assert_eq!(quest_counts(&gateway, ROYAL_HONEY), [0, 0], "no honey in a wave");
        now += 700;
        let Some((target, at)) = targets.first().copied() else {
            heard.extend(gateway.tick_at(now).unwrap().replies);
            continue;
        };
        heard.extend(walk_to(&mut gateway, [at[0] + 1.5, at[1], at[2]], &mut now));
        now += 700;
        heard.extend(send(&mut gateway, &melee(target), now).replies);
    }
    assert!(said(&heard, guid("speaker"), 38625), "the Queen calls for help");
    assert!(
        packets(&heard, [32, 0, 9, 0])
            .iter()
            .any(|p| p[4..12] == guid("boss").to_le_bytes()),
        "Drone Fauzz arrives after the waves, with his boss bar"
    );
    // Drone Fauzz falls: Royal Honey counts now, and the exit opens.
    let mut replies = Vec::new();
    for _ in 0..400 {
        let Some((_, at)) = standing(&gateway).into_iter().find(|(g, _)| *g == guid("boss")) else {
            break;
        };
        replies.extend(walk_to(&mut gateway, [at[0] + 1.5, at[1], at[2]], &mut now));
        now += 700;
        replies.extend(send(&mut gateway, &melee(guid("boss")), now).replies);
    }
    assert_eq!(quest_counts(&gateway, ROYAL_HONEY), [1, 0], "Fauzz's honey, at the end");
    assert_eq!(won_flag(&replies), None, "no results before the exit");
    // Drone Fauzz's fall meets the main goal on the card's record and on the Goals panel.
    assert!(panel_twins(&replies) > 0, "the main goal met on both lists");
    // The exit door: the results card.
    replies.extend(walk_to(&mut gateway, fight.exit_spot(), &mut now));
    assert_eq!(won_flag(&replies), Some(1), "the results say won");
    assert!(
        replies.iter().any(|r| unframe(r).starts_with(&[39, 0, 47])),
        "the score card"
    );
    let closed = send(&mut gateway, &[26, 0, 42, 0], now + 1_000);
    assert!(closed.began_zoning, "the close leaves the hive");
    send(&mut gateway, &13_u16.to_le_bytes(), now + 1_100);
    send(&mut gateway, &10_u16.to_le_bytes(), now + 1_200);
    assert_returned(&gateway, hive.at);
}

/// A wandering battle's starter (the Frosty Yeti of "Petty Yetis") starts its battle in its own
/// scene; a combat job is not switched, so no notice is sent. A starter with no fight answers
/// with the client's own refusal and changes nothing.
#[test]
fn a_battle_starter_enters_its_scene_and_a_closed_one_refuses() {
    if !available() {
        eprintln!("skipped: the private fixture is missing");
        return;
    }
    let table = table();
    let yeti = row(&table, "petty_yetis");
    let scene = table.scene("sh_random_encounter_01").unwrap();
    let fixture = fixture_with_fights();
    let (mut gateway, loaded) = fixture.session(yeti.at, 2902);
    assert!(
        (packets(&loaded.replies, [35, 0, 2, 0]).iter())
            .any(|packet| packet[4..12] == yeti.guid.to_le_bytes()),
        "the starter is drawn"
    );
    assert!(
        send(
            &mut gateway,
            &field_battle_fixture::switch_job(BRAWLER),
            300
        )
        .state_changed
    );
    let screen = send(&mut gateway, &interact(yeti.guid), 400);
    let card = details(&screen).expect("the click opens 41/114").to_vec();
    assert_eq!(i32_at(&card, 4), yeti.encounter_id as i32);
    let entered = send(&mut gateway, &go(&card), 500);
    assert!(entered.began_zoning, "{}", chat_text(&entered));
    assert!(
        packets(&entered.replies, [41, 0, 120, 0]).is_empty(),
        "a combat job is not switched, so there is no notice"
    );
    assert_eq!(gateway.state().zone_id, scene.zone_id);
    // Fix F5: the fork's battle, whose first yetis come 1.5 s in.
    let mut ready = arena_spawned(&mut gateway, 600).replies;
    ready.extend(gateway.tick_at(2_600).unwrap().replies);
    assert!(
        !packets(&ready, [35, 0, 2, 0]).is_empty(),
        "the yetis stand in the arena"
    );

    // "Mushroom Mania" has no fight: its starter stands closed.
    let pest = row(&table, "mushroom_mania");
    assert_eq!(pest.fight, None);
    let (mut gateway, loaded) = fixture.session(pest.at, 2903);
    assert!(
        (packets(&loaded.replies, [35, 0, 2, 0]).iter())
            .any(|packet| packet[4..12] == pest.guid.to_le_bytes()),
        "the closed starter is drawn too"
    );
    let before = gateway.snapshot();
    let refused = send(&mut gateway, &interact(pest.guid), 400);
    assert!(!refused.began_zoning && !refused.state_changed);
    let inner: Vec<&[u8]> = refused
        .replies
        .iter()
        .map(|packet| unframe(packet))
        .collect();
    assert_eq!(inner.len(), 1, "one packet: {inner:?}");
    assert!(inner[0].starts_with(&[41, 0, 4, 0]) && inner[0].len() == 16);
    assert_eq!(i32_at(inner[0], 4), pest.encounter_id as i32);
    assert_eq!(
        i32_at(inner[0], 12),
        139541,
        "the client's own refusal text"
    );
    assert_eq!(
        gateway.snapshot(),
        before,
        "a closed entrance changes nothing"
    );
}

/// Go (41/108) as the client sent it in the owner's recheck (trace 20261112T020333Z,
/// `29006c00 cb320000 09000000 01000000 00000000`): the card's id and instance, then 1 and 0.
fn client_go(card: &[u8]) -> Vec<u8> {
    [&[41, 0, 108, 0], &card[4..12], &[1, 0, 0, 0, 0, 0, 0, 0][..]].concat()
}

/// The leave (41/109) as the client sent it: no encounter id, the run's instance (trace
/// `29006d00 00000000 02000000`).
fn client_leave(instance: i32) -> Vec<u8> {
    [&[41, 0, 109, 0, 0, 0, 0, 0][..], &instance.to_le_bytes()].concat()
}

/// The BaseUi (47) or objective (45) rows with sub `sub` in one reply batch.
fn rows(replies: &[Vec<u8>], family: u8, sub: u8) -> Vec<&[u8]> {
    (replies.iter().map(|r| unframe(r)))
        .filter(|p| p.len() >= 7 && p[..3] == [family, 0, sub])
        .collect()
}

/// In one reply batch, every goal the card's records move (45/1, 45/2, 45/3) moves on the
/// in-game Goals panel too (47/1 with the same total, 47/2 with the same count, 47/3), the
/// panel's row first. Returns how many 45 rows it checked.
fn panel_twins(replies: &[Vec<u8>]) -> usize {
    let inner: Vec<&[u8]> = replies.iter().map(|r| unframe(r)).collect();
    let mut checked = 0;
    for (i, p) in inner.iter().enumerate() {
        if p.len() < 7 || p[..2] != [45, 0] || !(1..=3).contains(&p[2]) {
            continue;
        }
        let key = i32_at(p, 3);
        let twin = (inner.iter())
            .position(|q| q.len() >= 7 && q[..3] == [47, 0, p[2]] && i32_at(q, 3) == key)
            .unwrap_or_else(|| panic!("45/{} {key} has no 47 row", p[2]));
        assert!(twin < i, "the panel row comes first");
        match p[2] {
            1 => assert_eq!(i32_at(inner[twin], 22), i32_at(p, 7), "47/1 {key}: the record's total"),
            2 => assert_eq!(i32_at(inner[twin], 7), i32_at(p, 7), "47/2 {key}: the record's count"),
            _ => {}
        }
        checked += 1;
    }
    checked
}

/// Fight the run down: attack whatever stands, let the next wave come, until `exit` is drawn.
fn fight_to_the_exit(gateway: &mut Gateway, exit: u64, now: &mut u64) -> Vec<Vec<u8>> {
    let mut replies = Vec::new();
    for _ in 0..900 {
        if packets(&replies, [35, 0, 2, 0]).iter().any(|p| p[4..12] == exit.to_le_bytes()) {
            return replies;
        }
        *now += 700;
        let Some((target, at)) = standing(gateway).first().copied() else {
            replies.extend(gateway.tick_at(*now).unwrap().replies);
            continue;
        };
        replies.extend(walk_to(gateway, [at[0] + 1.5, at[1], at[2]], now));
        *now += 700;
        replies.extend(send(gateway, &melee(target), *now).replies);
    }
    panic!("the exit door never opened");
}

/// A fight file shipped beside the content.
fn shipped_fight(id: u32) -> director::fight::Fight {
    (director::load_dir(&content_dir()).unwrap().into_iter())
        .find(|f| f.id == id)
        .unwrap()
}

/// rc-0137 recheck (trace 20261112T020333Z): the owner's Frostfang Snarlers loaded the grass
/// clearing (zone load 31/28, the 28-letter `sg_random_encounter_clearing`), every wolf wore the
/// den's name, no Goals panel showed, and after the last wolf the owner left by the menu (41/109)
/// with nothing paid. The 2014 video (5-IM2pBh2yY 01002) shows a snowy arena and "Goals (1)".
/// Now: Snow Hill's arena, Frostfang Wolves and the Frostfang Alpha, the goal on the panel with
/// its count, the Brawler's own wheel on the card, and the leave after the last wolf pays the win.
#[test]
fn the_snarlers_den_runs_in_snow_hill_and_leaving_after_the_last_wolf_pays_the_win() {
    if !available() {
        eprintln!("skipped: the private fixture is missing");
        return;
    }
    let table = table();
    let den = row(&table, "frostfang_snarlers");
    let fight = shipped_fight(den.fight.unwrap());
    let fixture = fixture_with_fights();
    let (mut gateway, _) = fixture.session(den.at, 2903);
    assert!(send(&mut gateway, &field_battle_fixture::switch_job(BRAWLER), 300).state_changed);
    // The arena and the pay, not the fight: at the owner's x2 health and x1.5 hits (rc-0137) a
    // fresh level-1 Brawler who walks past the hearts is knocked out by the ten wolves.
    send(&mut gateway, &chat("realm god"), 350);
    let card = details(&send(&mut gateway, &interact(den.guid), 400))
        .expect("the click opens 41/114")
        .to_vec();
    let read = encounter_card::read_card(&card);
    // "Knock out all Snarlers!": two, three and four wolves, and the Alpha.
    let goals: Vec<_> = (read.goals.iter()).map(|g| (g.key, g.category, g.counters)).collect();
    assert_eq!(goals, [(78093, 0, [0, 0, 10, 0])]);
    let job_wheel = |job: i64| -> Vec<i32> {
        (fight.rewards.wheel.iter())
            .filter(|p| p["job"].as_i64() == Some(job))
            .map(|p| p["item_id"].as_i64().unwrap() as i32)
            .collect()
    };
    let shown: Vec<i32> = read.preview.items.iter().map(|i| i.0).collect();
    assert_eq!(shown, job_wheel(i64::from(BRAWLER)), "the Brawler's wheel");
    assert_ne!(shown, job_wheel(2), "not the Ninja's");
    assert_eq!(read.preview.coins, fight.rewards.wheel_coins as i32, "and the coin slot");

    let entered = send(&mut gateway, &client_go(&card), 500);
    assert!(entered.began_zoning, "{}", chat_text(&entered));
    assert!(
        (packets(&entered.replies, [31, 0, 22, 0]).iter())
            .any(|p| p[6..28] == *b"sh_random_encounter_01"),
        "the zone load names Snow Hill's arena"
    );
    assert_eq!(gateway.state().zone_id, 1240);
    let ready = arena_spawned(&mut gateway, 600);
    let instance = i32_at(packets(&ready.replies, [41, 0, 114, 0])[0], 8);
    assert_eq!(panel_twins(&ready.replies), 2, "the goal's row and count on both lists");

    let mut now = 1_000;
    let wheel: BTreeSet<i32> = job_wheel(i64::from(BRAWLER)).into_iter().collect();
    let before = field_battle_fixture::economy(&gateway, &wheel);
    let fought = fight_to_the_exit(&mut gateway, fight.guid(None), &mut now);
    let ours: BTreeSet<u64> = (0..fight.roster.len()).map(|i| fight.guid(Some(i))).collect();
    let names: BTreeSet<i32> = (packets(&fought, [35, 0, 2, 0]).into_iter())
        .map(|p| parse_add_npc(p).unwrap())
        .filter(|npc| ours.contains(&npc.actor_id) || npc.actor_id == fight.guid(None))
        .map(|npc| npc.name_string_id)
        .collect();
    assert_eq!(names, BTreeSet::from([6468, 423045, 4826]), "Frostfang Wolves, the Alpha, the Exit");
    assert!(rows(&fought, 47, 2).len() >= 10, "every knockout counted on the panel");
    assert_eq!(won_flag(&fought), None, "the door is open, the results wait for it");

    // The owner's way out: the leave, with the door open. The win is paid and the player goes home.
    let left = send(&mut gateway, &client_leave(instance), now + 500);
    assert!(left.began_zoning, "the leave goes home");
    let after = field_battle_fixture::economy(&gateway, &wheel);
    let prizes: u32 = after.prizes.values().sum::<u32>() - before.prizes.values().sum::<u32>();
    let coins = after.coins - before.coins;
    let pay = fight.rewards.coins;
    assert!(
        (prizes == 1 && coins == pay) || (prizes == 0 && coins == pay + fight.rewards.wheel_coins),
        "one slot of the Brawler's wheel and the fight's coins: {prizes} prizes, {coins} coins"
    );
    // The wheel stops where the client stops it: the 39/45 read in `008e7930`'s order lands on
    // the card's prize that was paid, or on the coin slot after them with the wheel's coins.
    let landed = rows(&left.replies, 39, 45);
    assert_eq!(landed.len(), 1, "the win's one wheel landing");
    let landing = encounter_card::read_landing(landed[0]);
    let slot = encounter_card::landing_slot(&read.preview, &landing)
        .expect("the wheel stops on a slot the card showed");
    let gained: Vec<i32> = (after.prizes.iter())
        .filter(|(id, n)| **n > before.prizes.get(*id).copied().unwrap_or(0))
        .map(|(id, _)| *id)
        .collect();
    match read.preview.items.get(slot) {
        Some(item) => assert_eq!(gained, [item.0], "the prize the wheel stopped on was paid"),
        None => assert_eq!(
            (gained.len(), landing.coins),
            (0, fight.rewards.wheel_coins as i32),
            "the coin slot pays the wheel's coins"
        ),
    }
}

/// rc-0137 recheck: the owner clicked the old Petty Yetis marker (the wandering Yeti, guid
/// 100001231000, trace 20261112T020333Z) and got the grass clearing (31/28) and three yetis 3 m
/// apart that fell to one hit. Its row runs in Snow Hill's arena with its yetis far apart now.
#[test]
fn the_old_petty_yetis_marker_runs_in_snow_hill_with_its_yetis_apart() {
    if !available() {
        eprintln!("skipped: the private fixture is missing");
        return;
    }
    let file = battles();
    let row = battle(&file, 12310);
    let marker = field_battle_fixture::position(&row["spawner"]["position"]);
    let fixture = fixture_with_fights();
    let (mut gateway, _) = fixture.session(marker, 2904);
    assert!(send(&mut gateway, &field_battle_fixture::switch_job(BRAWLER), 300).state_changed);
    let guid = row["spawner"]["guid"].as_u64().unwrap();
    let card = details(&send(&mut gateway, &interact(guid), 400))
        .expect("the click opens 41/114")
        .to_vec();
    assert_eq!(encounter_card::read_card(&card).id, 12310);
    let entered = send(&mut gateway, &client_go(&card), 500);
    assert!(entered.began_zoning, "{}", chat_text(&entered));
    assert!(
        (packets(&entered.replies, [31, 0, 22, 0]).iter())
            .any(|p| p[6..28] == *b"sh_random_encounter_01"),
        "the zone load names Snow Hill's arena"
    );
    assert_eq!(gateway.state().zone_id, 1240);
    arena_spawned(&mut gateway, 600);
    let yetis: Vec<[f32; 3]> = standing(&gateway).into_iter().map(|(_, at)| at).collect();
    assert_eq!(yetis.len(), 3);
    // The fork's first-wave spots for Petty Yetis! (8 to 29 m apart), not 3 m: one hit no longer
    // reaches all three.
    let mut apart = Vec::new();
    for (i, a) in yetis.iter().enumerate() {
        for b in &yetis[i + 1..] {
            apart.push((a[0] - b[0]).hypot(a[2] - b[2]));
        }
    }
    apart.sort_by(f32::total_cmp);
    assert!(apart[0] > 7.5 && apart[2] > 25.0, "the yetis stand apart: {apart:?}");
}

/// rc-0137 recheck: in Hooligan Bullies the owner saw no Goals panel and no count; the trace
/// has only the record's total (45/1 `2d000105240100 09000000`, key 74757, 9). The panel row and
/// every knockout reach the Goals panel too now.
#[test]
fn the_hooligan_goal_shows_on_the_side_panel_with_its_count() {
    if !available() {
        eprintln!("skipped: the private fixture is missing");
        return;
    }
    let table = table();
    let hut = row(&table, "hooligan_bullies");
    let fixture = fixture_with_fights();
    let (mut gateway, _) = fixture.session(hut.at, 2905);
    assert!(send(&mut gateway, &field_battle_fixture::switch_job(BRAWLER), 300).state_changed);
    let card = details(&send(&mut gateway, &interact(hut.guid), 400))
        .expect("the click opens 41/114")
        .to_vec();
    assert_eq!(encounter_card::read_card(&card).goals[0].counters, [0, 0, 9, 0]);
    assert!(send(&mut gateway, &client_go(&card), 500).began_zoning);
    let ready = arena_spawned(&mut gateway, 600);
    let panel = rows(&ready.replies, 47, 1);
    assert_eq!(panel.len(), 1, "one goal row on the panel");
    assert_eq!(
        (i32_at(panel[0], 3), i32_at(panel[0], 11), i32_at(panel[0], 22)),
        (74757, 74757, 9),
        "its key, its name and how many to defeat"
    );
    assert_eq!(panel_twins(&ready.replies), 2, "the row and its count, on both lists");
    // The first knockout counts on both lists.
    let mut now = 1_000;
    for _ in 0..200 {
        now += 700;
        let Some((target, at)) = standing(&gateway).first().copied() else {
            gateway.tick_at(now).unwrap();
            continue;
        };
        walk_to(&mut gateway, [at[0] + 1.5, at[1], at[2]], &mut now);
        now += 700;
        let hit = send(&mut gateway, &melee(target), now);
        if !rows(&hit.replies, 45, 2).is_empty() {
            assert_eq!(panel_twins(&hit.replies), rows(&hit.replies, 45, 2).len());
            assert_eq!(i32_at(rows(&hit.replies, 47, 2)[0], 7), 1, "one down");
            return;
        }
    }
    panic!("no knockout counted");
}

/// C2 (known bugs, 2026-11-12): 37 of the Games window's 38 Battles rows started nothing; the
/// window joined a row only to a `field_battles.json` battle, and the dungeons are fight files
/// behind their doors. Each row now runs the fight its dungeon's door runs: Play (167/1/2, the
/// row's own id) from the open world zones into the door's scene through the same gates (a
/// combat job, the level floor, the prize wheel). A row whose dungeon has no fight to run (no
/// door, or a door that stands closed) still refuses and changes nothing.
#[test]
fn every_games_window_battle_row_starts_its_doors_fight() {
    if !available() {
        eprintln!("skipped: the private fixture is missing");
        return;
    }
    let table = table();
    let engine = engine();
    let catalogue: Value =
        serde_json::from_slice(&fs::read(content_dir().join("activity_catalogue.json")).unwrap())
            .unwrap();
    let rows: Vec<&Value> = (catalogue["rows"].as_array().unwrap().iter())
        .filter(|row| row["category"] == 10)
        .collect();
    let fixture = fixture_with_fights();
    let lakeshore = [-733.952, 9.09, 945.973];
    let (mut gateway, _) = fixture.session(lakeshore, 2906);
    assert!(send(&mut gateway, &field_battle_fixture::switch_job(BRAWLER), 300).state_changed);
    send(&mut gateway, &osfr_tools::chat::local_request("realm level 20"), 400);
    let (mut now, mut started, mut refused) = (1_000, 0, Vec::new());
    for row in rows {
        let (id, name_id) = (row["id"].as_u64().unwrap() as u32, row["name_id"].as_i64().unwrap());
        let door = (table.entrances.iter()).find(|e| {
            e.kind == EntranceKind::Door
                && i64::from(e.name_id) == name_id
                && engine.entrance(e.guid).is_some_and(|(_, open)| open)
        });
        now += 1_000;
        let before = gateway.snapshot();
        let out = send(&mut gateway, &field_battle_fixture::portal_play(id), now);
        let Some(door) = door else {
            assert!(!out.began_zoning, "row {id} ({name_id}): no fight to run");
            assert_eq!(gateway.snapshot(), before, "row {id}: a refused Play changes nothing");
            refused.push(id);
            continue;
        };
        assert!(out.began_zoning, "row {id} ({}): {}", door.id, chat_text(&out));
        let (encounter, _) = engine.entrance(door.guid).unwrap();
        let mut world = engine.clone();
        world.set_player_job(BRAWLER, 20);
        let zone = (world.begin(encounter, at(door.at), BRAWLER).unwrap().zoning).unwrap();
        assert_eq!(gateway.state().zone_id, zone.zone_id, "row {id}: the door's own scene");
        let ready = arena_spawned(&mut gateway, now + 100);
        let card = packets(&ready.replies, [41, 0, 114, 0]);
        assert_eq!(i32_at(card[0], 4), encounter as i32, "row {id}: the door's own fight");
        // Leave, back to the open world, for the next row.
        now += 1_000;
        assert!(send(&mut gateway, &client_leave(i32_at(card[0], 8)), now).began_zoning);
        send(&mut gateway, &13_u16.to_le_bytes(), now + 100);
        send(&mut gateway, &10_u16.to_le_bytes(), now + 200);
        started += 1;
    }
    eprintln!("Games window Battles rows: {started} start their fight; no fight to run: {refused:?}");
    assert!(started > 1, "the window's dungeons start ({started})");
}

/// PB2: a party enters an entrance's run together (T440/T441). The leader's Go opens the hive;
/// a member who is not in a combat job is still invited and joins the leader's run in their best
/// combat job, as their own Go would switch them. A battle marker keeps its old gate.
#[test]
fn pb2_a_member_in_any_job_joins_the_leaders_entrance_run() {
    if !available() {
        eprintln!("skipped: the private fixture is missing");
        return;
    }
    let table = table();
    let hive = row(&table, "bixie_hive");
    let scene = table.scene("sg_bixie_hive").unwrap();
    // Fix F5: the hive's door runs its own fight file, which the fixture needs beside its content.
    let fixture = fixture_with_fights();
    let (mut leader, _) = fixture.session(hive.at, 2901);
    let (mut member, _) = fixture.session(hive.at, 2902);
    let card = details(&send(&mut leader, &interact(hive.guid), 400))
        .expect("the click opens 41/114")
        .to_vec();
    assert!(send(&mut leader, &go(&card), 500).began_zoning);
    let (battle, run, _) = leader.joinable_run().expect("an entrance run is joinable");

    let job_before = active_profile(&member);
    assert_ne!(job_before, BRAWLER, "the member is not in a combat job");
    assert!(member.can_join_group_battle(battle), "and is still invited");
    let joined = member.stage_encounter_join(battle, run, 600).unwrap();
    assert!(joined.began_zoning, "Accept zones the member");
    assert_eq!(member.state().zone_id, scene.zone_id, "into the hive");
    assert_ne!(active_profile(&member), job_before, "in a combat job");
    assert_eq!(member.joinable_run().map(|(b, ..)| b), Some(battle), "the leader's battle");
}

/// The shipped camp members of `camp` with their drawn positions.
fn camp_members(camp: &str) -> Vec<(u64, [f32; 3])> {
    let read = |name: &str| -> Value {
        serde_json::from_slice(&fs::read(content_dir().join(name)).unwrap()).unwrap()
    };
    let (camps, spawns) = (read("field_camps.json"), read("npc_spawns_extra.json"));
    let row = camps["camps"].as_array().unwrap().iter().find(|c| c["id"] == camp).unwrap();
    row["members"]
        .as_array()
        .unwrap()
        .iter()
        .map(|guid| {
            let spawn = spawns["records"].as_array().unwrap().iter().find(|s| s["spawn_guid"] == *guid).unwrap();
            let p = spawn["position"].as_array().unwrap();
            (guid.as_u64().unwrap(), [0, 1, 2].map(|i| p[i].as_f64().unwrap() as f32))
        })
        .collect()
}

/// Owner 2026-11-12: "the black spore cray claw cavern ... has the enemies attack me instead of
/// letting me click the dungeon to start the dungeon instance." The marauders by the Cracked Claw
/// Caverns door no longer chase (the no-chase rule is `behaviour_examples`), so the door is free to
/// click; each is an `instance_enemy` whose click opens its own battle, Cray Marauders (the wiki's
/// FR_Mob Cray Marauder: "Battle Starter", Battles: "Cray Marauders"), as that starter's click does.
#[test]
fn an_instance_enemy_opens_its_entrance_card() {
    let table = table();
    let door = table.entrances.iter().find(|r| r.id == "cray_marauders").unwrap();
    let mut base = engine();
    base.load_behaviours(&content_dir().join("behaviours.json")).unwrap();
    base.load_field_enemies(&content_dir().join("field_enemies.json")).unwrap();
    base.load_field_camps(&content_dir()).unwrap();
    let at_door = base.clone().handle(&click(door.guid), 77, door.at, BRAWLER, 1_000).unwrap();
    let subs = |out: &osfr_tools::encounter::EncounterOutcome| -> Vec<(u8, u8)> {
        out.replies.iter().map(|p| (p[0], p[2])).collect()
    };
    assert!(!at_door.replies.is_empty(), "the starter answers its own click");
    let marauders = camp_members("anon-cray-arctic-swamp-w58b-cray-marauder");
    assert_eq!(marauders.len(), 8);
    // The camp's bodies are spread over the map: only those homed in the starter's area link to it.
    let at_this_door = |at: [f32; 3]| (at[0] - door.at[0]).hypot(at[2] - door.at[2]) < 400.0;
    assert_eq!(marauders.iter().filter(|m| at_this_door(m.1)).count(), 4, "the four by the Cracked Claw door");
    for &(guid, at) in &marauders {
        let out = base.clone().handle(&click(guid), 77, at, BRAWLER, 1_000).unwrap();
        if !at_this_door(at) {
            assert_ne!(subs(&out), subs(&at_door), "Cray Marauder {guid}, homed elsewhere, opens no Cray Marauders card");
            continue;
        }
        assert_eq!(subs(&out), subs(&at_door), "Cray Marauder {guid} opens the Cray Marauders card");
        // Go (41/108) from that card starts the battle where the player stands, beside the
        // marauder, and not after walking away.
        let mut go = base.clone();
        go.handle(&click(guid), 77, at, BRAWLER, 1_000).unwrap();
        assert!(go.take_pending_battle(at).is_some(), "Go beside Cray Marauder {guid} starts it");
        go.handle(&click(guid), 77, at, BRAWLER, 1_000).unwrap();
        let walked = [at[0] + 40.0, at[1], at[2]];
        assert_eq!(go.take_pending_battle(walked), None, "Go after walking away starts nothing");
        let far = [at[0] + 40.0, at[1], at[2]];
        let missed = base.clone().handle(&click(guid), 77, far, BRAWLER, 1_000).unwrap();
        assert_ne!(subs(&missed), subs(&at_door), "out of reach, {guid} opens nothing");
    }
}

/// Owner 2026-11-12: "doors for dungeons can be hotswapped to open a specific dungeon or
/// instance". Any drawn object marked `interactable` with an `instance` opens that starter's own
/// card when clicked, and nothing out of reach.
#[test]
fn an_interactable_object_opens_its_linked_dungeon_card() {
    let door = table().entrances.into_iter().find(|r| r.id == "cracked_claw_caverns").unwrap();
    let dir = std::env::temp_dir().join(format!("osfr-object-link-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    content_copy::copy_tree(&content_dir(), &dir);
    let shipped = engine();
    let path = dir.join("npc_spawns_extra.json");
    let mut extras: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    let row = extras["records"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|r| {
            let guid = r["spawn_guid"].as_u64().unwrap();
            !shipped.is_battle_spawner(guid) && !shipped.is_field_enemy(guid)
        })
        .unwrap();
    row["interactable"] = json!(true);
    row["instance"] = json!(door.guid);
    let (guid, p) = (row["spawn_guid"].as_u64().unwrap(), row["position"].clone());
    let at: [f32; 3] = serde_json::from_value(p).unwrap();
    // The copy may hard-link the shipped file: replace it rather than write through it.
    fs::remove_file(&path).unwrap();
    fs::write(&path, serde_json::to_vec(&extras).unwrap()).unwrap();
    let mut linked = engine_in(&dir);
    linked.load_object_links(&dir).unwrap();
    assert!(linked.is_field_actor(guid), "the encounter owns the linked object's click");
    let subs = |out: &osfr_tools::encounter::EncounterOutcome| -> Vec<(u8, u8)> {
        out.replies.iter().map(|p| (p[0], p[2])).collect()
    };
    let at_door = linked.clone().handle(&click(door.guid), 77, door.at, BRAWLER, 1_000).unwrap();
    let out = linked.clone().handle(&click(guid), 77, at, BRAWLER, 1_000).unwrap();
    assert_eq!(subs(&out), subs(&at_door), "object {guid} opens the door's card");
    let far = [at[0] + 40.0, at[1], at[2]];
    let missed = linked.clone().handle(&click(guid), 77, far, BRAWLER, 1_000).unwrap();
    assert_ne!(subs(&missed), subs(&at_door), "out of reach it opens nothing");
    let _ = fs::remove_dir_all(&dir);
}

/// L1191: every shipped instance link is live. A field enemy or camp with `instance_enemy` names a
/// placed door, starter or field battle; the row (or at least one camp member) is homed within the
/// 400 m the click honours; each such actor's click opens exactly that starter's own card; and no
/// row carries an `instance` its switch leaves unread. Object rows with `interactable` name one too.
#[test]
fn every_shipped_instance_link_opens_its_starter() {
    let read = |name: &str| -> Value {
        serde_json::from_slice(&fs::read(content_dir().join(name)).unwrap()).unwrap()
    };
    let mut starters: Vec<(u64, [f32; 3])> = table().entrances.iter().map(|e| (e.guid, e.at)).collect();
    for b in read("field_battles.json")["battles"].as_array().unwrap() {
        let p = b["spawner"]["position"].as_array().unwrap();
        starters.push((b["spawner"]["guid"].as_u64().unwrap(), [0, 1, 2].map(|i| p[i].as_f64().unwrap() as f32)));
    }
    let mut base = engine();
    base.load_behaviours(&content_dir().join("behaviours.json")).unwrap();
    base.load_field_enemies(&content_dir().join("field_enemies.json")).unwrap();
    base.load_field_camps(&content_dir()).unwrap();
    let subs = |out: &osfr_tools::encounter::EncounterOutcome| -> Vec<(u8, u8)> {
        out.replies.iter().map(|p| (p[0], p[2])).collect()
    };
    let point = |v: &Value| -> [f32; 3] {
        let p = v.as_array().unwrap();
        [0, 1, 2].map(|i| p[i].as_f64().unwrap() as f32)
    };
    // (who, instance, actors homed with their positions, drawn without a quest)
    let mut links: Vec<(String, u64, Vec<(u64, [f32; 3])>, bool)> = Vec::new();
    for e in read("field_enemies.json")["enemies"].as_array().unwrap() {
        let who = format!("field enemy {}", e["guid"]);
        if e["instance_enemy"] != true {
            assert!(e["instance"].is_null(), "{who}: an instance its switch leaves unread");
            continue;
        }
        let drawn = e["quest_ids"].as_array().is_none_or(|q| q.is_empty());
        links.push((who, e["instance"].as_u64().unwrap(), vec![(e["guid"].as_u64().unwrap(), point(&e["position"]))], drawn));
    }
    for c in read("field_camps.json")["camps"].as_array().unwrap() {
        let who = format!("camp {}", c["id"]);
        if c["instance_enemy"] != true {
            assert!(c["instance"].is_null(), "{who}: an instance its switch leaves unread");
            continue;
        }
        links.push((who, c["instance"].as_u64().unwrap(), camp_members(c["id"].as_str().unwrap()), true));
    }
    assert!(links.len() >= 9, "the shipped links: {}", links.len());
    for (who, instance, actors, drawn) in &links {
        assert!(base.is_battle_spawner(*instance), "{who}: instance {instance} is no placed starter");
        let marker = starters.iter().find(|s| s.0 == *instance).unwrap().1;
        let own = base.clone().handle(&click(*instance), 77, marker, BRAWLER, 1_000).unwrap();
        assert!(!own.replies.is_empty(), "{who}: starter {instance} answers its own click");
        let near: Vec<_> = actors.iter().filter(|a| (a.1[0] - marker[0]).hypot(a.1[2] - marker[2]) < 390.0).collect();
        assert!(!near.is_empty(), "{who}: nothing homed within 400 m of {instance}, so the link is dead");
        // A quest-gated row is drawn only while its quest runs; its link and area are checked above.
        for &&(guid, at) in near.iter().filter(|_| *drawn) {
            let out = base.clone().handle(&click(guid), 77, at, BRAWLER, 1_000).unwrap();
            assert_eq!(subs(&out), subs(&own), "{who}: {guid} opens {instance}'s card");
        }
    }
    let mut objects = read("npc_spawns_extra.json")["records"].as_array().unwrap().clone();
    for path in fs::read_dir(content_dir().join("npcs")).unwrap() {
        let file: Value = serde_json::from_slice(&fs::read(path.unwrap().path()).unwrap()).unwrap();
        objects.extend(file["records"].as_array().unwrap().iter().cloned());
    }
    for r in objects.iter().filter(|r| r["interactable"] == true) {
        let starter = r["instance"].as_u64().unwrap_or(0);
        assert!(base.is_battle_spawner(starter), "object {}: instance {starter} is no placed starter", r["spawn_guid"]);
    }
}
