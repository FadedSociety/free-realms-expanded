//! T409: Soccer Star's first chapter. Manager Landon (9000000140) is drawn at the
//! Snowhill Soccer Field, teaches Soccer Star (profile 52) from his trainer menu, and
//! gives "Soccer Star: Snowy Scrimmage" (3140004), whose one objective is a
//! server-settled win of Snowhill Soccer (MiniGameData row 43).
//!
//! Content only: `npc_spawns_extra.json` (the drawable Landon), `job_trainers.json`
//! (the trainer row), `npcs/snowhill.json` (Landon gives 3140004) and
//! `quests/soccer_star.json` (3140004 enabled and authored). Every test drives the
//! production seams: `world::NpcVisibility` / `Gameplay::update_npc_visibility` (35/2),
//! `Gameplay::handle_in_world` (26/8, 26/10, 26/6, 26/13, 49/2, 49/14, 167/1/2 and the
//! family-39/76 soccer requests) and `Gameplay::tick` (the authoritative soccer clock;
//! `tick_world` settles the match and credits `FinishMinigame` through
//! `npc_minigame_win`, T259).
//!
//! The fixture is the reviewed opening content (`tests/support/npc_quest_chain.rs`)
//! with two shipped files copied in: the active play package's `profiles.json`
//! (its `starter_items.52` dresses Soccer Star; the opening copy predates it) and
//! `data/content/soccer.json` (the shipped match, rows 42/43/44, 10-coin reward).
//!
//! Client evidence (2014 client, `lab/working/ida/FreeRealms.exe.i64`):
//! - 26/8 interact constructor `008b6bc0`: u64 actor + one flag byte.
//! - 26/9 menu reader chain `00aa2360 -> 00aa2220 -> 00aa1600 -> 00aa04d0`: u64
//!   actor, a bool, the option rows, a length-prefixed string, a bool.
//! - 26/10 selection writer `00a36f80`: u64 actor + i32 option id.
//! - 26/6 dialogue reply constructor `009f5890`: one i32 row.
//! - 49/1 QuestInfo reader `00c7bb60 -> 00c7b7a0`: quest id, then the screen dwords.
//! - 49/3 objective row `008fd770`: key, definition id, name and description ids.
//! - 49/2 reply constructor `00a86400`: quest id + accept byte.
//! - 49/13 QuestEnd reader `00c7b990`: 8-byte actor, quest id, end text, bundle.
//! - 49/14 end reply constructor `00a864c0`: one quest id.
//! - 35/2 AddNpc reader `00921a10`; the name field is a Global.Text id (the client
//!   formats `Global.Text.%d`, string `01804c14`).
//! - 39/18 game-over body reader `009b7ec0`: shared header (game id at 11..15), won byte.
//!
//! No player names or chat text are embedded.

#[path = "support/npc_quest_chain.rs"]
mod npc_quest_chain;
#[path = "support/soccer_play.rs"]
mod soccer_play;

use npc_quest_chain::*;
use osfr_tools::{
    character_store::CharacterState,
    gameplay::{Gameplay, Outcome},
    npc_interaction::{decode_dialogue_26_3, decode_quest_offer_49_1},
    quest_engine::QuestStatus,
    startup_world::parse_add_npc,
    station_cash::EarnSource,
    world::NpcVisibility,
};
use serde_json::json;
use soccer_play::{enter_match, lifecycle, play, tick_to_full_time, win_match};
use std::{
    fs, io,
    path::{Path, PathBuf},
};

const LANDON: u64 = 9_000_000_140;
const OTHER: u64 = CHARACTER + 1;
const ADVENTURER: i32 = 1;
const BRAWLER: i32 = 43;
const SOCCER_STAR: i32 = 52;
const SCRIMMAGE: u32 = 3_140_004;
/// The next record stays disabled; Landon gives it but it is never offered.
const MEETING_THE_TEAM: u32 = 3_140_005;
/// MiniGameData row 43, client locale 37945 "Snowhill Soccer".
const SNOWHILL_SOCCER: u32 = 43;
/// MiniGameData row 42 (Sacred Grove), a different field.
const SACRED_GROVE_SOCCER: u32 = 42;
/// `Models.txt` row 1457 = `human_m_manager_landon.agr`.
const LANDON_MODEL: i32 = 1_457;
/// Global.Text 401744 "Manager Landon" (ucdt 2754079242 maps to it).
const LANDON_NAME: i32 = 401_744;
/// Client locale 412463 "Become a Soccer Star".
const BECOME_A_SOCCER_STAR: u32 = 412_463;
const TITLE: u32 = 407_408;
const INTRO: u32 = 418_984;
const DESCRIPTION: u32 = 402_583;
const END_TEXT: u32 = 402_584;
/// Global.Text 441183 "Win a Snowhill Soccer minigame" (guide 2216136620), the goal line.
const GOAL_TEXT: u32 = 441_183;
const QUEST_COINS: u32 = 10;
const QUEST_XP: u32 = 25;
/// The shipped `soccer.json` reward for a won match (not the quest's).
const MATCH_COINS: u32 = 10;
/// The play package's Soccer Star outfit (`starter_items.52`).
const STARTER: [i32; 5] = [61_907, 61_353, 62_169, 61_609, 62_473];
/// Results-screen Close; it ends the soccer session and zones back.
const RESULTS_CLOSE: [u8; 4] = [26, 0, 42, 0];

// ---- fixture ------------------------------------------------------------------------------

fn play_package_profiles() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../lab/bundles/playtest-20261016T115811Z/content/profiles.json")
}

/// The reviewed opening fixture plus the active package's starter tables and the
/// shipped soccer package. NPCs, spawns, trainers and quests are the repository's.
fn chapter_fixture() -> Fixture {
    let fixture = Fixture::new();
    let profiles = play_package_profiles();
    assert_eq!(
        read_json(&profiles)["starter_items"]["52"],
        json!(STARTER),
        "the active package dresses Soccer Star (52)"
    );
    fs::copy(&profiles, fixture.path().join("profiles.json")).unwrap();
    fs::copy(
        content_dir().join("soccer.json"),
        fixture.path().join("soccer.json"),
    )
    .unwrap();
    fixture
}

fn landon_position() -> [f32; 3] {
    content_position("snowhill.json", LANDON)
}

/// One session standing at Landon in the shared open-world zone.
struct Player {
    game: Gameplay,
    character: CharacterState,
    now: u64,
}

impl Player {
    fn join(fixture: &Fixture, guid: u64) -> Self {
        let mut game = fixture.game_as(guid, None);
        let character = character_at_for(guid, 0, landon_position());
        game.initialize_world(&character, 1_000).unwrap();
        Self {
            game,
            character,
            now: 10_000,
        }
    }

    fn stamp(&mut self) -> u64 {
        self.now += 1_000;
        self.now
    }

    fn send(&mut self, packet: &[u8]) -> Outcome {
        let now = self.stamp();
        send(&mut self.game, &self.character, packet, now)
    }

    fn try_send(&mut self, packet: &[u8]) -> io::Result<Outcome> {
        let now = self.stamp();
        self.game.handle_in_world(packet, &self.character, now)
    }

    fn menu(&mut self) -> Outcome {
        let now = self.stamp();
        interact(&mut self.game, &self.character, LANDON, now)
    }

    /// Landon's trainer choice through the real 26/8 -> 26/9 -> 26/10 path.
    fn learn(&mut self) -> Outcome {
        let menu = self.menu();
        let learn = menu_option(&menu, BECOME_A_SOCCER_STAR);
        self.send(&npc_select(LANDON, learn))
    }

    fn become_soccer_star(&mut self) {
        let learned = self.learn();
        assert!(learned.changed, "Landon's trainer choice must commit");
        let text = String::from_utf8_lossy(&learned.replies.concat()).into_owned();
        assert!(
            text.contains("You learned the Soccer Star job"),
            "Landon must confirm the unlock: {text}"
        );
        assert!(
            !text.contains("cannot be selected yet"),
            "the package dresses 52, so the outfit is complete: {text}"
        );
        let switched = self.send(&select_job(SOCCER_STAR));
        assert!(
            switched.changed,
            "a dressed, unlocked Soccer Star activates"
        );
        assert!(
            switched
                .replies
                .iter()
                .any(|packet| packet.starts_with(&[38, 0, 21, 0])),
            "the switch sends the 38/21 activation"
        );
        assert_eq!(active_job(&self.game), SOCCER_STAR);
    }

    fn take_scrimmage(&mut self) -> Outcome {
        let now = self.stamp();
        let offered = offer(&mut self.game, &self.character, LANDON, SCRIMMAGE, now);
        let now = self.stamp();
        let accepted = accept(&mut self.game, &self.character, SCRIMMAGE, now);
        assert!(
            objective_activated(&accepted, SCRIMMAGE, 1),
            "the one win objective is active"
        );
        // 49/3 objective rows (`008fd770`): count at 109, then key, definition id,
        // name and description; the name is the quest panel's goal line (T403).
        let add = accepted
            .replies
            .iter()
            .find(|packet| has_family(packet, 49, 3, Some(SCRIMMAGE)))
            .unwrap();
        let word = |at: usize| u32::from_le_bytes(add[at..at + 4].try_into().unwrap());
        assert_eq!(
            [109, 113, 117, 121, 125].map(word),
            [1, 1, 1, GOAL_TEXT, GOAL_TEXT],
            "one goal, keyed 1, reading 'Win a Snowhill Soccer minigame'"
        );
        assert_eq!(counts(&self.game, SCRIMMAGE), [0]);
        offered
    }

    /// Play one won match of `activity` through the portal, then Close the results.
    fn win(&mut self, activity: u32) -> Outcome {
        let at = self.stamp();
        let (replies, settled) = win_match(&mut self.game, &self.character, activity, at);
        self.now = settled;
        self.close_results();
        Outcome {
            replies,
            ..Outcome::default()
        }
    }

    fn close_results(&mut self) {
        let closed = self.send(&RESULTS_CLOSE);
        assert!(closed.scene_zoning.is_some(), "Close returns to the world");
        assert!(!self.game.is_soccer_active(), "Close ends the match");
    }

    /// The return Talk holds the 49/13 end screen at Landon; 26/6 releases it.
    fn return_to_landon(&mut self) -> Vec<u8> {
        let now = self.stamp();
        talk(&mut self.game, &self.character, LANDON, now);
        assert_eq!(pending_turnin(&self.game), Some(SCRIMMAGE));
        let now = self.stamp();
        release_end_screen(
            &mut self.game,
            &self.character,
            SCRIMMAGE,
            LANDON,
            END_TEXT,
            now,
        )
    }

    /// Reconnect: the typed save through JSON into a fresh session, then world entry.
    fn reconnect(&mut self, fixture: &Fixture) {
        self.game = fixture.reload_serialized_as(self.character.guid, &self.game);
        let now = self.stamp();
        self.game.initialize_world(&self.character, now).unwrap();
    }
}

/// The 39/18 game-over in a settling tick: (game id, won).
fn game_over(replies: &[Vec<u8>]) -> (i32, u8) {
    let packet = replies
        .iter()
        .find(|packet| packet.starts_with(&[39, 0, 18]))
        .expect("the settling tick carries the 39/18 result");
    (
        i32::from_le_bytes(packet[11..15].try_into().unwrap()),
        packet[15],
    )
}

fn any_quest_progress(outcome: &Outcome) -> bool {
    outcome.replies.iter().any(|packet| {
        has_family(packet, 49, 9, Some(SCRIMMAGE)) || has_family(packet, 49, 10, Some(SCRIMMAGE))
    })
}

fn owned(game: &Gameplay, definition: i32) -> u32 {
    game.state()
        .inventory
        .items
        .iter()
        .filter(|item| item.definition_id == definition && item.rental.is_none())
        .map(|item| item.count)
        .sum()
}

/// A refused request: an error or an inert reply, never a quest packet or a state change.
fn assert_refused(player: &mut Player, packet: &[u8], label: &str) {
    let before = saved(&player.game);
    if let Ok(outcome) = player.try_send(packet) {
        assert!(!outcome.changed, "{label} changed gameplay");
        assert!(
            !outcome.replies.iter().any(|reply| [1, 3, 4, 9, 10, 13]
                .iter()
                .any(|&sub| has_family(reply, 49, sub, None))),
            "{label} sent a quest packet"
        );
    }
    assert_eq!(saved(&player.game), before, "{label} changed the save");
}

// ---- gates --------------------------------------------------------------------------------

#[test]
fn manager_landon_is_drawable_and_offers_become_a_soccer_star() {
    let at = landon_position();
    // The production draw list: exactly one actor binds Landon's content key, with his
    // client model at the authored position.
    let visibility = NpcVisibility::load(&content_dir()).expect("NPC visibility loads");
    let landons: Vec<_> = visibility
        .teleport_targets(0)
        .into_iter()
        .filter(|target| visibility.content_key(target.guid) == Some(LANDON))
        .collect();
    assert_eq!(landons.len(), 1, "one drawable Landon, no duplicate actor");
    assert_eq!(
        landons[0].guid, LANDON,
        "the runtime GUID is the content key"
    );
    assert!(
        visibility
            .model_actors()
            .any(|(_, model, position)| { model == LANDON_MODEL as u32 && position == at }),
        "model 1457 is placed at the authored position"
    );

    // Two sessions near the field receive the same 35/2 AddNpc for the same actor.
    let fixture = chapter_fixture();
    let mut first = Player::join(&fixture, CHARACTER);
    let mut second = Player::join(&fixture, OTHER);
    let mut adds = Vec::new();
    for player in [&mut first, &mut second] {
        let packets = player
            .game
            .update_npc_visibility(&player.character)
            .expect("the visibility batch builds");
        let landon: Vec<_> = packets
            .iter()
            .filter_map(|packet| parse_add_npc(packet).ok())
            .filter(|spawn| spawn.actor_id == LANDON)
            .collect();
        assert_eq!(landon.len(), 1, "each session draws Landon once");
        assert_eq!(landon[0].model_id, LANDON_MODEL);
        assert_eq!(landon[0].name_string_id, LANDON_NAME);
        assert_eq!(&landon[0].position[..3], &at);
        adds.push(landon[0].clone());
    }
    assert_eq!(adds[0], adds[1], "both sessions see the same actor");

    // The real 26/8 -> 26/9 menu offers the trainer label beside Talk.
    let menu = first.menu();
    assert!(menu_option(&menu, TALK_LABEL) > 0);
    assert!(menu_option(&menu, BECOME_A_SOCCER_STAR) > 0);

    // An out-of-range click and an unknown actor change nothing.
    let mut far = Player::join(&fixture, CHARACTER + 2);
    far.character.position[0] += 100.0;
    assert_refused(&mut far, &npc_interact(LANDON), "an out-of-range 26/8");
    assert_refused(
        &mut far,
        &npc_interact(0x7fff_0000_0000_0001),
        "an unknown actor's 26/8",
    );
    assert_refused(
        &mut first,
        &npc_interact(9_000_000_181),
        "another trainer's 26/8 from Snowhill",
    );
}

#[test]
fn learning_soccer_star_unlocks_and_activates_profile_52() {
    let fixture = chapter_fixture();
    let mut player = Player::join(&fixture, CHARACTER);
    {
        let state = player.game.state();
        let profiles = state.profiles.as_ref().unwrap();
        assert_eq!(profiles.active, ADVENTURER, "the real initial_state");
        assert!(!profiles.unlocked.contains(&SOCCER_STAR));
    }
    for definition in STARTER {
        assert_eq!(owned(&player.game, definition), 0, "no outfit before");
    }

    // Learn: unlocked, not yet active, the outfit granted and dressed once.
    let menu = player.menu();
    let learn = menu_option(&menu, BECOME_A_SOCCER_STAR);
    let learned = player.send(&npc_select(LANDON, learn));
    assert!(learned.changed);
    {
        let state = player.game.state();
        let profiles = state.profiles.as_ref().unwrap();
        assert!(profiles.unlocked.contains(&SOCCER_STAR));
        assert_eq!(profiles.active, ADVENTURER, "learning does not activate");
        for slot in [3, 4] {
            assert!(
                state.inventory.equipment.iter().any(|entry| {
                    entry.profile_id == SOCCER_STAR && entry.slot == slot && entry.item_id != 0
                }),
                "the starter outfit fills required slot {slot}"
            );
        }
    }
    for definition in STARTER {
        assert_eq!(owned(&player.game, definition), 1, "granted exactly once");
    }

    // A replayed selection and a fresh Learn change nothing and grant nothing.
    assert_refused(
        &mut player,
        &npc_select(LANDON, learn),
        "a replayed Learn selection",
    );
    let before = saved(&player.game);
    let again = player.learn();
    assert!(!again.changed, "a second Learn changes nothing");
    assert!(
        String::from_utf8_lossy(&again.replies.concat()).contains("You already know that job"),
        "the trainer refuses a known job"
    );
    assert_eq!(saved(&player.game), before);
    for definition in STARTER {
        assert_eq!(owned(&player.game, definition), 1, "no second outfit");
    }

    // Activate through the real 26/13 switch, then reconnect: the job survives.
    let switched = player.send(&select_job(SOCCER_STAR));
    assert!(switched.changed);
    assert_eq!(active_job(&player.game), SOCCER_STAR);
    player.reconnect(&fixture);
    let state = player.game.state();
    let profiles = state.profiles.as_ref().unwrap();
    assert_eq!(
        profiles.active, SOCCER_STAR,
        "the save keeps the active job"
    );
    assert!(profiles.unlocked.contains(&SOCCER_STAR));
    for definition in STARTER {
        assert_eq!(owned(&player.game, definition), 1);
    }

    // Busy: a menu opened before a soccer match cannot teach during it.
    let mut busy = Player::join(&fixture, OTHER);
    let menu = busy.menu();
    let learn = menu_option(&menu, BECOME_A_SOCCER_STAR);
    let played = busy.send(&play(SNOWHILL_SOCCER));
    assert!(played.scene_zoning.is_some(), "the portal admits the match");
    assert!(busy.game.is_soccer_active());
    let before = saved(&busy.game);
    let refused = busy.send(&npc_select(LANDON, learn));
    assert!(!refused.changed, "a busy Learn changes nothing");
    assert_eq!(saved(&busy.game), before);
    assert!(
        !busy
            .game
            .state()
            .profiles
            .unwrap()
            .unlocked
            .contains(&SOCCER_STAR)
    );
}

#[test]
fn the_snowy_scrimmage_is_offered_only_to_a_soccer_star() {
    let fixture = chapter_fixture();
    let mut player = Player::join(&fixture, CHARACTER);

    // Not a Soccer Star: nothing is offered, and forged quest replies change nothing.
    let mut now = player.stamp();
    assert_no_offer(
        &mut player.game,
        &player.character,
        LANDON,
        &mut now,
        "an Adventurer at Landon",
    );
    player.now = now;
    assert_refused(&mut player, &quest_reply(SCRIMMAGE, true), "a forged 49/2");
    assert_refused(&mut player, &quest_end_reply(SCRIMMAGE), "a forged 49/14");

    // Soccer Star unlocked but Brawler active: still nothing.
    player.learn();
    player.send(&select_job(BRAWLER));
    assert_eq!(active_job(&player.game), BRAWLER);
    let mut now = player.stamp();
    assert_no_offer(
        &mut player.game,
        &player.character,
        LANDON,
        &mut now,
        "Soccer Star unlocked, Brawler active",
    );
    player.now = now;

    // Active Soccer Star: the Talk dialogue lists the scrimmage and 49/1 carries its text.
    player.send(&select_job(SOCCER_STAR));
    let now = player.stamp();
    let dialogue = talk(&mut player.game, &player.character, LANDON, now);
    let rows = decode_dialogue_26_3(
        dialogue
            .replies
            .iter()
            .find(|packet| packet.starts_with(&[26, 0, 3, 0]))
            .expect("Talk opens the 26/3 dialogue"),
    )
    .expect("26/3 decodes")
    .choices;
    assert_eq!(
        rows.iter()
            .find(|row| row[0] == QUEST_CHOICE as u32)
            .map(|row| row[2]),
        Some(TITLE),
        "the quest row names Snowy Scrimmage"
    );
    let offered = player.take_scrimmage();
    let offer = decode_quest_offer_49_1(
        offered
            .replies
            .iter()
            .find(|packet| packet.starts_with(&[49, 0, 1, 0]))
            .unwrap(),
    )
    .unwrap();
    assert_eq!(
        (
            offer.quest_id,
            offer.intro_text_string_id,
            offer.name_string_id,
            offer.description_string_id
        ),
        (SCRIMMAGE, INTRO, TITLE, DESCRIPTION)
    );
    assert_eq!(quest_status(&player.game, SCRIMMAGE), QuestStatus::Active);
    assert!(
        player
            .game
            .state()
            .npc_quests
            .unwrap()
            .quests
            .get(&MEETING_THE_TEAM)
            .is_none(),
        "the disabled next record is never offered"
    );
}

#[test]
fn a_snowhill_soccer_win_advances_the_quest_and_pays_once() {
    let fixture = chapter_fixture();
    let mut player = Player::join(&fixture, CHARACTER);
    player.become_soccer_star();
    player.take_scrimmage();
    let coins_before = coins(&player.game);
    let xp_before = profile_xp(&player.game, SOCCER_STAR);
    let items_before = player.game.state().inventory.items.len();

    // Portal Play of Snowhill Soccer, a server-scored goal and the full-time settlement.
    let settled = player.win(SNOWHILL_SOCCER);
    assert_eq!(game_over(&settled.replies), (SNOWHILL_SOCCER as i32, 1));
    assert!(objective_update(&settled, SCRIMMAGE, 1, 1), "49/9 0 -> 1");
    assert!(objective_complete(&settled, SCRIMMAGE, 1), "49/10");
    assert_eq!(counts(&player.game, SCRIMMAGE), [1]);
    assert_eq!(quest_status(&player.game, SCRIMMAGE), QuestStatus::Active);
    assert_eq!(coins(&player.game), coins_before + MATCH_COINS);
    assert_eq!(profile_xp(&player.game, SOCCER_STAR), xp_before);

    // A reconnect before the return keeps the counter and pays nothing.
    player.reconnect(&fixture);
    assert_eq!(counts(&player.game, SCRIMMAGE), [1]);
    assert_eq!(pending_turnin(&player.game), None);
    assert_eq!(coins(&player.game), coins_before + MATCH_COINS);

    // The reward map names 52, so switching away before the turn-in still pays 52.
    player.send(&select_job(BRAWLER));
    let brawler_xp = profile_xp(&player.game, BRAWLER);

    // The return Talk holds the 49/13 and 26/6 releases it; the pending end screen
    // survives a reconnect and nothing is paid before the 49/14.
    let end = player.return_to_landon();
    assert_eq!(end_screen_reward(&end), (QUEST_COINS, QUEST_XP));
    player.reconnect(&fixture);
    assert_eq!(pending_turnin(&player.game), Some(SCRIMMAGE));
    assert_eq!(coins(&player.game), coins_before + MATCH_COINS);
    assert_eq!(profile_xp(&player.game, SOCCER_STAR), xp_before);

    let now = player.stamp();
    finish_quest(
        &mut player.game,
        &player.character,
        SCRIMMAGE,
        SOCCER_STAR,
        QUEST_COINS,
        QUEST_XP,
        now,
    );
    assert_eq!(
        profile_xp(&player.game, BRAWLER),
        brawler_xp,
        "the active Brawler gains none of the quest XP"
    );
    assert_eq!(
        player.game.state().inventory.items.len(),
        items_before,
        "the quest grants no item"
    );
    let coins_after = coins(&player.game);
    let xp_after = profile_xp(&player.game, SOCCER_STAR);
    assert_eq!(coins_after, coins_before + MATCH_COINS + QUEST_COINS);
    assert_eq!(xp_after, xp_before + QUEST_XP);

    // A replayed 49/14, a reconnect and a second Talk pay nothing and re-offer nothing.
    assert_refused(&mut player, &quest_end_reply(SCRIMMAGE), "a replayed 49/14");
    player.reconnect(&fixture);
    assert_eq!(
        quest_status(&player.game, SCRIMMAGE),
        QuestStatus::Completed
    );
    assert_refused(
        &mut player,
        &quest_end_reply(SCRIMMAGE),
        "a 49/14 after the reconnect",
    );
    player.send(&select_job(SOCCER_STAR));
    let mut now = player.stamp();
    assert_no_offer(
        &mut player.game,
        &player.character,
        LANDON,
        &mut now,
        "after the turn-in",
    );
    assert_eq!(coins(&player.game), coins_after);
    assert_eq!(profile_xp(&player.game, SOCCER_STAR), xp_after);
}

#[test]
fn a_loss_or_quit_does_not_credit_the_quest() {
    let fixture = chapter_fixture();
    let mut player = Player::join(&fixture, CHARACTER);
    player.become_soccer_star();
    player.take_scrimmage();

    // A goalless full time is a loss: a result, no grant and no quest packet.
    let quest_before = player.game.state().npc_quests;
    let coins_before = coins(&player.game);
    // T229: full time is a completed match, so a loss still pays the soccer Station Cash kind
    // (from the shipped earn table); nothing else in the save changes.
    let mut after_loss = player.game.state();
    after_loss.sc_wallet = wallet_after(after_loss.sc_wallet, EarnSource::SoccerMatch);
    let after_loss = serde_json::to_vec(&after_loss).unwrap();
    let at = player.stamp();
    enter_match(&mut player.game, &player.character, SNOWHILL_SOCCER, at);
    let (replies, lost_at) = tick_to_full_time(&mut player.game, &player.character, at + 200);
    player.now = lost_at;
    let lost = Outcome {
        replies,
        ..Outcome::default()
    };
    assert_eq!(game_over(&lost.replies), (SNOWHILL_SOCCER as i32, 0));
    assert!(!any_quest_progress(&lost), "a loss sends no 49/9 or 49/10");
    player.close_results();
    assert_eq!(player.game.state().npc_quests, quest_before);
    assert_eq!(coins(&player.game), coins_before, "a loss pays nothing");
    assert_eq!(
        saved(&player.game),
        after_loss,
        "a loss changes only the Station Cash wallet"
    );

    // A quit before full time removes the run and credits nothing.
    let at = player.stamp();
    let instance = enter_match(&mut player.game, &player.character, SNOWHILL_SOCCER, at);
    let quit = send(
        &mut player.game,
        &player.character,
        &lifecycle(6, instance),
        at + 500,
    );
    player.now = at + 500;
    assert!(
        quit.replies
            .iter()
            .any(|packet| packet.starts_with(&[39, 0, 19])),
        "the quit removes the run"
    );
    assert!(!any_quest_progress(&quit));
    assert!(
        !player.game.is_soccer_active(),
        "the quit releases the player"
    );
    for _ in 0..4 {
        let now = player.stamp();
        let later = player.game.tick(&player.character, now).unwrap();
        assert!(!any_quest_progress(&later));
    }
    assert_eq!(player.game.state().npc_quests, quest_before);
    assert_eq!(coins(&player.game), coins_before);
    assert_eq!(
        saved(&player.game),
        after_loss,
        "a quit leaves the save as it was"
    );

    // A win on another field (Sacred Grove, row 42) is not the Snowhill objective.
    let other = player.win(SACRED_GROVE_SOCCER);
    assert_eq!(game_over(&other.replies), (SACRED_GROVE_SOCCER as i32, 1));
    assert!(!any_quest_progress(&other), "game 42 does not credit 43");
    assert_eq!(player.game.state().npc_quests, quest_before);
    assert_eq!(coins(&player.game), coins_before + MATCH_COINS);
    assert_eq!(counts(&player.game, SCRIMMAGE), [0]);
}

#[test]
fn refusals_and_the_two_player_case() {
    let fixture = chapter_fixture();
    let mut first = Player::join(&fixture, CHARACTER);
    let mut second = Player::join(&fixture, OTHER);
    first.become_soccer_star();
    second.become_soccer_star();

    // Malformed requests end only that request.
    for (packet, label) in [
        (vec![26, 0, 8, 0, 1, 2, 3], "a short 26/8"),
        (vec![26, 0, 10, 0, 1, 2], "a short 26/10"),
        (vec![49, 0, 2, 0, 0, 0, 4, 0], "a short 49/2"),
        (vec![49, 0, 14, 0, 0, 0, 4], "a short 49/14"),
    ] {
        assert_refused(&mut second, &packet, label);
    }

    // The first player holds an offer; the second cannot accept or turn in its quest.
    let now = first.stamp();
    offer(&mut first.game, &first.character, LANDON, SCRIMMAGE, now);
    assert_refused(
        &mut second,
        &quest_reply(SCRIMMAGE, true),
        "another's offer",
    );
    assert_refused(
        &mut second,
        &quest_end_reply(SCRIMMAGE),
        "another's turn-in",
    );
    let now = first.stamp();
    accept(&mut first.game, &first.character, SCRIMMAGE, now);
    second.take_scrimmage();

    // Busy: during a match Landon's menu is refused and nothing changes.
    let at = second.stamp();
    let instance = enter_match(&mut second.game, &second.character, SNOWHILL_SOCCER, at);
    second.now = at + 300;
    let before = saved(&second.game);
    let busy = second.send(&npc_interact(LANDON));
    assert!(!busy.changed, "a busy 26/8 changes nothing");
    assert!(
        !busy
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[26, 0, 9, 0])),
        "no menu opens during a match"
    );
    assert_eq!(saved(&second.game), before);

    // Wrong place: a Soccer Star out of range gets nothing from Landon.
    let mut far = Player::join(&fixture, CHARACTER + 2);
    far.become_soccer_star();
    far.character.position[2] += 100.0;
    assert_refused(&mut far, &npc_interact(LANDON), "an out-of-range 26/8");

    // Independent progress: the first player's win credits only the first player,
    // while the second player's own match is still running and then quits.
    first.win(SNOWHILL_SOCCER);
    assert_eq!(counts(&first.game, SCRIMMAGE), [1]);
    assert_eq!(counts(&second.game, SCRIMMAGE), [0]);
    assert!(second.game.is_soccer_active());
    second.send(&lifecycle(6, instance));
    assert!(!second.game.is_soccer_active());
    assert_eq!(counts(&second.game, SCRIMMAGE), [0]);

    first.return_to_landon();
    let now = first.stamp();
    finish_quest(
        &mut first.game,
        &first.character,
        SCRIMMAGE,
        SOCCER_STAR,
        QUEST_COINS,
        QUEST_XP,
        now,
    );
    assert_eq!(quest_status(&second.game, SCRIMMAGE), QuestStatus::Active);
    assert_refused(
        &mut second,
        &quest_end_reply(SCRIMMAGE),
        "a 49/14 without its own win",
    );
}
