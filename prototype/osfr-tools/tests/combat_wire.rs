//! Combat wire (piece P1): every writer against the bytes the March 2014 server sent.
//!
//! `fixtures/combat_capture_rows.json` holds one captured packet per row with its capture
//! file and time; each test builds the same packet from the row's decoded fields and
//! compares the whole packet. The templates of `data/content/combat_wire.json` are replayed
//! with patched ids and compared with a second captured row.
use osfr_tools::{
    character_store::CharacterState,
    encounter::{
        EncounterEngine,
        wire::{self, DefinitionPatch, EnemyHit, Settings, StatValue, WireData, bits, templates},
    },
};
use serde_json::Value;
use std::path::PathBuf;

const BRAWLER: i32 = 43;

fn shipped() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content")
}

fn rows() -> Vec<Value> {
    serde_json::from_str(include_str!("fixtures/combat_capture_rows.json")).unwrap()
}

fn row(name: &str) -> Value {
    rows()
        .into_iter()
        .find(|r| r["name"] == name)
        .unwrap_or_else(|| panic!("no fixture row {name}"))
}

fn bytes(row: &Value) -> Vec<u8> {
    let hex = row["hex"].as_str().unwrap();
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
        .collect()
}

fn u(row: &Value, field: &str) -> u64 {
    row["fields"][field]
        .as_u64()
        .unwrap_or_else(|| panic!("{}: no {field}", row["name"]))
}

fn i(row: &Value, field: &str) -> i32 {
    row["fields"][field].as_i64().unwrap() as i32
}

fn flag(row: &Value, field: &str) -> bool {
    row["fields"][field].as_bool().unwrap()
}

fn same(name: &str, ours: Vec<u8>) {
    assert_eq!(ours, bytes(&row(name)), "{name}");
}

#[test]
fn every_fixture_row_names_its_capture_and_is_used() {
    let rows = rows();
    assert_eq!(rows.len(), 39);
    for r in &rows {
        assert!(r["capture"].as_str().unwrap().contains(".pcap"), "{r}");
        assert!(r["t"].as_f64().unwrap() >= 0.0);
        assert!(!bytes(r).is_empty());
    }
}

#[test]
fn hitpoints_is_max_current_delta_with_real_ids() {
    for name in [
        "hitpoints_damage",
        "hitpoints_overkill",
        "hitpoints_regen",
        "hitpoints_statement",
        "hitpoints_crit",
        "hitpoints_heal",
        "hitpoints_player_regen",
    ] {
        let r = row(name);
        same(
            name,
            wire::hitpoints(
                u(&r, "source"),
                u(&r, "target"),
                flag(&r, "show"),
                i(&r, "max"),
                i(&r, "current"),
                i(&r, "delta"),
                flag(&r, "crit"),
            ),
        );
    }
    // The order the client reads (reader 0x008d6c50): max at 21, current at 25, delta at 29.
    let heal = wire::hitpoints(7, 9, true, 2500, 2100, 394, false);
    assert_eq!(heal.len(), 34);
    assert_eq!(heal[21..25], 2500_i32.to_le_bytes());
    assert_eq!(heal[25..29], 2100_i32.to_le_bytes());
    assert_eq!(heal[29..33], 394_i32.to_le_bytes());
    assert_eq!((heal[20], heal[33]), (1, 0));
}

#[test]
fn states_and_status_bits_match_the_live_values() {
    for (name, value) in [
        ("state_idle", bits::IDLE),
        ("state_fighting", bits::IN_FIGHT),
        ("state_going_home", bits::RETURNING),
        ("state_nonattackable", bits::ALIVE | bits::NONATTACKABLE),
        ("state_knocked_out_fighting", bits::FIGHTING | bits::KNOCKED_OUT),
        ("state_knocked_out", bits::ALIVE | bits::KNOCKED_OUT),
    ] {
        let r = row(name);
        assert_eq!(u(&r, "bits"), u64::from(value), "{name}");
        same(name, wire::state(u(&r, "guid"), value));
    }
    assert_eq!(
        (bits::STUNNED, bits::ROOTED, bits::FROZEN),
        (0x40, 0x20, 0x1_0000)
    );
}

#[test]
fn speeds_match() {
    for name in ["speed_walk", "speed_run", "speed_home"] {
        let r = row(name);
        let speed = r["fields"]["metres_per_s"].as_f64().unwrap() as f32;
        same(name, wire::speed(u(&r, "guid"), speed));
    }
}

#[test]
fn death_carries_the_delay_the_effect_and_its_duration() {
    let data = WireData::load(&shipped()).unwrap();
    for name in ["death_field", "death_battle"] {
        let r = row(name);
        let delay = i(&r, "delay_ms") as u32;
        same(name, wire::death(u(&r, "guid"), delay));
        same(name, data.death(u(&r, "guid"), delay));
        assert_eq!(i(&r, "effect_id"), data.death.effect_id);
        assert_eq!(i(&r, "duration_ms"), data.death.duration_ms);
    }
    assert_eq!(i(&row("death_field"), "delay_ms"), 5000);
    assert_eq!(i(&row("death_battle"), "delay_ms"), 2000);
}

#[test]
fn enemy_hits_carry_the_effect_and_the_flag_bytes() {
    let effects = WireData::load(&shipped()).unwrap().hit_effects.clone();
    for name in ["enemy_hit", "enemy_hit_special", "enemy_hit_knockout"] {
        let r = row(name);
        let (special, knockout) = (flag(&r, "special"), flag(&r, "knockout"));
        assert_eq!(effects.pick(special, knockout), i(&r, "effect"), "{name}");
        same(
            name,
            wire::enemy_hit(&EnemyHit {
                attacker: u(&r, "attacker"),
                target: u(&r, "target"),
                damage: i(&r, "damage") as u32,
                target_max: i(&r, "target_max") as u32,
                target_current: i(&r, "target_current") as u32,
                effect: effects.pick(special, knockout),
                special,
                knockout,
            }),
        );
    }
    let r = row("dodge");
    same("dodge", wire::dodge(u(&r, "attacker"), u(&r, "target")));
}

#[test]
fn casts_and_reactions_match() {
    for name in ["cast_player_basic", "cast_enemy"] {
        let r = row(name);
        same(
            name,
            wire::cast(u(&r, "caster"), u(&r, "target"), i(&r, "ability")),
        );
    }
    let own = row("cast_player_basic");
    assert_eq!(u(&own, "caster"), u(&own, "target"), "a player's own cast");
    for name in ["reaction_recoil", "reaction_knock_down"] {
        let r = row(name);
        let seconds = r["fields"]["seconds"].as_f64().unwrap() as f32;
        same(
            name,
            wire::reaction(u(&r, "guid"), i(&r, "animation"), seconds),
        );
    }
}

#[test]
fn attached_effects_disposition_tags_and_the_respawn_window_match() {
    let r = row("attach_going_home");
    let attach = wire::attach(u(&r, "guid"), i(&r, "tag"), i(&r, "effect"), u(&r, "source"));
    assert_eq!(attach.len(), 36, "the strict reader wants all 36 bytes");
    same("attach_going_home", attach);
    let r = row("detach_going_home");
    same("detach_going_home", wire::detach(u(&r, "guid"), i(&r, "tag")));
    let r = row("disposition");
    same("disposition", wire::disposition(u(&r, "guid"), i(&r, "value")));
    let r = row("tag_remove");
    same("tag_remove", wire::tag_remove(i(&r, "id")));
    let r = row("respawn_window");
    same(
        "respawn_window",
        wire::respawn_window(i(&r, "timer_ms"), i(&r, "cost") as u32),
    );
}

#[test]
fn encounter_settings_match_both_captured_records() {
    let data = WireData::load(&shipped()).unwrap();
    same("settings_world", wire::encounter_settings(&Settings::world()));
    same("settings_battle", wire::encounter_settings(&Settings::battle()));
    same("settings_world", wire::encounter_settings(&data.settings.world));
    same("settings_battle", wire::encounter_settings(&data.settings.battle));
    let own = wire::encounter_settings(&Settings::battle().with_score(150, 9000, 700));
    assert_eq!(own[33..37], 150_i32.to_le_bytes());
    assert_eq!(own[37..41], 9000_i32.to_le_bytes());
    assert_eq!(own[41..45], 700_i32.to_le_bytes());
}

#[test]
fn a_level_20_stat_block_matches() {
    let r = row("stats_level_20");
    let stats: Vec<(i32, StatValue)> = r["fields"]["rows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| {
            let id = s[0].as_i64().unwrap() as i32;
            match s[1].as_str().unwrap() {
                "float" => (id, StatValue::Float(s[2].as_f64().unwrap() as f32)),
                _ => (id, StatValue::Int(s[2].as_i64().unwrap() as i32)),
            }
        })
        .collect();
    assert_eq!(stats.len(), 25);
    assert!(stats.contains(&(1, StatValue::Int(7828))));
    assert!(stats.contains(&(20, StatValue::Float(1.05))));
    same("stats_level_20", wire::stats(u(&r, "guid"), &stats));
}

/// Live sent an NPC line as 15/4 with all flag bytes clear and the listener filled in; ours
/// (`chat::string_say`, already shown natively) differs in exactly the chat-log byte and
/// the listener.
#[test]
fn speech_differs_from_live_only_in_the_log_byte_and_the_listener() {
    let r = row("speech");
    let live = bytes(&r);
    let ours = wire::say(
        u(&r, "speaker"),
        &wire::Line {
            text_id: Some(i(&r, "string_id")),
            ..Default::default()
        },
    );
    assert_eq!(ours.len(), live.len());
    let differing: Vec<usize> = (0..live.len()).filter(|at| ours[*at] != live[*at]).collect();
    assert!(differing.iter().all(|at| *at == 17 || (23..31).contains(at)));
    assert!(differing.contains(&17));
}

#[test]
fn the_shipped_wire_data_equals_the_built_in_scalars() {
    let data = WireData::load(&shipped()).unwrap();
    let built_in = WireData::default();
    assert_eq!(data.death, built_in.death);
    assert_eq!(data.hit_effects, built_in.hit_effects);
    assert_eq!(data.settings, built_in.settings);
    assert_eq!(data.live_swing, None);
    assert_eq!((data.launches.len(), data.definitions.len()), (27, 33));
    // No file: the defaults, with no templates.
    let empty = std::env::temp_dir().join(format!("combat-wire-none-{}", std::process::id()));
    std::fs::create_dir_all(&empty).unwrap();
    assert_eq!(*WireData::load(&empty).unwrap(), built_in);
    // A template whose offsets leave the packet refuses the load.
    std::fs::write(
        empty.join("combat_wire.json"),
        r#"{"launches":{"x":{"hex":"24000400","caster_at":[4],"target_at":[],"capture":"c","t":0}}}"#,
    )
    .unwrap();
    assert!(WireData::load(&empty).is_err());
    std::fs::remove_dir_all(&empty).unwrap();
}

/// A launch template patched with a second captured row's ids is that row, but for the cast
/// counter the template names.
#[test]
fn launch_templates_round_trip_with_patched_ids() {
    let data = WireData::load(&shipped()).unwrap();
    for name in ["launch_4895_second", "launch_5994_second", "launch_5060_second"] {
        let r = row(name);
        let key = r["fields"]["template"].as_str().unwrap();
        let template = &data.launches[key];
        let live = bytes(&r);
        let mut ours = data
            .launch(key, u(&r, "caster"), u(&r, "target"))
            .unwrap();
        assert_eq!(ours.len(), live.len());
        assert!(!template.counter_at.is_empty(), "{name}");
        for at in &template.counter_at {
            ours[*at] = live[*at];
        }
        assert_eq!(ours, live, "{name}");
    }
    // Every template: its own ids patched back give the captured bytes, and new ids land on
    // every listed offset and nowhere else.
    for (name, t) in &data.launches {
        let caster = u64::from_le_bytes(t.hex[4..12].try_into().unwrap());
        assert!(t.caster_at.contains(&4), "{name}");
        let patched = data.launch(name, 0x1111_2222_3333_4444, 0x5555_6666_7777_8888);
        let patched = patched.unwrap();
        let moved: Vec<usize> = (0..t.hex.len()).filter(|at| patched[*at] != t.hex[*at]).collect();
        assert!(moved.iter().all(|at| {
            t.caster_at.iter().chain(&t.target_at).any(|o| (*o..*o + 8).contains(at))
        }));
        if let [target_at] = t.target_at[..] {
            let target = u64::from_le_bytes(t.hex[target_at..target_at + 8].try_into().unwrap());
            assert_eq!(data.launch(name, caster, target).unwrap(), t.hex, "{name}");
        }
        assert_eq!(t.targets as i32, i32::from_le_bytes(t.hex[12..16].try_into().unwrap()));
    }
    assert!(data.launch("no such", 1, 2).is_none());
    assert!(wire::launch("4895", 1, 2).is_none(), "the free function has no content");
    let toast = data.kill_toast(7, 9).unwrap();
    assert_eq!(toast.len(), 72);
    assert_eq!(toast[40..48], 7_u64.to_le_bytes());
    assert_eq!(toast[48..56], 9_u64.to_le_bytes());
}

#[test]
fn definitions_round_trip_and_take_new_ids_and_values() {
    let data = WireData::load(&shipped()).unwrap();
    for (name, d) in &data.definitions {
        let id: i32 = name.parse().unwrap();
        // The effect rows tile the packet exactly: the count at 155, the rows, one last byte.
        let count = i32::from_le_bytes(d.hex[155..159].try_into().unwrap());
        assert_eq!(count as usize, d.rows.len(), "{name}");
        assert_eq!(i32::from_le_bytes(d.hex[4..8].try_into().unwrap()), id);
        for r in &d.rows {
            let at = |o: usize| i32::from_le_bytes(d.hex[o..o + 4].try_into().unwrap());
            assert_eq!(at(r.ability_at), id, "{name}");
            assert!((1..=100).contains(&at(r.chance_at)), "{name}: a percent");
            // The row's `low` is the packet's float rounded to three places, for reading.
            let low = f32::from_le_bytes(d.hex[r.low_at..r.low_at + 4].try_into().unwrap());
            assert!((low - r.low).abs() < 0.01, "{name}");
        }
        let unchanged = DefinitionPatch { id, ..Default::default() };
        assert_eq!(data.definition(id, &unchanged).unwrap(), d.hex, "{name}");
    }
    // Flame Flash (4895) at level 20 and the five tiers of 5235.
    assert_eq!(data.definitions["4895"].rows[0].low, 2609.2);
    assert_eq!(data.definitions["4899"].rows[0].low, 10674.0);
    let ladder: Vec<f32> = data.definitions["5235"].rows.iter().map(|r| r.low).collect();
    assert_eq!(ladder, [254.0, 444.0, 776.0, 1357.0, 2372.0]);

    let base = &data.definitions["4895"];
    let patch = DefinitionPatch {
        id: 70_001,
        name_id: Some(11),
        description_id: Some(12),
        values: vec![(254.0, 279.0)],
    };
    let ours = data.definition(4895, &patch).unwrap();
    assert_eq!(ours.len(), base.hex.len());
    let at = |o: usize| i32::from_le_bytes(ours[o..o + 4].try_into().unwrap());
    assert_eq!(at(templates::DEFINITION_ID_AT), 70_001);
    assert_eq!(at(templates::DEFINITION_NAME_AT), 11);
    assert_eq!(at(templates::DEFINITION_DESCRIPTION_AT), 12);
    assert_eq!(at(templates::DEFINITION_ANIMATION_AT), 1099, "the swing stays");
    assert_eq!(at(base.rows[0].ability_at), 70_001);
    assert_eq!(at(base.rows[1].ability_at), 70_001);
    assert_eq!(ours[base.rows[0].low_at..base.rows[0].low_at + 4], 254_f32.to_le_bytes());
    assert_eq!(ours[base.rows[0].high_at..base.rows[0].high_at + 4], 279_f32.to_le_bytes());
    assert_eq!(
        ours[base.rows[1].low_at..base.rows[1].low_at + 4],
        base.hex[base.rows[1].low_at..base.rows[1].low_at + 4],
        "a row without a value keeps the capture"
    );
    assert!(data.definition(1, &patch).is_none());
}

/// The live shape of one player hit: 36/3 with the caster twice, the 36/4 of the ability,
/// 35/35 on the enemy, then its bar; the classic shape stays 35/8, 32/7, 35/5.
#[test]
fn the_live_player_hit_is_cast_launch_hitpoints_and_bar() {
    let data = WireData::load(&shipped()).unwrap();
    let hit = wire::PlayerHit {
        attacker: 7,
        target: 9,
        dealt: 215,
        health: 45,
        max_health: 260,
        critical: false,
        swing: true,
        ability: Some(4895),
        ..Default::default()
    };
    let live = data.player_hit_live(&hit);
    assert_eq!(live.len(), 4);
    assert_eq!(live[0], wire::cast(7, 7, 4895));
    assert_eq!(live[1], data.launch("4895", 7, 9).unwrap());
    assert_eq!(live[2], wire::hitpoints(7, 9, true, 260, 45, -215, false));
    assert_eq!(live[3][..4], [35, 0, 5, 0]);
    // No template for the ability: a swing keeps its animation, an ability hit sends none.
    let hit = wire::PlayerHit {
        ability: None,
        ..hit
    };
    let plain = data.player_hit_live(&hit);
    assert_eq!(plain.len(), 3);
    assert_eq!(plain[0], wire::animation(7, 1099, false));
    let classic = wire::player_hit(&hit);
    let heads: Vec<[u8; 4]> = classic.iter().map(|p| p[..4].try_into().unwrap()).collect();
    // rc-0137: no per-hit 35/35, as the live path above; the 32/7 moves the boss bar.
    assert_eq!(heads, [[35, 0, 8, 0], [32, 0, 7, 0], [35, 0, 5, 0]]);
    assert_eq!(classic[1][36..40], [0; 4], "the classic hit names no effect");
}

/// The real entry path: the shipped opening cave, fought through begin, zone ready, tick and
/// attack. Every hit is swing, 32/7, bar; the defeat sends the graceful remove with the
/// death effect and the delay of `combat_rules.json`.
#[test]
fn a_kill_through_the_engine_sends_the_hit_and_the_death_packets_in_order() {
    const CAVE: u32 = 411;
    let mut engine = EncounterEngine::load(shipped(), None).unwrap();
    assert_eq!(engine.wire_data().launches.len(), 27, "load_wire read the templates");
    engine.set_player_job(BRAWLER, 20);
    let world = engine.world().clone();
    let home = CharacterState {
        guid: 1,
        zone_id: world.zone_id,
        position: world.position,
        rotation: world.rotation,
    };
    engine.begin(CAVE, home, BRAWLER).unwrap();
    engine.on_zone_ready(BRAWLER, 0).unwrap();
    engine.toggle_god();
    let (target, at, _) = engine.standing_enemies()[0];
    let me = [at[0] + 1.0, at[1], at[2]];
    let mut now = 0;
    let lethal = loop {
        now += 700;
        assert!(now < 120_000, "the first enemy never fell");
        engine.tick(me, BRAWLER, now).unwrap();
        let out = engine.attack(target, me, BRAWLER, 0, now).unwrap();
        let heads: Vec<&[u8]> = out.replies.iter().map(|p| &p[..4]).collect();
        let hit = heads.iter().position(|h| *h == [32, 0, 7, 0]).expect("32/7");
        assert_eq!(heads[hit - 1], [35, 0, 8, 0], "the swing comes first");
        assert_eq!(heads[hit + 1], [35, 0, 5, 0], "then the bar");
        assert_eq!(out.replies[hit][20..28], target.to_le_bytes());
        if !out.defeated.is_empty() {
            break out;
        }
        assert!(!heads.contains(&&[35, 0, 3, 0][..]), "no remove before the defeat");
    };
    let hit = lethal.replies.iter().position(|p| p[..4] == [32, 0, 7, 0]).unwrap();
    let gone = lethal.replies.iter().position(|p| p[..4] == [35, 0, 3, 0]).unwrap();
    assert!(hit < gone, "the hit, then the remove");
    let remove = &lethal.replies[gone];
    assert_eq!(*remove, wire::death(target, 0), "delay 0 until combat_rules.json says more");
    assert_eq!(remove.len(), 31);
    assert_eq!(remove[14], 1, "animate");
    assert_eq!(remove[23..27], 5017_i32.to_le_bytes());
    assert_eq!(remove[27..31], 1000_i32.to_le_bytes());
}
