//! Fights as JSON (piece `combat-jobs`, P5 the director): the loader and its refusals, every
//! fight shape of the combat design written as a file, the shipped files, and the rows of
//! `field_battles.json` that have no fight file still playing as before.
use osfr_tools::encounter::{
    EncounterEngine, EncounterPhase, FightFact,
    director::{self, DirectorAct, fight::Kind},
};
use serde_json::{Value, json};

#[path = "support/fight_kit.rs"]
mod fight_kit;
use fight_kit::*;

/// A two-wave battle that loads; each refusal test breaks one thing in it.
fn battle() -> Value {
    json!({
        "schema_version": 1, "id": 500, "kind": "battle", "title_id": 12, "description_id": 13,
        "place": arena(), "kinds": kinds(),
        "actors": {"chief": {"definition": "chief", "at": [0, 0, 30], "tags": ["leaders"]}},
        "points": {"ring": [[10, 0, 10], [-10, 0, 10]]},
        "waves": [
            {"id": "w1", "when": {"fight_started": {"after_ms": 1000}},
             "spawn": [{"definition": "grunt", "count": 2, "at": "ring"}]},
            {"id": "w2", "when": {"wave_left": {"wave": "w1", "left": 0}}, "or_after_ms": 5000,
             "spawn": [{"definition": "grunt", "count": 3, "at": "ring"}]}
        ],
        "objectives": [{"id": "all", "kind": "defeat", "text_id": 14, "target": {"all": true}, "primary": true}],
        "source": source()
    })
}

fn broken(change: impl FnOnce(&mut Value)) -> String {
    let mut fight = battle();
    change(&mut fight);
    refusal(&fight)
}

#[test]
fn a_well_formed_fight_loads_with_its_roster_in_order() {
    let fight = parse(&battle());
    let keys: Vec<&str> = fight.roster.iter().map(|slot| slot.key.as_str()).collect();
    assert_eq!(
        keys,
        [
            "chief", "w1.0.0.0", "w1.0.0.1", "w2.0.0.0", "w2.0.0.1", "w2.0.0.2"
        ]
    );
    assert!(fight.roster[0].start && !fight.roster[1].start);
    assert!(fight.roster[0].tags.iter().any(|t| t == "actor:chief"));
    assert!(fight.roster[0].tags.iter().any(|t| t == "boss"));
    assert!(fight.roster[1].tags.iter().any(|t| t == "wave:w1"));
    assert_eq!(fight.knockouts(), 10, "a battle allows ten knockouts");
    // Guids are one block per fight: the door, then the roster.
    assert_eq!(fight.guid(Some(0)), fight.guid(None) + 1);
    assert_ne!(
        fight.guid(None) >> 8,
        parse(&{
            let mut other = battle();
            other["id"] = json!(501);
            other
        })
        .guid(None)
            >> 8
    );
}

#[test]
fn an_unknown_tag_is_refused_with_its_pointer() {
    let error = broken(|f| f["objectives"][0]["target"] = json!({"tag": "nobody"}));
    assert!(
        error.contains("/objectives/0/target/tag") && error.contains("unknown tag 'nobody'"),
        "{error}"
    );
    let error = broken(|f| {
        f["script"] =
            json!([{"id": "r", "when": {"tag_dead": {"tag": "ghosts"}}, "do": [{"win": {}}]}]);
    });
    assert!(
        error.contains("/script/0/when") && error.contains("unknown tag 'ghosts'"),
        "{error}"
    );
    let error = broken(|f| f["actors"]["chief"]["guarded_by"] = json!("sentries"));
    assert!(
        error.contains("guarded_by") && error.contains("sentries"),
        "{error}"
    );
    let error = broken(|f| {
        f["script"] = json!([{"id": "r", "when": {"clock": {"ms": 1000}}, "do": [{"wake": {"tag": "sleepers"}}]}]);
    });
    assert!(
        error.contains("/script/0/do/0") && error.contains("sleepers"),
        "{error}"
    );
}

#[test]
fn an_objective_nobody_can_finish_is_refused() {
    // A dormant actor nothing wakes.
    let error = broken(|f| f["actors"]["chief"]["dormant"] = json!(true));
    assert!(
        error.contains("unreachable objective") && error.contains("chief"),
        "{error}"
    );
    // More to defeat than the fight holds.
    let error = broken(|f| f["objectives"][0]["count"] = json!(7));
    assert!(error.contains("unreachable objective"), "{error}");
    // A place that does not exist.
    let error = broken(|f| {
        f["objectives"][0] = json!({"id": "go", "kind": "reach", "target": {"point": "nowhere"}});
    });
    assert!(
        error.contains("unreachable objective") && error.contains("nowhere"),
        "{error}"
    );
    // A ward that enemies could be told to defeat.
    let error = broken(|f| {
        f["objectives"][0] =
            json!({"id": "hold", "kind": "defend", "target": {"actor": "chief"}, "ms": 5000});
    });
    assert!(
        error.contains("unreachable objective") && error.contains("ally"),
        "{error}"
    );
    // Only bonus goals: the fight could never end.
    let error = broken(|f| f["objectives"][0]["bonus"] = json!(true));
    assert!(error.contains("primary"), "{error}");
}

#[test]
fn a_wave_that_cannot_fire_is_refused() {
    let error = broken(|f| f["waves"][1]["when"] = json!({"wave_left": {"wave": "w9"}}));
    assert!(
        error.contains("/waves/1/when") && error.contains("unknown wave 'w9'"),
        "{error}"
    );
    // Two waves that wait on each other.
    let error = broken(|f| f["waves"][0]["when"] = json!({"wave_left": {"wave": "w2"}}));
    assert!(error.contains("can never fire"), "{error}");
    // No trigger and no rule that spawns it.
    let error = broken(|f| {
        f["waves"][0].as_object_mut().unwrap().remove("when");
        f["waves"][1]["when"] = json!({"clock": {"ms": 4000}});
    });
    assert!(error.contains("wave 'w1' can never fire"), "{error}");
    // The same wave is fine once a rule spawns it.
    let mut fight = battle();
    fight["waves"][0].as_object_mut().unwrap().remove("when");
    fight["script"] = json!([{"id": "go", "when": {"clock": {"ms": 2000}}, "do": [{"spawn_wave": {"id": "w1"}}]}]);
    parse(&fight);
}

#[test]
fn limits_and_loops_are_refused() {
    let error = broken(|f| f["waves"][0]["spawn"][0]["count"] = json!(33));
    assert!(error.contains("count"), "{error}");
    let error = broken(|f| {
        f["waves"][0]["repeat"] = json!({"times": 32, "every_ms": 1000});
        f["waves"][0]["spawn"][0]["count"] = json!(3);
    });
    assert!(error.contains("1..64 actors"), "{error}");
    let error = broken(|f| f["waves"][0]["repeat"] = json!({"times": 2, "every_ms": 100}));
    assert!(error.contains("repeat"), "{error}");
    let error = broken(|f| {
        f["script"] =
            json!([{"id": "r", "when": {"every": {"ms": 50}}, "do": [{"signal": {"name": "x"}}]}]);
    });
    assert!(error.contains("250"), "{error}");
    // A rule that raises the signal that starts it, with nothing to slow it down.
    let looping = |extra: Value| {
        let mut fight = battle();
        let mut rule = json!({"id": "r", "when": {"signal": {"name": "again"}}, "do": [{"signal": {"name": "again"}}]});
        rule.as_object_mut()
            .unwrap()
            .extend(extra.as_object().unwrap().clone());
        fight["script"] = json!([rule]);
        try_parse(&fight)
    };
    let error = looping(json!({})).unwrap_err();
    assert!(error.contains("cooldown"), "{error}");
    looping(json!({"cooldown_ms": 1000})).unwrap();
    looping(json!({"once": true})).unwrap();
    let error = broken(|f| {
        f["script"] = json!([{"id": "r", "when": {"clock": {"ms": 1}}, "do": [{"dance": {}}]}]);
    });
    assert!(error.contains("unknown action 'dance'"), "{error}");
    let error = broken(|f| f["source"] = json!([{"kind": "wiki", "evidence": "a page"}]));
    assert!(error.contains("/source"), "{error}");
    let error = broken(|f| f["surprise"] = json!(1));
    assert!(error.contains("surprise"), "{error}");
}

/// Every fight shape of the design's section 11 as a file the loader accepts.
#[test]
fn every_fight_shape_is_expressible() {
    // A dungeon: rooms, a three-goal chain, a bonus goal, fifteen knockouts.
    let dungeon = parse(&json!({
        "schema_version": 1, "id": 601, "kind": "dungeon", "difficulty": 2,
        "place": {"zone": 9611, "arena": "sg_bixie_hive", "arrive": [0, 0, 0],
                  "rooms": [{"id": "gate", "box": [-20, -5, -20, 20, 10, 20]},
                            {"id": "hall", "box": [20, -5, -20, 80, 10, 20]}]},
        "kinds": kinds(),
        "actors": {"g1": {"definition": "grunt", "at": [30, 0, 4], "room": "hall", "tags": ["guards"]},
                   "cocoon1": {"definition": "captive", "at": [44, 0, -8], "tags": ["cocoon"], "guarded_by": "guards"},
                   "fauzz": {"definition": "chief", "at": [70, 0, 0], "boss": true, "dormant": true}},
        "points": {"door": [[60, 0, 0]]},
        "waves": [{"id": "m1", "when": {"objective_done": {"id": "guards"}},
                   "spawn": [{"definition": "mage", "count": 1, "at": "door"}, {"definition": "grunt", "count": 2, "at": "door"}]},
                  {"id": "m2", "when": {"wave_left": {"wave": "m1", "left": 0}},
                   "spawn": [{"definition": "mage", "count": 1, "at": "door"}, {"definition": "grunt", "count": 2, "at": "door"}]}],
        "objectives": [{"id": "guards", "kind": "defeat", "target": {"tag": "guards"}, "primary": true},
                       {"id": "waves", "kind": "defeat", "target": {"waves": ["m1", "m2"]}, "primary": true},
                       {"id": "boss", "kind": "defeat", "target": {"actor": "fauzz"}, "primary": true},
                       {"id": "free", "kind": "free", "target": {"tag": "cocoon"}, "bonus": true}],
        "script": [{"id": "boss_in", "when": {"objective_done": {"id": "waves"}}, "do": [{"wake": {"tag": "actor:fauzz"}}]}],
        "source": source()
    }));
    assert_eq!((dungeon.kind, dungeon.knockouts()), (Kind::Dungeon, 15));

    // An open-world named boss: attack to start, a chest for taking part.
    let named = parse(&json!({
        "schema_version": 1, "id": 602, "kind": "world_boss",
        "place": {"open_world": true, "centre": [100, 0, 100], "radius": 40},
        "kinds": kinds(),
        "actors": {"boss": {"definition": "chief", "at": [100, 0, 100], "boss": true}},
        "objectives": [{"id": "boss", "kind": "defeat", "target": {"actor": "boss"}, "primary": true}],
        "rewards": {"participation_chest": {"coins": 100}},
        "source": source()
    }));
    assert!(!named.instanced() && named.knockouts() == 0);

    // A seasonal world boss: a timetable, a route, a banner, a borrowed weapon.
    let seasonal = parse(&json!({
        "schema_version": 1, "id": 603, "kind": "world_boss",
        "place": {"open_world": true, "centre": [0, 0, 0], "radius": 60},
        "schedule": {"every_s": 900, "towns": ["snowhill", "seaside"], "route": "march", "banner": "The Snowman King marches!"},
        "rules": {"power_up": "snowball_launcher"},
        "kinds": kinds(), "points": {"march": [[0, 0, 0], [50, 0, 0]]},
        "actors": {"king": {"definition": "chief", "at": "march", "boss": true}},
        "objectives": [{"id": "king", "kind": "defeat", "target": {"actor": "king"}, "primary": true}],
        "script": [{"id": "taunt", "when": {"every": {"ms": 48000, "first_ms": 5000}},
                    "do": [{"telegraph": {"say": {"text": "Brrr!", "how": "red"}, "ms": 2000, "attack": "blizzard"}, "on": "actor:king"}]}],
        "source": source()
    }));
    assert_eq!(seasonal.schedule.as_ref().map(|s| s.every_s), Some(900));

    // A quest fight: defend the camp until twenty attackers are down.
    let quest = parse(&defend_the_camp());
    assert_eq!(quest.kind, Kind::QuestFight);
    assert_eq!(quest.roster.len(), 21, "the tent and five waves of four");

    // An NPC duel: one enemy, one player.
    let duel = parse(&json!({
        "schema_version": 1, "id": 605, "kind": "duel", "place": arena(), "rules": {"solo": true},
        "kinds": kinds(),
        "actors": {"bullseye": {"definition": "chief", "at": [0, 0, 10]}},
        "objectives": [{"id": "win", "kind": "defeat", "text_id": 382960, "target": {"actor": "bullseye"}, "primary": true}],
        "source": source()
    }));
    assert_eq!((duel.group_max(), duel.knockouts()), (1, 0));
    let error = refusal(&json!({
        "schema_version": 1, "id": 606, "kind": "duel", "place": arena(), "kinds": kinds(),
        "actors": {"a": {"definition": "chief", "at": [0, 0, 10]}},
        "objectives": [{"id": "win", "kind": "defeat", "target": {"actor": "a"}}],
        "source": source()
    }));
    assert!(error.contains("one on one"), "{error}");
}

fn defend_the_camp() -> Value {
    json!({
        "schema_version": 1, "id": 604, "kind": "quest_fight",
        "entry": {"quest_ids": [3050021], "interact": {"npc_guid": 100000012345_u64}},
        "place": {"open_world": true, "centre": [-640, 0, 2280], "radius": 30},
        "kinds": kinds(), "points": {"ring": [[-620, 0, 2280], [-660, 0, 2280]]},
        "actors": {"camp": {"definition": "tent", "at": [-640, 0, 2280], "tags": ["ward"]}},
        "waves": [{"id": "a", "when": {"fight_started": {"after_ms": 3000}}, "repeat": {"times": 5, "every_ms": 12000},
                   "spawn": [{"definition": "grunt", "count": 4, "at": "ring"}]}],
        "objectives": [{"id": "hold", "kind": "defend", "target": {"actor": "camp"},
                        "until": {"defeated": {"definition": "grunt", "count": 20}}, "primary": true}],
        "source": source()
    })
}

/// The files the repository ships load through the same loader the engine uses.
#[test]
fn the_shipped_fights_load() {
    let fights = director::load_dir(&content()).unwrap_or_else(|e| panic!("{e}"));
    let ids: Vec<(u32, bool)> = fights.iter().map(|f| (f.id, f.is_enabled())).collect();
    // File order: the names sort as text. 12365 is the captured battle, 96 the conversion that
    // waits for its native run; the 14xxx files are the fights the combat quests stand on. Fix
    // F5: the 13xxx files are dungeon doors' own fights (they ran clearing battles before), and
    // 14008, 14029, 14037, 14043 and 14045 are dungeons built from their fork battles.
    assert_eq!(ids.first(), Some(&(12365, true)));
    assert_eq!(ids.last(), Some(&(96, false)));
    let quest_fights: Vec<u32> = ids[1..ids.len() - 1].iter().map(|(id, _)| *id).collect();
    assert_eq!(
        quest_fights,
        [
            13002, 13003, 13035, 13041, 14001, 14003, 14006, 14007, 14008, 14010, 14011, 14013,
            14016, 14017, 14018, 14019, 14021, 14022, 14023, 14025, 14027, 14029, 14030, 14033, 14034,
            14037, 14038, 14039, 14040, 14041, 14042, 14043, 14045, 14049, 14050, 14052, 14055,
            14057, 14059, 14061, 14069, 14070, 14072, 14073, 14074, 14076, 14077, 14078, 14079
        ]
    );
    assert!(ids[1..ids.len() - 1].iter().all(|(_, enabled)| *enabled));
    for fight in &fights {
        assert!(fight.roster.len() <= director::fight::MAX_ACTORS);
        // A fight that takes over a battle row has that row.
        if fight.entry.battle_row {
            let battles: Value = serde_json::from_slice(
                &std::fs::read(content().join("field_battles.json")).unwrap(),
            )
            .unwrap();
            let rows = battles["battles"].as_array().unwrap();
            assert!(
                rows.iter().any(|row| row["id"] == fight.id),
                "fight {} has no battle row",
                fight.id
            );
        }
    }
    // Battle 96 as a fight: the same seven actors, the helpers asleep until the ladder.
    let troll = fights.iter().find(|f| f.id == 96).unwrap();
    assert_eq!(troll.roster.iter().filter(|slot| slot.start).count(), 1);
    assert_eq!(troll.roster.len(), 7);
    let mut replay = Replay::start(troll, 1);
    let summoner = replay.guid("summoner");
    for (pct, standing) in [(80, 0), (74, 2), (49, 4), (24, 6)] {
        replay.run.set_health(summoner, pct);
        replay.step(100);
        assert_eq!(replay.living("helpers").len(), standing, "at {pct}%");
    }
    replay.kill(summoner);
    assert!(replay.run.exit_is_open(), "the goal is the summoner alone");
}

/// A directory with a bad fight refuses the engine load and names the file.
#[test]
fn the_engine_refuses_a_bad_fight_file_and_names_it() {
    let (dir, _) = engine_with(&[]);
    let mut fight = battle();
    fight["objectives"][0]["target"] = json!({"tag": "nobody"});
    std::fs::write(dir.join("fights/500.json"), fight.to_string()).unwrap();
    let error = EncounterEngine::load(&dir, None)
        .err()
        .expect("loaded")
        .to_string();
    assert!(
        error.contains("fights/500.json") && error.contains("unknown tag"),
        "{error}"
    );
    // A file named after another id, and one that reuses the cave's id.
    std::fs::write(dir.join("fights/500.json"), battle().to_string()).unwrap();
    std::fs::rename(dir.join("fights/500.json"), dir.join("fights/77.json")).unwrap();
    let error = EncounterEngine::load(&dir, None)
        .err()
        .expect("loaded")
        .to_string();
    assert!(error.contains("named after its id"), "{error}");
    let mut cave = battle();
    cave["id"] = json!(411);
    std::fs::remove_file(dir.join("fights/77.json")).unwrap();
    std::fs::write(dir.join("fights/411.json"), cave.to_string()).unwrap();
    let error = EncounterEngine::load(&dir, None)
        .err()
        .expect("loaded")
        .to_string();
    assert!(error.contains("encounter"), "{error}");
    let _ = std::fs::remove_dir_all(dir);
}

/// A fight file plays in the engine: the start actors stand at arrival, the wave comes on its
/// trigger, and the engine's old single-goal win does not end the run.
#[test]
fn a_fight_file_runs_in_the_engine() {
    let (dir, mut engine) = engine_with(&[battle()]);
    assert_eq!(engine.fight_ids(), [500]);
    let arrival = enter(&mut engine, 500, 1_000);
    let run = engine.fight_run().expect("a fight file's run");
    let chief = run.definition().guid(Some(0));
    assert_eq!(
        spawned(&arrival),
        [chief],
        "only the start actor stands at arrival"
    );
    engine.toggle_god();
    // One second in: wave one, two grunts.
    let mut now = 1_000;
    let mut waves = Vec::new();
    while now < 2_500 {
        now += 100;
        let out = engine.tick([0.0; 3], BRAWLER, now).unwrap();
        if !spawned(&out.replies).is_empty() {
            waves.push(spawned(&out.replies).len());
        }
    }
    assert_eq!(waves, [2]);
    // The chief falls first: the old engine would have called that the win.
    let mut facts = Vec::new();
    for _ in 0..5_000 {
        now += 700;
        let standing = engine.standing_enemies();
        let Some((guid, at, _)) = standing.iter().find(|e| e.0 == chief).or(standing.first())
        else {
            break;
        };
        let hit = engine.attack(*guid, *at, BRAWLER, 0, now).unwrap();
        let tick = engine.tick(*at, BRAWLER, now + 100).unwrap();
        facts.extend(hit.facts.into_iter().chain(tick.facts));
        assert_eq!(
            engine.run_phase(),
            Some(EncounterPhase::Fighting),
            "the door ends it, not a kill"
        );
    }
    let run = engine.fight_run().unwrap();
    assert_eq!(
        run.kills, 6,
        "the chief, two grunts, then the three of wave two"
    );
    assert!(run.exit_is_open());
    assert!(
        facts.contains(&FightFact::Goal {
            fight: 500,
            goal: "all".into(),
            done: 6,
            of: 6
        }),
        "{facts:?}"
    );
    let _ = std::fs::remove_dir_all(dir);
}

/// A run of the pure director asks for each wave once, in order.
#[test]
fn waves_fire_on_their_trigger_or_their_clock() {
    let fight = parse(&battle());
    // Clear wave one: wave two comes at once.
    let mut fast = Replay::start(&fight, 1);
    assert!(fast.step(900).is_empty(), "not before its second");
    assert_eq!(fast.step(100), [DirectorAct::Wave("w1".into())]);
    assert_eq!(fast.clear("wave:w1"), 2);
    assert_eq!(fast.waves(), ["w1", "w2"]);
    // Leave wave one standing: wave two comes five seconds after it.
    let mut slow = Replay::start(&fight, 1);
    slow.step(1_000);
    slow.step(4_900);
    assert_eq!(slow.waves(), ["w1"]);
    slow.step(100);
    assert_eq!(slow.waves(), ["w1", "w2"]);
    slow.step(60_000);
    assert_eq!(slow.waves(), ["w1", "w2"], "each wave once");
    assert_eq!(slow.living("wave:w2").len(), 3);
}

/// A `field_battles.json` row with no fight file plays as it always did: its run only reports.
#[test]
fn a_battle_row_without_a_fight_file_plays_as_before() {
    let (dir, mut engine) = engine_with(&[battle()]);
    engine
        .load_field_battles(&content().join("field_battles.json"))
        .unwrap();
    let arrival = enter(&mut engine, 96, 1_000);
    assert!(
        engine.fight_run().is_none(),
        "battle 96 has no enabled fight file"
    );
    assert_eq!(
        spawned(&arrival),
        [100_000_096_001],
        "the summoner alone, helpers asleep"
    );
    let won = engine.debug_win(2_000).unwrap();
    assert_eq!(engine.run_phase(), Some(EncounterPhase::Won));
    assert_eq!((won.rewards.len(), won.win_completed), (1, Some(96)));
    assert!(
        won.replies.iter().any(|r| r.starts_with(&[39, 0, 47])),
        "the old results card"
    );
    // The first knockout still loses such a battle.
    engine.exit().unwrap();
    enter(&mut engine, 96, 3_000);
    engine.receive_arena_damage(u32::MAX, 4_000);
    assert_eq!(engine.run_phase(), Some(EncounterPhase::Lost));
    let _ = std::fs::remove_dir_all(dir);
}
