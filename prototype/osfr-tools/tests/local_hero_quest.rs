//! T277 / L1193: the Royal Courier's "Local Hero" entry quest (3000024).
//!
//! The quest's own 2014 locale block names one enemy: "Defeat the Thugawug Thug in south
//! Greenwood Forest" (389192, goal 389193). L1193 binds goal 1 to winning the Thugawug Thug!
//! battle (fight 14080, BJX fork battle 60, started from the Legacy body 4115 that
//! `entrances.json` row `thugawug_thug` places), then a final Talk to the Courier, then
//! 13 coins and 25 Adventurer (profile 1) XP. Before L1193 the goal was five kills of the
//! field Hooligan 930050, the Alpha Wolf's borrowed id (KNOWN-BUGS A25).
//!
//! These tests drive the production in-world router (`Gameplay::handle_in_world` ->
//! `gameplay/npc.rs` -> `QuestEngine`), the 41/108 Go and 41/109 leave of the battle and the
//! production shared-field seams, never engine internals. The starter, its fight and the
//! arena's arrival spot are read from the shipped data, not pinned here.
//!
//! Client wire evidence used here:
//! - 26/8 NPC interact reader `0x008b6bc0`; 26/10 menu writer `0x00a36f80`;
//!   26/6 dialogue reader `0x009f5890`; 26/4 ends the dialogue.
//! - 41/114 start screen; 41/108 Go (ctor `0x008b6e70`, sender `0x00938f00`); 41/109 leave
//!   (`tests/brawler_job_chain.rs`, L1192).
//! - 49/1 QuestInfo reader `0x00c7bb60`: quest id dword 0 at 6..10, intro at
//!   10..14, card title at 18..22 (the writer also repeats the title in the two
//!   probe-unresolved dwords).
//! - 49/2 accept constructor `0x00a86400`; 49/3 QuestAdd reader `0x00c7bd10`;
//!   49/13 end-screen writer/reader `0x00c7bbd0`/`0x00c7b990`; 49/14 reply
//!   constructor `0x00a864c0`.
//! - 49/9 ObjectiveUpdate / 49/10 GoalComplete / 49/8 ObjectiveActivated.
//!
//! The shared fixture, packet builders and shared-field/objective helpers live in
//! `tests/support/npc_quest_chain.rs`.

#[path = "support/npc_quest_chain.rs"]
mod npc_quest_chain;

use npc_quest_chain::*;
use osfr_tools::{
    character_store::CharacterState,
    encounter::EncounterPhase,
    game_store,
    gameplay::{Gameplay, Outcome},
    quest_engine::QuestStatus,
};
use std::collections::BTreeSet;

const QUEST: u32 = 3_000_024;
const COURIER: u64 = 9_000_000_444;
/// `title_string_id`, `intro_text_string_id`, `description_string_id` and
/// `end_text_string_id` of 3000024.
const TITLE_ID: u32 = 389_187;
const INTRO_ID: u32 = 389_186;
const DESCRIPTION_ID: u32 = 389_188;
const END_TEXT: u32 = 389_189;
/// The record's reward row and objective ordinals.
const REWARD_COINS: u32 = 13;
const REWARD_XP: u32 = 25;
const ADVENTURER: i32 = 1;
const MEDIC: i32 = 11;
const BRAWLER: i32 = 43;
const DEFEAT_OBJECTIVE: u32 = 1;
const TALK_OBJECTIVE: u32 = 2;
/// The `entrances.json` row whose battle is Local Hero's goal.
const THUG_ENTRANCE: &str = "thugawug_thug";
/// Two of the always-on level-3 930050 Hooligans of `starting_highroad_hooligans`
/// (data/content/field_enemies.json): the old goal's kind, which no longer counts.
const LIVE_HOOLIGANS: [u64; 2] = [131_000_102_004, 131_000_102_005];
const ATTACK_CHOICE: i32 = 103;
const A: u64 = 101;
const B: u64 = 202;

fn courier_character(guid: u64) -> CharacterState {
    character_at_for(guid, 0, content_position("greenwood_forest.json", COURIER))
}

/// The Thugawug Thug! starter, read from the shipped data: its body guid, where it stands,
/// its fight id and where that fight's arena puts the player.
struct ThugBattle {
    starter: u64,
    at: [f32; 3],
    fight: u32,
    arrive: [f32; 3],
}

fn thug_battle() -> ThugBattle {
    let point = |value: &serde_json::Value| -> [f32; 3] {
        let row = value.as_array().expect("a point is [x, y, z]");
        [0, 1, 2].map(|axis| row[axis].as_f64().unwrap() as f32)
    };
    let entrances = read_json(&content_dir().join("entrances.json"));
    let row = entrances["entrances"]
        .as_array()
        .and_then(|rows| rows.iter().find(|row| row["id"] == THUG_ENTRANCE))
        .expect("entrances.json places the Thugawug Thug! starter");
    let fight = u32::try_from(row["fight"].as_u64().unwrap()).unwrap();
    let file = read_json(&content_dir().join("fights").join(format!("{fight}.json")));
    ThugBattle {
        starter: row["guid"].as_u64().unwrap(),
        at: point(&row["at"]),
        fight,
        arrive: point(&file["place"]["arrive"]),
    }
}

/// The opening fixture plus the shipped fight files and the catalogue and pacing table they
/// load against: the opening package carries no fights, so the Thug's starter would open
/// nothing (as `tests/brawler_job_chain.rs` does for the Brawling Club, L1192).
fn battle_fixture() -> Fixture {
    let fixture = Fixture::new();
    copy_tree(&content_dir().join("fights"), &fixture.path().join("fights"));
    for file in ["enemies.json", "instance_pacing.json"] {
        fixture.copy_content(file);
    }
    fixture
}

/// The opening fixture's default profile state, with the chosen job active. The
/// quest has no `job` gate; a battle's Go needs a combat job.
fn game_with_job(fixture: &Fixture, character_id: u64, job: i32) -> Gameplay {
    let mut state = Gameplay::load(fixture.path(), None)
        .expect("the opening fixture loads")
        .state();
    let profiles = state
        .profiles
        .as_mut()
        .expect("the opening fixture stores profiles");
    profiles.active = job;
    profiles.unlocked.insert(job);
    fixture.game_as(character_id, Some(state))
}

/// The 49/1 offer for the record carries the four screen fields at the client
/// reader's offsets (intro, repeated title, card title, description).
fn assert_offer(outcome: &Outcome) {
    let offer = outcome
        .replies
        .iter()
        .find(|packet| has_family(packet, 49, 1, Some(QUEST)))
        .expect("the Courier sends the 49/1 Local Hero offer");
    let word = |offset: usize| u32::from_le_bytes(offer[offset..offset + 4].try_into().unwrap());
    assert_eq!(
        word(10),
        INTRO_ID,
        "49/1 intro field (client reader 0x00c7b7a0)"
    );
    assert_eq!(word(14), TITLE_ID, "49/1 repeats the card title");
    assert_eq!(word(18), TITLE_ID, "49/1 card title is Local Hero (389187)");
    assert_eq!(word(22), DESCRIPTION_ID, "49/1 description field");
}

/// Accept Local Hero at the Courier through the real 26/8 -> 26/10 -> 26/6 ->
/// 49/1 -> 49/2 path. Returns the game and the next free timestamp.
fn start_local_hero(fixture: &Fixture, character_id: u64) -> (Gameplay, u64) {
    let mut game = game_with_job(fixture, character_id, BRAWLER);
    let character = courier_character(character_id);
    let offered = offer(&mut game, &character, COURIER, QUEST, 1_000);
    assert_offer(&offered);
    accept(&mut game, &character, QUEST, 1_010);
    (game, 1_020)
}

/// Win Thugawug Thug! through the production route: click the starter, Go (41/108), fight
/// every enemy in the arena until it is quiet, then leave through the open exit (41/109) and
/// return to the world beside the Courier. Returns every reply from the Go on, where the
/// quest packets of the win are.
fn win_thug_battle(game: &mut Gameplay, home: &CharacterState, now: &mut u64) -> Outcome {
    let battle = thug_battle();
    let mut character = home.clone();
    character.zone_id = 0;
    character.position = battle.at;
    *now += 100;
    let card = game
        .handle_in_world(&npc_interact(battle.starter), &character, *now)
        .expect("the starter's click is valid");
    let details = card
        .replies
        .iter()
        .find(|packet| packet.starts_with(&[41, 0, 114, 0]))
        .cloned()
        .expect("the Thugawug Thug! starter opens its battle's start screen");
    let mut go = vec![41, 0, 108, 0];
    go.extend(&details[4..12]);
    go.extend(0_u64.to_le_bytes());
    *now += 100;
    let mut all = game
        .handle_in_world(&go, &character, *now)
        .expect("Go is valid");
    let zone = all.zoning.clone().unwrap_or_else(|| {
        let text: Vec<_> = all
            .replies
            .iter()
            .map(|p| String::from_utf8_lossy(p).into_owned())
            .collect();
        panic!("Go zones into Thugawug Thug! (fight {}): {text:?}", battle.fight)
    });
    character.zone_id = zone.zone_id;
    character.position = battle.arrive;
    *now += 100;
    let loaded = game
        .initialize_world(&character, *now)
        .expect("the arena loads");
    all.replies.extend(loaded.replies);

    // A fight file does not end by itself: after the last wave its exit opens, and the win
    // waits for the player's leave (41/109).
    let (mut target, mut quiet) = (None, 0);
    for _ in 0..3_000 {
        let Some(active) = game.state().encounter.and_then(|state| state.active) else {
            break;
        };
        assert_eq!(
            active.phase,
            EncounterPhase::Fighting,
            "Thugawug Thug! is lost"
        );
        *now += 700;
        let living = active
            .enemies
            .iter()
            .find(|enemy| enemy.health > 0 && !enemy.dormant);
        if let Some(enemy) = living {
            quiet = 0;
            character.position = enemy.position;
            if target != Some(enemy.guid) {
                target = Some(enemy.guid);
                let _ = game.handle_in_world(&npc_interact(enemy.guid), &character, *now);
            }
            if let Ok(out) =
                game.handle_in_world(&npc_select(enemy.guid, ATTACK_CHOICE), &character, *now)
            {
                all.replies.extend(out.replies);
            }
        } else {
            quiet += 1;
            if quiet >= 12 {
                break;
            }
        }
        let ticked = game.tick(&character, *now).expect("the arena ticks");
        all.replies.extend(ticked.replies);
    }
    let mut leave = vec![41, 0, 109, 0];
    leave.extend(0_i32.to_le_bytes());
    leave.extend(&details[8..12]);
    *now += 100;
    let left = game
        .handle_in_world(&leave, &character, *now)
        .expect("the leave is valid");
    assert!(left.zoning.is_some(), "the leave returns to the world");
    all.replies.extend(left.replies);
    *now += 100;
    let back = game
        .initialize_world(home, *now)
        .expect("the world return loads");
    all.replies.extend(back.replies);
    *now += 100;
    all
}

fn reward_packets(outcome: &Outcome, subtype: u16) -> usize {
    outcome
        .replies
        .iter()
        .filter(|packet| has_family(packet, 38, subtype, None))
        .count()
}

#[test]
fn the_royal_courier_offers_local_hero_to_every_job() {
    let fixture = Fixture::new();
    for (index, job) in [BRAWLER, MEDIC, ADVENTURER].into_iter().enumerate() {
        let character_id = 300 + index as u64;
        let mut game = game_with_job(&fixture, character_id, job);
        let character = courier_character(character_id);
        let offered = offer(&mut game, &character, COURIER, QUEST, 1_000);
        assert_offer(&offered);
        assert_eq!(
            quest_status(&game, QUEST),
            QuestStatus::Offered,
            "job {job} sees exactly one Local Hero offer"
        );
        accept(&mut game, &character, QUEST, 1_010);
        assert_eq!(quest_status(&game, QUEST), QuestStatus::Active);

        // A second Talk while the quest is active must not offer it again.
        let mut now = 1_100;
        assert_no_offer(
            &mut game,
            &character,
            COURIER,
            &mut now,
            &format!("active Local Hero talk as job {job}"),
        );
        assert_eq!(counts(&game, QUEST), [0, 0]);
        assert_eq!(quest_status(&game, QUEST), QuestStatus::Active);
    }
}

/// (Name kept from T277.) Winning Thugawug Thug! completes the goal once; the old goal's
/// field Hooligans (930050) no longer count, and a second win changes nothing.
#[test]
fn five_hooligan_kills_advance_the_hunt_once() {
    let fixture = battle_fixture();

    // The field Hooligans the old goal counted are just field enemies now, killed in the
    // production shared field by a second player on the quest.
    let (mut hunter, mut hunter_now) = start_local_hero(&fixture, B);
    let hunter_at = courier_character(B);
    let mut field = hunter
        .take_shared_fields()
        .expect("the opening field is available");
    let known = BTreeSet::from(LIVE_HOOLIGANS);
    for target in LIVE_HOOLIGANS {
        let (out, next) =
            kill_shared_target(&mut hunter, &mut field, &known, &hunter_at, target, hunter_now);
        hunter_now = next;
        assert_eq!(field.health(target), Some(0));
        assert!(
            !objective_update(&out, QUEST, DEFEAT_OBJECTIVE, 1)
                && !objective_complete(&out, QUEST, DEFEAT_OBJECTIVE),
            "a 930050 kill produces no Local Hero objective packet"
        );
    }
    assert_eq!(
        counts(&hunter, QUEST),
        [0, 0],
        "field Hooligans no longer count"
    );

    let (mut game, mut now) = start_local_hero(&fixture, A);
    let courier = courier_character(A);
    let won = win_thug_battle(&mut game, &courier, &mut now);
    assert_eq!(counts(&game, QUEST), [1, 0], "one win is the goal");
    assert_eq!(quest_status(&game, QUEST), QuestStatus::Active);
    assert!(
        objective_complete(&won, QUEST, DEFEAT_OBJECTIVE),
        "the win sends 49/10 GoalComplete"
    );
    assert!(
        objective_activated(&won, QUEST, TALK_OBJECTIVE),
        "the completed goal activates the Courier Talk objective"
    );

    // A second win of the repeatable battle does not advance or re-complete the goal.
    let again = win_thug_battle(&mut game, &courier, &mut now);
    assert_eq!(counts(&game, QUEST), [1, 0]);
    assert!(
        !objective_complete(&again, QUEST, DEFEAT_OBJECTIVE),
        "a second win sends no second GoalComplete"
    );
}

#[test]
fn the_courier_turn_in_pays_thirteen_coins_and_adventurer_xp_once() {
    let fixture = battle_fixture();
    let (mut game, mut now) = start_local_hero(&fixture, A);
    let courier = courier_character(A);
    win_thug_battle(&mut game, &courier, &mut now);
    assert_eq!(counts(&game, QUEST), [1, 0]);

    let talked = talk(&mut game, &courier, COURIER, now);
    now += 10;
    assert!(talked.changed, "the final Talk advances the objective");
    assert_eq!(counts(&game, QUEST), [1, 1]);
    assert_eq!(pending_turnin(&game), Some(QUEST));
    release_end_screen(&mut game, &courier, QUEST, COURIER, END_TEXT, now);
    now += 10;

    let coins_before = coins(&game);
    let adventurer_before = profile_xp(&game, ADVENTURER);
    let brawler_before = profile_xp(&game, BRAWLER);
    assert_eq!(
        active_job(&game),
        BRAWLER,
        "the reward must not come from the active job"
    );
    let completed = finish_quest(
        &mut game,
        &courier,
        QUEST,
        ADVENTURER,
        REWARD_COINS,
        REWARD_XP,
        now,
    );
    now += 10;
    assert_eq!(coins(&game), coins_before + REWARD_COINS);
    assert_eq!(
        profile_xp(&game, ADVENTURER),
        adventurer_before + REWARD_XP,
        "profile 1 (Adventurer) receives the XP"
    );
    assert_eq!(
        profile_xp(&game, BRAWLER),
        brawler_before,
        "the active Brawler job receives no quest XP"
    );
    assert_eq!(reward_packets(&completed, 19), 1, "one coin grant");
    assert_eq!(reward_packets(&completed, 14), 1, "one XP grant");

    // Paid once: a replayed 49/14 changes neither state nor save.
    assert_packet_changes_nothing(
        &mut game,
        &courier,
        &quest_end_reply(QUEST),
        now,
        "replayed Local Hero turn-in",
    );
    assert_eq!(coins(&game), coins_before + REWARD_COINS);
    assert_eq!(profile_xp(&game, ADVENTURER), adventurer_before + REWARD_XP);
}

#[test]
fn a_refused_or_out_of_range_request_changes_nothing() {
    let fixture = Fixture::new();
    let mut game = game_with_job(&fixture, A, BRAWLER);
    let courier = courier_character(A);
    let mut now = 5_000;

    // A non-giver NPC's Talk cannot offer or start Local Hero.
    let vendor = character_at_for(A, 0, content_position("greenwood_forest.json", 9_500_000_285));
    assert_no_offer(
        &mut game,
        &vendor,
        9_500_000_285,
        &mut now,
        "Local Hero at a non-giver",
    );

    // Forged accept with no offer open.
    assert_packet_changes_nothing(
        &mut game,
        &courier,
        &quest_reply(QUEST, true),
        now + 1,
        "forged Local Hero accept",
    );
    // A 49/14 with no pending end screen.
    assert_packet_changes_nothing(
        &mut game,
        &courier,
        &quest_end_reply(QUEST),
        now + 2,
        "Local Hero turn-in with no end screen",
    );

    // 26/8 out of range is refused before any menu opens.
    let far = character_at_for(
        A,
        0,
        [
            courier.position[0] + 400.0,
            courier.position[1],
            courier.position[2],
        ],
    );
    let before = saved(&game);
    let refused = game.handle_in_world(&npc_interact(COURIER), &far, now + 3);
    assert!(
        refused.is_ok_and(|out| out.replies.is_empty() && !out.changed),
        "the out-of-range interact is refused"
    );
    assert_eq!(
        saved(&game),
        before,
        "the out-of-range interact changed the state"
    );

    // A real offer, then a duplicate offer attempt keeps the open offer intact.
    now += 10;
    let offered = offer(&mut game, &courier, COURIER, QUEST, now);
    assert_offer(&offered);
    let held = saved(&game);
    now += 10;
    let menu = interact(&mut game, &courier, COURIER, now);
    let choice = menu_option(&menu, TALK_LABEL);
    let dialogue = send(&mut game, &courier, &npc_select(COURIER, choice), now + 1);
    assert!(
        dialogue
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[26, 0, 3, 0])),
        "the second Talk still opens the dialogue"
    );
    let duplicate = send(&mut game, &courier, &dialogue_choice(QUEST_CHOICE), now + 2);
    assert!(
        !duplicate
            .replies
            .iter()
            .any(|packet| has_family(packet, 49, 1, Some(QUEST))),
        "an open offer is not offered a second time"
    );
    assert!(!duplicate.changed);
    assert_eq!(
        saved(&game),
        held,
        "the duplicate offer attempt changed the state"
    );

    // Accept, then a declined 49/2 with no pending offer is a refusal.
    accept(&mut game, &courier, QUEST, now + 3);
    assert_packet_changes_nothing(
        &mut game,
        &courier,
        &quest_reply(QUEST, false),
        now + 4,
        "declined 49/2 with no pending offer",
    );
    assert_eq!(quest_status(&game, QUEST), QuestStatus::Active);
}

/// Save and reload through the real character-document envelope.
fn reload(fixture: &Fixture, game: &Gameplay, character: &CharacterState, name: &str) -> Gameplay {
    let save_path = fixture.path().join(name);
    let held = game.state();
    game_store::save(
        &save_path,
        &game_store::Snapshot {
            character: character.clone(),
            gameplay: Some(held.clone()),
            pets: None,
        },
    )
    .expect("the character document saves");
    let loaded = game_store::load_or_create(&save_path, character.clone())
        .expect("the character document loads");
    assert_eq!(loaded.gameplay.as_ref(), Some(&held));
    fixture.game_as(character.guid, loaded.gameplay)
}

#[test]
fn progress_survives_a_reconnect() {
    let fixture = battle_fixture();
    let (game, mut now) = start_local_hero(&fixture, A);
    let character = courier_character(A);

    // Disconnect with the quest just accepted, then reconnect.
    let mut game = reload(&fixture, &game, &character, "reconnect-accepted.json");
    assert_eq!(counts(&game, QUEST), [0, 0]);
    assert_eq!(quest_status(&game, QUEST), QuestStatus::Active);

    // Win, then disconnect with the goal done: it survives the reload.
    win_thug_battle(&mut game, &character, &mut now);
    assert_eq!(counts(&game, QUEST), [1, 0]);
    let mut game = reload(&fixture, &game, &character, "reconnect-won.json");
    assert_eq!(counts(&game, QUEST), [1, 0], "the won goal survives reload");
    assert_eq!(quest_status(&game, QUEST), QuestStatus::Active);
    // After the battle's own payout: the quest's reward is counted from here.
    let coins_before = coins(&game);
    let adventurer_before = profile_xp(&game, ADVENTURER);

    let talked = talk(&mut game, &character, COURIER, now);
    now += 10;
    assert!(talked.changed);
    release_end_screen(&mut game, &character, QUEST, COURIER, END_TEXT, now);
    now += 10;
    finish_quest(
        &mut game,
        &character,
        QUEST,
        ADVENTURER,
        REWARD_COINS,
        REWARD_XP,
        now,
    );
    let expected_bytes = saved(&game);

    // A further reload keeps the completed history and the paid balance.
    let mut game = reload(&fixture, &game, &character, "reconnect-finished.json");
    assert_eq!(
        saved(&game),
        expected_bytes,
        "the reload changed the save"
    );
    assert_eq!(counts(&game, QUEST), [1, 1]);
    assert_eq!(quest_status(&game, QUEST), QuestStatus::Completed);
    assert_eq!(coins(&game), coins_before + REWARD_COINS);
    assert_eq!(profile_xp(&game, ADVENTURER), adventurer_before + REWARD_XP);
    assert_packet_changes_nothing(
        &mut game,
        &character,
        &quest_end_reply(QUEST),
        now + 10,
        "replayed turn-in after reload",
    );
    assert_eq!(coins(&game), coins_before + REWARD_COINS);
    assert_eq!(profile_xp(&game, ADVENTURER), adventurer_before + REWARD_XP);
}

/// (Name kept from T277.) One player's win of Thugawug Thug! credits that player only.
#[test]
fn a_second_player_killing_the_same_hooligans_does_not_advance_the_first() {
    let fixture = battle_fixture();
    let (mut a, mut now) = start_local_hero(&fixture, A);
    let (mut b, _) = start_local_hero(&fixture, B);
    let a_character = courier_character(A);
    let b_character = courier_character(B);

    let a_before = a.state();
    win_thug_battle(&mut b, &b_character, &mut now);
    assert_eq!(counts(&b, QUEST), [1, 0], "the winner is credited");
    assert_eq!(counts(&a, QUEST), [0, 0], "the other player is not credited");
    assert_eq!(a.state(), a_before, "B's win changed A's saved state");

    let b_after = b.state();
    win_thug_battle(&mut a, &a_character, &mut now);
    assert_eq!(counts(&a, QUEST), [1, 0], "A's own win credits A");
    assert_eq!(b.state(), b_after, "A's win changed B's saved state");
}

#[test]
fn a_failing_quest_event_for_one_player_does_not_stop_the_other() {
    let fixture = battle_fixture();

    // A's save belongs to another character: every production bind refuses it.
    let (valid, _) = start_local_hero(&fixture, A);
    let mut stale = valid.state();
    stale
        .npc_quests
        .as_mut()
        .expect("Local Hero is active")
        .character_id = B;
    let mut a = Gameplay::load(fixture.path(), Some(stale)).expect("A's stale session loads");
    let error = a
        .bind_npc_quest_catalogue(&content_dir(), A)
        .err()
        .expect("A's stale quest save is refused");
    assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);

    let (mut b, mut now) = start_local_hero(&fixture, B);
    let b_character = courier_character(B);

    // A's win is played but cannot credit the unbound stale state.
    win_thug_battle(&mut a, &courier_character(A), &mut now);
    assert_eq!(counts(&a, QUEST), [0, 0], "the stale save is not credited");

    // B's engine still advances normally.
    let out = win_thug_battle(&mut b, &b_character, &mut now);
    assert_eq!(counts(&b, QUEST), [1, 0]);
    assert!(
        objective_complete(&out, QUEST, DEFEAT_OBJECTIVE),
        "B's win still completes B's goal"
    );

    // Near-cap variant: A's reward cannot fit the wire i32 coin field. The
    // refusal is A's alone; B stays usable.
    let (mut capped, mut now_c) = start_local_hero(&fixture, A);
    let a_character = courier_character(A);
    win_thug_battle(&mut capped, &a_character, &mut now_c);
    let talked = talk(&mut capped, &a_character, COURIER, now_c);
    now_c += 10;
    assert!(talked.changed);
    assert_eq!(pending_turnin(&capped), Some(QUEST));
    release_end_screen(&mut capped, &a_character, QUEST, COURIER, END_TEXT, now_c);
    now_c += 10;
    let mut state = capped.state();
    state.inventory.coins = i32::MAX as u32;
    let mut capped = fixture.game_as(A, Some(state));
    let before = saved(&capped);
    let refused = capped
        .handle_in_world(&quest_end_reply(QUEST), &a_character, now_c)
        .expect("a coin grant past i32::MAX is a refusal, not an error (T382)");
    assert!(
        !refused.changed,
        "a coin grant past i32::MAX is refused for that player"
    );
    assert_eq!(
        saved(&capped),
        before,
        "the refused payout changed A's save"
    );
    assert_eq!(pending_turnin(&capped), Some(QUEST));

    // B still finishes its own goal at the Courier.
    let talked = talk(&mut b, &b_character, COURIER, now);
    assert!(talked.changed);
    assert_eq!(counts(&b, QUEST), [1, 1]);
}
