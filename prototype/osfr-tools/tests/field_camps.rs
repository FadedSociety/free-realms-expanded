//! T892: the Legacy overworld spawns that `npc_spawns_extra.json` drew as friendly NPC actors
//! although they are enemies (Sandscale, Skullz, cultists, crays, widows, ...) fight through the
//! field enemy system: `field_camps.json` takes each body over by `spawn_guid`, so it keeps its
//! place and model, shows a hostile name, can be fought and respawns, and the passive table no
//! longer draws it. Hooligan family bodies belong to T889; bosses, Group N enemies without a
//! definition and the Stone Heart battle marker are not routed here.
//!
//! The fight runs through the real 26/8 -> Attack -> 26/10 path (`kill_field_enemy`); a body is
//! friendly exactly when the passive visibility table still knows it (`Gameplay::npc_model`).

#[path = "common/field_camp_centres.rs"]
mod field_camp_centres;
#[path = "common/field_enemy_fixture.rs"]
mod field_enemy_fixture;
#[path = "common/job_playtest.rs"]
mod job_playtest;
#[path = "support/mining_quest_chain.rs"]
mod mining_quest_chain;
#[path = "support/npc_quest_chain.rs"]
mod npc_quest_chain;
#[path = "support/quest_marker_packets.rs"]
mod quest_marker_packets;
#[path = "support/scratch.rs"]
mod scratch;

use field_camp_centres::VERIFIED_CENTRES;
use field_enemy_fixture::Fixture as EngineFixture;
use npc_quest_chain::{ADVENTURER, CHARACTER, character_at_for, content_dir, read_json};
use osfr_tools::gameplay::Gameplay;
use quest_marker_packets::spawn_index;
use scratch::Scratch;
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    sync::LazyLock,
};

const MINER: i32 = 14;
const BRAWLER: i32 = 43;
/// The 4 m chase hysteresis in the field tick (`encounter/field.rs`).
const CHASE_MARGIN: f32 = 4.0;
/// The top of the starter bands; a player of these levels meets only its own band at the
/// starter camps (T415, `field_camp_separation.rs`).
const STARTER_TOP: u64 = 4;
/// (R) The clearance T415 keeps between a higher band and where a low-level player goes.
const LOW_LEVEL_MARGIN: f32 = 20.0;

static EXTRAS: LazyLock<BTreeMap<u64, Value>> = LazyLock::new(|| {
    read_json(&content_dir().join("npc_spawns_extra.json"))["records"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| (row["spawn_guid"].as_u64().unwrap(), row.clone()))
        .collect()
});

fn camps() -> Vec<Value> {
    read_json(&content_dir().join("field_camps.json"))["camps"]
        .as_array()
        .unwrap()
        .clone()
}

fn members(camp: &Value) -> impl Iterator<Item = u64> + '_ {
    camp["members"]
        .as_array()
        .unwrap()
        .iter()
        .map(|guid| guid.as_u64().unwrap())
}

fn home(guid: u64) -> [f32; 3] {
    serde_json::from_value(EXTRAS[&guid]["position"].clone()).unwrap()
}

/// The named Sandscale bodies of Sunstone Valley (27 Scavengers and 18 Hunters) with the
/// respawn timer of the camp that took them over.
fn sandscale() -> Vec<(u64, [f32; 3], u64)> {
    camps()
        .iter()
        .flat_map(|camp| {
            let respawn_ms = camp["respawn_seconds"].as_u64().unwrap() * 1000;
            members(camp)
                .filter(|guid| {
                    matches!(
                        EXTRAS[guid]["name"].as_str(),
                        Some("Sandscale Scavenger" | "Sandscale Hunter")
                    )
                })
                .map(move |guid| (guid, home(guid), respawn_ms))
                .collect::<Vec<_>>()
        })
        .collect()
}

/// The shipped content as a Brawler plays it; the fixture owns the content copy.
fn rig() -> (mining_quest_chain::Fixture, Gameplay) {
    let fixture = mining_quest_chain::Fixture::new();
    let game = fixture.game(BRAWLER, &[ADVENTURER, MINER, BRAWLER]);
    (fixture, game)
}

#[test]
fn the_sandscale_camp_fights_and_respawns() {
    let camp = sandscale();
    assert_eq!(camp.len(), 45, "Sunstone Valley's Sandscale camp");
    let (_fixture, mut game) = rig();
    let mut now = 1_000;

    // It fights back: standing at a body, the camp hits the player.
    let (first, at, _) = camp[0];
    let character = character_at_for(CHARACTER, 0, at);
    let hits: Vec<u32> = (0..30)
        .flat_map(|step| game.tick(&character, now + step * 100).unwrap().replies)
        .flat_map(|packet| job_playtest::hits_from(&[packet], first))
        .collect();
    assert!(
        hits.iter().any(|damage| *damage > 0),
        "{first} swings at a player in reach"
    );
    now += 3_100;

    // Every body is fightable, dies, and returns exactly on its camp's timer.
    let away = character_at_for(CHARACTER, 0, [10_000.0, 0.0, 10_000.0]);
    for (guid, at, respawn_ms) in camp {
        assert!(
            npc_quest_chain::kill_field_enemy(&mut game, at, guid, &mut now),
            "{guid} dies to basic attacks"
        );
        let killed = now;
        let early = game.tick(&away, killed + respawn_ms - 1_000).unwrap();
        assert!(
            spawn_index(&early.replies, guid).is_none(),
            "{guid} returned early"
        );
        let due = game.tick(&away, killed + respawn_ms + 1_000).unwrap();
        assert!(
            spawn_index(&due.replies, guid).is_some(),
            "{guid} did not respawn"
        );
        now = killed + respawn_ms + 1_000;
    }
}

#[test]
fn camp_bodies_are_no_longer_drawn_as_friendly_actors() {
    let camps = camps();
    let guids: Vec<u64> = camps.iter().flat_map(members).collect();
    assert!(guids.len() > 300, "the routed census records");

    // Control: without the camp file the same bodies are friendly actors.
    let control = npc_quest_chain::Fixture::new().with_field_enemies(&[]);
    let without = control.game(None);
    for guid in &guids {
        assert!(without.npc_model(*guid).is_some(), "{guid} is drawn today");
    }

    let (_fixture, routed) = rig();
    for guid in &guids {
        assert_eq!(routed.npc_model(*guid), None, "{guid} is still friendly");
    }
}

#[test]
fn every_named_enemy_body_is_routed_unless_it_belongs_elsewhere() {
    // Not routed here: the Hooligan family (T889), bosses (a boss cannot be a field enemy), the
    // census Group N enemies that have no definition yet, and the Stone Heart battle marker.
    let elsewhere = [
        "Hooligan",
        "Hooligan Archer",
        "Hooligan Wolf",
        "Hooligan Bullies!",
        "Alpha Wolf",
        "Surfmonger Arthur",
        "The Pumpkin Prince",
        "Stone Heart's Hideaway",
        "Ninja Apprentice",
        "Robgoblin Kaboomer",
        "Robgoblin Tinkerer",
        "Robgoblin Mechanic",
        "Robgoblin Shinyseeker",
        "Mini Necrowart Zombie",
        "Skullz Roughrider",
        "Jintaka",
        "King of Low Tide",
        "Three of Three",
    ];
    // The census's named enemy records this task routes.
    let routed = [
        "Skullz Thug",
        "Skullz Hooligan",
        "Skullz Commander",
        "Sandscale Scavenger",
        "Sandscale Hunter",
        "Sandscale Water Seeker",
        "Cultist Initiate",
        "Fairy Cultist",
        "Cray Snapper",
        "Cray Bubbler",
        "Ghost Ninja",
        "Ghost Ninja Striker",
        "Floren Leafling",
        "Floren Defender",
        "Bitterleaf",
        "Scavenger Swooper",
        "Lazy Swooper",
        "Nettleseed Pest",
        "Nettleseed Nibbler",
        "Bristlewood Vulture",
        "Bristlewood Arachnid",
        "Stealthy Despoiler",
        "Shady Smuggler",
        "Troll Guard",
        "Frostfang Snarler",
        "Evil Postman",
        "Brutus the Brute",
        "Thugawug Thug!",
        "Snake",
    ];
    let camps = camps();
    let taken: BTreeSet<u64> = camps.iter().flat_map(members).collect();
    let mut seen = BTreeSet::new();
    for (guid, row) in EXTRAS.iter() {
        let Some(name) = row["name"].as_str() else {
            continue;
        };
        if routed.contains(&name) {
            seen.insert(name);
            // A Shady Smuggler on the boss smuggler's spot is the one named exception.
            let boss_spot = name == "Shady Smuggler" && row["legacy_npc_id"] == 2834;
            assert!(
                taken.contains(guid) || boss_spot,
                "{name} {guid} still stands as a friendly actor"
            );
        }
        if elsewhere.contains(&name) {
            assert!(!taken.contains(guid), "{name} {guid} is not routed here");
        }
    }
    assert_eq!(seen.len(), routed.len(), "every routed name has bodies");
}

#[test]
fn camps_keep_to_their_places_in_the_level_bands() {
    // No camp body above the starter bands stands where a low-level player goes: a starter camp,
    // a difficulty 1-2 battle marker or an opening stage enemy (T415 clearance), and no body is
    // within camp-overlap range of a field actor whose band is more than two levels away.
    let battles = read_json(&content_dir().join("field_battles.json"));
    let encounters = read_json(&content_dir().join("encounters.json"));
    let field = read_json(&content_dir().join("field_enemies.json"));
    let xz = |at: &Value| {
        [
            at[0].as_f64().unwrap() as f32,
            at[2].as_f64().unwrap() as f32,
        ]
    };
    let mut low: Vec<[f32; 2]> = VERIFIED_CENTRES[..12]
        .iter()
        .map(|(_, at)| [at[0], at[2]])
        .collect();
    low.extend(
        battles["battles"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|battle| battle["difficulty"].as_u64().unwrap() <= 2)
            .map(|battle| xz(&battle["spawner"]["position"])),
    );
    low.extend(
        encounters["world_enemies"]
            .as_array()
            .unwrap()
            .iter()
            .map(|enemy| xz(&enemy["position"])),
    );
    let groups = field["spawn_groups"].as_array().unwrap();
    // Each shipped field actor: its level band (its group's, else its own level) and place.
    let existing: Vec<(u64, u64, [f32; 2])> = field["enemies"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| {
            let group = groups
                .iter()
                .find(|g| g["members"].as_array().unwrap().contains(&row["guid"]));
            let level = row["level"].as_u64().unwrap_or(1);
            let [lo, hi] = group.map_or([level, level], |g| {
                [
                    g["level_band"][0].as_u64().unwrap(),
                    g["level_band"][1].as_u64().unwrap(),
                ]
            });
            (lo, hi, xz(&row["position"]))
        })
        .collect();
    let distance = |a: [f32; 2], b: [f32; 2]| (a[0] - b[0]).hypot(a[1] - b[1]);
    let camps = camps();
    for camp in &camps {
        let level = camp["level"].as_u64().unwrap();
        let leash = camp["leash_radius"].as_f64().unwrap() as f32;
        let aggro = camp["aggro_range"].as_f64().unwrap() as f32;
        for guid in members(camp) {
            let at = xz(&json!(home(guid)));
            if level > STARTER_TOP {
                let need = leash + aggro + CHASE_MARGIN + LOW_LEVEL_MARGIN;
                for place in &low {
                    assert!(
                        distance(at, *place) >= need,
                        "level {level} body {guid} is {:.1} m from a low-level place",
                        distance(at, *place)
                    );
                }
            }
            for (lo, hi, other) in &existing {
                if *lo > level + 2 || level > hi + 2 {
                    // The shipped field actors share the camps' 12 m leash and 8 m aggro.
                    let need = leash + leash + aggro + CHASE_MARGIN;
                    assert!(
                        distance(at, *other) >= need,
                        "level {level} body {guid} is {:.1} m from a band {lo}-{hi} camp",
                        distance(at, *other)
                    );
                }
            }
        }
    }
}

/// A scratch content root holding what `load_field_camps` reads, with the camp file `edit`ed.
fn camp_root(edit: impl FnOnce(&mut Value)) -> Scratch {
    let root = Scratch::new("t892-camps");
    for name in [
        "enemies.json",
        "npc_spawns_extra.json",
        "field_enemies.json",
    ] {
        fs::copy(content_dir().join(name), root.path().join(name)).unwrap();
    }
    let mut camps = read_json(&content_dir().join("field_camps.json"));
    edit(&mut camps);
    fs::write(root.path().join("field_camps.json"), camps.to_string()).unwrap();
    root
}

#[test]
fn a_refused_camp_file_changes_nothing() {
    let refusals: [(&str, fn(&mut Value)); 6] = [
        ("a boss definition", |c| {
            c["camps"][0]["definition"] = json!("alpha_wolf");
        }),
        ("an unknown definition", |c| {
            c["camps"][0]["definition"] = json!("no_such_enemy");
        }),
        ("a body that is not an extras row", |c| {
            c["camps"][0]["members"][0] = json!(1_u64 << 40);
        }),
        ("a guid that a field enemy already holds", |c| {
            c["camps"][0]["members"][0] = json!(100_000_004_154_u64);
        }),
        ("a source with no (R) marker", |c| {
            c["camps"][0]["source"] = json!([{"kind": "reconstructed", "evidence": "a guess"}]);
        }),
        ("a level past the table", |c| {
            c["camps"][0]["level"] = json!(21);
        }),
    ];
    for (what, edit) in refusals {
        let root = camp_root(edit);
        let mut engine = EngineFixture::shipped().load();
        let before = engine.field_guids();
        assert!(
            engine.load_field_camps(root.path()).is_err(),
            "{what} must be refused"
        );
        assert_eq!(engine.field_guids(), before, "{what} changed the engine");
    }
}

#[test]
fn camps_load_once_and_survive_a_field_reload() {
    let root = camp_root(|_| {});
    let mut engine = EngineFixture::shipped().load();
    let field_rows = engine.field_guids().len();
    engine.load_field_camps(root.path()).unwrap();
    let camps = camps();
    let camp_rows: usize = camps.iter().map(|camp| members(camp).count()).sum();
    assert_eq!(engine.field_guids().len(), field_rows + camp_rows);
    // Loading the same camps again would stack duplicates: refused.
    assert!(engine.load_field_camps(root.path()).is_err());
    assert_eq!(engine.field_guids().len(), field_rows + camp_rows);
    // The developer reload re-reads the field file and its camps together.
    engine
        .reload_field_enemies(&root.path().join("field_enemies.json"))
        .unwrap();
    assert_eq!(engine.field_guids().len(), field_rows + camp_rows);
}
