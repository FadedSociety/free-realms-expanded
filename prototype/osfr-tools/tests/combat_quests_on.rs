//! The combat quests switched on over the combat engine (piece `quests-dialog`).
//!
//! * Every fight file a quest stands on is played in the real engine: arrival, real attacks on
//!   whatever stands, every wave, the boss a rule wakes, the exit door at the last kill and
//!   the click on it. The kills the engine reports there are kept.
//! * Every enabled quest's fight steps are then counted in the production quest engine from
//!   exactly those kills: a `defeat` inside a fight only from a kind that fight reported, a
//!   `complete_dungeon` or `enter_fight` only for a fight that was played or a shipped battle.
//!
//! The giver, the turn-in and the reward of every enabled quest are the reachability gate's
//! (`tests/enabled_quest_reachability.rs`); the packets of a field kill, a fight kill and the
//! start screen are `tests/quest_combat_steps.rs` and `tests/shared_field_kill_quest_credit.rs`.
#[path = "support/fight_kit.rs"]
mod fight_kit;

use fight_kit::*;
use osfr_tools::{
    encounter::{EncounterPhase, FightFact, KillFact, Rank, director, director::fight::Side},
    quest_engine::{
        CharacterEligibility, QuestEngine, QuestEvent, QuestProgress, QuestStatus,
    },
};
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    sync::OnceLock,
};

const CHARACTER: u64 = 7;
/// The Frostfang battle has its own suite (`tests/fight_frostfang.rs`); it is still played here.
const MOST_SWINGS: usize = 60_000;

fn read(path: &std::path::Path) -> Value {
    serde_json::from_slice(&fs::read(path).unwrap_or_else(|e| panic!("{}: {e}", path.display())))
        .unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

/// The enabled fight files, in id order.
fn fight_files() -> Vec<Value> {
    let mut fights: Vec<Value> = fs::read_dir(content().join("fights"))
        .unwrap()
        .map(|entry| read(&entry.unwrap().path()))
        .filter(|fight| fight["enabled"] != false)
        .collect();
    fights.sort_by_key(|fight| fight["id"].as_u64());
    fights
}

/// Play one fight to its exit door and through it; the kinds its kills reported.
fn play(file: &Value) -> BTreeSet<u32> {
    let id = file["id"].as_u64().unwrap() as u32;
    let (dir, mut engine) = engine_with(std::slice::from_ref(file));
    enter(&mut engine, id, 1_000);
    let fight = engine
        .fight_run()
        .unwrap_or_else(|| panic!("fight {id}: the file's own run"))
        .definition()
        .clone();
    engine.toggle_god();
    let arrive = fight.place.arrive;
    // With nothing standing the player walks the fight's named points, so a wave that waits
    // for a player there (a dungeon's finale) is called in.
    let points: Vec<[f32; 3]> = fight.points.values().flatten().copied().collect();
    let (mut now, mut kinds, mut kills) = (1_000, BTreeSet::new(), 0);
    for step in 0..MOST_SWINGS {
        if engine.fight_run().is_some_and(|run| run.exit_is_open()) {
            break;
        }
        now += 700;
        let mut at_tick = arrive;
        if let Some((guid, at, _)) = engine.standing_enemies().first().copied() {
            let hit = engine.attack(guid, at, BRAWLER, 0, now).unwrap();
            for kill in &hit.kills {
                assert_eq!(kill.fight, Some(id), "fight {id}: a kill names its fight");
                kinds.insert(kill.kind);
            }
            kills += hit.kills.len();
        } else if !points.is_empty() {
            at_tick = points[step % points.len()];
        }
        let tick = engine.tick(at_tick, BRAWLER, now + 100).unwrap();
        assert!(tick.kills.is_empty(), "fight {id}: only attacks kill here");
        assert_eq!(
            engine.run_phase(),
            Some(EncounterPhase::Fighting),
            "fight {id}: nothing ends it before the door"
        );
    }
    let run = engine.fight_run().unwrap();
    assert!(
        run.exit_is_open(),
        "fight {id}: the exit door never opened ({kills} kills, {} standing)",
        engine.standing_enemies().len()
    );
    // Allies (a dungeon's captives and the NPC it is fought for) are not fought.
    let enemies = (fight.roster.iter())
        .filter(|slot| slot.side == Side::Enemy)
        .count();
    assert_eq!(
        run.kills as usize, enemies,
        "fight {id}: every enemy of the roster was fought"
    );
    // A kind with no kill-credit id reports no kill (the captured battle's Prowlers); the
    // quest fights give every kind one.
    assert!(
        kills == enemies || id == 12365,
        "fight {id}: {kills} kills reported for {enemies} enemies"
    );
    let exit = fight.exit_spot();
    let won = engine
        .handle(
            &click(fight.guid(None)),
            1,
            [exit[0], exit[1], exit[2] - 2.0],
            BRAWLER,
            now + 5_000,
        )
        .unwrap();
    assert_eq!(engine.run_phase(), Some(EncounterPhase::Won), "fight {id}");
    assert!(won.facts.contains(&FightFact::Won { fight: id }), "fight {id}");
    assert_eq!(
        u64::from(won.rewards[0].coins),
        file["rewards"]["coins"].as_u64().unwrap_or(0),
        "fight {id}: the file's coins are paid"
    );
    let _ = fs::remove_dir_all(dir);
    kinds
}

/// Fight id to the kinds its kills report: every fight file played, plus the rosters of the
/// `field_battles.json` rows (their runs are `tests/field_battle_roster.rs`).
fn fights_played() -> &'static BTreeMap<u32, BTreeSet<u32>> {
    static PLAYED: OnceLock<BTreeMap<u32, BTreeSet<u32>>> = OnceLock::new();
    PLAYED.get_or_init(|| {
        let mut played: BTreeMap<u32, BTreeSet<u32>> = fight_files()
            .iter()
            .map(|file| (file["id"].as_u64().unwrap() as u32, play(file)))
            .collect();
        let battles = read(&content().join("field_battles.json"));
        for battle in battles["battles"].as_array().unwrap() {
            let kinds = battle["enemies"]
                .as_array()
                .unwrap()
                .iter()
                .map(|enemy| enemy["definition_id"].as_u64().unwrap() as u32);
            played
                .entry(battle["id"].as_u64().unwrap() as u32)
                .or_default()
                .extend(kinds);
        }
        played
    })
}

#[test]
fn every_quest_fight_plays_from_arrival_through_the_exit_door() {
    let played = fights_played();
    let files = fight_files();
    assert!(files.len() >= 40, "the quest fights ship: {}", files.len());
    // Every scene but the clearing still needs its first load in the client; the file says so.
    for file in &files {
        let id = file["id"].as_u64().unwrap() as u32;
        assert!(!played[&id].is_empty(), "fight {id} reported no kill");
        let arena = file["place"]["arena"].as_str().unwrap();
        let said = file["source"]
            .as_array()
            .unwrap()
            .iter()
            .any(|row| row["evidence"].as_str().unwrap().contains("needs one native load"));
        assert!(
            arena == "sg_random_encounter_clearing" || said,
            "fight {id}: scene {arena} is unproven and the file must say so"
        );
    }
    // The loader the engine uses takes the whole folder at once (ids, limits, the catalogue).
    let loaded = director::load_dir(&content()).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(loaded.iter().filter(|f| f.is_enabled()).count(), files.len());
}

fn eligibility() -> CharacterEligibility {
    CharacterEligibility {
        player_level: 20,
        nonmember_quest_limit: 30,
        member_quest_limit: 30,
        ..Default::default()
    }
}

/// The shipped catalogue with `quest` active and every step before `step` done.
fn engine_at(quest: u32, required: &[u32], step: usize) -> QuestEngine {
    let engine =
        QuestEngine::from_split_content(&content(), CHARACTER, eligibility(), None).unwrap();
    let mut state = engine.state().clone();
    let counts = required
        .iter()
        .enumerate()
        .map(|(index, required)| if index < step { *required } else { 0 })
        .collect();
    state.quests.insert(
        quest,
        QuestProgress {
            status: QuestStatus::Active,
            counts,
        },
    );
    QuestEngine::from_split_content(&content(), CHARACTER, eligibility(), Some(state)).unwrap()
}

fn kill(kind: u32, fight: Option<u32>, job: u32) -> QuestEvent {
    QuestEvent::Kill {
        fact: KillFact {
            kind,
            guid: 1,
            rank: Rank::Normal,
            tags: Vec::new(),
            zone: 0,
            area: None,
            fight,
            solo: true,
            job,
            ability: None,
            item: None,
        },
    }
}

fn number(value: &Value) -> Option<u32> {
    value.as_u64().and_then(|n| u32::try_from(n).ok())
}

/// Every enabled quest: each `defeat` that names kinds or a fight, each `complete_dungeon` and
/// each `enter_fight` counts from what a played fight (or the open world) reports.
#[test]
fn every_enabled_fight_step_counts_from_the_kills_the_fights_report() {
    let played = fights_played();
    let (mut steps, mut in_fights) = (0, BTreeSet::new());
    for entry in fs::read_dir(content().join("quests")).unwrap() {
        let document = read(&entry.unwrap().path());
        for record in document["records"].as_array().unwrap() {
            if record["enabled"] == false {
                continue;
            }
            let quest = number(&record["id"]).unwrap();
            let goals = record["objectives"].as_array().unwrap();
            let required: Vec<u32> = goals
                .iter()
                .map(|goal| number(&goal["count"]).or(number(&goal["quantity"])).unwrap_or(1))
                .collect();
            for (step, goal) in goals.iter().enumerate() {
                let fight = number(&goal["in_fight"]);
                let event = match goal["kind"].as_str().unwrap_or_default() {
                    "defeat" => {
                        // A step with an area, zone, ability or item rule is its own suite's.
                        if ["in_area", "in_zone", "ability", "item"]
                            .iter()
                            .any(|key| !goal[*key].is_null() || !goal["with"][*key].is_null())
                        {
                            continue;
                        }
                        let kinds: Vec<u32> = number(&goal["model_id"])
                            .into_iter()
                            .chain(goal["kinds"].as_array().into_iter().flatten().filter_map(number))
                            .collect();
                        let kind = match (fight, kinds.first()) {
                            // Inside a fight: a kind that fight really reported.
                            (Some(id), _) => {
                                let reported = played.get(&id).unwrap_or_else(|| {
                                    panic!("quest {quest}: fight {id} is not a played fight")
                                });
                                in_fights.insert(id);
                                *reported
                                    .iter()
                                    .find(|kind| kinds.is_empty() || kinds.contains(kind))
                                    .unwrap_or_else(|| {
                                        panic!("quest {quest}: fight {id} reports none of {kinds:?}")
                                    })
                            }
                            (None, Some(kind)) => *kind,
                            (None, None) => continue,
                        };
                        let job = number(&goal["with"]["job"]).unwrap_or(43);
                        if fight.is_some() {
                            let mut engine = engine_at(quest, &required, step);
                            let outside = engine.on_event(kill(kind, None, job)).unwrap();
                            assert!(
                                !outside.changed,
                                "quest {quest} step {step}: a kill outside fight {fight:?} counts nothing"
                            );
                        }
                        kill(kind, fight, job)
                    }
                    "complete_dungeon" => {
                        let id = number(&goal["encounter_id"]).unwrap();
                        assert!(
                            played.contains_key(&id) || id == 411,
                            "quest {quest}: fight {id} is not a played fight"
                        );
                        QuestEvent::CompleteDungeon { encounter_id: id }
                    }
                    "enter_fight" => {
                        let id = number(&goal["fight_id"]).unwrap();
                        assert!(
                            played.contains_key(&id),
                            "quest {quest}: fight {id} is not a played fight"
                        );
                        QuestEvent::EnterFight { fight_id: id }
                    }
                    _ => continue,
                };
                let mut engine = engine_at(quest, &required, step);
                assert!(
                    engine.on_event(event).unwrap().changed,
                    "quest {quest} step {step}: the fight's own fact counts"
                );
                assert_eq!(engine.progress(quest).unwrap().counts[step], 1);
                steps += 1;
            }
        }
    }
    assert!(steps >= 35, "fight steps counted: {steps}");
    assert!(!in_fights.is_empty(), "some step names its fight");
}
