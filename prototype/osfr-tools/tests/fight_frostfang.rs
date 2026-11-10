//! The Frostfang Growlers wave battle rebuilt from the March 2014 capture (p12.pcap
//! t=546232..546321) as `data/content/fights/12365.json`: one Howler on arrival, waves of 6, 9,
//! 10 and 11 that come when one wolf of the wave before is left or 16 s after it, the Alpha's
//! boss bar, the exit door at the last kill, and a score built from 300 / 25000 / 5000.
//!
//! The first tests replay the shipped file from facts; the last one plays it in the engine with
//! real attacks, ticks and the click on the door.
use osfr_tools::encounter::{
    EncounterPhase, FightFact,
    director::{DirectorAct, fight::GoalState},
};
use serde_json::Value;

#[path = "support/fight_kit.rs"]
mod fight_kit;
use fight_kit::*;

const FROSTFANG: u32 = 12365;

fn shipped() -> Value {
    serde_json::from_slice(&std::fs::read(content().join("fights/12365.json")).unwrap()).unwrap()
}

#[test]
fn the_roster_is_the_captured_pack() {
    let fight = parse(&shipped());
    assert_eq!(fight.roster.len(), 37, "the capture's 37 wolves");
    let count = |tag: &str| {
        fight
            .roster
            .iter()
            .filter(|slot| slot.tags.iter().any(|t| t == tag))
            .count()
    };
    assert_eq!(
        [
            count("wave:w1"),
            count("wave:w2"),
            count("wave:w3"),
            count("wave:w4")
        ],
        [6, 9, 10, 11]
    );
    assert_eq!(
        (
            count("kind:frostfang_growler"),
            count("kind:frostfang_prowler")
        ),
        (27, 8)
    );
    assert_eq!(
        (
            count("kind:frostfang_howler"),
            count("kind:frostfang_alpha"),
            count("boss")
        ),
        (1, 1, 1)
    );
    let standing: Vec<&str> = fight
        .roster
        .iter()
        .filter(|slot| slot.start)
        .map(|slot| slot.key.as_str())
        .collect();
    assert_eq!(standing, ["howler"], "only the Howler stands on arrival");
    // Every wolf appears 27 to 75 m from where the player arrives, as captured.
    for slot in fight.roster.iter().filter(|slot| slot.wave.is_some()) {
        let (dx, dz) = (
            slot.spot[0] - fight.place.arrive[0],
            slot.spot[2] - fight.place.arrive[2],
        );
        let far = (dx * dx + dz * dz).sqrt();
        assert!((27.0..=75.0).contains(&far), "{} at {far} m", slot.key);
    }
    assert_eq!(fight.knockouts(), 10);
    let score = &fight.rules.score;
    assert_eq!(
        (
            score.per_enemy,
            score.per_knockout_left,
            score.knockout_penalty
        ),
        (300, 25_000, 5_000)
    );
    assert_eq!(
        (
            score.quick_value,
            score.quick_loss_per_s,
            score.quick_time_s
        ),
        (100_000, 500, 1)
    );
}

/// Left alone, the waves come on the clock: 7.4 s after arrival, then 16, 16 and 17 s apart.
#[test]
fn the_waves_come_on_the_clock_when_the_pack_is_left_standing() {
    let mut run = Replay::start(&parse(&shipped()), 1);
    let mut at = Vec::new();
    for _ in 0..700 {
        let elapsed = run.now + 100 - 1_000;
        for act in run.step(100) {
            if let DirectorAct::Wave(id) = act {
                at.push((id, elapsed));
            }
        }
    }
    let expected = [
        ("w1", 7_400),
        ("w2", 23_400),
        ("w3", 39_400),
        ("w4", 56_400),
    ];
    assert_eq!(at, expected.map(|(id, ms)| (id.to_owned(), ms)));
    assert_eq!(
        run.living("kind:frostfang_alpha").len(),
        1,
        "the Alpha comes with the last wave"
    );
}

/// Fought well, each wave comes as soon as one wolf of the wave before is left.
#[test]
fn the_next_wave_comes_when_one_wolf_is_left() {
    let mut run = Replay::start(&parse(&shipped()), 1);
    run.step(7_400);
    assert_eq!(run.waves(), ["w1"]);
    for (wave, next, size) in [("w1", "w2", 6), ("w2", "w3", 9), ("w3", "w4", 10)] {
        let pack = run.living(&format!("wave:{wave}"));
        assert_eq!(pack.len(), size);
        for guid in &pack[..size - 2] {
            run.kill(*guid);
        }
        run.step(1_000);
        assert_eq!(
            run.waves().last().map(String::as_str),
            Some(wave),
            "two left: not yet"
        );
        run.kill(pack[size - 2]);
        assert_eq!(
            run.waves().last().map(String::as_str),
            Some(next),
            "one left: the next wave"
        );
    }
    // The goal counts every wolf, and the door waits for the last of them.
    assert_eq!(run.run.goal("wolves"), Some((GoalState::Active, 22, 37)));
    run.clear("kind:frostfang_growler");
    run.clear("kind:frostfang_prowler");
    run.clear("kind:frostfang_howler");
    assert!(!run.run.exit_is_open(), "the Alpha still stands");
    run.clear("boss");
    assert_eq!(run.run.goal("wolves"), Some((GoalState::Done, 37, 37)));
    assert_eq!(run.count(&DirectorAct::OpenExit), 1);
    assert_eq!(run.leave(), [DirectorAct::Win]);
    let card = run.run.score(run.now);
    assert_eq!((card.enemies, card.enemy_points), (37, 11_100));
    assert_eq!(card.knockout_points, 25_000);
}

/// The whole battle in the engine: arrive, fight the four waves with real attacks, see the
/// Alpha's boss bar, find the exit door at the last kill and end the battle by clicking it.
#[test]
fn the_battle_plays_in_the_engine_to_the_door_and_the_score() {
    let (dir, mut engine) = engine_with(&[shipped()]);
    let arrival = enter(&mut engine, FROSTFANG, 1_000);
    let fight = engine
        .fight_run()
        .expect("the fight file's run")
        .definition()
        .clone();
    let slot = |key: &str| {
        let i = fight
            .roster
            .iter()
            .position(|slot| slot.key == key)
            .unwrap();
        fight.guid(Some(i))
    };
    let (howler, alpha, door) = (slot("howler"), slot("alpha"), fight.guid(None));
    assert_eq!(spawned(&arrival), [howler]);
    assert!(
        arrival.iter().any(|r| r.starts_with(&[45, 0, 1])
            && r[3..7] == 104_177_i32.to_le_bytes()
            && r[7..11] == 37_i32.to_le_bytes()),
        "the goal row: Defeat all of the Frostfang Growlers, 37"
    );
    engine.toggle_god();

    let mut now = 1_000;
    let mut waves: Vec<(u64, Vec<u64>)> = Vec::new();
    let mut replies: Vec<Vec<u8>> = Vec::new();
    let mut facts = Vec::new();
    let mut door_at = None;
    for _ in 0..20_000 {
        now += 700;
        let standing = engine.standing_enemies();
        let at = standing.first().map_or([136.0, -0.5, 155.0], |e| e.1);
        let mut step = Vec::new();
        if let Some((guid, _, _)) = standing.first() {
            let hit = engine.attack(*guid, at, BRAWLER, 0, now).unwrap();
            step.extend(hit.replies);
            facts.extend(hit.facts);
        }
        // Stand clear of the door while fighting: walking into it also leaves.
        let tick = engine
            .tick([136.0, -0.5, 150.0], BRAWLER, now + 100)
            .unwrap();
        step.extend(tick.replies);
        facts.extend(tick.facts);
        let new: Vec<u64> = spawned(&step)
            .into_iter()
            .filter(|guid| *guid != door)
            .collect();
        if !new.is_empty() {
            waves.push((now, new));
        }
        if door_at.is_none() && spawned(&step).contains(&door) {
            door_at = Some(
                replies.len()
                    + step
                        .iter()
                        .position(|r| {
                            r.starts_with(&[35, 0, 2, 0]) && r[4..12] == door.to_le_bytes()
                        })
                        .unwrap(),
            );
        }
        replies.extend(step);
        assert_eq!(
            engine.run_phase(),
            Some(EncounterPhase::Fighting),
            "a kill never ends this battle"
        );
        if door_at.is_some() {
            break;
        }
    }
    let sizes: Vec<usize> = waves.iter().map(|(_, guids)| guids.len()).collect();
    assert_eq!(sizes, [6, 9, 10, 11], "the captured waves");
    assert!(
        waves[0].0 - 1_000 >= 7_400 && waves[0].0 - 1_000 <= 8_200,
        "the first wave 7.4 s in"
    );
    for pair in waves.windows(2) {
        assert!(
            pair[1].0 - pair[0].0 <= 17_800,
            "at the latest 16 to 17 s after the wave before"
        );
    }
    assert!(waves[3].1.contains(&alpha));
    let boss_bar = |on: u8| {
        let mut packet = vec![32, 0, 9, 0];
        packet.extend(alpha.to_le_bytes());
        packet.push(on);
        packet
    };
    assert!(
        replies.contains(&boss_bar(1)),
        "the Alpha's boss bar comes with it"
    );
    assert!(replies.contains(&boss_bar(0)), "and goes when it falls");

    // The door: the captured Exit actor, added and made friendly at once, after the last kill.
    let door_at = door_at.expect("the exit door");
    let add = &replies[door_at];
    assert_eq!(add[12..16], 4826_i32.to_le_bytes(), "name id 4826, Exit");
    assert_eq!(add[16..20], 846_i32.to_le_bytes(), "model 846");
    let mut friendly = vec![35, 0, 28, 0];
    friendly.extend(door.to_le_bytes());
    friendly.extend(1_i32.to_le_bytes());
    assert_eq!(replies[door_at + 1], friendly, "SetDisposition 1");
    let run = engine.fight_run().unwrap();
    assert_eq!((run.kills, run.exit_is_open()), (37, true));
    assert!(facts.contains(&FightFact::Goal {
        fight: FROSTFANG,
        goal: "wolves".into(),
        done: 37,
        of: 37
    }));
    assert!(
        replies
            .iter()
            .any(|r| r.starts_with(&[45, 0, 3]) && r[3..7] == 104_177_i32.to_le_bytes()),
        "Goal Complete"
    );
    assert!(engine.standing_enemies().is_empty());

    // Clicking the door ends the battle: results, the score, the pay.
    let exit = fight.exit_spot();
    let won = engine
        .handle(
            &click(door),
            1,
            [exit[0], exit[1], exit[2] - 2.0],
            BRAWLER,
            now + 5_000,
        )
        .unwrap();
    assert_eq!(engine.run_phase(), Some(EncounterPhase::Won));
    assert_eq!(won.win_completed, Some(FROSTFANG));
    assert!(won.facts.contains(&FightFact::Won { fight: FROSTFANG }));
    assert_eq!(won.rewards.len(), 1);
    assert_eq!((won.rewards[0].coins, won.rewards[0].job_xp), (25, 50));
    let card = engine.fight_run().unwrap().score(now + 5_000);
    let rows = score_rows(&won.replies);
    assert_eq!(rows["scoreEnemiesDefeated"], (37, -1, 37 * 300));
    assert_eq!(rows["scorePlayerKnockouts"], (10, 10, 25_000));
    assert_eq!(rows["scoreObjectives"], (1, 1, 5_000));
    assert_eq!(rows["scoreTimeBonus"].2, card.time_bonus as i32);
    assert_eq!(
        rows["scoreTotalScore"].2,
        11_100 + 25_000 + 5_000 + card.time_bonus as i32
    );
    assert!(
        !rows.contains_key("scoreBonusObjectives"),
        "the battle has no bonus goal"
    );
    assert!(
        won.replies.iter().any(|r| r.starts_with(&[39, 0, 18])),
        "game over, won"
    );
    assert_eq!(
        won.replies
            .iter()
            .filter(|r| r.starts_with(&[39, 0, 47]))
            .count(),
        1,
        "one results card"
    );
    let _ = std::fs::remove_dir_all(dir);
}
