//! T259: a settled soccer match advances `FinishMinigame` quest objectives once.
//!
//! Every test drives a production seam: `Gameplay::handle_in_world` (the 167/1/2
//! Play and family-76 requests the gateway sends), `Gameplay::tick` (the session
//! tick whose `tick_world` settles the match), `Gameplay::load` +
//! `bind_npc_quest_catalogue` (the reconnect bind) and `Gameplay::state` (the
//! persisted quest state).
//!
//! The quest is the shipped split-content record 3000021 "Introduce Yourself"
//! (`data/content/quests/adventurer.json`, giver 9000000189) with its single
//! objective replaced by a synthetic two-match `finish_minigame` goal for
//! minigame 42 (R). Two matches keep "one finished match = one increment"
//! observable: a one-match goal is capped by the quest engine, so a duplicated
//! credit could not be told apart from a correct one. The shipped MiniGameData
//! rows 42/43/44 and the shipped 10-coin soccer reward stay content. The match
//! is a short synthetic fixture so the full-time settle is bounded and the
//! server-scored goal is deterministic.
//!
//! Client evidence: the terminal `39/18` (`ApplyMiniGameGameOverResult`
//! 0x009ad480; body reader 0x009b7ec0 over the shared header then `u8 won` and
//! three `i32`) and the `39/14` payload reader 0x009b89e0 are what the hook
//! parses; the objective rows are the ordinary `49/9`/`49/10` readers
//! 0x00c7b5e0/0x00c7b650 (applied by 0x00c7a990/0x00c7aa10).
//!
//! No player names or chat text are embedded.

use osfr_tools::{
    character_store::CharacterState,
    gameplay::{Gameplay, GameplayState},
    quest_engine::{CharacterEligibility, QuestEngine, QuestState},
    soccer::{self, SoccerPacket},
    soccer_match::MatchPhase,
};
use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

#[allow(dead_code)] // only `copy_missing` is used here
#[path = "common/content_copy.rs"]
mod content_copy;

const GUID: u64 = 42_059;
const OTHER: u64 = 42_060;
/// Shipped split quest "Introduce Yourself" (adventurer chapter).
const QUEST: u32 = 3_000_021;
const GIVER: u64 = 9_000_000_189;
const SOCCER_42: u32 = 42;
const SOCCER_43: u32 = 43;
const SOCCER_44: u32 = 44;
/// The synthetic goal: two finished matches of MiniGameData row 42 (R).
const GOAL_COUNT: u32 = 2;
/// `data/content/soccer.json` reward.
const REWARD_COINS: u32 = 10;
static NEXT: AtomicU64 = AtomicU64::new(0);

struct Fixture {
    dir: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!(
            "t259-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&dir).unwrap();
        content_copy::copy_missing(&opening_content(), &dir);
        content_copy::copy_missing(&content_dir(), &dir);
        // The opening inventory does not carry every optional farming reward
        // (the same rule as the field-combat fixtures); this suite exercises soccer.
        let _ = fs::remove_file(dir.join("harvesting.json"));
        fs::write(dir.join("soccer.json"), soccer_content()).unwrap();
        patch_quest(&dir);
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

/// The shipped rows 42/43/44 with the traced short match. Every physics and
/// adapter value is the existing test fixture shape (R); only the clock is short.
fn soccer_content() -> Vec<u8> {
    let games: Vec<Value> = [
        (SOCCER_42, 37_944, 37_947, 15, 1),
        (SOCCER_43, 37_945, 37_948, 20_992, 2),
        (SOCCER_44, 37_946, 37_949, 20_992, 3),
    ]
    .into_iter()
    .map(|(id, name_id, description_id, icon_id, encounter_id)| {
        json!({
            "id": id,
            "name_id": name_id,
            "description_id": description_id,
            "icon_id": icon_id,
            "difficulty": 1,
            "required_level": 0,
            "encounter_id": encounter_id
        })
    })
    .collect();
    serde_json::to_vec(&json!({
        "schema_version": 1,
        "source": "T259 short-match fixture; shipped MiniGameData rows 42/43/44 and the shipped 10-coin reward; match values are reconstructed (R)",
        "games": games,
        "match_config": {
            "bounds": {
                "half_x": 50.0,
                "half_z": 10.0,
                "goal_half_width": 8.0,
                "goal_height": 5.0
            },
            "physics": {
                "gravity": 0.0,
                "max_position_step": 10.0,
                "max_speed": 40.0,
                "control_radius": 2.0,
                "max_kick_power": 4.0,
                "pass_radius": 40.0,
                "max_tick": 0.25
            },
            "half_seconds": 6.0,
            "kickoff_spawn": [0.0, 0.0, 0.0],
            "half_policy": "keep_sides",
            "adapter": {
                "max_power": 4.0,
                "charge_floor": 0.01,
                "min_direction_norm": 0.001
            },
            "entry_position": [0.0, 0.0, 0.0, 1.0],
            "entry_rotation": [0.0, 0.0, 1.0, 0.0]
        },
        "reward": {"coins": REWARD_COINS, "job_xp": {}}
    }))
    .unwrap()
}

/// Replace the accepted quest's single Talk goal with a two-match soccer goal.
fn patch_quest(dir: &Path) {
    let path = dir.join("quests/adventurer.json");
    let mut document: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    let mut patched = false;
    for quest in document["records"].as_array_mut().unwrap() {
        if quest["id"] == QUEST {
            quest["objectives"] = json!([{
                "kind": "finish_minigame",
                "id": 1,
                "minigame_id": SOCCER_42,
                "count": GOAL_COUNT,
                "source": [{
                    "kind": "reconstructed",
                    "evidence": "(R) T259 synthetic soccer completion goal; not a recovered retail objective"
                }]
            }]);
            patched = true;
        }
    }
    assert!(patched, "quest {QUEST} is in the fixture content");
    fs::write(&path, serde_json::to_vec(&document).unwrap()).unwrap();
}

fn character_at(guid: u64) -> CharacterState {
    CharacterState {
        guid,
        zone_id: 0,
        position: [0.0; 3],
        rotation: [0.0, 0.0, 1.0],
    }
}

fn eligibility() -> CharacterEligibility {
    CharacterEligibility {
        member: true,
        nonmember_quest_limit: 30,
        member_quest_limit: 30,
        ..Default::default()
    }
}

/// Build the real quest engine and accept the synthetic completion goal. The
/// saved `QuestState` is what `bind_npc_quest_catalogue` restores on reconnect.
fn active_quests(dir: &Path, guid: u64) -> QuestState {
    let mut engine = QuestEngine::from_split_content(dir, guid, eligibility(), None).unwrap();
    assert!(
        engine.offer(QUEST, GIVER).changed,
        "quest {QUEST} is offered"
    );
    assert!(engine.accept(QUEST).changed, "quest {QUEST} is accepted");
    assert_eq!(engine.progress(QUEST).unwrap().counts, [0]);
    engine.state().clone()
}

/// The same accepted goal saved with `counts` already earned (the near-cap
/// fixtures). A saved count is repaired/validated by the engine on load.
fn quests_at(dir: &Path, guid: u64, earned: u32) -> QuestState {
    let mut state = active_quests(dir, guid);
    state.quests.get_mut(&QUEST).unwrap().counts = vec![earned];
    state
}

fn base_state(fixture: &Fixture) -> GameplayState {
    Gameplay::load(&fixture.dir, None).unwrap().state()
}

/// One player's session with the opening engine the soccer entry needs and the
/// given saved quest state. `bind_npc_quest_catalogue` is the call the gateway
/// makes at character load (`gateway.rs:816`).
fn player_with(fixture: &Fixture, guid: u64, quests: Option<QuestState>) -> Gameplay {
    let mut state = base_state(fixture);
    state.npc_quests = quests;
    let mut game = Gameplay::load(&fixture.dir, Some(state)).unwrap();
    game.bind_npc_quest_catalogue(&fixture.dir, guid).unwrap();
    game.initialize_world(&character_at(guid), 0).unwrap();
    game
}

fn player(fixture: &Fixture, guid: u64) -> Gameplay {
    player_with(fixture, guid, Some(active_quests(&fixture.dir, guid)))
}

fn play(activity: u32) -> Vec<u8> {
    let mut inner = vec![167, 0, 1, 2];
    inner.extend(activity.to_le_bytes());
    inner.extend(0u32.to_le_bytes());
    inner
}

fn lifecycle(sub: u8, instance: i32) -> Vec<u8> {
    let mut packet = vec![39, 0, sub];
    for value in [instance, -1, -1] {
        packet.extend(value.to_le_bytes());
    }
    packet
}

/// A client's own `39/14` result claim: the exact shape the server writes for
/// `OnGameWonMsg`, but sent client-to-server.
fn client_won_payload(instance: i32) -> Vec<u8> {
    let mut packet = vec![39, 0, 14];
    for value in [instance, -1, SOCCER_42 as i32] {
        packet.extend(value.to_le_bytes());
    }
    let text = b"OnGameWonMsg\0";
    packet.extend((text.len() as i32).to_le_bytes());
    packet.extend_from_slice(text);
    packet
}

fn offer_instance(replies: &[Vec<u8>]) -> i32 {
    let offer = replies
        .iter()
        .find(|reply| reply.starts_with(&[39, 0, 16]))
        .expect("39/16 soccer offer");
    i32::from_le_bytes(offer[3..7].try_into().unwrap())
}

fn position(player: u64, position: [f32; 3]) -> Vec<u8> {
    soccer::encode_inner(&SoccerPacket::UpdatePlayerPos {
        player,
        position,
        tail: [0; 45],
    })
}

fn shot(player: u64) -> Vec<u8> {
    soccer::encode_inner(&SoccerPacket::UpdatePlayerState {
        player,
        state: 17,
        aux: 255,
        state_counter: 1,
        vectors: [[0.0, 0.0, 1.0, 1.0], [0.0; 4], [0.0; 4]],
    })
}

fn client_goal() -> Vec<u8> {
    soccer::encode_inner(&SoccerPacket::GoalMade { fields: [0, 1, 1] })
}

/// Portal Play through Loaded and the start request. Returns the run instance.
fn enter_match(game: &mut Gameplay, character: &CharacterState, activity: u32, at: u64) -> i32 {
    let played = game
        .handle_in_world(&play(activity), character, at)
        .unwrap();
    assert!(played.scene_zoning.is_some(), "Play zones into sg_soccer");
    let loaded = game.initialize_world(character, at + 100).unwrap();
    let instance = offer_instance(&loaded.replies);
    let started = game
        .handle_in_world(&lifecycle(5, instance), character, at + 200)
        .unwrap();
    assert!(
        started
            .replies
            .iter()
            .any(|reply| reply.starts_with(&[41, 0, 2, 0])),
        "shared encounter start"
    );
    instance
}

/// The server-scored goal a real client produces: report the pose, let the
/// server step, then send the sub-14 charged shot.
fn shoot(game: &mut Gameplay, character: &CharacterState, at: u64) {
    game.handle_in_world(&position(character.guid, [0.0, 0.0, 0.0]), character, at)
        .unwrap();
    game.tick(character, at + 150).unwrap();
    game.handle_in_world(&shot(character.guid), character, at + 150)
        .unwrap();
}

/// Step the authoritative clock until full time settles the match. Returns the
/// settling tick's replies and the tick time.
fn tick_to_full_time(
    game: &mut Gameplay,
    character: &CharacterState,
    mut now: u64,
) -> (Vec<Vec<u8>>, u64) {
    for _ in 0..160 {
        now += 250;
        let out = game.tick(character, now).unwrap();
        if index(&out.replies, &[39, 0, 18]).is_some() {
            return (out.replies, now);
        }
    }
    panic!("the soccer clock reached full time");
}

/// One full match: Play, start, server goal, full time. Returns the settling
/// tick's replies and its time.
fn win_match(
    game: &mut Gameplay,
    character: &CharacterState,
    activity: u32,
    at: u64,
) -> (Vec<Vec<u8>>, u64) {
    enter_match(game, character, activity, at);
    shoot(game, character, at + 200);
    tick_to_full_time(game, character, at + 350)
}

fn index(packets: &[Vec<u8>], prefix: &[u8]) -> Option<usize> {
    packets.iter().position(|packet| packet.starts_with(prefix))
}

fn index_requiring(packets: &[Vec<u8>], prefix: &[u8], what: &str) -> usize {
    index(packets, prefix).unwrap_or_else(|| panic!("{what} is in the settled replies"))
}

/// `49/9` ObjectiveUpdate: six-byte header, then quest, objective key, current,
/// progress `f32`, notification byte, trailing `i32` (27 bytes).
fn objective_updates(packets: &[Vec<u8>], quest: u32) -> Vec<(u32, u32)> {
    packets
        .iter()
        .filter(|packet| packet.len() == 27 && packet.starts_with(&[49, 0, 9, 0]))
        .filter_map(|packet| {
            let quest_id = u32::from_le_bytes(packet[6..10].try_into().unwrap());
            let key = u32::from_le_bytes(packet[10..14].try_into().unwrap());
            let current = u32::from_le_bytes(packet[14..18].try_into().unwrap());
            (quest_id == quest).then_some((key, current))
        })
        .collect()
}

/// `49/10` ObjectiveComplete: six-byte header, then quest, objective key,
/// progress `f32`, an `i32` and the notification byte (23 bytes).
fn objective_completions(packets: &[Vec<u8>], quest: u32) -> Vec<u32> {
    packets
        .iter()
        .filter(|packet| packet.len() == 23 && packet.starts_with(&[49, 0, 10, 0]))
        .filter_map(|packet| {
            let quest_id = u32::from_le_bytes(packet[6..10].try_into().unwrap());
            let key = u32::from_le_bytes(packet[10..14].try_into().unwrap());
            (quest_id == quest).then_some(key)
        })
        .collect()
}

fn counts(game: &Gameplay) -> Vec<u32> {
    game.state().npc_quests.as_ref().unwrap().quests[&QUEST]
        .counts
        .clone()
}

#[test]
fn a_finished_soccer_match_advances_the_quest_once() {
    let fixture = Fixture::new();
    let character = character_at(GUID);
    let mut game = player(&fixture, GUID);
    let coins_before = game.inventory_state().coins;

    let (replies, settled_at) = win_match(&mut game, &character, SOCCER_42, 1_000);

    let game_over = index_requiring(&replies, &[39, 0, 18], "the server result");
    assert_eq!(replies[game_over][15], 1, "the server scored a win");
    assert_eq!(
        i32::from_le_bytes(replies[game_over][11..15].try_into().unwrap()),
        SOCCER_42 as i32,
        "the settled session's own game id"
    );
    assert_eq!(
        objective_updates(&replies, QUEST),
        [(1, 1)],
        "one server objective update goes 0 -> 1"
    );
    assert!(
        objective_completions(&replies, QUEST).is_empty(),
        "one of two matches is not completion"
    );
    assert_eq!(counts(&game), [1]);
    assert_eq!(
        game.inventory_state().coins,
        coins_before + REWARD_COINS,
        "one payout"
    );

    // The settlement marker is the once-only guard: the next tick, the results
    // Close and later world ticks add no objective and no coin.
    let after = game.tick(&character, settled_at + 250).unwrap();
    assert!(after.replies.is_empty(), "{:?}", after.replies);
    let closed = game
        .handle_in_world(&[26, 0, 42, 0], &character, settled_at + 500)
        .unwrap();
    assert!(!game.is_soccer_active(), "results Close releases the match");
    assert!(closed.scene_zoning.is_some(), "Close returns to the world");
    assert!(objective_updates(&closed.replies, QUEST).is_empty());
    assert!(
        closed
            .replies
            .iter()
            .all(|p| !p.starts_with(&[49, 0, 10, 0]))
    );
    assert_eq!(counts(&game), [1]);
    assert_eq!(game.inventory_state().coins, coins_before + REWARD_COINS);
    let later = game.tick(&character, settled_at + 750).unwrap();
    assert!(objective_updates(&later.replies, QUEST).is_empty());
    assert_eq!(counts(&game), [1]);
    assert_eq!(game.inventory_state().coins, coins_before + REWARD_COINS);

    // The near-cap fixture: one earned match plus one settled match completes
    // the goal, and the counter stops exactly at the required count.
    let mut near = player_with(&fixture, OTHER, Some(quests_at(&fixture.dir, OTHER, 1)));
    let other = character_at(OTHER);
    let near_coins = near.inventory_state().coins;
    let (near_replies, _) = win_match(&mut near, &other, SOCCER_42, 1_000);
    assert_eq!(
        objective_updates(&near_replies, QUEST),
        [(1, 2)],
        "one more match adds exactly one"
    );
    assert_eq!(objective_completions(&near_replies, QUEST), [1]);
    assert_eq!(counts(&near), [2], "capped at the required count");
    assert_eq!(near.inventory_state().coins, near_coins + REWARD_COINS);
}

#[test]
fn the_reward_is_paid_once_and_the_result_precedes_the_event() {
    let fixture = Fixture::new();
    let character = character_at(GUID);
    let mut game = player(&fixture, GUID);
    let coins_before = game.inventory_state().coins;
    let xp_before = game.state().profiles.unwrap().experience;

    let (replies, _) = win_match(&mut game, &character, SOCCER_42, 1_000);

    // The lifecycle burst is the pinned order, then the payout, then the goal.
    let won_msg = index_requiring(&replies, &[39, 0, 14], "OnGameWonMsg");
    let game_over = index_requiring(&replies, &[39, 0, 18], "39/18");
    let scores = index_requiring(&replies, &[39, 0, 47], "39/47");
    let removed = index_requiring(&replies, &[39, 0, 19], "39/19");
    let coins = index_requiring(&replies, &[38, 0, 19, 0], "the coin update");
    let bundle = index_requiring(&replies, &[50, 0, 1], "the reward bundle");
    let update = index_requiring(&replies, &[49, 0, 9, 0], "the objective update");
    assert!(
        won_msg < game_over && game_over < scores && scores < removed,
        "result order is OnGameWonMsg, 39/18, 39/47, 39/19: {:?}",
        replies
    );
    assert!(
        removed < coins && coins < bundle,
        "the payout follows the result"
    );
    assert!(
        bundle < update,
        "the objective follows the result and its payout"
    );
    assert_eq!(
        i32::from_le_bytes(replies[coins][4..8].try_into().unwrap()) as u32,
        coins_before + REWARD_COINS,
        "the coin update carries the new balance"
    );
    assert_eq!(game.inventory_state().coins, coins_before + REWARD_COINS);
    assert_eq!(
        game.state().profiles.unwrap().experience,
        xp_before,
        "the shipped soccer reward pays no job XP"
    );
    assert_eq!(objective_updates(&replies, QUEST), [(1, 1)]);
    assert_eq!(counts(&game), [1]);
}

#[test]
fn a_loss_or_cancel_does_not_advance() {
    let fixture = Fixture::new();
    let character = character_at(GUID);

    // A tie at full time is the per-player generic loss: result and no grant.
    let mut game = player(&fixture, GUID);
    let coins_before = game.inventory_state().coins;
    enter_match(&mut game, &character, SOCCER_42, 1_000);
    let (replies, lost_at) = tick_to_full_time(&mut game, &character, 1_200);
    let game_over = index_requiring(&replies, &[39, 0, 18], "the loss result");
    assert_eq!(replies[game_over][15], 0, "win flag 0");
    assert_eq!(counts(&game), [0], "a loss credits nothing");
    assert!(objective_updates(&replies, QUEST).is_empty());
    assert_eq!(
        game.inventory_state().coins,
        coins_before,
        "a loss pays nothing"
    );
    game.handle_in_world(&[26, 0, 42, 0], &character, lost_at + 250)
        .unwrap();
    assert_eq!(counts(&game), [0]);

    // A cancel before full time sends 39/19, pays nothing and credits nothing.
    let mut game = player(&fixture, OTHER);
    let other = character_at(OTHER);
    let coins_before = game.inventory_state().coins;
    let instance = enter_match(&mut game, &other, SOCCER_42, 1_000);
    game.tick(&other, 1_300).unwrap();
    let quit = game
        .handle_in_world(&lifecycle(6, instance), &other, 1_350)
        .unwrap();
    assert!(
        quit.replies
            .iter()
            .any(|reply| reply.starts_with(&[39, 0, 19])),
        "the quit removes the run"
    );
    assert!(!game.is_soccer_active());
    assert_eq!(counts(&game), [0]);
    assert_eq!(game.inventory_state().coins, coins_before);
    let later = game.tick(&other, 1_600).unwrap();
    assert!(later.replies.iter().all(|p| !p.starts_with(&[39, 0, 18])));
    assert_eq!(counts(&game), [0]);

    // A close before full time is not a settlement.
    let mut game = player(&fixture, OTHER);
    let coins_before = game.inventory_state().coins;
    enter_match(&mut game, &other, SOCCER_42, 1_000);
    let early = game
        .handle_in_world(&[26, 0, 42, 0], &other, 1_300)
        .unwrap();
    assert!(index(&early.replies, &[39, 0, 18]).is_none());
    assert_eq!(counts(&game), [0]);
    assert_eq!(game.inventory_state().coins, coins_before);
}

#[test]
fn a_different_game_id_does_not_advance() {
    let fixture = Fixture::new();
    for activity in [SOCCER_43, SOCCER_44] {
        let guid = GUID + u64::from(activity);
        let character = character_at(guid);
        let mut game = player(&fixture, guid);
        let coins_before = game.inventory_state().coins;
        let (replies, _) = win_match(&mut game, &character, activity, 1_000);
        let game_over = index_requiring(&replies, &[39, 0, 18], "the server result");
        assert_eq!(replies[game_over][15], 1, "match {activity} is a win");
        assert_eq!(
            i32::from_le_bytes(replies[game_over][11..15].try_into().unwrap()),
            activity as i32
        );
        assert_eq!(counts(&game), [0], "match {activity} does not credit 42");
        assert!(objective_updates(&replies, QUEST).is_empty());
        assert_eq!(
            game.inventory_state().coins,
            coins_before + REWARD_COINS,
            "the match's own reward still pays"
        );
    }
}

#[test]
fn a_duplicate_settle_or_close_does_not_replay() {
    let fixture = Fixture::new();
    let character = character_at(GUID);
    let mut game = player(&fixture, GUID);
    let coins_before = game.inventory_state().coins;
    let (replies, settled_at) = win_match(&mut game, &character, SOCCER_42, 1_000);

    // The settling tick itself carries exactly one update.
    assert_eq!(objective_updates(&replies, QUEST), [(1, 1)]);
    assert_eq!(counts(&game), [1]);

    // The client's automatic 39/7 acknowledgement, a post-result quit and the
    // results Close are all inert; a second tick sends no second objective.
    let acked = game
        .handle_in_world(&lifecycle(7, 1), &character, settled_at + 150)
        .unwrap();
    assert!(acked.replies.is_empty(), "{:?}", acked.replies);
    let quit = game
        .handle_in_world(&lifecycle(6, 1), &character, settled_at + 200)
        .unwrap();
    assert!(quit.replies.is_empty(), "{:?}", quit.replies);
    let tick = game.tick(&character, settled_at + 250).unwrap();
    assert!(tick.replies.is_empty(), "{:?}", tick.replies);
    let closed = game
        .handle_in_world(&[26, 0, 42, 0], &character, settled_at + 300)
        .unwrap();
    assert!(objective_updates(&closed.replies, QUEST).is_empty());
    assert!(
        closed
            .replies
            .iter()
            .all(|p| !p.starts_with(&[49, 0, 10, 0]))
    );
    assert_eq!(counts(&game), [1]);
    assert_eq!(game.inventory_state().coins, coins_before + REWARD_COINS);
    for step in 1..=4 {
        let later = game
            .tick(&character, settled_at + 300 + step * 250)
            .unwrap();
        assert!(objective_updates(&later.replies, QUEST).is_empty());
    }
    assert_eq!(counts(&game), [1]);

    // A goal already at its required count does not over-credit or emit a row.
    let mut capped = player_with(&fixture, OTHER, Some(quests_at(&fixture.dir, OTHER, 2)));
    let capped_character = character_at(OTHER);
    let coins_before = capped.inventory_state().coins;
    let (replies, _) = win_match(&mut capped, &capped_character, SOCCER_42, 1_000);
    assert_eq!(counts(&capped), [2], "the counter stays capped");
    assert!(objective_updates(&replies, QUEST).is_empty());
    assert!(objective_completions(&replies, QUEST).is_empty());
    assert_eq!(
        capped.inventory_state().coins,
        coins_before + REWARD_COINS,
        "the match reward is not the quest credit"
    );
}

#[test]
fn the_event_is_not_client_reported() {
    let fixture = Fixture::new();
    let character = character_at(GUID);
    let mut game = player(&fixture, GUID);
    let coins_before = game.inventory_state().coins;
    let instance = enter_match(&mut game, &character, SOCCER_42, 1_000);

    // A client GoalMade is refused and changes no score or state.
    let forged_goal = match game.handle_in_world(&client_goal(), &character, 1_200) {
        Err(error) => error,
        Ok(_) => panic!("client GoalMade is refused"),
    };
    assert!(forged_goal.to_string().contains("goal"), "{forged_goal}");
    assert_eq!(game.soccer_score(), Some([0, 0]));
    assert_eq!(game.soccer_phase(), Some(MatchPhase::Running));

    // A client 39/14 won claim and a client 39/18 result claim are ignored.
    for claim in [client_won_payload(instance), lifecycle(18, instance)] {
        let out = game.handle_in_world(&claim, &character, 1_250).unwrap();
        assert!(out.replies.is_empty(), "{:?}", out.replies);
        assert_eq!(game.soccer_phase(), Some(MatchPhase::Running));
        assert_eq!(counts(&game), [0]);
        assert_eq!(game.inventory_state().coins, coins_before);
    }

    // The world tick does not turn a claim into a settlement either.
    let tick = game.tick(&character, 1_400).unwrap();
    assert!(tick.replies.iter().all(|p| !p.starts_with(&[39, 0, 18])));
    assert_eq!(counts(&game), [0]);
    assert_eq!(game.inventory_state().coins, coins_before);

    // The same session still settles only through the server-built result.
    shoot(&mut game, &character, 1_450);
    let (replies, _) = tick_to_full_time(&mut game, &character, 1_600);
    index_requiring(&replies, &[39, 0, 18], "the server result");
    assert_eq!(counts(&game), [1]);
}

#[test]
fn the_credit_survives_a_reconnect() {
    let fixture = Fixture::new();
    let character = character_at(GUID);
    let mut game = player(&fixture, GUID);
    let coins_before = game.inventory_state().coins;
    win_match(&mut game, &character, SOCCER_42, 1_000);
    assert_eq!(counts(&game), [1]);
    let coins_after = game.inventory_state().coins;
    let saved = game.state();
    drop(game);

    // The gateway's reconnect: load the save, bind the catalogue, world entry.
    let mut restored = Gameplay::load(&fixture.dir, Some(saved)).unwrap();
    restored
        .bind_npc_quest_catalogue(&fixture.dir, GUID)
        .unwrap();
    let loaded = restored.initialize_world(&character, 10_000).unwrap();
    assert!(!restored.is_soccer_active(), "a match is session-local");
    assert!(
        loaded.replies.iter().all(|p| !p.starts_with(&[39, 0, 18])),
        "no replayed result"
    );
    // World entry rehydrates the saved journal row (the existing login sync);
    // it carries the saved counter, and nothing new is credited or paid.
    assert_eq!(
        objective_updates(&loaded.replies, QUEST),
        [(1, 1)],
        "the login sync carries the saved counter"
    );
    assert!(objective_completions(&loaded.replies, QUEST).is_empty());
    assert_eq!(counts(&restored), [1], "the objective survives the save");
    assert_eq!(
        restored.inventory_state().coins,
        coins_after,
        "the reward survives the save"
    );

    // Nothing settles or pays again on the reconnected session.
    for step in 1..=4 {
        restored.tick(&character, 10_000 + step * 250).unwrap();
    }
    assert_eq!(counts(&restored), [1]);
    assert_eq!(restored.inventory_state().coins, coins_after);
    assert_eq!(coins_after, coins_before + REWARD_COINS);
}

#[test]
fn kart_chess_and_fishing_completions_are_unchanged() {
    // The credit is reached only from the settled soccer block. An ordinary
    // world tick with the same active soccer goal emits none, so the kart,
    // chess, fishing and type-8 producers keep their own suites and packets;
    // those suites run as separate gates (`kart_*`, `chess`, `fishing`).
    let fixture = Fixture::new();
    let character = character_at(GUID);
    let mut game = player(&fixture, GUID);
    for step in 1..=8 {
        let out = game.tick(&character, step * 250).unwrap();
        assert!(objective_updates(&out.replies, QUEST).is_empty());
    }
    assert_eq!(counts(&game), [0]);
}

#[test]
fn a_failing_quest_event_for_one_player_does_not_stop_the_other() {
    // A stale save belongs to another character, so A's catalogue bind is the
    // refusal such a save gets (the server-built soccer event itself is
    // validated and cannot be forged). The refused bind keeps the stale save
    // for a later retry instead of consuming it; no engine is bound, so A's
    // match still settles and pays without crediting it, while B's session
    // credits its own quest.
    let fixture = Fixture::new();
    let mut stale = active_quests(&fixture.dir, GUID);
    stale.character_id = OTHER;
    let mut state = base_state(&fixture);
    state.npc_quests = Some(stale);
    let mut a = Gameplay::load(&fixture.dir, Some(state)).unwrap();
    let error = a
        .bind_npc_quest_catalogue(&fixture.dir, GUID)
        .err()
        .expect("A's stale quest save is refused");
    assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
    let kept = a
        .state()
        .npc_quests
        .expect("the refused bind keeps the saved state");
    assert_eq!(kept.character_id, OTHER);
    a.initialize_world(&character_at(GUID), 0).unwrap();

    let a_character = character_at(GUID);
    let a_coins = a.inventory_state().coins;
    let (a_replies, _) = win_match(&mut a, &a_character, SOCCER_42, 1_000);
    index_requiring(&a_replies, &[39, 0, 18], "A's server result");
    assert!(objective_updates(&a_replies, QUEST).is_empty());
    assert_eq!(a.inventory_state().coins, a_coins + REWARD_COINS);

    // B's session in the same fixture still credits its own active goal.
    let b_character = character_at(OTHER);
    let mut b = player(&fixture, OTHER);
    let b_coins = b.inventory_state().coins;
    let (b_replies, _) = win_match(&mut b, &b_character, SOCCER_42, 2_000);
    assert_eq!(objective_updates(&b_replies, QUEST), [(1, 1)]);
    assert_eq!(counts(&b), [1]);
    assert_eq!(b.inventory_state().coins, b_coins + REWARD_COINS);
}
