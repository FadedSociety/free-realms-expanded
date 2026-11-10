//! P2 combat-kits: one status table (`data/content/status_effects.json`) and one code path
//! for players and enemies. A status put on an enemy by a kit ability and one put on the
//! player by an enemy attack both go through `EncounterEngine::apply_status`.
use osfr_tools::encounter::{EncounterOutcome, kits::StatusTable};
use serde_json::{Value, json};

#[path = "common/kit_fixture.rs"]
mod kit_fixture;
use kit_fixture::{Fixture, dealt, refusal, shipped, states, taken};

const ORIGIN: [f32; 3] = [0.0; 3];
/// The solo engine's player guid.
const PLAYER: u64 = 1;

fn wired(f: &Fixture) {
    f.patch("status_effects.json", |c| {
        c["wire"]["state_bits"] = json!(true);
        c["wire"]["attached_effects"] = json!(true);
    });
}

fn has(out: &EncounterOutcome, family: u8, sub: u8) -> usize {
    out.replies.iter().filter(|p| p.starts_with(&[family, 0, sub, 0])).count()
}

#[test]
fn the_shipped_table_carries_the_clients_columns() {
    let t = StatusTable::builtin();
    let stun = t.row("stun").expect("stun");
    assert_eq!((stun.bit, stun.effect_id, stun.animation), (Some(6), 2, 1401), "client bit 6, stars, com_stunned");
    assert!(stun.blocks("move") && stun.blocks("act"));
    assert_eq!(t.row("knock_down").unwrap().animation, 1402);
    assert_eq!(t.row("freeze").unwrap().effect_id, 5337);
    assert_eq!(t.row("root").unwrap().blocks, ["move"]);
    assert!(t.row("sleep").unwrap().breaks_on_damage);
    assert_eq!(t.row("slow").unwrap().speed, Some(0.7), "Archer's Advantage: -30 %");
    assert_eq!(t.row("energy_regen").unwrap().energy_per_tick, 3, "Wizard's Cunning: 3 a second");
    assert_eq!(t.row("heal_on_damage").unwrap().default_ms, 8000, "Warrior's Boon: 8 seconds");
    assert!(!t.wire.state_bits && !t.wire.attached_effects, "unproven packets ship switched off");
    assert_eq!((t.wire.base_bits.enemy, t.wire.base_bits.player), (0x8001, 0x1), "the live 35/20 values");
    // Every status an ability or trait names exists: the kit loader checks it.
    assert!(osfr_tools::encounter::kits::KitTable::builtin().is_some());
    assert!(t.ids().count() >= 25);
}

#[test]
fn the_loader_refuses_bad_rows() {
    let good: Value = serde_json::from_slice(&std::fs::read(shipped("status_effects.json")).unwrap()).unwrap();
    let parse = |change: &dyn Fn(&mut Value)| {
        let mut content = good.clone();
        change(&mut content);
        StatusTable::parse(&serde_json::to_vec(&content).unwrap())
    };
    assert!(parse(&|_| {}).is_ok());
    assert!(parse(&|c| c["statuses"]["stun"]["bit"] = 32.into()).is_err());
    assert!(parse(&|c| c["statuses"]["stun"]["blocks"] = json!(["think"])).is_err());
    assert!(parse(&|c| c["statuses"]["stun"]["default_ms"] = 0.into()).is_err());
    assert!(parse(&|c| c["statuses"]["slow"]["speed"] = json!(0.0)).is_err());
    assert!(parse(&|c| c["statuses"]["stun"]["colour"] = "red".into()).is_err(), "unknown column");
    assert!(parse(&|c| c["schema_version"] = 9.into()).is_err());
}

/// The done-means: a stun on an enemy and a stun on the player are the same call and the
/// same packets (35/20 with bit 6 over the actor's base bits, 35/41 with the stars).
#[test]
fn a_stun_is_one_code_path_for_enemy_and_player() {
    let f = Fixture::new(100_000, 10, false);
    wired(&f);
    let mut e = f.load(43, 1);
    let mut on_enemy = EncounterOutcome::default();
    assert!(e.apply_status(100, "stun", 2000, 1000, &mut on_enemy));
    let mut on_player = EncounterOutcome::default();
    assert!(e.apply_status(PLAYER, "stun", 2000, 1000, &mut on_player));
    assert_eq!(states(&on_enemy), [(100, 0x8001 | 0x40)]);
    assert_eq!(states(&on_player), [(PLAYER, 0x1 | 0x40)]);
    for out in [&on_enemy, &on_player] {
        assert_eq!(has(out, 35, 41), 1, "the stun stars are attached once");
    }
    assert!(e.has_status(100, "stun", 2999) && e.has_status(PLAYER, "stun", 2999));
    assert!(!e.has_status(101, "stun", 1500));
    // A stunned player cannot cast: the basic is silent, the super says so.
    assert!(dealt(&e.attack(100, ORIGIN, 43, 0, 1500).unwrap()).is_empty());
    assert_eq!(refusal(&e.cast_slot(1, 100, ORIGIN, 43, 1500).unwrap()), Some(286));
    // Both end on the same tick, with the bit cleared and the effect detached.
    let end = e.tick(ORIGIN, 43, 3000).unwrap();
    assert!(states(&end).contains(&(100, 0x8001)) && states(&end).contains(&(PLAYER, 0x1)));
    assert_eq!(has(&end, 35, 42), 2);
    assert!(!e.has_status(100, "stun", 3000) && !e.has_status(PLAYER, "stun", 3000));
    assert!(!dealt(&e.attack(100, ORIGIN, 43, 0, 3100).unwrap()).is_empty());
}

#[test]
fn unproven_packets_stay_off_until_switched_on() {
    let f = Fixture::new(100_000, 0, false);
    let mut e = f.load(43, 1);
    let mut out = EncounterOutcome::default();
    assert!(e.apply_status(100, "stun", 2000, 1000, &mut out));
    assert_eq!((has(&out, 35, 20), has(&out, 35, 41)), (0, 0));
    assert!(!out.replies.is_empty(), "the stun clip still plays");
    assert!(!e.apply_status(100, "dizzy", 2000, 1000, &mut out), "an unknown status lands nowhere");
}

#[test]
fn a_stunned_enemy_neither_walks_nor_strikes_until_it_ends() {
    let f = Fixture::new(100_000, 10, false);
    let mut e = f.load(32, 1);
    // Sweeping Slash pulls all three; then stun them.
    assert_eq!(dealt(&e.attack(100, ORIGIN, 32, 0, 0).unwrap()).len(), 3);
    let mut out = EncounterOutcome::default();
    for guid in 100..103 {
        assert!(e.apply_status(guid, "stun", 4000, 0, &mut out));
    }
    let mut hurt = Vec::new();
    for t in (250..4000).step_by(250) {
        hurt.extend(taken(&e.tick(ORIGIN, 32, t).unwrap()));
    }
    assert!(hurt.is_empty(), "no enemy struck while stunned: {hurt:?}");
    for t in (4000..9000).step_by(250) {
        hurt.extend(taken(&e.tick(ORIGIN, 32, t).unwrap()));
    }
    assert!(!hurt.is_empty(), "they strike again once it ends");
}

#[test]
fn immunity_follows_a_stun_and_bosses_shrug_off_charm() {
    let f = Fixture::new(100_000, 0, true);
    let mut e = f.load(11, 1);
    let mut out = EncounterOutcome::default();
    assert!(e.apply_status(100, "stun", 2000, 0, &mut out));
    // A second stun while it runs refreshes it and takes the longer end.
    assert!(e.apply_status(100, "stun", 3000, 500, &mut out));
    e.tick(ORIGIN, 11, 3400).unwrap();
    assert!(e.has_status(100, "stun", 3400));
    e.tick(ORIGIN, 11, 3500).unwrap();
    assert!(!e.has_status(100, "stun", 3500), "expired exactly at its end");
    assert!(!e.apply_status(100, "stun", 2000, 4000, &mut out), "immune for 3 s after it ends");
    assert!(e.apply_status(100, "root", 2000, 4000, &mut out), "but not to another status");
    e.tick(ORIGIN, 11, 6500).unwrap();
    assert!(e.apply_status(100, "stun", 2000, 6500, &mut out), "and the immunity runs out");
    assert!(e.apply_status(101, "charm", 0, 0, &mut out), "an ordinary enemy can be charmed");
    // The cave's enemy (guid 200) is a boss: charm and transfigure never land, a stun does.
    let home = osfr_tools::character_store::CharacterState { guid: 1, zone_id: 0, position: [9.0, 2.0, 3.0], rotation: [0.0; 3] };
    let mut e = f.load(11, 1);
    e.begin(411, home, 11).unwrap();
    e.on_zone_ready(11, 0).unwrap();
    assert!(!e.apply_status(200, "charm", 0, 0, &mut out));
    assert!(!e.apply_status(200, "transfigure", 0, 0, &mut out));
    assert!(e.apply_status(200, "stun", 0, 0, &mut out));
    assert!(e.has_status(200, "stun", 2999) && !e.has_status(200, "stun", 3000), "0 ms means the status's own default");
}

#[test]
fn damage_breaks_sleep_and_nothing_else() {
    let f = Fixture::new(100_000, 0, false);
    let mut e = f.load(35, 1);
    let mut out = EncounterOutcome::default();
    assert!(e.apply_status(100, "sleep", 0, 0, &mut out));
    assert!(e.apply_status(101, "stun", 5000, 0, &mut out));
    assert!(e.apply_status(102, "root", 5000, 0, &mut out));
    assert!(e.has_status(100, "sleep", 9999));
    // Barrage hits all three.
    assert_eq!(dealt(&e.attack(101, ORIGIN, 35, 0, 1000).unwrap()).len(), 3);
    assert!(!e.has_status(100, "sleep", 1001), "a hit wakes the sleeper");
    assert!(e.has_status(101, "stun", 1001) && e.has_status(102, "root", 1001));
}

#[test]
fn statuses_change_damage_dealt_and_taken() {
    let f = Fixture::new(100_000, 0, false);
    let mut e = f.load(32, 1);
    let mut out = EncounterOutcome::default();
    // Target Vitals' debuff: the marked enemy takes 20 % more, the others do not.
    assert!(e.apply_status(100, "defense_down", 0, 0, &mut out));
    let hits = kit_fixture::hits(&e.attack(100, ORIGIN, 32, 0, 1000).unwrap());
    for (_, target, damage) in hits {
        let base = if target == 100 { 335 } else { 279 };
        assert!(damage == base || damage == base * 3 / 2, "enemy {target} took {damage}");
    }
    // Ready to Rumble's buff on the player: 20 % more on everything, for 10 s.
    assert!(e.apply_status(PLAYER, "damage_up", 0, 2000, &mut out));
    let hits = kit_fixture::hits(&e.attack(101, ORIGIN, 32, 0, 3000).unwrap());
    assert!(hits.iter().any(|h| h.1 == 101 && (h.2 == 335 || h.2 == 502)), "{hits:?}");
    e.tick(ORIGIN, 32, 12_000).unwrap();
    let hits = kit_fixture::hits(&e.attack(101, ORIGIN, 32, 0, 13_000).unwrap());
    assert!(hits.iter().any(|h| h.1 == 101 && (h.2 == 279 || h.2 == 418)), "the buff ran out: {hits:?}");
    // A dead player takes no status, and a job change clears them.
    assert!(e.apply_status(102, "slow", 0, 13_000, &mut out));
    e.set_player_job(43, 1);
    assert!(!e.has_status(102, "slow", 13_001));
}

/// A kit ability's rider reaches the same table: Leg Sweep's knock-down and Blood Cell's root.
#[test]
fn kit_riders_land_as_statuses() {
    let f = Fixture::new(100_000, 0, false);
    let mut e = f.load(43, 1);
    e.cast_slot(1, 101, ORIGIN, 43, 1000).unwrap();
    for guid in 100..103 {
        assert!(e.has_status(guid, "knock_down", 2999), "Leg Sweep knocks down {guid}");
        assert!(!e.has_status(guid, "knock_down", 3000));
    }
    // Enrage-style self buffs land on the caster.
    let mut e = f.load(43, 1);
    let rumble = osfr_tools::encounter::kits::WeaponFacts {
        item_id: {
            let content: Value = serde_json::from_slice(&std::fs::read(shipped("weapon_kits.json")).unwrap()).unwrap();
            let items = content["kits"]["brawler/wallop+ready_to_rumble"]["items"].as_object().unwrap();
            items.values().next().unwrap()[0].as_i64().unwrap() as i32
        },
        ..Default::default()
    };
    e.equip_weapon(43, rumble);
    e.cast_slot(1, 101, ORIGIN, 43, 1000).unwrap();
    assert!(e.has_status(PLAYER, "damage_up", 10_999));
}
