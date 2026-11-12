//! Fight goals (piece `combat-jobs`, P5 the director): each objective kind, the primary chain
//! and a bonus goal in a dungeon excerpt, the fight script, the shared knockout allowance and
//! the score arithmetic. The runs are replayed from facts; the knockout test also walks the
//! engine (damage, the knockout window, the client's stand-up request).
use osfr_tools::encounter::{
    EncounterPhase, FightFact,
    brain::Act,
    director::{DirectorAct, Signal, fight::GoalState},
};
use serde_json::{Value, json};

#[path = "support/fight_kit.rs"]
mod fight_kit;
use fight_kit::*;

fn fight(id: u32, extra: Value) -> Value {
    let mut fight = json!({"schema_version": 1, "id": id, "kind": "battle", "place": arena(),
                           "kinds": kinds(), "source": source()});
    fight
        .as_object_mut()
        .unwrap()
        .extend(extra.as_object().unwrap().clone());
    fight
}

fn state(replay: &Replay, goal: &str) -> (GoalState, u32, u32) {
    replay
        .run
        .goal(goal)
        .unwrap_or_else(|| panic!("no goal {goal}"))
}

/// The design's dungeon excerpt: two rooms, a chain of three goals in order, a bonus goal.
#[test]
fn a_dungeon_chain_completes_in_order_with_its_bonus_goal() {
    let dungeon = parse(&json!({
        "schema_version": 1, "id": 601, "kind": "dungeon",
        "place": {"zone": 9611, "arena": "sg_bixie_hive", "arrive": [0, 0, 0], "exit_door": [75, 0, 0],
                  "rooms": [{"id": "gate", "box": [-20, -5, -20, 20, 10, 20]},
                            {"id": "hall", "box": [20, -5, -20, 80, 10, 20]}]},
        "kinds": kinds(),
        "actors": {"g1": {"definition": "grunt", "at": [30, 0, 4], "room": "hall", "tags": ["guards"]},
                   "g2": {"definition": "grunt", "at": [32, 0, 4], "room": "hall", "tags": ["guards"]},
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
        "script": [{"id": "hello", "when": {"room_entered": {"id": "hall"}}, "once": true,
                    "do": [{"say": {"text": "Intruders!", "how": "red"}, "on": "guards"}]},
                   {"id": "boss_in", "when": {"objective_done": {"id": "waves"}}, "do": [{"wake": {"tag": "actor:fauzz"}}]}],
        "source": source()
    }));
    let mut run = Replay::start(&dungeon, 1);
    assert_eq!(state(&run, "guards"), (GoalState::Active, 0, 2));
    assert_eq!(
        state(&run, "waves").0,
        GoalState::Waiting,
        "one primary goal at a time"
    );
    assert_eq!(
        state(&run, "free").0,
        GoalState::Active,
        "a bonus goal is open from the start"
    );

    // Walking into the hall makes both guards shout, once.
    let (g1, g2, cocoon, boss) = (
        run.guid("g1"),
        run.guid("g2"),
        run.guid("cocoon1"),
        run.guid("fauzz"),
    );
    let shout = run.fact(Signal::Reached {
        place: "hall".into(),
    });
    let said = |guid| DirectorAct::Actor {
        guid,
        act: Act::Say(osfr_tools::encounter::wire::Line {
            text_id: None,
            text: Some("Intruders!".into()),
            how: osfr_tools::encounter::wire::How::Red,
            text_ids: Vec::new(),
        }),
    };
    assert_eq!(shout, [said(g1), said(g2)]);
    run.fact(Signal::Reached {
        place: "left:hall".into(),
    });
    assert!(
        run.fact(Signal::Reached {
            place: "hall".into()
        })
        .is_empty(),
        "once"
    );

    // The captive cannot be freed while a guard lives.
    run.talk(cocoon);
    assert_eq!(state(&run, "free"), (GoalState::Active, 0, 1));
    run.kill(g1);
    assert_eq!(state(&run, "guards"), (GoalState::Active, 1, 2));
    assert!(run.waves().is_empty());
    run.kill(g2);
    assert_eq!(state(&run, "guards").0, GoalState::Done);
    assert_eq!(state(&run, "waves"), (GoalState::Active, 0, 6));
    assert_eq!(run.waves(), ["m1"], "the first goal brings the first wave");
    let freed = run.talk(cocoon);
    assert!(
        freed.contains(&DirectorAct::Fact(FightFact::Freed {
            kind: 7005,
            guid: cocoon
        })),
        "{freed:?}"
    );
    assert_eq!(state(&run, "free").0, GoalState::Done);

    assert_eq!(run.clear("wave:m1"), 3);
    assert_eq!(run.waves(), ["m1", "m2"]);
    assert_eq!(
        run.run.life(boss),
        Some(osfr_tools::encounter::director::Life::Waiting)
    );
    assert_eq!(run.clear("wave:m2"), 3);
    assert_eq!(state(&run, "waves").0, GoalState::Done);
    assert_eq!(
        run.living("actor:fauzz"),
        [boss],
        "the second goal wakes the boss"
    );
    assert!(!run.run.exit_is_open());
    run.kill(boss);
    assert_eq!(state(&run, "boss").0, GoalState::Done);
    assert_eq!(
        run.count(&DirectorAct::OpenExit),
        1,
        "the door opens at the end of the chain"
    );
    assert_eq!(run.run.outcome(), None, "and the fight ends at the door");
    assert_eq!(run.leave(), [DirectorAct::Win]);

    // The goals were reported in order, each finishing before the next began.
    let done: Vec<String> = run
        .goal_facts()
        .into_iter()
        .filter(|(_, done, of)| done == of)
        .map(|(goal, _, _)| goal)
        .collect();
    assert_eq!(done, ["guards", "free", "waves", "boss"]);
    let card = run.run.score(run.now);
    assert_eq!((card.goals, card.bonus_goals, card.enemies), (3, 1, 9));
}

#[test]
fn destroy_collect_reach_and_survive_goals() {
    let fight = parse(&fight(
        610,
        json!({
            "actors": {"c1": {"definition": "crate", "at": [5, 0, 5], "tags": ["crates"]},
                       "c2": {"definition": "crate", "at": [6, 0, 5], "tags": ["crates"]},
                       "c3": {"definition": "crate", "at": [7, 0, 5], "tags": ["crates"]}},
            "points": {"shrine": [[40, 0, 40]]},
            "objectives": [
                {"id": "smash", "kind": "destroy", "target": {"tag": "crates"}, "count": 2},
                {"id": "keys", "kind": "collect", "target": {"pickup": "key"}, "count": 2},
                {"id": "go", "kind": "reach", "target": {"point": "shrine"}, "radius": 4},
                {"id": "hold", "kind": "survive", "ms": 10000}
            ]
        }),
    ));
    let mut run = Replay::start(&fight, 1);
    // Reaching the shrine early does not finish a goal that is not open yet.
    run.fact(Signal::Reached {
        place: "shrine".into(),
    });
    run.fact(Signal::Reached {
        place: "left:shrine".into(),
    });
    assert_eq!(state(&run, "go").0, GoalState::Waiting);
    run.kill(run.guid("c1"));
    run.kill(run.guid("c3"));
    assert_eq!(
        state(&run, "smash"),
        (GoalState::Done, 2, 2),
        "two of the three crates is enough"
    );
    run.fact(Signal::Collected {
        pickup: "coin".into(),
    });
    run.fact(Signal::Collected {
        pickup: "key".into(),
    });
    assert_eq!(state(&run, "keys"), (GoalState::Active, 1, 2));
    run.fact(Signal::Collected {
        pickup: "key".into(),
    });
    assert_eq!(state(&run, "go"), (GoalState::Active, 0, 1));
    run.fact(Signal::Reached {
        place: "shrine".into(),
    });
    assert_eq!(state(&run, "go").0, GoalState::Done);
    assert_eq!(state(&run, "hold"), (GoalState::Active, 0, 1));
    run.step(9_900);
    assert_eq!(
        state(&run, "hold").0,
        GoalState::Active,
        "ten seconds from when it opened"
    );
    assert!(run.step(100).contains(&DirectorAct::OpenExit));
}

fn camp() -> Value {
    json!({
        "schema_version": 1, "id": 604, "kind": "quest_fight",
        "place": {"open_world": true, "centre": [-640, 0, 2280], "radius": 30},
        "kinds": kinds(), "points": {"ring": [[-620, 0, 2280], [-660, 0, 2280]]},
        "actors": {"camp": {"definition": "tent", "at": [-640, 0, 2280], "tags": ["ward"]}},
        "waves": [{"id": "a", "when": {"fight_started": {"after_ms": 3000}}, "repeat": {"times": 5, "every_ms": 12000},
                   "spawn": [{"definition": "grunt", "count": 4, "at": "ring"}]}],
        "objectives": [{"id": "hold", "kind": "defend", "target": {"actor": "camp"},
                        "until": {"defeated": {"definition": "grunt", "count": 20}},
                        "pressure": {"within": 6.0, "damage": 5, "every_ms": 2300}}],
        "source": source()
    })
}

/// "Defend the camp against 20 Tarantula Ambushers": won at the twentieth kill.
#[test]
fn a_defend_fight_wins_at_twenty_kills() {
    let mut run = Replay::start(&parse(&camp()), 1);
    let mut kills = 0;
    for wave in 0..5 {
        run.step(if wave == 0 { 3_000 } else { 12_000 });
        assert_eq!(
            run.waves().len(),
            wave + 1,
            "a wave of four every twelve seconds"
        );
        kills += run.clear("wave:a");
        assert_eq!(state(&run, "hold").1, kills as u32);
        assert_eq!(run.run.outcome(), (kills == 20).then_some(true));
    }
    assert_eq!(kills, 20);
    // The open world has no door: the last goal is the win.
    assert_eq!(run.count(&DirectorAct::Win), 1);
    assert_eq!(run.count(&DirectorAct::OpenExit), 0);
    assert_eq!(run.goal_facts().last(), Some(&("hold".to_owned(), 20, 20)));
}

/// The same fight is lost the moment the ward falls.
#[test]
fn a_defend_fight_fails_when_the_ward_dies() {
    let mut run = Replay::start(&parse(&camp()), 1);
    run.step(3_000);
    assert_eq!(run.clear("wave:a"), 4);
    let lost = run.kill(run.guid("camp"));
    assert!(lost.contains(&DirectorAct::Lose), "{lost:?}");
    assert_eq!(state(&run, "hold").0, GoalState::Failed);
    assert_eq!(run.run.outcome(), Some(false));
    // Nothing more happens in a lost run.
    assert!(run.step(60_000).is_empty());
    assert_eq!(run.waves().len(), 1);
    assert_eq!(run.run.kills, 4, "the ward is not an enemy defeated");
}

#[test]
fn an_escort_arrives_or_is_lost() {
    let escort = parse(&fight(
        611,
        json!({
            "points": {"road": [[0, 0, 10], [0, 0, 40]]},
            "actors": {"vip": {"definition": "captive", "at": [0, 0, 0]},
                       "thug": {"definition": "grunt", "at": [0, 0, 30]}},
            "objectives": [{"id": "walk", "kind": "escort", "target": {"actor": "vip"}, "route": "road", "speed": 2.0}]
        }),
    ));
    let mut safe = Replay::start(&escort, 1);
    safe.fact(Signal::Named("arrived:vip".into()));
    assert_eq!(state(&safe, "walk").0, GoalState::Done);
    assert!(safe.run.exit_is_open());
    let mut lost = Replay::start(&escort, 1);
    assert!(lost.kill(lost.guid("vip")).contains(&DirectorAct::Lose));
}

/// Signals, phases, conditions, waits and the fight-only actions.
#[test]
fn the_fight_script_runs_in_the_behaviour_language() {
    let scripted = parse(&fight(
        612,
        json!({
            "actors": {"boss": {"definition": "chief", "at": [0, 0, 10]},
                       "add1": {"definition": "grunt", "at": [3, 0, 10], "dormant": true, "tags": ["adds"]},
                       "add2": {"definition": "grunt", "at": [-3, 0, 10], "dormant": true, "tags": ["adds"]}},
            "points": {"ring": [[9, 0, 9]]},
            "waves": [{"id": "late", "spawn": [{"definition": "mage", "count": 2, "at": "ring"}]}],
            "objectives": [{"id": "boss", "kind": "defeat", "target": {"actor": "boss"}},
                           {"id": "style", "kind": "collect", "target": {"pickup": "never"}, "count": 3, "bonus": true}],
            "script": [
                {"id": "half", "when": {"health_below": {"tag": "actor:boss", "pct": 50}}, "once": true,
                 "do": [{"say": {"text_id": 4242}, "on": "actor:boss"}, {"phase": {"n": 2}},
                        {"wake": {"tag": "adds"}, "wait_ms": 1500}, {"signal": {"name": "enraged"}}]},
                {"id": "more", "when": {"signal": {"name": "enraged"}}, "if": [{"phase": 2}, {"tag_alive": {"tag": "adds", "min": 2}}],
                 "do": [{"spawn_wave": {"id": "late"}}, {"credit": {"id": "style", "n": 3}}]},
                {"id": "pulse", "when": {"every": {"ms": 1000, "first_ms": 500}}, "max": 2, "do": [{"signal": {"name": "pulse"}}]},
                {"id": "pity", "when": {"tag_dead": {"tag": "adds", "left": 0}}, "do": [{"open_exit": {}}]}
            ]
        }),
    ));
    let mut run = Replay::start(&scripted, 1);
    let boss = run.guid("boss");
    run.step(3_000);
    assert_eq!(
        run.count(&DirectorAct::Signal("pulse".into())),
        2,
        "`max` stops the timer rule"
    );
    run.run.set_health(boss, 60);
    assert!(run.step(100).is_empty());
    run.run.set_health(boss, 40);
    let at_half = run.step(100);
    assert_eq!(
        at_half.len(),
        1,
        "the line now, the rest after the wait: {at_half:?}"
    );
    assert!(
        matches!(&at_half[0], DirectorAct::Actor { guid, act: Act::Say(line) } if *guid == boss && line.text_id == Some(4242))
    );
    assert!(run.living("adds").is_empty());
    run.step(1_400);
    assert!(
        run.living("adds").is_empty(),
        "the wake waits a second and a half"
    );
    let woke = run.step(100);
    assert_eq!(run.living("adds").len(), 2);
    assert!(woke.contains(&DirectorAct::Signal("enraged".into())));
    assert!(
        woke.contains(&DirectorAct::Wave("late".into())),
        "phase 2 with both adds up: {woke:?}"
    );
    assert_eq!(
        state(&run, "style").0,
        GoalState::Done,
        "`credit` finished the bonus goal"
    );
    // The health trigger fired once, though the boss stays under half.
    run.run.set_health(boss, 10);
    run.step(5_000);
    assert_eq!(run.waves(), ["late"]);
    // Both adds down opens the exit although the boss lives; the door then wins the fight.
    run.clear("adds");
    assert!(run.run.exit_is_open());
    assert_eq!(state(&run, "boss").0, GoalState::Active);
    assert_eq!(run.leave(), [DirectorAct::Win]);
}

#[test]
fn the_script_can_fail_win_and_set_goals_and_a_clock_can_run_out() {
    let timed = parse(&fight(
        613,
        json!({
            "rules": {"time_limit_s": 60},
            "actors": {"boss": {"definition": "chief", "at": [0, 0, 10]}},
            "objectives": [{"id": "boss", "kind": "defeat", "target": {"actor": "boss"}}],
            "script": [{"id": "mercy", "when": {"signal": {"name": "spared"}}, "do": [{"set_objective": {"id": "boss", "state": "done"}}]},
                       {"id": "flee", "when": {"signal": {"name": "fled"}}, "do": [{"fail": {}}]},
                       {"id": "gift", "when": {"signal": {"name": "gift"}}, "do": [{"win": {}}]}]
        }),
    ));
    let mut spared = Replay::start(&timed, 1);
    spared.fact(Signal::Named("spared".into()));
    assert_eq!(state(&spared, "boss"), (GoalState::Done, 1, 1));
    assert!(spared.run.exit_is_open());
    let mut fled = Replay::start(&timed, 1);
    assert_eq!(fled.fact(Signal::Named("fled".into())), [DirectorAct::Lose]);
    let mut gift = Replay::start(&timed, 1);
    assert_eq!(gift.fact(Signal::Named("gift".into())), [DirectorAct::Win]);
    let mut slow = Replay::start(&timed, 1);
    assert!(slow.step(59_900).is_empty());
    assert_eq!(
        slow.step(100),
        [DirectorAct::Lose],
        "the fight's clock ran out"
    );
}

/// Ten knockouts are allowed in a battle and shared by the group; the eleventh loses it.
#[test]
fn the_eleventh_knockout_loses_a_battle() {
    let battle = parse(&fight(
        620,
        json!({
            "actors": {"boss": {"definition": "chief", "at": [0, 0, 10]}},
            "objectives": [{"id": "boss", "kind": "defeat", "target": {"actor": "boss"}}]
        }),
    ));
    let mut run = Replay::start(&battle, 2);
    for taken in 1..=10 {
        // Either member's knockout comes off the same allowance.
        let fact = Signal::PlayerDown {
            guid: 1 + taken % 2,
        };
        assert!(run.fact(fact).is_empty(), "knockout {taken} is allowed");
        assert_eq!(run.run.knockouts_left(), 10 - taken as u32);
    }
    assert_eq!(run.knock_out(), [DirectorAct::Lose]);
    assert_eq!(run.run.outcome(), Some(false));
    // A dungeon allows fifteen, a file may say otherwise, a duel none.
    let mut three = fight(
        621,
        json!({
            "rules": {"knockouts": 3},
            "actors": {"boss": {"definition": "chief", "at": [0, 0, 10]}},
            "objectives": [{"id": "boss", "kind": "defeat", "target": {"actor": "boss"}}]
        }),
    );
    assert_eq!(parse(&three).knockouts(), 3);
    three["rules"] = json!({});
    three["kind"] = json!("dungeon");
    assert_eq!(parse(&three).knockouts(), 15);
}

/// The same limit through the engine: damage knocks the player out, the window opens, the
/// client's request stands the player up where they fell, and the eleventh ends the run.
#[test]
fn knockouts_in_the_engine_stand_the_player_up_until_the_allowance_is_gone() {
    let (dir, mut engine) = engine_with(&[fight(
        620,
        json!({
            "actors": {"boss": {"definition": "chief", "at": [0, 0, 10]}},
            "objectives": [{"id": "boss", "kind": "defeat", "text_id": 14, "target": {"actor": "boss"}}]
        }),
    )]);
    enter(&mut engine, 620, 1_000);
    let mut now = 1_000;
    for taken in 1..=10_u32 {
        now += 1_000;
        let down = engine.receive_arena_damage(u32::MAX, now);
        assert_eq!(
            engine.run_phase(),
            Some(EncounterPhase::Fighting),
            "knockout {taken}"
        );
        assert_eq!(engine.player_health().0, 0);
        let window = down
            .replies
            .iter()
            .find(|r| r.starts_with(&[41, 0, 125, 0]))
            .expect("the knockout window");
        assert_eq!(
            window[12..16],
            10_000_i32.to_le_bytes(),
            "ten seconds to recover"
        );
        assert_eq!(
            window[16..20],
            0_i32.to_le_bytes(),
            "and no price inside a fight"
        );
        assert_eq!(engine.fight_run().unwrap().knockouts_left(), 10 - taken);
        // The fight goes on around a player who is down.
        assert!(down.replies.iter().all(|r| !r.starts_with(&[39, 0, 18])));
        let up = engine
            .handle(&stand_up(), 1, [0.0; 3], BRAWLER, now + 500)
            .unwrap();
        assert_eq!(
            engine.player_health().0,
            engine.player_health().1,
            "full health"
        );
        assert!(
            up.replies.iter().any(|r| r.starts_with(&[41, 0, 126, 0])),
            "the window closes"
        );
        assert!(up.zoning.is_none(), "where they fell, not at the safe spot");
    }
    let last = engine.receive_arena_damage(u32::MAX, now + 2_000);
    assert_eq!(engine.run_phase(), Some(EncounterPhase::Lost));
    assert!(last.facts.contains(&FightFact::Lost { fight: 620 }));
    assert!(
        last.replies.iter().any(|r| r.starts_with(&[39, 0, 18])),
        "game over"
    );
    let rows = score_rows(&last.replies);
    assert_eq!(
        rows["scorePlayerKnockouts"],
        (0, 10, 0),
        "none left of ten, and no points"
    );
    assert_eq!(rows["scoreTotalScore"].2, 0);
    let _ = std::fs::remove_dir_all(dir);
}

/// The score is built from the fight's own values (packet 62's names).
#[test]
fn the_score_is_built_from_the_fights_values() {
    let scored = parse(&fight(
        630,
        json!({
            "rules": {"score": {"per_enemy": 300, "per_knockout_left": 25000, "knockout_penalty": 5000,
                                "quick_value": 100000, "quick_loss_per_s": 500, "quick_time_s": 1}},
            "actors": {"a": {"definition": "grunt", "at": [0, 0, 10]}, "b": {"definition": "grunt", "at": [1, 0, 10]},
                       "c": {"definition": "grunt", "at": [2, 0, 10]}},
            "objectives": [{"id": "all", "kind": "defeat", "target": {"all": true}},
                           {"id": "keys", "kind": "collect", "target": {"pickup": "key"}, "bonus": true}]
        }),
    ));
    let mut run = Replay::start(&scored, 1);
    run.step(21_000);
    run.knock_out();
    run.knock_out();
    run.clear("kind:grunt");
    run.leave();
    let card = run.run.score(run.now + 99_000);
    assert_eq!(card.enemy_points, 3 * 300);
    assert_eq!(
        card.time_bonus,
        100_000 - 20 * 500,
        "500 a second after the first, frozen at the win"
    );
    assert_eq!((card.knockouts_left, card.knockouts_allowed), (8, 10));
    assert_eq!(card.knockout_points, 25_000 - 2 * 5_000);
    assert_eq!(
        (
            card.goals,
            card.goal_points,
            card.bonus_goals,
            card.bonus_points
        ),
        (1, 5_000, 0, 0)
    );
    assert_eq!(card.total, 900 + 90_000 + 15_000 + 5_000);
    // Five knockouts use the whole reward up; more cost nothing further.
    let mut clumsy = Replay::start(&scored, 1);
    for _ in 0..7 {
        clumsy.knock_out();
    }
    clumsy.clear("kind:grunt");
    clumsy.leave();
    assert_eq!(clumsy.run.score(clumsy.now).knockout_points, 0);
    // A lost run keeps its kills and earns no bonus.
    let mut lost = Replay::start(&scored, 1);
    lost.kill(lost.guid("a"));
    for _ in 0..11 {
        lost.knock_out();
    }
    let card = lost.run.score(lost.now);
    assert_eq!(
        (
            card.enemy_points,
            card.time_bonus,
            card.knockout_points,
            card.total
        ),
        (300, 0, 0, 300)
    );
}

/// A group changes what an enemy is worth in health; the roster and goals stay the same.
#[test]
fn a_run_remembers_its_party_size() {
    let battle = parse(&fight(
        640,
        json!({
            "actors": {"boss": {"definition": "chief", "at": [0, 0, 10]}},
            "objectives": [{"id": "boss", "kind": "defeat", "target": {"actor": "boss"}}]
        }),
    ));
    assert_eq!(Replay::start(&battle, 1).run.group, 1);
    assert_eq!(Replay::start(&battle, 3).run.group, 3);
    assert_eq!(battle.group_max(), 4);
    let small = parse(&fight(
        641,
        json!({
            "rules": {"group_max": 2},
            "actors": {"boss": {"definition": "chief", "at": [0, 0, 10]}},
            "objectives": [{"id": "boss", "kind": "defeat", "target": {"actor": "boss"}}]
        }),
    ));
    assert_eq!(small.group_max(), 2);
}

/// A relog resumes the run: what was done stays done, and nothing already answered fires again.
#[test]
fn a_saved_run_resumes_where_it_stopped() {
    use osfr_tools::encounter::director::{Run, run::Saved};
    let battle = parse(&fight(
        650,
        json!({
            "points": {"ring": [[9, 0, 9]]},
            "actors": {"boss": {"definition": "chief", "at": [0, 0, 10]}},
            "waves": [{"id": "w1", "when": {"fight_started": {"after_ms": 1000}},
                       "spawn": [{"definition": "grunt", "count": 3, "at": "ring"}]},
                      {"id": "w2", "when": {"wave_left": {"wave": "w1", "left": 0}},
                       "spawn": [{"definition": "grunt", "count": 2, "at": "ring"}]}],
            "objectives": [{"id": "all", "kind": "defeat", "target": {"all": true}}],
            "script": [{"id": "hello", "when": {"clock": {"ms": 500}}, "once": true, "do": [{"signal": {"name": "hello"}}]}]
        }),
    ));
    let mut before = Replay::start(&battle, 1);
    before.step(1_000);
    before.knock_out();
    let pack = before.living("wave:w1");
    before.kill(pack[0]);
    before.kill(pack[1]);
    assert_eq!(before.count(&DirectorAct::Signal("hello".into())), 1);
    // The save goes through JSON, as it does in the character file.
    let text = serde_json::to_string(&before.run.save()).unwrap();
    let saved: Saved = serde_json::from_str(&text).unwrap();
    let run = Run::restore(&battle, &saved, 900_000).expect("the save fits its fight");
    let mut after = Replay {
        run,
        now: 900_000,
        acts: Vec::new(),
        plain: Vec::new(),
    };
    assert!(
        after.step(100).is_empty(),
        "nothing already answered fires again"
    );
    assert_eq!(after.run.goal("all"), Some((GoalState::Active, 2, 6)));
    assert_eq!((after.run.kills, after.run.knockouts_left()), (2, 9));
    assert_eq!(after.living("wave:w1"), [pack[2]]);
    after.kill(pack[2]);
    assert_eq!(
        after.waves(),
        ["w2"],
        "the next wave still follows the last wolf"
    );
    // A save of another fight, or of a file that changed shape, is dropped.
    let other = parse(&fight(
        651,
        json!({
            "actors": {"boss": {"definition": "chief", "at": [0, 0, 10]}},
            "objectives": [{"id": "all", "kind": "defeat", "target": {"all": true}}]
        }),
    ));
    assert!(Run::restore(&other, &saved, 0).is_none());
    assert!(Run::restore(&battle, &Saved::default(), 0).is_none());
}
