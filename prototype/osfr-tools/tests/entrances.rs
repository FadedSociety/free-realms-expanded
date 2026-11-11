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
    character_store::CharacterState,
    encounter::{
        EncounterEngine, EncounterPhase, FightFact,
        director::entrances::{EntranceKind, Launch, Table},
    },
    startup_world::parse_add_npc,
};
use serde_json::{Value, json};
use std::{collections::BTreeSet, fs, path::PathBuf};

#[path = "common/field_battle_fixture.rs"]
mod field_battle_fixture;
use field_battle_fixture::{
    Fixture, active_profile, arena_spawned, assert_returned, available, battle, battles, chat_text,
    content_dir, details, fight_to_win, go, i32_at, interact, npc_select, send, unframe, won_flag,
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
        (50, 35, 44),
        "35 dungeon doors and 15 battle starters in 44 scenes"
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
    // The evidence proves 63 entrances: each is placed, or held with its reason. The other 55
    // are parked, each with its reason.
    let proven_held = (table.held.iter())
        .filter(|held| held.reason.starts_with("proven, held: "))
        .count();
    let parked = (table.held.iter())
        .filter(|held| held.reason.starts_with("parked: "))
        .count();
    assert_eq!((table.entrances.len() + proven_held, parked), (63, 55));
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
        let fight = row.fight.unwrap();
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
            "frostfang_caverns",
            "frostfang_growler",
            "frostfang_snarlers",
            "grexan_s_camp",
            "hot_springs_haven",
            "howling_hills",
            "ice_troll_scout",
            "moldy_shamblers",
            "mugwort_s_hollow",
            "petty_yetis",
            "robgoblin_treasure_trove",
            "shady_smugglers",
            "sheep_watch",
            "snowy_canyon",
            "tanglewood_fort",
            "tormented_spirits",
            "troll_summoner_madness",
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

    // A developer reload of the battle file places every entrance again, once.
    assert_eq!(
        engine.battle_spawners().len(),
        11,
        "the file's own markers stay apart"
    );
    assert_eq!(engine.entrance_spawners().len(), table.entrances.len());
    engine
        .reload_field_battles(&content_dir().join("field_battles.json"))
        .unwrap();
    assert_eq!(engine.battle_spawners().len(), 11);
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

/// The Bixie Hive door, end to end: drawn at its point of interest, the click opens the hive's
/// card, Go switches the job with the live notice, zones into `sg_bixie_hive`, the battle is
/// won through real attacks and the close returns the player to the door.
#[test]
fn the_bixie_hive_door_runs_the_whole_route() {
    if !available() {
        eprintln!("skipped: the private fixture is missing");
        return;
    }
    let table = table();
    let hive = row(&table, "bixie_hive");
    let scene = table.scene("sg_bixie_hive").unwrap();
    let fixture = Fixture::new();
    let (mut gateway, loaded) = fixture.session(hive.at, 2901);

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

    // Go: the job is switched, the live notice says so, and the zone is the hive.
    let entered = send(&mut gateway, &go(&card), 500);
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

    let ready = arena_spawned(&mut gateway, 600);
    let spawned = packets(&ready.replies, [35, 0, 2, 0]).len();
    let roster = base["enemies"].as_array().unwrap();
    let standing = roster
        .iter()
        .filter(|e| e["summon_below_percent"].is_null())
        .count();
    assert!(
        spawned >= standing,
        "{spawned} of {standing} enemies spawned"
    );

    let mut now = 1_000;
    let arena = json!({"zone": {"position": scene.arrive}});
    let replies = fight_to_win(&mut gateway, &mut now, &arena);
    assert_eq!(won_flag(&replies), Some(1), "the results say won");
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
    let fixture = Fixture::new();
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
    let ready = arena_spawned(&mut gateway, 600);
    assert!(
        !packets(&ready.replies, [35, 0, 2, 0]).is_empty(),
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
