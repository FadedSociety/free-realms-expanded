//! D004: minigame reward adoption regressions. The opening reference and the family-138 wire
//! builders live in `t013_objective_events`; importing them keeps one fishing request path.

use super::t013_objective_events::{
    FISHING_T0, fishing_cast, fishing_reel, fishing_register, game, install,
};
use super::*;
use crate::fishing_service::{FISHERMAN_PROFILE, FishingSpots};
use crate::quest_engine::ObjectiveKind;
use serde_json::json;

const SPOTS_JSON: &str = include_str!("../../data/content/fishing_spots.json");
const FISHING_GATE: u64 = 9_000_000_181;

fn character() -> CharacterState {
    CharacterState {
        guid: 1,
        zone_id: 0,
        position: [0.; 3],
        rotation: [0., 0., 1.],
    }
}

fn start(game: &mut Gameplay, at: u64) {
    let instance = game.fishing.instance_id(1).expect("fishing round");
    let mut packet = vec![39, 0, 5];
    packet.extend(instance.to_le_bytes());
    packet.extend((-1i32).to_le_bytes());
    packet.extend((-1i32).to_le_bytes());
    let outcome = game
        .handle_in_world(&packet, &character(), at)
        .expect("fishing lifecycle starts");
    assert!(!outcome.replies.is_empty());
    game.handle_in_world(&fishing_register(), &character(), at + 1)
        .expect("fishing registration");
}

fn begin(game: &mut Gameplay) {
    let mut spots: serde_json::Value = serde_json::from_str(SPOTS_JSON).unwrap();
    for sizes in spots["items"].as_object_mut().unwrap().values_mut() {
        for item in sizes.as_array_mut().unwrap() {
            if item.as_i64().unwrap() > 0 {
                *item = json!(2884);
            }
        }
    }
    game.fishing_spots = Some(Arc::new(
        FishingSpots::parse(&serde_json::to_vec(&spots).unwrap()).unwrap(),
    ));
    let outcome = game
        .begin_fishing(FISHING_GATE, &character())
        .expect("fishing begins");
    assert!(outcome.changed);
    assert!(
        game.profiles
            .as_ref()
            .unwrap()
            .state()
            .unlocked
            .contains(&FISHERMAN_PROFILE)
    );
    start(game, FISHING_T0);
}

fn catch(game: &mut Gameplay, at: u64) -> io::Result<Outcome> {
    game.handle_in_world(&fishing_cast(), &character(), at + 10)?;
    game.tick(&character(), at + 1_510)?;
    game.tick(&character(), at + 2_410)?;
    game.handle_in_world(&fishing_reel(), &character(), at + 2_420)
}

fn item_count(game: &Gameplay, definition_id: i32) -> u32 {
    game.inventory
        .state()
        .items
        .iter()
        .filter(|item| item.definition_id == definition_id)
        .map(|item| item.count)
        .sum()
}

#[test]
fn fishing_catch_reloads_unlocked_fisherman_profile() {
    let mut game = game();
    begin(&mut game);
    let active = game.profiles.as_ref().unwrap().active();
    assert!(
        game.profiles
            .as_ref()
            .unwrap()
            .state()
            .experience
            .get(&FISHERMAN_PROFILE)
            .is_none()
    );
    game.inventory.grant(0, 2884, 3).expect("live item grant");
    let held = item_count(&game, 2884);

    let outcome = catch(&mut game, FISHING_T0 + 10_000).expect("paid catch");
    assert!(outcome.changed);
    let profiles = game.profiles.as_ref().unwrap();
    assert_eq!(profiles.active(), active, "fishing does not change jobs");
    assert_eq!(
        profiles.state().experience.get(&FISHERMAN_PROFILE),
        Some(&10),
        "the catch reloads the Fisherman XP snapshot"
    );
    assert_eq!(
        item_count(&game, 2884),
        held + 1,
        "adoption keeps items granted after the round began"
    );
}

#[test]
fn fishing_catch_without_profile_content_changes_no_reward_state() {
    let mut game = game();
    begin(&mut game);
    game.profile_content_directory = None;
    let before = (
        game.inventory.state().clone(),
        game.inventory.state().coins,
        game.profiles.as_ref().unwrap().state().clone(),
        game.profiles.as_ref().unwrap().active(),
    );

    let error = match catch(&mut game, FISHING_T0 + 10_000) {
        Ok(_) => panic!("profile reload is required"),
        Err(error) => error,
    };
    assert_eq!(error.to_string(), "minigame XP needs profile content");
    assert_eq!(
        (
            game.inventory.state().clone(),
            game.inventory.state().coins,
            game.profiles.as_ref().unwrap().state().clone(),
            game.profiles.as_ref().unwrap().active(),
        ),
        before,
        "a refused catch adopts neither inventory nor profile state"
    );
}

/// F576: a minigame payout replaces the profiles through `adopt_reward_state`. A catch
/// whose Fisherman XP crosses a job-level goal's rank credits it in the same outcome,
/// not at a later accept, turn-in or reload.
#[test]
fn a_fishing_payout_that_crosses_a_job_level_goal_credits_it_at_once() {
    let mut game = game();
    begin(&mut game);
    let mut state = game.profiles.as_ref().unwrap().state().clone();
    state.experience.insert(FISHERMAN_PROFILE, 90);
    let directory = game.profile_content_directory.clone().unwrap();
    game.profiles = Some(Profiles::load(&directory, Some(state)).unwrap());
    let goal = ObjectiveKind::JobLevel {
        profile_id: FISHERMAN_PROFILE,
        level: 2,
    };
    install(&mut game, goal, 1);

    let outcome = catch(&mut game, FISHING_T0 + 10_000).expect("paid catch");
    assert_eq!(
        game.job_level(FISHERMAN_PROFILE),
        2,
        "the 10-XP catch reaches rank 2"
    );
    let quests = game.npc_quests.as_ref().unwrap();
    assert_eq!(quests.progress(1).unwrap().counts, vec![1]);
    assert!(
        outcome
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[49, 0, 10, 0, 0, 0])),
        "the payout outcome carries the 49/10 completion"
    );
}

/// L059 (the L052 replay's one conflict): a minigame payout reloads the saved progression
/// through `Profiles::adopt`, so the session's staff jobs (never saved) survive it.
#[test]
fn a_fishing_payout_keeps_the_sessions_staff_jobs() {
    let mut game = game();
    begin(&mut game);
    game.set_staff_jobs(true);
    catch(&mut game, FISHING_T0 + 10_000).expect("paid catch");
    assert_eq!(
        game.staff_job_rows().len(),
        2,
        "both staff jobs stay granted"
    );
}
