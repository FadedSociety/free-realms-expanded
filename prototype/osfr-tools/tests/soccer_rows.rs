//! L932: the Soccer Star quest rows (MiniGameData type 12: 322-333) credit through the one soccer
//! pitch.
//!
//! Client facts: `Resources/MiniGameData.txt` rows 322-333 are `<id>^12^<name>^37947^20992^<difficulty>`
//! (names Global.Text 402582, 402586, 402590, 402598, 402606, 402610, 402618, 414908-414912). The
//! 39/18 result reader (`ApplyMiniGameGameOverResult` 0x009ad480, body 0x009b7ec0) names the game at
//! bytes 11..15, so the row a match was offered under is the row it credits.
//!
//! Every test drives the production seams (`Gameplay::handle_in_world` for the 167/1/2 portal Play,
//! the family-39/76 requests and the results Close; `Gameplay::tick` for the clock whose
//! `tick_world` settles the match) with the shipped `soccer.json` rows and a short synthetic match.
//!
//! No player names or chat text are embedded.

use osfr_tools::{
    character_store::CharacterState,
    gameplay::{Gameplay, GameplayState},
    quest_engine::{CharacterEligibility, QuestEngine, QuestState},
    soccer::{self, SoccerPacket},
};
use serde_json::{Value, json};
use soccer_play::*;
use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

#[allow(dead_code)] // only `copy_missing` is used here
#[path = "common/content_copy.rs"]
mod content_copy;
#[path = "support/soccer_play.rs"]
mod soccer_play;

const GUID: u64 = 93_201;
/// Shipped split quest "Introduce Yourself" (adventurer chapter), its objective rewritten per test.
const QUEST: u32 = 3_000_021;
const GIVER: u64 = 9_000_000_189;
const SNOWHILL: u32 = 43;
const SLOWING_THE_SNOWSPURS: u32 = 326;
const GOAL_STOPPER: u32 = 323;
const CHEETAHS_DOUBLE_CHALLENGE: u32 = 333;
static NEXT: AtomicU64 = AtomicU64::new(0);

struct Fixture {
    dir: PathBuf,
}

impl Fixture {
    /// The shipped content with `objective_row` as the quest's only goal, `count` wins of it.
    fn new(objective_row: u32, count: u32) -> Self {
        let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!(
            "l932-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&dir).unwrap();
        content_copy::copy_missing(&opening_content(), &dir);
        content_copy::copy_missing(&content_dir(), &dir);
        let _ = fs::remove_file(dir.join("harvesting.json"));
        fs::write(dir.join("soccer.json"), short_match_content()).unwrap();
        patch_quest(&dir, objective_row, count);
        Self { dir }
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.dir);
    }
}

fn opening_content() -> PathBuf {
    std::env::var_os("OSFR_SOCCER_CONTENT")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../../lab/references/rust-opening-cooking-original-assets")
        })
}

fn content_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content")
}

/// The shipped `soccer.json` (every row, including the quest-only ones) with the short synthetic
/// match of `soccer_quest_credit`, so the full-time settle is bounded and a server goal is
/// deterministic. Goal Stopper allows no goal against it here: the shipped limit of 4 cannot be
/// reached in a 12 second match.
fn short_match_content() -> Vec<u8> {
    let mut shipped: Value =
        serde_json::from_slice(&fs::read(content_dir().join("soccer.json")).unwrap()).unwrap();
    for game in shipped["games"].as_array_mut().unwrap() {
        if game["id"] == GOAL_STOPPER {
            game["max_conceded"] = json!(0);
        }
    }
    shipped["match_config"] = json!({
        "bounds": {"half_x": 10.0, "half_z": 50.0, "goal_half_width": 8.0, "goal_height": 5.0},
        "physics": {
            "gravity": 0.0, "max_position_step": 10.0, "max_speed": 40.0,
            "control_radius": 2.0, "max_kick_power": 4.0, "pass_radius": 40.0, "max_tick": 0.25
        },
        "half_seconds": 30.0,
        "kickoff_spawn": [0.0, 0.0, 0.0],
        "half_policy": "keep_sides",
        "adapter": {"max_power": 4.0, "charge_floor": 0.01, "min_direction_norm": 0.001},
        "entry_position": [0.0, 0.0, 0.0, 1.0],
        "entry_rotation": [0.0, 0.0, 1.0, 0.0]
    });
    serde_json::to_vec(&shipped).unwrap()
}

fn patch_quest(dir: &Path, minigame_id: u32, count: u32) {
    let path = dir.join("quests/adventurer.json");
    let mut document: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    let quest = document["records"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|quest| quest["id"] == QUEST)
        .expect("the fixture carries the quest");
    quest["objectives"] = json!([{
        "kind": "finish_minigame",
        "id": 1,
        "minigame_id": minigame_id,
        "count": count,
        "source": [{
            "kind": "reconstructed",
            "evidence": "(R) L932 synthetic soccer quest-row goal; not a recovered retail objective"
        }]
    }]);
    fs::write(&path, serde_json::to_vec(&document).unwrap()).unwrap();
}

fn character() -> CharacterState {
    CharacterState {
        guid: GUID,
        zone_id: 0,
        position: [0.0; 3],
        rotation: [0.0, 0.0, 1.0],
    }
}

fn accepted_quest(dir: &Path) -> QuestState {
    let eligibility = CharacterEligibility {
        member: true,
        nonmember_quest_limit: 30,
        member_quest_limit: 30,
        ..Default::default()
    };
    let mut engine = QuestEngine::from_split_content(dir, GUID, eligibility, None).unwrap();
    assert!(engine.offer(QUEST, GIVER).changed);
    assert!(engine.accept(QUEST).changed);
    engine.state().clone()
}

/// A session with the quest accepted (`accepted`) or not offered at all.
fn player(fixture: &Fixture, accepted: bool) -> Gameplay {
    let mut state: GameplayState = Gameplay::load(&fixture.dir, None).unwrap().state();
    state.npc_quests = accepted.then(|| accepted_quest(&fixture.dir));
    let mut game = Gameplay::load(&fixture.dir, Some(state)).unwrap();
    game.bind_npc_quest_catalogue(&fixture.dir, GUID).unwrap();
    game.initialize_world(&character(), 0).unwrap();
    game
}

fn counts(game: &Gameplay) -> Vec<u32> {
    let state = game.state();
    (state.npc_quests.as_ref())
        .and_then(|quests| quests.quests.get(&QUEST))
        .map(|quest| quest.counts.clone())
        .unwrap_or_default()
}

/// The game id of a message that names one at bytes 11..15 (the 39/16 offer, the 39/18 result).
fn game_id(replies: &[Vec<u8>], prefix: &[u8]) -> i32 {
    let packet = replies
        .iter()
        .find(|packet| packet.starts_with(prefix))
        .unwrap_or_else(|| panic!("{prefix:?} is in the replies"));
    i32::from_le_bytes(packet[11..15].try_into().unwrap())
}

#[test]
fn an_open_quest_objective_picks_the_row_a_games_window_match_credits() {
    let fixture = Fixture::new(SLOWING_THE_SNOWSPURS, 1);
    let mut game = player(&fixture, true);
    let character = character();

    // Snowhill Soccer (43) is entered; the open objective names row 326, so the offer does too.
    let played = game
        .handle_in_world(&play(SNOWHILL), &character, 1_000)
        .unwrap();
    assert!(played.scene_zoning.is_some(), "Play zones into sg_soccer");
    let loaded = game.initialize_world(&character, 1_100).unwrap();
    assert_eq!(
        game_id(&loaded.replies, &[39, 0, 16]),
        SLOWING_THE_SNOWSPURS as i32
    );
    game.handle_in_world(
        &lifecycle(5, offer_instance(&loaded.replies)),
        &character,
        1_200,
    )
    .unwrap();
    shoot(&mut game, &character, 1_400);
    let (replies, _) = tick_to_full_time(&mut game, &character, 1_550);

    assert_eq!(
        game_id(&replies, &[39, 0, 18]),
        SLOWING_THE_SNOWSPURS as i32
    );
    assert_eq!(counts(&game), [1], "the win credits the quest row once");
}

#[test]
fn without_an_open_objective_the_entered_games_window_row_credits() {
    let fixture = Fixture::new(SLOWING_THE_SNOWSPURS, 1);
    let mut game = player(&fixture, false);
    let character = character();

    let (replies, _) = win_match(&mut game, &character, SNOWHILL, 1_000);

    assert_eq!(game_id(&replies, &[39, 0, 18]), SNOWHILL as i32);
    assert!(counts(&game).is_empty(), "no quest was accepted");
}

#[test]
fn an_objective_on_the_entered_row_keeps_that_row() {
    let fixture = Fixture::new(SNOWHILL, 1);
    let mut game = player(&fixture, true);
    let character = character();

    let (replies, _) = win_match(&mut game, &character, SNOWHILL, 1_000);

    assert_eq!(game_id(&replies, &[39, 0, 18]), SNOWHILL as i32);
    assert_eq!(counts(&game), [1]);
}

#[test]
fn goal_stopper_and_the_double_challenge_credit_their_own_rows() {
    for (row, wins) in [(GOAL_STOPPER, 1), (CHEETAHS_DOUBLE_CHALLENGE, 2)] {
        let fixture = Fixture::new(row, wins);
        let mut game = player(&fixture, true);
        let character = character();
        let mut at = 1_000;
        for win in 1..=wins {
            let (replies, settled) = win_match(&mut game, &character, SNOWHILL, at);
            assert_eq!(game_id(&replies, &[39, 0, 18]), row as i32, "row {row}");
            assert_eq!(counts(&game), [win], "row {row} after {win} wins");
            let closed = game
                .handle_in_world(&[26, 0, 42, 0], &character, settled + 500)
                .unwrap();
            assert!(closed.scene_zoning.is_some(), "Close returns to the world");
            at = settled + 1_000;
        }
    }
}

#[test]
fn a_quest_row_cannot_be_started_from_the_games_window() {
    let fixture = Fixture::new(SLOWING_THE_SNOWSPURS, 1);
    let mut game = player(&fixture, true);
    let character = character();

    // 326 is no Games window tile: a client that asks for it is refused and stays put.
    for id in [
        SLOWING_THE_SNOWSPURS,
        GOAL_STOPPER,
        CHEETAHS_DOUBLE_CHALLENGE,
    ] {
        let played = game.handle_in_world(&play(id), &character, 1_000).unwrap();
        assert!(played.scene_zoning.is_none(), "row {id} zoned");
        assert!(!game.is_soccer_active(), "row {id} started a match");
    }
}

/// A charged shot from the kickoff spot along `z` (+1 at the other team's goal, -1 at the
/// player's own), then the clock until the ball is in a goal. Returns the time it ended.
fn shot_along(game: &mut Gameplay, character: &CharacterState, at: u64, z: f32, n: u32) -> u64 {
    // L946: after a goal the kickoff waits for the celebration (3.1 s); this tick starts it.
    game.tick(character, at).unwrap();
    // After a goal the last toucher must step clear of the reset ball before the ball can be
    // taken again, so walk away and back.
    for (offset, spot) in [(0, [3.0, 0.0, 0.0]), (150, [0.0; 3])] {
        game.handle_in_world(&position(character.guid, spot), character, at + offset)
            .unwrap();
        game.tick(character, at + offset + 100).unwrap();
    }
    let at = at + 150;
    let shot = soccer::encode_inner(&SoccerPacket::UpdatePlayerState {
        player: character.guid,
        state: 17,
        aux: 255,
        state_counter: n,
        // L932: the goals are on the x ends; `z` > 0 is the attacking (-x) goal for team 0.
        vectors: [[-z, 0.0, 0.0, 1.0], [0.0; 4], [0.0; 4]],
    });
    game.handle_in_world(&shot, character, at + 150).unwrap();
    let before = game.soccer_score();
    let mut now = at + 150;
    while game.soccer_score() == before {
        now += 250;
        assert!(
            now < at + 20_000,
            "shot {n} never reached a goal: {:?}",
            game.soccer_score()
        );
        game.tick(character, now).unwrap();
    }
    now
}

#[test]
fn a_win_that_concedes_more_than_the_row_allows_does_not_credit() {
    let fixture = Fixture::new(GOAL_STOPPER, 1);
    let mut game = player(&fixture, true);
    let character = character();
    enter_match(&mut game, &character, SNOWHILL, 1_000);

    // An own goal, then two goals at the other end: 2-1 is a win that let one in.
    let mut now = shot_along(&mut game, &character, 1_300, -1.0, 1);
    assert_eq!(game.soccer_score(), Some([0, 1]), "the own goal");
    now = shot_along(&mut game, &character, now + 3_500, 1.0, 2);
    now = shot_along(&mut game, &character, now + 3_500, 1.0, 3);
    assert_eq!(game.soccer_score(), Some([2, 1]));
    let (replies, _) = tick_to_full_time(&mut game, &character, now + 250);

    let game_over = replies
        .iter()
        .find(|p| p.starts_with(&[39, 0, 18]))
        .unwrap();
    assert_eq!(game_over[15], 1, "the team won 2-1");
    assert_eq!(game_id(&replies, &[39, 0, 18]), GOAL_STOPPER as i32);
    assert_eq!(counts(&game), [0], "one goal against fails the row's limit");
}
