//! The behaviour language's runtime (piece `combat-jobs`, P3 `combat-brain`): `brain::step` is
//! pure, so every trigger, condition and action is tested here without an engine or a client.
//! Each test builds a program from the same JSON an author writes and steps it against views.
use osfr_tools::encounter::{
    brain::{self, Act, Attack, Memory, Move, Place, Program, TargetRule, View, Who},
    wire::How,
};
use serde_json::{Value, json};
use std::collections::BTreeMap;

fn attacks() -> BTreeMap<String, Attack> {
    brain::parse_attacks(&json!({
        "bite": {"kind": "melee", "reach": 2.5, "interval_ms": 2300},
        "slam": {"kind": "burst", "radius": 6.0, "hit": 7.0, "windup_ms": 500}
    }))
    .unwrap()
}

fn program(behaviour: Value) -> Program {
    Program::from_json(&behaviour, &attacks()).unwrap()
}

fn refused(behaviour: Value) -> String {
    Program::from_json(&behaviour, &attacks()).unwrap_err()
}

fn rules(rules: Value) -> Program {
    program(json!({"stance": {"move": "hold", "attack": null}, "rules": rules}))
}

/// What the enemy sees; tests change one field at a time.
#[derive(Clone)]
struct Sight {
    health_pct: u32,
    target_distance: Option<f32>,
    nearest_player: Option<f32>,
    tags: BTreeMap<String, u32>,
    signals: Vec<String>,
    night: bool,
}

impl Default for Sight {
    fn default() -> Self {
        Self {
            health_pct: 100,
            target_distance: Some(5.0),
            nearest_player: Some(5.0),
            tags: BTreeMap::new(),
            signals: Vec::new(),
            night: false,
        }
    }
}

impl Sight {
    fn tag(mut self, tag: &str, alive: u32) -> Self {
        self.tags.insert(tag.into(), alive);
        self
    }
}

fn step(p: &Program, m: &mut Memory, sight: &Sight, now_ms: u64) -> Vec<Act> {
    let view = View {
        now_ms,
        health_pct: sight.health_pct,
        position: [0.0; 3],
        home: [0.0; 3],
        target: Some(1),
        target_distance: sight.target_distance,
        nearest_player: sight.nearest_player,
        tags_alive: &sight.tags,
        signals: &sight.signals,
        phase: m.phase(),
        night: sight.night,
    };
    let mut out = Vec::new();
    brain::step(p, m, &view, &mut out);
    out
}

fn signal(name: &str) -> Act {
    Act::Signal(name.into())
}

fn raise(name: &str) -> Value {
    json!([{"signal": {"name": name}}])
}

// ---- triggers ----

#[test]
fn spawned_fires_once_after_its_delay() {
    let p = rules(json!([{"when": {"spawned": {"after_ms": 2200}}, "do": raise("rush")}]));
    let mut m = Memory::default();
    let see = Sight::default();
    assert!(step(&p, &mut m, &see, 1000).is_empty(), "the clock starts at the first step");
    assert!(step(&p, &mut m, &see, 3100).is_empty());
    assert_eq!(step(&p, &mut m, &see, 3200), [signal("rush")]);
    assert!(step(&p, &mut m, &see, 9000).is_empty(), "never twice");
}

#[test]
fn engaged_fires_when_it_first_has_a_target_and_not_again() {
    let p = rules(json!([{"when": {"engaged": {}}, "do": raise("attack")}]));
    let mut m = Memory::default();
    let see = Sight::default();
    assert!(step(&p, &mut m, &see, 0).is_empty());
    m.engaged(250);
    assert!(m.is_engaged());
    assert_eq!(step(&p, &mut m, &see, 250), [signal("attack")]);
    m.engaged(500);
    assert!(step(&p, &mut m, &see, 500).is_empty(), "still the same engagement");
}

#[test]
fn hit_counts_hits_and_does_not_save_them_up_during_a_cooldown() {
    let p = rules(json!([
        {"when": {"hit": {"count": 2}}, "do": raise("slip"), "cooldown_ms": 4000}
    ]));
    let mut m = Memory::default();
    let see = Sight::default();
    m.hit();
    assert!(step(&p, &mut m, &see, 0).is_empty(), "one hit is not two");
    m.hit();
    assert_eq!(step(&p, &mut m, &see, 250), [signal("slip")]);
    // Three hits inside the cooldown are forgotten; two fresh ones are needed afterwards.
    for _ in 0..3 {
        m.hit();
    }
    assert!(step(&p, &mut m, &see, 1000).is_empty());
    assert!(step(&p, &mut m, &see, 4250).is_empty(), "nothing was saved up");
    m.hit();
    m.hit();
    assert_eq!(step(&p, &mut m, &see, 4500), [signal("slip")]);
}

#[test]
fn health_below_fires_each_time_health_crosses_down() {
    let p = rules(json!([{"when": {"health_below": {"pct": 50}}, "do": raise("low")}]));
    let mut m = Memory::default();
    let at = |health_pct| Sight {
        health_pct,
        ..Sight::default()
    };
    assert!(step(&p, &mut m, &at(50), 0).is_empty(), "50 is not below 50");
    assert_eq!(step(&p, &mut m, &at(49), 250), [signal("low")]);
    assert!(step(&p, &mut m, &at(30), 500).is_empty(), "still below: no new crossing");
    assert!(step(&p, &mut m, &at(80), 750).is_empty());
    assert_eq!(step(&p, &mut m, &at(10), 1000), [signal("low")], "healed and crossed again");
}

#[test]
fn a_resumed_enemy_keeps_quiet_about_thresholds_it_passed_before() {
    let p = rules(json!([
        {"id": "seventy", "when": {"health_below": {"pct": 70}}, "do": raise("seventy")},
        {"id": "forty", "when": {"health_below": {"pct": 40}}, "do": raise("forty")}
    ]));
    // A relog finds the enemy at 55 %: the 70 rung was climbed in the lost session.
    let mut m = Memory::resumed(7, 55);
    let at = |health_pct| Sight {
        health_pct,
        ..Sight::default()
    };
    assert!(step(&p, &mut m, &at(55), 0).is_empty());
    assert_eq!(step(&p, &mut m, &at(39), 250), [signal("forty")]);
    // A fresh enemy that is first seen already hurt does answer.
    let mut fresh = Memory::default();
    assert_eq!(step(&p, &mut fresh, &at(55), 0), [signal("seventy")]);
}

#[test]
fn every_runs_on_a_timer_only_while_engaged() {
    let p = rules(json!([
        {"when": {"every": {"ms": 1000, "first_ms": 400}}, "do": raise("tick")}
    ]));
    let mut m = Memory::default();
    let see = Sight::default();
    assert!(step(&p, &mut m, &see, 5000).is_empty(), "not engaged: no timer");
    m.engaged(10_000);
    assert!(step(&p, &mut m, &see, 10_000).is_empty());
    assert_eq!(step(&p, &mut m, &see, 10_400), [signal("tick")]);
    assert!(step(&p, &mut m, &see, 11_000).is_empty());
    assert_eq!(step(&p, &mut m, &see, 11_400), [signal("tick")]);
    // A long gap rings once, not once per missed period.
    assert_eq!(step(&p, &mut m, &see, 20_000), [signal("tick")]);
    assert!(step(&p, &mut m, &see, 20_250).is_empty());
}

#[test]
fn player_within_and_beyond_fire_on_the_crossing() {
    let p = rules(json!([
        {"id": "in", "when": {"player_within": {"m": 8}}, "do": raise("near")},
        {"id": "out", "when": {"player_beyond": {"m": 20}}, "do": raise("far")}
    ]));
    let mut m = Memory::default();
    let at = |d| Sight {
        nearest_player: d,
        ..Sight::default()
    };
    assert!(step(&p, &mut m, &at(Some(30.0)), 0).is_empty(), "starting far is not a crossing");
    assert!(step(&p, &mut m, &at(Some(10.0)), 250).is_empty());
    assert_eq!(step(&p, &mut m, &at(Some(8.0)), 500), [signal("near")]);
    assert!(step(&p, &mut m, &at(Some(3.0)), 750).is_empty());
    assert_eq!(step(&p, &mut m, &at(Some(21.0)), 1000), [signal("far")]);
    assert!(step(&p, &mut m, &at(None), 1250).is_empty(), "nobody there is neither");
    assert_eq!(step(&p, &mut m, &at(Some(2.0)), 1500), [signal("near")]);
}

#[test]
fn ally_engaged_hears_a_neighbour_in_range_with_the_tag() {
    let p = rules(json!([
        {"id": "any", "when": {"ally_engaged": {"m": 6}}, "do": raise("join")},
        {"id": "pack", "when": {"ally_engaged": {"m": 6, "tag": "wolf"}}, "do": raise("howl")}
    ]));
    let mut m = Memory::default();
    let see = Sight::default();
    m.ally_engaged(9.0, &["wolf".into()]);
    assert!(step(&p, &mut m, &see, 0).is_empty(), "too far");
    m.ally_engaged(5.0, &["bear".into()]);
    assert_eq!(step(&p, &mut m, &see, 250), [signal("join")]);
    m.ally_engaged(5.0, &["wolf".into()]);
    assert_eq!(step(&p, &mut m, &see, 500), [signal("join"), signal("howl")]);
    assert!(step(&p, &mut m, &see, 750).is_empty(), "an event is heard once");
}

#[test]
fn tag_dead_waits_for_the_group_to_have_been_there() {
    let p = rules(json!([
        {"when": {"tag_dead": {"tag": "pack", "left": 0}}, "do": raise("next")}
    ]));
    let mut m = Memory::default();
    let none = Sight::default();
    assert!(step(&p, &mut m, &none, 0).is_empty(), "a pack still asleep is not a dead pack");
    assert!(step(&p, &mut m, &none.clone().tag("pack", 2), 250).is_empty());
    assert!(step(&p, &mut m, &none.clone().tag("pack", 1), 500).is_empty());
    assert_eq!(step(&p, &mut m, &none.clone().tag("pack", 0), 750), [signal("next")]);
    assert!(step(&p, &mut m, &none, 1000).is_empty());
    // A second pack under the same tag counts again.
    assert!(step(&p, &mut m, &none.clone().tag("pack", 3), 1250).is_empty());
    assert_eq!(step(&p, &mut m, &none, 1500), [signal("next")]);
}

#[test]
fn target_down_died_talked_and_signal_are_events() {
    let p = rules(json!([
        {"id": "a", "when": {"target_down": {}}, "do": raise("gloat")},
        {"id": "b", "when": {"died": {}}, "do": raise("last words")},
        {"id": "c", "when": {"talked": {}}, "do": raise("hello")},
        {"id": "d", "when": {"signal": {"name": "heal_me"}}, "do": raise("healing")}
    ]));
    let mut m = Memory::default();
    let see = Sight::default();
    assert!(step(&p, &mut m, &see, 0).is_empty());
    m.target_down();
    assert_eq!(step(&p, &mut m, &see, 250), [signal("gloat")]);
    m.talked();
    assert_eq!(step(&p, &mut m, &see, 500), [signal("hello")]);
    let heard = Sight {
        signals: vec!["other".into(), "heal_me".into()],
        ..Sight::default()
    };
    assert_eq!(step(&p, &mut m, &heard, 750), [signal("healing")]);
    m.died();
    assert_eq!(step(&p, &mut m, &see, 1000), [signal("last words")]);
    assert!(step(&p, &mut m, &see, 1250).is_empty());
}

// ---- conditions and rule limits ----

#[test]
fn every_condition_must_hold() {
    let p = rules(json!([{
        "when": {"every": {"ms": 250}},
        "if": [{"health_below": 60}, {"health_above": 20}, {"tag_alive": {"tag": "bone", "min": 1, "max": 2}},
               {"phase": 0}, {"target_within": 10}, {"target_beyond": 2}, {"night": true}],
        "do": raise("ok")
    }]));
    let good = Sight {
        health_pct: 40,
        target_distance: Some(6.0),
        night: true,
        ..Sight::default()
    }
    .tag("bone", 2);
    let cases: Vec<(&str, Sight, bool)> = vec![
        ("all hold", good.clone(), true),
        ("health not below", Sight { health_pct: 60, ..good.clone() }, false),
        ("health not above", Sight { health_pct: 20, ..good.clone() }, false),
        ("too many alive", good.clone().tag("bone", 3), false),
        ("none alive", good.clone().tag("bone", 0), false),
        ("target too far", Sight { target_distance: Some(11.0), ..good.clone() }, false),
        ("target too near", Sight { target_distance: Some(2.0), ..good.clone() }, false),
        ("no target", Sight { target_distance: None, ..good.clone() }, false),
        ("day", Sight { night: false, ..good.clone() }, false),
    ];
    for (name, sight, fires) in cases {
        let mut m = Memory::default();
        m.engaged(0);
        assert_eq!(!step(&p, &mut m, &sight, 250).is_empty(), fires, "{name}");
    }
    // The phase is the enemy's own.
    let mut m = Memory::default();
    m.engaged(0);
    m.set_phase(1);
    assert!(step(&p, &mut m, &good, 250).is_empty(), "wrong phase");
}

#[test]
fn chance_is_a_seeded_roll() {
    let p = rules(json!([
        {"when": {"every": {"ms": 250}}, "if": [{"chance": 0.5}], "do": raise("lucky")}
    ]));
    let run = |seed| {
        let mut m = Memory::seeded(seed);
        m.engaged(0);
        (1..=200_u64)
            .map(|i| !step(&p, &mut m, &Sight::default(), i * 250).is_empty())
            .collect::<Vec<bool>>()
    };
    let first = run(42);
    assert_eq!(first, run(42), "the same seed replays the same fight");
    assert_ne!(first, run(43));
    let hits = first.iter().filter(|hit| **hit).count();
    assert!((60..=140).contains(&hits), "about half of 200, got {hits}");
    // Never and always.
    for (chance, expected) in [(0.0, 0), (1.0, 20)] {
        let p = rules(json!([
            {"when": {"every": {"ms": 250}}, "if": [{"chance": chance}], "do": raise("x")}
        ]));
        let mut m = Memory::seeded(1);
        m.engaged(0);
        let fired = (1..=20_u64)
            .filter(|i| !step(&p, &mut m, &Sight::default(), i * 250).is_empty())
            .count();
        assert_eq!(fired, expected);
    }
}

#[test]
fn once_max_and_cooldown_limit_a_rule() {
    let p = rules(json!([
        {"id": "once", "when": {"every": {"ms": 250}}, "do": raise("once"), "once": true},
        {"id": "twice", "when": {"every": {"ms": 250}}, "do": raise("twice"), "max": 2},
        {"id": "slow", "when": {"every": {"ms": 250}}, "do": raise("slow"), "cooldown_ms": 1000}
    ]));
    let mut m = Memory::default();
    m.engaged(0);
    let mut seen: BTreeMap<String, Vec<u64>> = BTreeMap::new();
    for now in (250..=2500).step_by(250) {
        for act in step(&p, &mut m, &Sight::default(), now) {
            let Act::Signal(name) = act else { panic!() };
            seen.entry(name).or_default().push(now);
        }
    }
    assert_eq!(seen["once"], [250]);
    assert_eq!(seen["twice"], [250, 500]);
    assert_eq!(seen["slow"], [250, 1250, 2250]);
}

// ---- actions ----

#[test]
fn every_action_reads_into_its_act() {
    let p = rules(json!([{"when": {"engaged": {}}, "do": [
        {"stance": {"move": "keep_away", "min": 8, "max": 14, "attack": "bite", "leash": false}},
        {"attack": {"id": "slam"}},
        {"say": {"text": "Spiders, attack!", "how": "red"}},
        {"summon": {"kind": "cave_bat", "count": 3, "at": "ring", "radius": 10, "tag": "adds", "visual": "mound"}},
        {"wake": {"tag": "pack1"}},
        {"relocate": {"to": "away", "min": 15, "max": 30, "effect": "smoke"}},
        {"heal": {"who": "boss", "pct": 15}},
        {"status": {"id": "stun", "to": "all_players", "ms": 3000}}
    ]}, {"when": {"engaged": {}}, "do": [
        {"set": {"invulnerable": true, "speed": 6.0, "disposition": 2}},
        {"phase": {"n": 2}},
        {"become": {"definition": "vengeful_vulture", "health_pct": 80}},
        {"signal": {"name": "banished"}},
        {"drop": {"pickup": "heart"}},
        "yield",
        {"leave": {"ms": 2000}},
        {"revive": {"pct": 100, "after_ms": 4000}}
    ]}, {"when": {"engaged": {}}, "do": [
        {"target": {"rule": "lowest_health"}},
        {"relocate": {"to": "point:stage2"}},
        {"heal": {"who": "tag:medics", "pct": 5}}
    ]}]));
    let mut m = Memory::default();
    m.engaged(0);
    let acts = step(&p, &mut m, &Sight::default(), 0);
    let stance = brain::Stance {
        movement: Move::KeepAway { min: 8.0, max: 14.0 },
        attack: Some("bite".into()),
        leash: false,
    };
    let expected = vec![
        Act::Stance(stance.clone()),
        Act::Attack("slam".into()),
        Act::Say(osfr_tools::encounter::wire::Line {
            text_id: None,
            text: Some("Spiders, attack!".into()),
            how: How::Red,
            text_ids: Vec::new(),
        }),
        Act::Summon {
            kind: "cave_bat".into(),
            count: 3,
            at: Place::Ring,
            radius: 10.0,
            tag: "adds".into(),
            mound: true,
        },
        Act::Wake("pack1".into()),
        Act::Relocate {
            to: Place::Away,
            min: 15.0,
            max: 30.0,
            effect: Some("smoke".into()),
        },
        Act::Heal { who: Who::Boss, pct: 15 },
        Act::Status {
            id: "stun".into(),
            to: Who::AllPlayers,
            ms: 3000,
        },
        Act::Set(brain::Flags {
            invulnerable: Some(true),
            speed: Some(6.0),
            disposition: Some(2),
            ..Default::default()
        }),
        Act::Phase(2),
        Act::Become {
            definition: "vengeful_vulture".into(),
            health_pct: 80,
        },
        Act::Signal("banished".into()),
        Act::Drop("heart".into()),
        Act::Yield,
        Act::Leave(2000),
        Act::Revive { pct: 100, after_ms: 4000 },
        Act::Target(TargetRule::LowestHealth),
        Act::Relocate {
            to: Place::Point("stage2".into()),
            min: 0.0,
            max: 0.0,
            effect: None,
        },
        Act::Heal {
            who: Who::Tag("medics".into()),
            pct: 5,
        },
    ];
    assert_eq!(acts, expected);
    // The stance and the phase are the enemy's from that step on.
    assert_eq!(m.stance(&p), &stance);
    assert_eq!(m.phase(), 2);
}

#[test]
fn every_move_reads_into_its_stance() {
    let cases = [
        (json!({"move": "hold"}), Move::Hold),
        (json!({"move": "chase"}), Move::Chase),
        (json!({"move": "flee", "ms": 4000, "speed": 3.5}), Move::Flee { ms: 4000, speed: 3.5 }),
        (json!({"move": "behind", "tag": "front", "m": 6}), Move::Behind { tag: "front".into(), m: 6.0 }),
        (json!({"move": "route", "id": "patrol"}), Move::Route("patrol".into())),
        (json!({"move": "perch", "points": "perches"}), Move::Perch("perches".into())),
        (json!({"move": "roam", "m": 4}), Move::Roam(4.0)),
        (json!({"move": "home"}), Move::Home),
    ];
    for (stance, movement) in cases {
        let p = program(json!({"stance": stance}));
        assert_eq!(p.stance.movement, movement);
        assert_eq!(p.stance.attack, None, "no attack unless one is named");
        assert!(p.stance.leash);
    }
    // No stance at all is the old default: chase and swing.
    let p = program(json!({}));
    assert_eq!((p.stance.movement.clone(), p.stance.attack.as_deref()), (Move::Chase, Some("melee")));
    assert_eq!(p.attack("melee").unwrap().interval_ms, 1000);
}

#[test]
fn wait_ms_delays_an_action_and_everything_after_it() {
    let p = rules(json!([{"when": {"engaged": {}}, "do": [
        {"signal": {"name": "now"}},
        {"signal": {"name": "later", "wait_ms": 1500}},
        {"signal": {"name": "with it"}},
        {"stance": {"move": "chase", "attack": "bite", "wait_ms": 500}}
    ]}]));
    let mut m = Memory::default();
    m.engaged(0);
    let see = Sight::default();
    assert_eq!(step(&p, &mut m, &see, 0), [signal("now")]);
    assert!(step(&p, &mut m, &see, 1250).is_empty());
    assert_eq!(step(&p, &mut m, &see, 1500), [signal("later"), signal("with it")]);
    assert_eq!(m.stance(&p).movement, Move::Hold, "the stance waits its turn");
    let acts = step(&p, &mut m, &see, 2000);
    assert!(matches!(acts.as_slice(), [Act::Stance(_)]));
    assert_eq!(m.stance(&p).movement, Move::Chase);
}

#[test]
fn a_telegraph_is_its_line_now_and_its_attack_after_the_wait() {
    let p = rules(json!([{"when": {"engaged": {}}, "do": [
        {"telegraph": {"say": {"text": "Drone Fauzz prepares for a big attack!", "how": "system"}, "ms": 3000, "attack": "slam"}}
    ]}]));
    let mut m = Memory::default();
    m.engaged(0);
    let see = Sight::default();
    let said = step(&p, &mut m, &see, 0);
    assert!(matches!(said.as_slice(), [Act::Say(line)] if line.how == How::System));
    assert!(step(&p, &mut m, &see, 2750).is_empty());
    assert_eq!(step(&p, &mut m, &see, 3000), [Act::Attack("slam".into())]);
    assert_eq!(p.attack("slam").unwrap().windup_ms, 500, "the named attack came with the program");
}

// ---- the loader's limits ----

#[test]
fn a_signal_loop_with_no_cooldown_is_refused_at_load() {
    let ping_pong = |brake: Value| {
        let mut second = json!({"id": "pong", "when": {"signal": {"name": "pong"}}, "do": raise("ping")});
        for (key, value) in brake.as_object().unwrap() {
            second[key] = value.clone();
        }
        json!({"rules": [
            {"id": "ping", "when": {"signal": {"name": "ping"}}, "do": raise("pong")},
            second
        ]})
    };
    let error = refused(ping_pong(json!({})));
    assert!(error.contains("loops back") && error.contains("ping"), "{error}");
    let alone = refused(json!({"rules": [
        {"id": "echo", "when": {"signal": {"name": "echo"}}, "do": raise("echo")}
    ]}));
    assert!(alone.contains("rule echo"), "{alone}");
    // Any brake on the way makes it legal.
    for brake in [json!({"cooldown_ms": 250}), json!({"once": true}), json!({"max": 3})] {
        assert!(Program::from_json(&ping_pong(brake.clone()), &attacks()).is_ok(), "{brake}");
    }
    // A chain that does not come back is no loop.
    program(json!({"rules": [
        {"id": "a", "when": {"signal": {"name": "a"}}, "do": raise("b")},
        {"id": "b", "when": {"signal": {"name": "b"}}, "do": raise("c")}
    ]}));
}

#[test]
fn the_loader_refuses_what_is_over_a_limit_or_unknown() {
    let rule = |when: Value, act: Value| json!({"rules": [{"when": when, "do": [act]}]});
    let engaged = json!({"engaged": {}});
    let ok = json!({"signal": {"name": "x"}});
    let many_rules: Vec<Value> = (0..33)
        .map(|i| json!({"id": format!("r{i}"), "when": {"engaged": {}}, "do": [ok.clone()]}))
        .collect();
    let many_acts: Vec<Value> = (0..9).map(|_| ok.clone()).collect();
    let cases: Vec<(&str, Value, &str)> = vec![
        ("33 rules", json!({"rules": many_rules}), "more than 32 rules"),
        ("9 actions", json!({"rules": [{"when": engaged, "do": many_acts}]}), "1..8 actions"),
        ("no actions", json!({"rules": [{"when": engaged, "do": []}]}), "1..8 actions"),
        ("fast timer", rule(json!({"every": {"ms": 100}}), ok.clone()), "must be 250"),
        ("short wait", rule(engaged.clone(), json!({"signal": {"name": "x", "wait_ms": 10}})), "must be 250"),
        ("short cooldown", json!({"rules": [{"when": engaged, "do": [ok], "cooldown_ms": 5}]}), "must be 250"),
        ("17 summons", rule(engaged.clone(), json!({"summon": {"kind": "bat", "count": 17}})), "count must be 1..16"),
        ("zero hits", rule(json!({"hit": {"count": 0}}), ok.clone()), "hit count"),
        ("percent", rule(json!({"health_below": {"pct": 101}}), ok.clone()), "1..100"),
        ("heal percent", rule(engaged.clone(), json!({"heal": {"pct": 0}})), "1..100"),
        ("unknown attack", rule(engaged.clone(), json!({"attack": {"id": "nope"}})), "unknown attack 'nope'"),
        ("unknown stance attack", json!({"stance": {"move": "chase", "attack": "nope"}}), "unknown attack 'nope'"),
        ("unknown action", rule(engaged.clone(), json!({"explode": {}})), "unknown action 'explode'"),
        ("unknown key", rule(engaged.clone(), json!({"wake": {"tag": "a", "tga": "b"}})), "unknown key 'tga'"),
        ("unknown trigger", rule(json!({"bored": {}}), ok.clone()), "unknown variant"),
        ("two triggers", rule(json!({"engaged": {}, "died": {}}), ok.clone()), "rule 1"),
        ("unknown move", json!({"stance": {"move": "dance"}}), "unknown move 'dance'"),
        ("keep_away order", json!({"stance": {"move": "keep_away", "min": 9, "max": 3}}), "min must be under max"),
        ("unknown place", rule(engaged.clone(), json!({"relocate": {"to": "moon"}})), "unknown place 'moon'"),
        ("unknown receiver", rule(engaged.clone(), json!({"heal": {"who": "everyone", "pct": 5}})), "unknown receiver"),
        ("silent say", rule(engaged.clone(), json!({"say": {"how": "red"}})), "needs text_id or text"),
        ("chance", json!({"rules": [{"when": engaged, "if": [{"chance": 1.5}], "do": [ok]}]}), "chance"),
        ("same id twice", json!({"rules": [
            {"id": "a", "when": engaged, "do": [ok]}, {"id": "a", "when": engaged, "do": [ok]}]}), "unique"),
        ("unknown top key", json!({"stanse": {}}), "unknown key 'stanse'"),
    ];
    for (name, behaviour, expected) in cases {
        let error = refused(behaviour);
        assert!(error.contains(expected), "{name}: {error}");
    }
}

#[test]
fn the_attack_table_is_checked_too() {
    let cases = [
        (json!({"melee": {"kind": "melee"}}), "built in"),
        (json!({"a": {"kind": "lob", "range": 20}}), "needs a radius"),
        (json!({"a": {"kind": "melee", "interval_ms": 100}}), "interval_ms"),
        (json!({"a": {"kind": "melee", "reach": 0}}), "reach nobody"),
        (json!({"a": {"kind": "burst", "radius": 5, "at": "moon"}}), "self or target"),
        (json!({"a": {"kind": "melee", "special": {"chance": 2.0, "hit": 1.5}}}), "special"),
        (json!({"a": {"kind": "melee", "raech": 2.0}}), "unknown field"),
    ];
    for (table, expected) in cases {
        let error = brain::parse_attacks(&table).unwrap_err();
        assert!(error.contains(expected), "{table}: {error}");
    }
    let table = attacks();
    assert_eq!(table["bite"].reach(), 2.5);
    assert_eq!(table["slam"].reach(), 6.0, "a burst reaches its radius");
}

#[test]
fn the_same_seed_and_the_same_views_give_the_same_acts() {
    let p = rules(json!([
        {"id": "a", "when": {"every": {"ms": 500}}, "if": [{"chance": 0.3}], "do": raise("a")},
        {"id": "b", "when": {"hit": {"count": 3}}, "do": [{"relocate": {"to": "away", "min": 5, "max": 9}}], "cooldown_ms": 1000},
        {"id": "c", "when": {"health_below": {"pct": 50}}, "do": [{"heal": {"who": "self", "pct": 30, "wait_ms": 750}}]}
    ]));
    let play = || {
        let mut m = Memory::seeded(96);
        m.engaged(0);
        let mut all = Vec::new();
        for i in 0..400_u64 {
            if i % 3 == 0 {
                m.hit();
            }
            let sight = Sight {
                health_pct: 100 - (i % 90) as u32,
                ..Sight::default()
            };
            all.push(step(&p, &mut m, &sight, i * 250));
        }
        (all, m)
    };
    let (first, memory) = play();
    let (second, again) = play();
    assert_eq!(first, second);
    assert_eq!(memory, again, "the memory ends the same too");
    assert!(first.iter().flatten().count() > 20, "the script did things");
}
