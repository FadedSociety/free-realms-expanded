//! The combat seam (piece `combat-jobs`, step P0): the packet writers, the simulator and the
//! quest-credit block moved out of the hub files and the interfaces of P1-P7 were frozen, with
//! no behaviour change. These tests pin that: the simulator plays battle 96 exactly as before
//! the move, and the new kill facts mirror the old `defeated` list.
//!
//! Needs the private opening content like `tests/activity_portal_route.rs`
//! (`OSFR_ACTIVITY_CONTENT` overrides the default directory).
use osfr_tools::{
    account_store::AccountRole,
    character_store::CharacterState,
    chat,
    encounter::{EncounterEngine, FightFact, Rank, brain, director, kits, scale, wire},
    gameplay::{Gameplay, Outcome},
};
use std::{collections::BTreeMap, path::PathBuf};

const LAKESHORE: [f32; 3] = [-733.952, 9.09, 945.973];
const BRAWLER: i32 = 43;

fn content() -> PathBuf {
    std::env::var_os("OSFR_ACTIVITY_CONTENT")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../../lab/references/rust-opening-cooking-original-assets")
        })
}

fn character() -> CharacterState {
    CharacterState {
        guid: 1,
        zone_id: 0,
        position: LAKESHORE,
        rotation: [0.0, 0.0, 1.0],
    }
}

fn chat_text(out: &Outcome) -> Vec<String> {
    out.replies
        .iter()
        .filter(|reply| reply.starts_with(&[15, 0, 3, 0]))
        .map(|reply| {
            let length = u32::from_le_bytes(reply[4..8].try_into().unwrap()) as usize;
            String::from_utf8_lossy(&reply[8..8 + length]).into_owned()
        })
        .collect()
}

fn brawler() -> Gameplay {
    let mut game = Gameplay::load(&content(), None).unwrap();
    game.initialize_world(&character(), 500).unwrap();
    let mut switch = vec![26, 0, 13, 0];
    switch.extend(BRAWLER.to_le_bytes());
    assert!(
        game.handle_in_world(&switch, &character(), 600)
            .unwrap()
            .changed
    );
    game.set_dev_tools(true);
    game.set_role(AccountRole::Admin);
    game
}

/// `realm sim 96` (the Troll Summoner battle) before the move, captured at base `75e3d982`;
/// rc-0136 retuned the battle to its tier's 2014 numbers (the level 1 bot now loses half its
/// health where it lost 80).
const SIM_96: [&str; 6] = [
    "Battle 96, job 43 (scripted bot):",
    "  level  1: won in 7s, lowest health 250/500, dealt 3266 (0 crits)",
    "  level  5: won in 7s, lowest health 525/875, dealt 4786 (0 crits)",
    "  level 10: won in 0s, lowest health 1761/1761, dealt 1726 (0 crits)",
    "  level 15: won in 0s, lowest health 3545/3545, dealt 1726 (0 crits)",
    "  level 20: won in 0s, lowest health 7135/7135, dealt 1726 (0 crits)",
];

#[test]
fn the_simulator_plays_battle_96_exactly_as_before_the_move() {
    let mut game = brawler();
    let out = game
        .handle_in_world(&chat::local_request("realm sim 96"), &character(), 700)
        .unwrap();
    assert_eq!(chat_text(&out), SIM_96);
}

fn shipped() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content")
}

/// The opening cave of the shipped `encounters.json`, fought to the win through the engine's
/// real entry points (begin, zone ready, attack, tick).
#[test]
fn every_defeat_has_its_kill_fact_and_the_run_reports_entered_and_won() {
    const CAVE: u32 = 411;
    let mut engine = EncounterEngine::load(shipped(), None).unwrap();
    engine.set_player_job(BRAWLER, 20);
    let world = engine.world().clone();
    let home = CharacterState {
        guid: 1,
        zone_id: world.zone_id,
        position: world.position,
        rotation: world.rotation,
    };
    assert!(engine.begin(CAVE, home, BRAWLER).unwrap().facts.is_empty());
    let ready = engine.on_zone_ready(BRAWLER, 0).unwrap();
    assert_eq!(ready.facts, [FightFact::Entered { fight: CAVE }]);
    // A reconnect into the running fight is not a second entry.
    assert!(engine.on_zone_ready(BRAWLER, 0).unwrap().facts.is_empty());
    engine.toggle_god();

    let (mut defeated, mut kills, mut facts, mut won) = (Vec::new(), Vec::new(), Vec::new(), None);
    let mut now = 0;
    while won.is_none() && now < 600_000 {
        now += 700;
        let Some(&(target, at, _)) = engine.standing_enemies().first() else {
            break;
        };
        let me = [at[0] + 1.0, at[1], at[2]];
        for out in [
            engine.tick(me, BRAWLER, now).unwrap(),
            engine.attack(target, me, BRAWLER, 0, now).unwrap(),
        ] {
            assert_eq!(out.kills.len(), out.defeated.len());
            defeated.extend(out.defeated);
            kills.extend(out.kills);
            facts.extend(out.facts);
            won = won.or(out.win_completed);
        }
    }

    assert_eq!(won, Some(CAVE));
    // P2 combat-kits (pin moved): with the weapon kits live the basic is a named ability, so
    // each accepted swing also reports `Used`; they are not what this test counts.
    facts.retain(|fact| !matches!(fact, FightFact::Used { .. }));
    assert_eq!(facts, [FightFact::Won { fight: CAVE }]);
    assert_eq!(defeated.len(), 4, "the cave's four enemies");
    assert_eq!(
        kills.iter().map(|k| k.kind as i32).collect::<Vec<_>>(),
        defeated
    );
    for kill in &kills {
        assert_eq!(
            (kill.fight, kill.zone, kill.job, kill.solo),
            (
                Some(CAVE),
                engine.zone().unwrap().zone_id,
                BRAWLER as u32,
                true
            )
        );
        // P2 combat-kits (pin moved): a kill names the kit ability that made it.
        assert!(kill.ability.is_some());
        assert_eq!((kill.item, kill.area), (None, None));
        assert!(kill.tags.is_empty() && kill.guid != 0);
    }
    // The Gloam King is the cave's one boss row.
    let ranks: Vec<Rank> = kills.iter().map(|k| k.rank).collect();
    assert_eq!(ranks.iter().filter(|r| **r == Rank::Boss).count(), 1);
    assert_eq!(ranks.iter().filter(|r| **r == Rank::Normal).count(), 3);
}

fn home(engine: &EncounterEngine) -> CharacterState {
    let world = engine.world();
    CharacterState {
        guid: 1,
        zone_id: world.zone_id,
        position: world.position,
        rotation: world.rotation,
    }
}

/// `realm win` (`debug_win`) goes through the normal hit path, so it reports what a fought
/// win reports: one kill fact per defeated entry and the win.
#[test]
fn the_developer_win_reports_every_kill_fact_and_the_won_fact() {
    const CAVE: u32 = 411;
    let mut engine = EncounterEngine::load(shipped(), None).unwrap();
    engine.set_player_job(BRAWLER, 20);
    engine.begin(CAVE, home(&engine), BRAWLER).unwrap();
    engine.on_zone_ready(BRAWLER, 0).unwrap();
    let out = engine.debug_win(1_000).unwrap();
    assert_eq!(out.win_completed, Some(CAVE));
    assert_eq!(out.defeated.len(), 4, "the cave's four enemies");
    assert_eq!(
        out.kills.iter().map(|k| k.kind as i32).collect::<Vec<_>>(),
        out.defeated
    );
    assert!(out.kills.iter().all(|k| k.fight == Some(CAVE)));
    assert_eq!(out.facts, [FightFact::Won { fight: CAVE }]);
}

/// An enemy row may carry `rank`, `tags` and `dormant`. A dormant row is not
/// there at the start, cannot be hit, and (unlike a summoned minion) still counts toward the
/// goal: the fight is not won while it sleeps.
#[test]
fn a_dormant_row_stays_out_of_the_fight_and_still_counts_toward_the_goal() {
    const CAVE: u32 = 411;
    let dir = std::env::temp_dir().join(format!("combat-seam-dormant-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let mut content: serde_json::Value =
        serde_json::from_slice(&std::fs::read(shipped().join("encounters.json")).unwrap()).unwrap();
    let template = content["template_file"].as_str().unwrap().to_owned();
    std::fs::copy(shipped().join(&template), dir.join(&template)).unwrap();
    let row = &mut content["dungeons"][0]["enemies"][0];
    let sleeper = row["guid"].as_u64().unwrap();
    row["dormant"] = true.into();
    row["rank"] = "elite".into();
    row["tags"] = serde_json::json!(["captain"]);
    std::fs::write(dir.join("encounters.json"), content.to_string()).unwrap();

    let mut engine = EncounterEngine::load(&dir, None).unwrap();
    engine.set_player_job(BRAWLER, 20);
    engine.begin(CAVE, home(&engine), BRAWLER).unwrap();
    engine.on_zone_ready(BRAWLER, 0).unwrap();
    let standing: Vec<u64> = engine.standing_enemies().iter().map(|e| e.0).collect();
    assert_eq!(standing.len(), 3);
    assert!(!standing.contains(&sleeper));
    let out = engine.debug_win(1_000).unwrap();
    assert_eq!(out.kills.len(), 3);
    assert_eq!(out.win_completed, None, "one goal enemy is still asleep");
    assert!(out.facts.is_empty());

    content["dungeons"][0]["enemies"][0]["tags"] = serde_json::json!([""]);
    std::fs::write(dir.join("encounters.json"), content.to_string()).unwrap();
    assert!(EncounterEngine::load(&dir, None).is_err(), "an empty tag");
    let _ = std::fs::remove_dir_all(dir);
}

/// The frozen `scale` signatures answer with the design's anchors (section 5) until P4 moves the
/// numbers into data; a later change of signature breaks this test at compile time.
#[test]
fn the_frozen_scale_interface_holds_the_captured_anchors() {
    assert_eq!(scale::enemy_health(Rank::Minion, 1, 1), 260);
    assert_eq!(scale::enemy_health(Rank::Normal, 1, 1), 760);
    assert_eq!(scale::enemy_health(Rank::Boss, 1, 1), 3_800);
    assert_eq!(scale::enemy_health(Rank::Tough, 5, 1), 16_143);
    assert_eq!(scale::enemy_health(Rank::Normal, 1, 3), 1_520);
    assert_eq!(scale::enemy_hit(Rank::Minion, 1), 25);
    assert_eq!(scale::ability_damage("super_heavy", 5), 10_674);
    assert_eq!(scale::ability_damage("no_such_class", 1), 0);
    assert_eq!(scale::mitigate(206, 348, 0, 5), 113);
    assert_eq!(
        (scale::player_health(1), scale::player_health(4)),
        (500, 761)
    );
    assert_eq!(
        [1, 3, 4, 8, 12, 16, 20].map(scale::tier_of_level),
        [1, 1, 2, 3, 4, 5, 5]
    );
}

/// The frozen `wire` writers that had a body before the seam still write those bytes, and the
/// death writer with no delay is the plain graceful remove.
#[test]
fn the_frozen_wire_interface_keeps_the_bytes_the_engine_sent_before() {
    let hit = wire::player_hit(&wire::PlayerHit {
        attacker: 1,
        target: 9,
        dealt: 30,
        health: 70,
        max_health: 100,
        critical: true,
        swing: true,
        ..Default::default()
    });
    let heads: Vec<&[u8]> = hit.iter().map(|p| &p[..4]).collect();
    // rc-0136: the 35/35 health statement the boss bar reads follows the nameplate bar.
    assert_eq!(heads, [[35, 0, 8, 0], [32, 0, 7, 0], [35, 0, 5, 0], [35, 0, 35, 0]]);
    assert_eq!(hit[3], wire::hitpoints(9, 9, false, 100, 70, 0, false));
    assert_eq!(hit[1].len(), 4 + 24 + 12 + 2 + 8);
    assert_eq!(hit[1][28..32], 30_i32.to_le_bytes());
    assert_eq!(hit[1][40], 1, "the critical flag");
    assert_eq!(hit[1][46..50], 70_i32.to_le_bytes());
    let quiet = wire::player_hit(&wire::PlayerHit {
        attacker: 1,
        target: 9,
        dealt: 30,
        health: 70,
        max_health: 100,
        critical: false,
        swing: false,
        ..Default::default()
    });
    assert_eq!(quiet.len(), 3, "an ability hit plays no swing");
    // The classic hit carries the kit's impact effect in 32/7 and nothing else new.
    let named = wire::player_hit(&wire::PlayerHit {
        attacker: 1,
        target: 9,
        dealt: 30,
        health: 70,
        max_health: 100,
        critical: true,
        swing: true,
        ability: Some(700),
        effect: Some(5409),
    });
    assert_eq!(named[1][36..40], 5409_i32.to_le_bytes());
    assert_eq!((&named[0], &named[2]), (&hit[0], &hit[2]));
    assert_eq!((&named[1][..36], &named[1][40..]), (&hit[1][..36], &hit[1][40..]));

    let gone = wire::death(9, 0);
    assert_eq!(gone[..4], [35, 0, 3, 0]);
    assert_eq!(gone[15..23], [0; 8], "no delay");
    let later = wire::death(9, 5_000);
    assert_eq!(later.len(), gone.len());
    assert_eq!(later[15..19], 5_000_u32.to_le_bytes());

    assert_eq!(
        wire::hitpoints(1, 9, true, 100, 70, -30, false)[29..33],
        (-30_i32).to_le_bytes()
    );
    assert!(wire::launch("ninja_basic", 1, 9).is_none());
    assert_eq!(wire::state(9, 0x8001)[12..], 0x8001_u32.to_le_bytes());
}

/// The frozen brain, kits and director entry points exist with their signatures and decide
/// nothing yet.
#[test]
fn the_frozen_brain_kits_and_director_stubs_decide_nothing() {
    let tags = BTreeMap::new();
    let view = brain::View {
        now_ms: 0,
        health_pct: 100,
        position: [0.0; 3],
        home: [0.0; 3],
        target: None,
        target_distance: None,
        nearest_player: None,
        tags_alive: &tags,
        signals: &[],
        phase: 0,
        night: false,
    };
    let mut acts: Vec<brain::Act> = Vec::new();
    brain::step(
        &brain::Program::default(),
        &mut brain::Memory::default(),
        &view,
        &mut acts,
    );
    assert!(acts.is_empty());

    // P2 combat-kits (pin moved): the kits are built, so the shipped table answers. A Brawler's
    // default weapon gives the basic and the super; a kit with no abilities resolves nothing.
    let bar = kits::bar_for(43, 4, &kits::WeaponFacts::default());
    assert!(bar.slots[0].is_some() && bar.slots[1].is_some());
    assert!(bar.slots[2..].iter().all(Option::is_none));
    let caster = kits::Caster {
        guid: 1,
        position: [0.0; 3],
        heading: [0.0, 0.0, 1.0],
        profile: 43,
        level: 4,
        tier: 2,
        energy: 100,
        now_ms: 0,
    };
    let targets = kits::Targets {
        selected: None,
        enemies: &[],
        allies: &[],
    };
    assert_eq!(
        kits::resolve(&kits::Kit::default(), 0, &caster, &targets),
        Err(286)
    );

    let fight = director::Fight { id: 96, ..Default::default() };
    let mut run = director::begin(&fight, &director::Party::default());
    assert_eq!(run.fight, 96);
    director::on_fact(&mut run, &director::Signal::Named("go".into()), &mut acts);
    let mut wants: Vec<director::DirectorAct> = Vec::new();
    director::tick(&mut run, 250, &mut wants);
    // The director is no longer a stub (P5): an empty fight may report its own state, but
    // it spawns nothing and the brain stub still acts on nothing.
    assert!(acts.is_empty(), "{acts:?}");
    assert!(
        !wants.iter().any(|w| matches!(w, director::DirectorAct::Wave(_))),
        "{wants:?}"
    );
}
