//! Work item 018: developer entry points for the supported minigames.
//!
//! The module is owned by item 018 and is included by path until the lead
//! registers `restoration::piece_018`. Every expectation below is derived from
//! already-reviewed evidence:
//!
//! * the recovered activity catalogue
//!   (`notes/Research/original-activity-coverage.json`, item 015);
//! * the accepted crafting movies (`src/{cooking,smelting,forging}.rs`);
//! * the code/test candidates (`src/{mining,harvesting,fishing,kart_racing,
//!   soccer,derby}.rs`) and their owned test targets;
//! * the reviewed `realm goto` standoff in `src/dev_teleport.rs`, which the
//!   world-actor routes reuse instead of computing their own arrival;
//! * the bound private opening content
//!   (`../lab/references/opening-anvil-placement-v1/content`), read only by
//!   the explicitly ignored tests at the end of this file.
//!
//! No native client, release build, save file or launcher script is touched.
#[path = "../src/restoration/piece_018.rs"]
mod piece_018;

use osfr_tools::{character_store::CharacterState, dev_teleport, opening::OpeningActor};
use piece_018::{ActorView, COMMAND, Command, Entry, GATE, Status, handle, resolve, route, routes};
use serde_json::Value;
use std::{fs, path::PathBuf};

/// Recipe ids bound by the recovered opening campaign content.
const COOKING_RECIPE: i32 = 2001;
const SMELTING_RECIPE: i32 = 2002;
const FORGING_RECIPE: i32 = 2003;
/// Recovered chef/miner/blacksmith profiles.
const CHEF: i32 = 45;
const MINER: i32 = 14;
const BLACKSMITH: i32 = 16;

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn project_dir() -> PathBuf {
    manifest_dir().join("../..")
}

fn content_dir() -> PathBuf {
    std::env::var_os("OSFR_PIECE_018_CONTENT")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../../lab/references/opening-anvil-placement-v1/content")
        })
}

fn content_actors() -> Vec<Value> {
    let opening: Value =
        serde_json::from_slice(&fs::read(content_dir().join("opening.json")).unwrap()).unwrap();
    opening["actors"].as_array().unwrap().clone()
}

fn content_actor(name: &str) -> Value {
    content_actors()
        .into_iter()
        .find(|a| a["name"].as_str() == Some(name))
        .unwrap_or_else(|| panic!("campaign actor {name}"))
}

fn float3(value: &Value) -> [f32; 3] {
    let a = value.as_array().unwrap();
    [
        a[0].as_f64().unwrap() as f32,
        a[1].as_f64().unwrap() as f32,
        a[2].as_f64().unwrap() as f32,
    ]
}

fn actor(guid: u64, name: &str, position: [f32; 3]) -> OpeningActor {
    OpeningActor {
        guid,
        definition_id: 1,
        name_id: 1,
        name: name.into(),
        model_id: 1,
        model_file_name: String::new(),
        zone_id: 0,
        position,
        heading: 0.0,
        interaction_radius: 12.0,
        kind: "resource".into(),
        action: None,
        required_profile: None,
        provenance: String::new(),
    }
}

fn character(position: [f32; 3]) -> CharacterState {
    CharacterState {
        guid: 1,
        zone_id: 0,
        position,
        rotation: [1.0, 0.0, 0.0],
    }
}

fn horizontal(left: [f32; 3], right: [f32; 3]) -> f32 {
    ((left[0] - right[0]).powi(2) + (left[2] - right[2]).powi(2)).sqrt()
}

fn actor_views<'a>(actors: &'a [OpeningActor]) -> Vec<ActorView<'a>> {
    actors
        .iter()
        .map(|a| ActorView {
            name: &a.name,
            guid: a.guid,
            zone_id: a.zone_id,
        })
        .collect()
}

fn reply_text(command: Command) -> String {
    match command {
        Command::Reply(text) => text,
        Command::Route { reply, .. } => reply,
    }
}

/// The crafting movies are started by interacting with a loaded world actor.
fn crafting_actors() -> Vec<OpeningActor> {
    vec![
        actor(
            5_715_144_128_442_925_070,
            "Cooking Table",
            [-2109.244, -56.61908, 210.265],
        ),
        actor(
            5_715_144_128_442_925_072,
            "Smelt 5 Copper Bars",
            [-2076.0, -55.6, 190.0],
        ),
        actor(5_715_144_128_442_925_073, "Anvil", [-2088.0, -54.79, 200.0]),
    ]
}

fn crafting_route(slug: &str) -> (&'static str, i32, i32) {
    let route = route(slug).expect("crafting route");
    match route.entry {
        Entry::WorldActor {
            actor,
            profile,
            recipe,
        } => (actor, profile, recipe),
        other => panic!("{slug} is not a world-actor route: {other:?}"),
    }
}

#[test]
fn developer_route_gate_defaults_off() {
    assert_eq!(GATE, "--dev-tools");
    assert_eq!(COMMAND, "game");
    let actors = crafting_actors();
    let views = actor_views(&actors);
    assert!(handle(false, "", &views).is_none());
    assert!(handle(false, "game", &views).is_none());
    assert!(handle(false, "game list", &views).is_none());
    for route in routes() {
        let text = format!("game {}", route.slug);
        assert!(
            handle(false, &text, &views).is_none(),
            "{text} must stay off without the developer gate"
        );
    }
}

#[test]
fn route_table_covers_every_supported_game_once() {
    let expected = [
        ("cooking", Status::NativeAccepted),
        ("smelting", Status::NativeAccepted),
        ("forging", Status::NativeAccepted),
        ("mining", Status::CodeTestCandidate),
        ("harvesting", Status::CodeTestCandidate),
        ("fishing", Status::CodeTestCandidate),
        ("kart_racing", Status::CodeTestCandidate),
        ("soccer", Status::CodeTestCandidate),
        ("derby", Status::WireStudyOnly),
    ];
    assert_eq!(routes().len(), expected.len());
    for (route, (slug, status)) in routes().iter().zip(expected) {
        assert_eq!(route.slug, slug);
        assert_eq!(route.status, status);
        assert!(!route.original_games.is_empty());
        assert!(!route.native_gate.is_empty());
        assert!(
            route
                .slug
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b == b'_')
        );
        for alias in route.aliases {
            assert!(
                routes()
                    .iter()
                    .filter(|r| r.aliases.contains(alias))
                    .count()
                    == 1,
                "alias {alias} is shared"
            );
        }
    }

    let world: Vec<_> = routes()
        .iter()
        .filter(|r| matches!(r.entry, Entry::WorldActor { .. }))
        .map(|r| r.slug)
        .collect();
    assert_eq!(world, ["cooking", "smelting", "forging"]);
    assert_eq!(
        crafting_route("cooking"),
        ("Cooking Table", CHEF, COOKING_RECIPE)
    );
    assert_eq!(
        crafting_route("smelting"),
        ("Smelt 5 Copper Bars", MINER, SMELTING_RECIPE)
    );
    assert_eq!(
        crafting_route("forging"),
        ("Anvil", BLACKSMITH, FORGING_RECIPE)
    );
}

#[test]
fn every_route_cites_existing_repository_evidence() {
    let root = project_dir();
    for route in routes() {
        for path in std::iter::once(route.module_path)
            .chain(route.tests.iter().copied())
            .chain(std::iter::once(route.notes))
        {
            let full = root.join(path);
            assert!(full.is_file(), "{path} for {} does not exist", route.slug);
        }
    }
}

#[test]
fn route_game_ids_match_the_recovered_activity_catalogue() {
    let catalogue: Value = serde_json::from_slice(
        &fs::read(project_dir().join("notes/Research/original-activity-coverage.json")).unwrap(),
    )
    .unwrap();
    let games = catalogue["games"].as_array().unwrap();
    for route in routes() {
        for id in route.original_games {
            let row = games
                .iter()
                .find(|g| g["id"].as_str() == Some(id.to_string().as_str()))
                .unwrap_or_else(|| panic!("{} cites missing game {id}", route.slug));
            assert!(
                row["name"].as_str().is_some_and(|n| !n.is_empty()),
                "{} cites unnamed game {id}",
                route.slug
            );
            assert_eq!(
                row["type"].as_str().unwrap().parse::<u32>().unwrap(),
                route.original_type as u32,
                "{} cites game {id} of the wrong type",
                route.slug
            );
        }
        let unique: std::collections::BTreeSet<_> = route.original_games.iter().collect();
        assert_eq!(unique.len(), route.original_games.len());
    }
}

#[test]
fn forging_row_139_is_type_8_match_three() {
    let forging = route("forging").unwrap();
    assert_eq!(forging.original_games, &[139]);
    assert_eq!(forging.original_type, 8);
    assert_eq!(forging.family, "TYPE 8 MatchThree (Forging row 139)");

    let catalogue: Value = serde_json::from_slice(
        &fs::read(project_dir().join("notes/Research/original-activity-coverage.json")).unwrap(),
    )
    .unwrap();
    let row = catalogue["games"]
        .as_array()
        .unwrap()
        .iter()
        .find(|game| game["id"].as_str() == Some("139"))
        .expect("Forging source row 139");
    assert_eq!(row["type"].as_str(), Some("8"));
}

#[test]
fn acceptance_status_is_limited_to_the_three_reviewed_movies() {
    for route in routes() {
        match route.status {
            Status::NativeAccepted => {
                assert!(
                    ["cooking", "smelting", "forging"].contains(&route.slug),
                    "{} is not a reviewed native-accepted movie",
                    route.slug
                );
                assert!(matches!(route.entry, Entry::WorldActor { .. }));
            }
            Status::CodeTestCandidate => {
                assert!(matches!(route.entry, Entry::ModuleDriver { .. }));
            }
            Status::WireStudyOnly => {
                assert_eq!(route.slug, "derby");
                assert!(matches!(route.entry, Entry::Unrecovered { .. }));
            }
        }
    }
}

#[test]
fn exact_alias_and_unique_prefix_resolve_without_guessing() {
    assert_eq!(resolve("cooking").unwrap().slug, "cooking");
    assert_eq!(resolve("cook").unwrap().slug, "cooking");
    assert_eq!(resolve("sm").unwrap().slug, "smelting");
    assert_eq!(resolve("racing").unwrap().slug, "kart_racing");
    assert!(resolve("s").is_err());
    assert!(resolve("nope").is_err());
    assert!(resolve("COOKING").is_err() || resolve("COOKING").unwrap().slug == "cooking");
}

#[test]
fn unknown_and_foreign_commands_are_left_alone() {
    let actors = crafting_actors();
    let views = actor_views(&actors);
    // Anything that is not our command body stays with the existing handlers.
    assert!(handle(true, "", &views).is_none());
    assert!(handle(true, "gamex", &views).is_none());
    assert!(handle(true, "goto mae", &views).is_none());
    // A real miss answers with candidates instead of moving anywhere.
    let reply = reply_text(handle(true, "game nope", &views).unwrap());
    for route in routes() {
        assert!(reply.contains(route.slug), "{reply} misses {}", route.slug);
    }
}

#[test]
fn crafting_routes_compose_with_the_reviewed_teleport_standoff() {
    let actors = crafting_actors();
    let teleport_targets: Vec<_> = actors.iter().map(dev_teleport::Target::from).collect();
    let views = actor_views(&actors);
    let mut current = character([-1000.0, -40.0, 1000.0]);
    for slug in ["cooking", "smelting", "forging"] {
        let (name, _, _) = crafting_route(slug);
        let command = handle(true, &format!("game {slug}"), &views).unwrap();
        let Command::Route { reply, actor, guid } = command else {
            panic!("{slug} must produce a relocation route");
        };
        assert_eq!(actor, name);
        assert!(reply.contains(name) && reply.contains(&guid.to_string()));

        let target = actors.iter().find(|a| a.name == name).unwrap();
        let relocation = dev_teleport::handle(&format!("goto {name}"), &teleport_targets, &current)
            .and_then(|c| match c {
                dev_teleport::Command::Relocate { relocation, .. } => Some(relocation),
                dev_teleport::Command::Reply(text) => panic!("{slug} refused: {text}"),
            })
            .expect("reviewed standoff");
        let distance = horizontal(relocation.position, target.position);
        assert!(
            (distance - dev_teleport::STANDOFF).abs() < 0.01,
            "{slug} arrived {distance}m away"
        );
        assert!(distance <= target.interaction_radius);
        assert!(relocation.position.iter().all(|c| c.is_finite()));
        assert!(
            relocation.position[1] == target.position[1],
            "the standoff keeps the actor's own Y"
        );

        // The sampler starts on the approach side, so a repeated route stays put.
        current.position = relocation.position;
        let again = dev_teleport::handle(&format!("goto {name}"), &teleport_targets, &current)
            .and_then(|c| match c {
                dev_teleport::Command::Relocate { relocation, .. } => Some(relocation.position),
                dev_teleport::Command::Reply(text) => panic!("{slug} refused on replay: {text}"),
            })
            .unwrap();
        for (left, right) in again.into_iter().zip(relocation.position) {
            assert!((left - right).abs() < 0.01);
        }
    }
}

#[test]
fn world_route_refuses_a_missing_entry_actor() {
    let actors = vec![actor(7, "Bartle", [-1860.185, -51.3044, 340.0])];
    let views = actor_views(&actors);
    let command = handle(true, "game cooking", &views).unwrap();
    let text = reply_text(command);
    assert!(text.contains("Cooking Table"));
    assert!(text.contains("Nothing changed"), "{text}");
}

#[test]
fn candidate_routes_report_driver_or_gap_without_relocation() {
    let actors = crafting_actors();
    let views = actor_views(&actors);
    for slug in ["mining", "harvesting", "fishing", "kart_racing", "soccer"] {
        let command = handle(true, &format!("game {slug}"), &views).unwrap();
        let Entry::ModuleDriver { api, driver } = route(slug).unwrap().entry else {
            panic!("{slug} must be a module driver");
        };
        let text = reply_text(command);
        assert!(text.contains(api), "{text}");
        assert!(text.contains(driver), "{text}");
        assert!(text.contains(route(slug).unwrap().module_path), "{text}");
    }
    let text = reply_text(handle(true, "game derby", &views).unwrap());
    let Entry::Unrecovered { missing } = route("derby").unwrap().entry else {
        panic!("derby must be unrecovered");
    };
    assert!(text.contains(missing), "{text}");
}

#[test]
fn every_reply_carries_the_route_native_gate() {
    let actors = crafting_actors();
    let views = actor_views(&actors);
    for route in routes() {
        let text = reply_text(handle(true, &format!("game {}", route.slug), &views).unwrap());
        assert!(
            text.contains(route.native_gate),
            "{} hides its gate",
            route.slug
        );
        if route.status != Status::NativeAccepted {
            let lowered = text.to_ascii_lowercase();
            for claim in ["native-accepted", "playable", "native accepted"] {
                assert!(
                    !lowered.contains(claim),
                    "{} overclaims: {text}",
                    route.slug
                );
            }
        }
    }
    let listing = reply_text(handle(true, "game", &views).unwrap());
    for route in routes() {
        assert!(
            listing.contains(route.slug),
            "{listing} misses {}",
            route.slug
        );
    }
}

/// Private-content checks. Run explicitly:
/// `cargo test --test piece_018 -- --ignored --test-threads=1`.
#[test]
#[ignore = "requires the private bound opening campaign; no native launch"]
fn crafting_entries_match_the_bound_private_content() {
    for slug in ["cooking", "smelting", "forging"] {
        let (name, profile, recipe) = crafting_route(slug);
        let row = content_actor(name);
        assert_eq!(row["zone_id"].as_u64().unwrap(), 0);
        assert_eq!(row["required_profile"].as_i64().unwrap(), profile as i64);
        assert_eq!(row["action"]["kind"].as_str().unwrap(), "craft");
        assert_eq!(row["action"]["recipe_id"].as_i64().unwrap(), recipe as i64);
        assert_eq!(row["interaction_radius"].as_f64().unwrap(), 12.0);
        assert!(float3(&row["position"]).iter().all(|c| c.is_finite()));

        let actors = content_actors();
        let views: Vec<ActorView<'_>> = actors
            .iter()
            .map(|a| ActorView {
                name: a["name"].as_str().unwrap(),
                guid: a["guid"].as_u64().unwrap(),
                zone_id: a["zone_id"].as_u64().unwrap() as u32,
            })
            .collect();
        let Command::Route { guid, .. } = handle(true, &format!("game {slug}"), &views).unwrap()
        else {
            panic!("{slug} must relocate against the bound content");
        };
        assert_eq!(guid, row["guid"].as_u64().unwrap());
    }
}

#[test]
#[ignore = "requires the private bound opening campaign; no native launch"]
fn private_content_composition_reaches_the_entry_actor() {
    let rows = content_actors();
    let actors: Vec<OpeningActor> = rows
        .iter()
        .map(|row| {
            let mut actor = actor(
                row["guid"].as_u64().unwrap(),
                row["name"].as_str().unwrap(),
                float3(&row["position"]),
            );
            actor.zone_id = row["zone_id"].as_u64().unwrap() as u32;
            actor.interaction_radius = row["interaction_radius"].as_f64().unwrap() as f32;
            actor
        })
        .collect();
    let teleport_targets: Vec<_> = actors.iter().map(dev_teleport::Target::from).collect();
    let views = actor_views(&actors);
    let mut character = character([-1054.135, 4.629629, 305.7209]);
    for slug in ["cooking", "smelting", "forging"] {
        let (name, _, _) = crafting_route(slug);
        let Command::Route { actor, .. } = handle(true, &format!("game {slug}"), &views).unwrap()
        else {
            panic!("{slug} must relocate against the bound content");
        };
        let target = actors.iter().find(|a| a.name == actor).unwrap();
        let dev_teleport::Command::Relocate { relocation, .. } =
            dev_teleport::handle(&format!("goto {name}"), &teleport_targets, &character).unwrap()
        else {
            panic!("{slug} refused against the bound content");
        };
        assert!(horizontal(relocation.position, target.position) <= target.interaction_radius);
        character.position = relocation.position;
    }
}
