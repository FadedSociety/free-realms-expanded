//! Every dungeon guide runs its own script (fix F5, release candidate 0.1.36).
//!
//! The owner's report: the Bixie Hive ended after its first spawn, showed none of the original
//! goals and credited the quests that need it at once. The hive's door ran a one-wave battle
//! written for a clearing (`field_battles.json` row 12370). `scripts/dungeon_fights.py` now maps
//! every guide (`data/content/dungeons/*.json` and its notes page) onto the fight its entrance
//! runs, and each record lists the fight goals it maps to, or says why no script runs.
//!
//! Client facts the goal rows rest on: the card's goal list is read by `008fd770` (key, name,
//! description, flag, reward bundle, four counters, members flag, category) and turned into the
//! panel's records by `00c42870`; 45/1, 45/2 and 45/3 find a record by its key (`009ad510`,
//! `009ad5a0`, `009b6700`); category 4 prefixes "Bonus" (`GetObjectiveCategoryPrefix`
//! `00a872f0`); a total of 1 shows no counter (`ObjectiveStatus_FormatText` `00c42440`).
//! The boss bar is 32/9; the results card 39/47. The gateway walk through the hive door, with
//! Royal Honey counted only when Drone Fauzz falls, is `entrances::the_bixie_hive_door_runs_the_whole_route`.
#[path = "support/fight_kit.rs"]
mod fight_kit;
#[path = "common/field_battle_fixture.rs"]
mod field_battle_fixture;
#[path = "support/encounter_card.rs"]
mod encounter_card;

use field_battle_fixture::{content_dir, unframe};
use fight_kit::{score_rows, spawned};
use osfr_tools::{
    character_store::CharacterState,
    encounter::{
        EncounterEngine, EncounterPhase, FightFact,
        director::{self, entrances::Table, fight::Fight},
    },
};
use serde_json::Value;
use std::{collections::BTreeMap, fs};

const BRAWLER: i32 = 43;
const HIVE_FIGHT: u32 = 13003;
const HIVE_BATTLE: u32 = 12370;
const MAIN_TEXT: i32 = 16791;
const BONUS_TEXT: i32 = 430337;
const WORKER_THANKS: i32 = 130474;
const QUEEN_LINES: [i32; 6] = [38625, 421146, 14787, 38626, 38627, 38629];
/// Drone Fauzz in the hive: his own kind, the one Royal Honey's drop follows.
const FAUZZ: u32 = 1_300_319;

fn read(path: &std::path::Path) -> Value {
    serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
}

fn fights() -> BTreeMap<u32, Fight> {
    let loaded = director::load_dir(&content_dir()).unwrap_or_else(|e| panic!("{e}"));
    loaded.into_iter().map(|f| (f.id, f)).collect()
}

/// The roster slot of a named actor, as (guid, spot).
fn actor(fight: &Fight, key: &str) -> (u64, [f32; 3]) {
    let i = (fight.roster.iter().position(|s| s.key == key)).unwrap_or_else(|| panic!("{key}"));
    (fight.guid(Some(i)), fight.roster[i].spot)
}

/// Guids of the guards along the tunnels.
fn guards(fight: &Fight) -> Vec<u64> {
    (0..fight.roster.len())
        .filter(|&i| fight.roster[i].tags.iter().any(|t| t == "guards"))
        .map(|i| fight.guid(Some(i)))
        .collect()
}

/// The card's goal rows (41/114) as (key, category), read the way the client reads the card.
fn goal_rows(card: &[u8]) -> Vec<(i32, i32)> {
    let card = encounter_card::read_card(card);
    assert!(card.goals.iter().all(|g| g.key == g.name && g.name == g.description), "{card:?}");
    card.goals.iter().map(|g| (g.key, g.category)).collect()
}

/// A 15/4 line said by `speaker` with client text `text`.
fn said(replies: &[Vec<u8>], speaker: u64, text: i32) -> bool {
    replies.iter().map(|r| unframe(r)).any(|r| {
        r.len() >= 16
            && r.starts_with(&[15, 0, 4, 0])
            && r[4..12] == speaker.to_le_bytes()
            && r[12..16] == text.to_le_bytes()
    })
}

/// The 45/<sub> rows of a batch: (key, value).
fn objective(replies: &[Vec<u8>], sub: u8) -> Vec<(i32, i32)> {
    let int = |r: &[u8], at: usize| i32::from_le_bytes(r[at..at + 4].try_into().unwrap());
    (replies.iter().map(|r| unframe(r)))
        .filter(|r| r.len() >= 11 && r.starts_with(&[45, 0, sub]))
        .map(|r| (int(r, 3), int(r, 7)))
        .collect()
}

#[test]
fn every_dungeon_record_runs_its_fight_or_says_why() {
    let fights = fights();
    let table = Table::load(&content_dir()).unwrap().unwrap();
    let (mut mapped, mut fallback, mut bonus) = (Vec::new(), Vec::new(), Vec::new());
    for entry in fs::read_dir(content_dir().join("dungeons")).unwrap() {
        let path = entry.unwrap().path();
        let file = read(&path);
        let record = &file["records"][0]["record"];
        let id = record["id"].as_str().unwrap().to_owned();
        let encounter = &record["encounters"][0];
        let goals = encounter["goals"].as_array().unwrap();
        let note = encounter["source"].as_array().unwrap().last().unwrap()["evidence"]
            .as_str()
            .unwrap();
        if goals.is_empty() {
            assert!(
                note.starts_with("(R) No script runs (scripts/dungeon_fights.py): ") && note.len() > 60,
                "{id} runs nothing and must say why: {note}"
            );
            fallback.push(id);
            continue;
        }
        assert!(note.contains("Goals map onto fights/"), "{id}: {note}");
        let fight_id = goals[0]["fight_id"].as_u64().unwrap() as u32;
        let fight = (fights.get(&fight_id)).unwrap_or_else(|| panic!("{id}: no fight {fight_id}"));
        assert!(fight.is_enabled(), "{id}: fight {fight_id} is switched off");
        // The record's entrance runs that fight.
        let door = (table.entrances.iter().find(|e| e.id == id))
            .unwrap_or_else(|| panic!("{id} has no entrance"));
        assert_eq!(door.fight, Some(fight_id), "{id}");
        // The main goal comes first, shows from the start and is the fight's last step: the
        // boss when there is one.
        let main = &fight.objectives[0];
        assert_eq!((main.id.as_str(), main.bonus), ("main", false), "{id}");
        assert!(main.text_id > 0, "{id}: the main goal shows a row");
        assert_eq!(fight.objectives.iter().filter(|o| !o.bonus).count(), 1, "{id}");
        if fight.actors.contains_key("boss") {
            assert_eq!(main.target.actor.as_deref(), Some("boss"), "{id}");
            assert!(fight.actors["boss"].dormant, "{id}: the boss waits for the waves");
        }
        for goal in goals {
            assert_eq!(goal["kind"], "fight_goal", "{id}");
            assert_eq!(goal["fight_id"].as_u64(), Some(u64::from(fight_id)), "{id}");
            let name = goal["goal"].as_str().unwrap();
            let row = (fight.objectives.iter().find(|o| o.id == name))
                .unwrap_or_else(|| panic!("{id}: fight {fight_id} has no goal {name}"));
            // A paced fight (`instance_pacing.json`) brings its waves again after its own last
            // one; the record maps the fight as authored, its own waves.
            let cycled = |id: &str| (fight.waves.iter()).any(|o| id.starts_with(&format!("{}_", o.id)));
            let own = (fight.waves.iter().position(|w| cycled(&w.id))).unwrap_or(fight.waves.len());
            let authored = (fight.targets(&row.target).into_iter())
                .filter(|&i| fight.roster[i].wave.is_none_or(|(w, _)| w < own))
                .count();
            let size = row.count.unwrap_or(authored as u32);
            assert_eq!(goal["count"].as_u64(), Some(u64::from(size)), "{id} {name}");
            if row.bonus {
                bonus.push(id.clone());
            }
        }
        mapped.push(id);
    }
    // 75 guides: 28 run their fight (9 with nothing left out, 19 with a goal the evidence does
    // not place, each named in the record), 47 run nothing, each with its reason. rc-0136 placed
    // the Snakes in a Maze starter, so its guide runs fight 14038.
    assert_eq!((mapped.len(), fallback.len()), (28, 47), "{mapped:?}");
    assert_eq!(bonus, ["bixie_hive"], "the one bonus goal the evidence places");
}

fn shipped_engine() -> EncounterEngine {
    let mut engine = EncounterEngine::load(&content_dir(), None).unwrap_or_else(|e| panic!("{e}"));
    engine
        .load_field_battles(&content_dir().join("field_battles.json"))
        .unwrap_or_else(|e| panic!("{e}"));
    engine
}

fn at(position: [f32; 3]) -> CharacterState {
    CharacterState {
        guid: 77,
        zone_id: 0,
        position,
        rotation: [0.0, 0.0, 1.0],
    }
}

fn click(guid: u64) -> Vec<u8> {
    let mut b = vec![26, 0, 8, 0];
    b.extend(guid.to_le_bytes());
    b.push(0);
    b
}

/// The Bixie Hive through its door, in the engine: the card lists both goals, the workers go
/// free one by one, the Queen's arrival brings three waves, Drone Fauzz only after them, and
/// the results only when the main goal is done and the player takes the exit.
#[test]
fn the_bixie_hive_plays_waves_bonus_boss_then_results() {
    let table = Table::load(&content_dir()).unwrap().unwrap();
    let hive = table.entrances.iter().find(|e| e.id == "bixie_hive").unwrap();
    assert_eq!((hive.fight, hive.counts_as), (Some(HIVE_FIGHT), Some(HIVE_BATTLE)));
    let mut engine = shipped_engine();
    assert_eq!(engine.entrance(hive.guid), Some((HIVE_FIGHT, true)));

    // The card: the main goal with no prefix, the bonus in the Bonus category.
    let card = engine
        .handle(&click(hive.guid), 77, hive.at, BRAWLER, 1_000)
        .unwrap();
    assert_eq!(goal_rows(&card.replies[0]), [(MAIN_TEXT, 0), (BONUS_TEXT, 4)]);
    assert_eq!(engine.take_pending_battle(hive.at), Some(HIVE_FIGHT));
    let zone = (engine.begin(HIVE_FIGHT, at(hive.at), BRAWLER).unwrap().zoning).unwrap();
    assert_eq!((zone.name.as_str(), zone.zone_id), ("sg_bixie_hive", 1203));
    let ready = engine.on_zone_ready(BRAWLER, 2_000).unwrap();
    assert!(ready.facts.contains(&FightFact::Entered { fight: HIVE_BATTLE }));
    let fight = engine.fight_run().expect("the hive runs its fight").definition().clone();
    let (queen, queen_at) = actor(&fight, "speaker");
    let (fauzz, fauzz_at) = actor(&fight, "boss");
    // 18 guards along the tunnels, the Queen and four workers; not Drone Fauzz.
    let standing = spawned(&ready.replies);
    assert_eq!(standing.len(), 23, "{standing:?}");
    assert!(standing.contains(&queen) && !standing.contains(&fauzz));
    // Goals (2): the main one with no counter (a total of 1), the bonus 0/4.
    let rows = objective(&ready.replies, 1);
    assert!(rows.contains(&(MAIN_TEXT, 1)) && rows.contains(&(BONUS_TEXT, 4)), "{rows:?}");
    engine.toggle_god();

    // The bonus: each worker goes free with a talk, thanks the player and runs.
    let mut now = 3_000;
    for n in 1..=4_u32 {
        let (worker, spot) = actor(&fight, &format!("captive_{n}"));
        now += 1_000;
        let out = engine.handle(&click(worker), 77, spot, BRAWLER, now).unwrap();
        assert!(said(&out.replies, worker, WORKER_THANKS), "worker {n} thanks");
        assert!(objective(&out.replies, 2).contains(&(BONUS_TEXT, n as i32)), "{n}/4");
        let complete = objective(&out.replies, 3).iter().any(|(key, _)| *key == BONUS_TEXT);
        assert_eq!(complete, n == 4, "the bonus's Goal Complete! with the fourth");
        let gone = engine.tick(spot, BRAWLER, now + 3_100).unwrap();
        assert!(
            gone.replies
                .iter()
                .any(|r| r.starts_with(&[35, 0, 3, 0]) && r[6..14] == worker.to_le_bytes()),
            "worker {n} has run off"
        );
        now += 3_100;
    }

    // The Queen: coming near her calls the first wave; each cleared wave calls the next.
    now += 500;
    let met = engine.tick(queen_at, BRAWLER, now).unwrap();
    assert!(said(&met.replies, queen, QUEEN_LINES[0]));
    let mut all = met.replies.clone();
    let (mut waves, mut kills, mut facts) = (vec![spawned(&met.replies).len()], Vec::new(), Vec::new());
    let guards = guards(&fight);
    for _ in 0..400 {
        if engine.standing_enemies().iter().any(|e| e.0 == fauzz) {
            break;
        }
        now += 700;
        let Some((guid, spot, _)) = (engine.standing_enemies().into_iter())
            .find(|e| !guards.contains(&e.0))
        else {
            let out = engine.tick(queen_at, BRAWLER, now).unwrap();
            if !spawned(&out.replies).is_empty() {
                waves.push(spawned(&out.replies).len());
            }
            all.extend(out.replies);
            facts.extend(out.facts);
            continue;
        };
        let hit = engine.attack(guid, spot, BRAWLER, 0, now).unwrap();
        kills.extend(hit.kills.iter().map(|k| k.kind));
        facts.extend(hit.facts);
        let out = engine.tick(queen_at, BRAWLER, now + 100).unwrap();
        if !spawned(&hit.replies).is_empty() || !spawned(&out.replies).is_empty() {
            waves.push(spawned(&hit.replies).len() + spawned(&out.replies).len());
        }
        all.extend(hit.replies);
        all.extend(out.replies);
        facts.extend(out.facts);
    }
    assert_eq!(waves[..3], [3, 3, 3], "three waves of two Soldiers and a Battlemage: {waves:?}");
    assert_eq!(kills.len(), 9, "the three waves, not a guard");
    assert!(!kills.contains(&FAUZZ), "Drone Fauzz is not in a wave");
    for line in &QUEEN_LINES[..4] {
        assert!(said(&all, queen, *line), "the Queen says {line}");
    }
    // Drone Fauzz arrives after the waves, with his boss bar and the Queen's line.
    assert!(
        all.iter().any(|r| r.starts_with(&[32, 0, 9, 0]) && r[4..12] == fauzz.to_le_bytes()),
        "his boss bar"
    );
    now += 600;
    all.extend(engine.tick(queen_at, BRAWLER, now).unwrap().replies);
    assert!(said(&all, queen, QUEEN_LINES[4]), "the betrayal line");
    assert!(!facts.iter().any(|f| matches!(f, FightFact::Won { .. })));

    // Fauzz falls: the main goal is done, the Queen thanks the player, the exit opens.
    let mut end = Vec::new();
    for _ in 0..200 {
        if !engine.standing_enemies().iter().any(|e| e.0 == fauzz) {
            break;
        }
        now += 700;
        let hit = engine.attack(fauzz, fauzz_at, BRAWLER, 0, now).unwrap();
        kills.extend(hit.kills.iter().map(|k| k.kind));
        end.extend(hit.replies);
        facts.extend(hit.facts);
        let out = engine.tick(fauzz_at, BRAWLER, now + 100).unwrap();
        end.extend(out.replies);
        facts.extend(out.facts);
    }
    assert_eq!(kills.last(), Some(&FAUZZ), "Fauzz's kill is the last");
    assert!(objective(&end, 3).iter().any(|(key, _)| *key == MAIN_TEXT), "Goal Complete!");
    assert!(said(&end, queen, QUEEN_LINES[5]), "the Queen's thanks");
    let door = fight.guid(None);
    assert!(spawned(&end).contains(&door), "the exit door");
    assert!(!facts.iter().any(|f| matches!(f, FightFact::Won { .. })), "no results yet");
    assert_eq!(engine.run_phase(), Some(EncounterPhase::Fighting));

    // The exit: the results card, the win counted as the hive's battle, once.
    now += 1_000;
    let out = engine.tick(fight.exit_spot(), BRAWLER, now).unwrap();
    assert_eq!(out.facts.iter().filter(|f| **f == FightFact::Won { fight: HIVE_BATTLE }).count(), 1);
    assert_eq!(out.win_completed, Some(HIVE_BATTLE));
    let card = score_rows(&out.replies);
    assert_eq!((card["scoreObjectives"].0, card["scoreObjectives"].1), (1, 1));
    assert_eq!((card["scoreBonusObjectives"].0, card["scoreBonusObjectives"].1), (1, 1));
    assert_eq!(engine.run_phase(), Some(EncounterPhase::Won));
}

/// Royal Honey ("Recover some Royal Honey from Fauzz") drops from the hive's own Drone Fauzz, who
/// comes after the waves; the clearing battle's copy (12370's 1237001), met in its only wave,
/// no longer pays it.
#[test]
fn royal_honey_drops_from_the_hives_fauzz_only() {
    let rewards = read(&content_dir().join("field_kill_rewards.json"));
    let drops: Vec<u64> = (rewards["quest_drops"].as_array().unwrap().iter())
        .filter(|row| row["quest_id"] == 3_957_507)
        .map(|row| row["definition_id"].as_u64().unwrap())
        .collect();
    assert_eq!(drops, [u64::from(FAUZZ)]);
    let hive = fights().remove(&HIVE_FIGHT).unwrap();
    let boss = &hive.resolved[&hive.actors["boss"].definition];
    assert_eq!((boss.definition_id, boss.name_id, boss.model_id), (FAUZZ as i32, 38510, 218));
}

/// rc-0137 recheck: the native client drops a 41/114 it cannot read to the last byte, and then
/// Go does nothing (the Bixie Hive's two goal rows were 21 bytes short each). Every open
/// entrance's card and every battle marker's card reads whole and opens the start screen.
#[test]
fn every_start_card_reads_to_its_last_byte() {
    let table = Table::load(&content_dir()).unwrap().unwrap();
    let engine = shipped_engine();
    let file = read(&content_dir().join("field_battles.json"));
    let markers = (file["battles"].as_array().unwrap().iter()).map(|b| {
        let at = &b["spawner"]["position"];
        let at = [0, 1, 2].map(|i| at[i].as_f64().unwrap() as f32);
        (b["spawner"]["guid"].as_u64().unwrap(), at, b["id"].to_string())
    });
    let doors = (table.entrances.iter()).map(|e| (e.guid, e.at, e.id.clone()));
    for (guid, at, name) in doors.chain(markers) {
        let mut engine = engine.clone();
        if engine.entrance(guid).is_some_and(|(_, open)| !open) {
            continue;
        }
        let out = engine.handle(&click(guid), 77, at, BRAWLER, 1_000).unwrap();
        let card = encounter_card::read_card(&out.replies[0]);
        assert!(!card.launch && !card.goals.is_empty(), "{name}");
    }
}
