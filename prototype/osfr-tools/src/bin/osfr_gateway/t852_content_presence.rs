//! T852: a release server starts with the repository's `data/content` alone.
//!
//! The clean-install launcher shipped `data/content` and the gateway exited at once
//! (`mining requires opening content`) because gameplay only ever ran on lab packages that add
//! the captured modules. This module drives the real load path: `Session::new` ->
//! `load_gateway` -> `Gameplay::load` with no `--reference` and no lab content, then the login
//! from `startup/manifest.json`, then Ready and Loaded to world entry.
//!
//! It pins the two sides of that rule. A module whose own file or prerequisite is absent is
//! skipped: one `gameplay_module_skipped` event names it, it stays inactive, and `Gameplay::state`
//! writes its saved state back unchanged. A module whose file IS present stays fail-closed: a
//! version or shape that disagrees with the loader refuses the load instead of half-loading.
use super::test_support::{self, Scratch};
use super::*;
// The shared content-root helper (`tests/support`), not a second copy of the same join.
#[path = "../../../tests/support/npc_visibility.rs"]
mod npc_visibility;
use osfr_tools::{
    encounter::EncounterState,
    game_store::Snapshot,
    gameplay::{Gameplay, GameplayState},
    inventory::Inventory,
    mining::Type8State,
    profiles::ProfileState,
};
use std::collections::{BTreeMap, BTreeSet};

fn shipped_options(state: PathBuf) -> Options {
    let mut options = test_support::options(state);
    // Content mode: no capture folder and no lab package, exactly like a clean install.
    options.reference = PathBuf::new();
    options.content = Some(npc_visibility::content_dir());
    options
}

#[test]
fn shipped_content_alone_reaches_world_entry_and_logs_every_skipped_module() {
    if !test_support::bin_test_target("t852") {
        return;
    }
    let scratch = Scratch::new("t852-shipped");
    let options = shipped_options(scratch.state());
    let mut scene = test_support::Scene::new(&options, 1);

    // The load bound gameplay from `data/content` and logged one record per skipped module.
    let records = scene
        .session
        .gateway
        .gameplay()
        .expect("content mode binds gameplay")
        .skipped_modules()
        .to_vec();
    let skipped: BTreeSet<(String, String)> = records
        .iter()
        .map(|record| {
            assert_eq!(record["event"], "gameplay_module_skipped");
            (
                record["module"].as_str().unwrap_or_default().to_owned(),
                record["missing"].as_str().unwrap_or_default().to_owned(),
            )
        })
        .collect();
    let expected: BTreeSet<(String, String)> = [
        ("profiles", "profiles.json"),
        ("opening", "opening.json"),
        ("smelting", "smelting.json"),
        ("forging", "forging-campaign.json"),
        ("cooking", "cooking-campaign.json"),
        ("mining", "the opening campaign"),
        ("harvesting", "the opening campaign"),
        ("inventory", "inventory.json"),
        ("categories", "categories.json"),
        ("quests", "quests.json"),
        ("combat", "combat.json"),
    ]
    .into_iter()
    .map(|(module, missing)| (module.to_owned(), missing.to_owned()))
    .collect();
    assert_eq!(
        records.len(),
        expected.len(),
        "one record per skipped module"
    );
    assert_eq!(skipped, expected);

    let replies = test_support::enter_world(&mut scene, &options);
    assert!(!replies.is_empty(), "world entry answers the client");
    // Inner application replies still carry the client envelope `[5, 0, 1]` and its length.
    let answers = |family: [u8; 2], sub: [u8; 2]| {
        replies.iter().any(|reply| {
            reply.len() >= 11
                && reply[..3] == [5, 0, 1]
                && reply[7..9] == family
                && reply[9..11] == sub
        })
    };
    // `Gameplay::initialization` ran (its system chat) and the shipped NPC placement fed the
    // client's world (35/2 AddNpc).
    assert!(answers([15, 0], [3, 0]), "the world-entry chat is sent");
    assert!(answers([35, 0], [2, 0]), "the shipped NPCs spawn");
    // The shipped job browser (`job_browser.json`, bound beside the split quest content) is
    // `u16 147`, `u8 1`, `u32 count`, one seven-i32 row each; those rows are content.
    let jobs = replies
        .iter()
        .find(|reply| reply.len() >= 14 && reply[..3] == [5, 0, 1] && reply[7..10] == [147, 0, 1])
        .expect("the shipped job list is sent");
    assert!(
        u32::from_le_bytes(jobs[10..14].try_into().unwrap()) > 0,
        "the shipped job rows are published"
    );
    assert_eq!(
        scene.session.gateway.state().guid,
        1,
        "the fresh character from the startup manifest entered"
    );
}

#[test]
fn a_skipped_modules_saved_state_survives_login_and_the_module_stays_inactive() {
    if !test_support::bin_test_target("t852") {
        return;
    }
    let scratch = Scratch::new("t852-dormant");
    let options = shipped_options(scratch.state());
    let content = options.content.clone().unwrap();
    let character = Gateway::content_state(&content).expect("the startup manifest names it");
    // What a character written by a server WITH those modules holds in its save.
    let profiles = ProfileState {
        version: 1,
        active: 43,
        unlocked: BTreeSet::from([43]),
        experience: BTreeMap::from([(43, 120)]),
    };
    let mining = Type8State {
        completed: 3,
        ..Type8State::default()
    };
    // The opening campaign's own saved state: a cave encounter a bigger content set wrote.
    let encounter = EncounterState {
        version: 1,
        content_sha256: "a-campaign-this-server-does-not-carry".into(),
        active: None,
        world_stage: 2,
        world_enemies: Vec::new(),
        player_health: 0,
        rewarded: BTreeSet::new(),
        next_instance_id: 1,
    };
    let mut saved: GameplayState = Gameplay::load(&content, None)
        .expect("shipped content loads")
        .state();
    saved.profiles = Some(profiles.clone());
    saved.encounter = Some(encounter.clone());
    saved.mining = Some(mining.clone());
    saved.inventory.coins = 1_234;
    test_support::save_character(&options.state, &character, &saved);

    let mut scene = test_support::Scene::new(&options, 1);
    let replies = test_support::enter_world(&mut scene, &options);
    assert!(!replies.is_empty(), "a dormant save still enters the world");
    let game = scene.session.gateway.gameplay().expect("gameplay is bound");
    // The modules are inactive: the engine fields are absent (the summary reads the live
    // modules, not the state written back to the save).
    assert!(
        game.summary()["profiles"].is_null(),
        "profiles stays inactive"
    );
    assert!(game.summary()["mining"].is_null(), "mining stays inactive");
    assert!(
        game.summary()["opening"].is_null(),
        "opening stays inactive"
    );

    let Snapshot {
        gameplay: Some(again),
        ..
    } = load_or_create(&options.state, character).expect("the save reads back")
    else {
        panic!("the login keeps the gameplay save");
    };
    // Nothing was wiped: the skipped modules' state and the item currency are what they were.
    assert_eq!(again.profiles, Some(profiles));
    assert_eq!(again.encounter, Some(encounter));
    assert_eq!(again.mining, Some(mining));
    assert_eq!(again.inventory.coins, 1_234);

    // Economy safety (T852): without definitions nothing can be granted, and a refused grant
    // changes nothing: no item, no coin.
    let mut empty = Inventory::without_content(Some(saved.inventory.clone()));
    let before = empty.state().clone();
    assert!(empty.grant(0, 12_345, 1).is_err(), "no definition, no item");
    assert!(
        empty.grant_bundle(7, &[(12_345, 1)]).is_err(),
        "a bundle with an unknown item refuses whole"
    );
    assert_eq!(empty.state(), &before, "a refused grant changes nothing");
}

#[test]
fn present_but_inconsistent_content_still_refuses_the_load() {
    if !test_support::bin_test_target("t852") {
        return;
    }
    // One example per fail-closed check this load keeps: the file is THERE, so skipping it
    // would run half-matching content. A version the loader does not know and a shape it
    // cannot read both refuse; only an absent file is a skip now.
    for (file, body, expected) in [
        (
            "categories.json",
            r#"{"schema_version":2,"categories":[],"groups":[],"packet_sha256":""}"#,
            "unsupported item category content",
        ),
        ("inventory.json", r#"{"schema_version":1}"#, "missing field"),
        (
            "quests.json",
            r#"{"schema_version":1}"#,
            "invalid quest content JSON",
        ),
        ("combat.json", r#"{"schema_version":1}"#, "missing field"),
        ("profiles.json", r#"{"schema_version":1}"#, "missing field"),
    ] {
        let scratch = Scratch::new("t852-inconsistent");
        fs::write(scratch.path().join(file), body).unwrap();
        let before = written(scratch.path());
        let error = Gameplay::load(scratch.path(), None)
            .err()
            .unwrap_or_else(|| panic!("{file} present and inconsistent must refuse"));
        let text = error.to_string();
        assert!(
            text.contains(expected),
            "{file}: unexpected refusal {text:?}"
        );
        assert_eq!(
            written(scratch.path()),
            before,
            "{file}: a refusal writes nothing"
        );
    }
}

/// Every file in `directory` with its bytes, so a refusal can prove it wrote nothing.
fn written(directory: &Path) -> BTreeMap<String, Vec<u8>> {
    fs::read_dir(directory)
        .unwrap()
        .map(|entry| {
            let path = entry.unwrap().path();
            let name = path.file_name().unwrap().to_string_lossy().into_owned();
            (name, fs::read(&path).unwrap())
        })
        .collect()
}
