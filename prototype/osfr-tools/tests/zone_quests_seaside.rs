//! T466: the Seaside Tour (`3000320`-`3000323`) and Feminine Touch (`3000360`),
//! community zone quests imported as data only into
//! `data/content/quests/zone_seaside.json` at existing Seaside `legacy_data` rows.
//!
//! Every test drives the production paths in `tests/support/npc_quest_chain.rs`:
//! 26/8 -> 26/10 -> 26/6 offers (49/1, decoded with the production reader), 49/2
//! accepts, `Gameplay::on_location` for the visit goals, Talk steps, the 26/6 that
//! releases the held 49/13, and the 49/14 turn-in.
//!
//! Client readers (IDA, 2014 client; no new wire): 26/3 payload reader `0x00a9ef10`
//! stores the first dword after the 4-byte header as the dialogue body, which a
//! completed talk step fills with its `dialogue_string_id`; 49/13
//! `QuestEndPacket::ReadFields` `0x00c7b990` reads the actor at `6..14`, quest
//! `14..18`, end text `18..22`, one dword, then the reward bundle `0x008e7930`.

#[path = "support/npc_quest_chain.rs"]
mod npc_quest_chain;

use npc_quest_chain::{
    CHARACTER, Fixture, accept, active_job, assert_no_offer, assert_packet_changes_nothing,
    character_at_for, coins, content_position, counts, end_screen_reward, finish_quest, has_family,
    interact, menu_option, npc_select, objective_complete, offer, pending_turnin, profile_xp,
    quest_end_reply, quest_reply, quest_status, release_end_screen, saved, seed_active_profile,
    talk,
};
use osfr_tools::{
    character_store::CharacterState,
    gameplay::{Gameplay, Outcome},
    npc_interaction::{decode_dialogue_26_3, decode_quest_offer_49_1},
    quest_engine::QuestStatus,
};

const SPRITZY: u64 = 9_500_000_629;
const COLE: u64 = 9_500_000_567;
const SEASPRAY: u64 = 9_500_000_623;
const DIGGY: u64 = 9_000_000_070;
const BRENNA: u64 = 9_500_000_562;
const YANCY: u64 = 9_500_000_645;
const TAZINNA: u64 = 9_500_000_633;
const EMUZZ: u64 = 9_500_000_580;
const OTHER: u64 = CHARACTER + 1;
const ADVENTURER: i32 = 1;
const MINER: i32 = 14;
/// The fork's RewardCoins on every quest here (R).
const COINS: u32 = 30;

/// The fork's hand-placed visit points (R), as authored in `zone_seaside.json`.
const GAZEBO: [f32; 3] = [-690.38, 2.3, -1060.25];
const KOODINI: [f32; 3] = [-379.92, -6.58, -1168.87];
const BOARDWALK: [f32; 3] = [-426.24, -18.72, -1223.69];

enum Step {
    Visit([f32; 3]),
    /// A talk goal and the 26/3 body its NPC speaks when the step completes (0: none).
    Talk(u64, u32),
}

struct Quest {
    id: u32,
    giver: u64,
    /// Title, intro and description ids the 49/1 card carries.
    card: [u32; 3],
    /// Every goal before the final Talk at `turnin`.
    steps: &'static [Step],
    turnin: u64,
    end_text: u32,
    xp: u32,
}

const WELCOME: Quest = Quest {
    id: 3_000_320,
    giver: SPRITZY,
    card: [93_283, 93_282, 93_284],
    steps: &[Step::Visit(GAZEBO), Step::Talk(SPRITZY, 93_316)],
    turnin: COLE,
    end_text: 93_335,
    xp: 20,
};
const OLDEST: Quest = Quest {
    id: 3_000_321,
    giver: COLE,
    card: [93_399, 93_398, 140_222],
    steps: &[Step::Talk(SEASPRAY, 93_409)],
    turnin: COLE,
    end_text: 93_401,
    xp: 20,
};
const TOURING: Quest = Quest {
    id: 3_000_322,
    giver: COLE,
    card: [93_420, 93_419, 93_421],
    steps: &[
        Step::Visit(KOODINI),
        Step::Visit(BOARDWALK),
        Step::Talk(DIGGY, 0),
    ],
    turnin: COLE,
    end_text: 93_422,
    xp: 30,
};
const TESTING: Quest = Quest {
    id: 3_000_323,
    giver: COLE,
    card: [93_447, 93_449, 93_454],
    steps: &[Step::Talk(BRENNA, 93_464)],
    turnin: COLE,
    end_text: 93_451,
    xp: 30,
};
const FEMININE: Quest = Quest {
    id: 3_000_360,
    giver: YANCY,
    card: [101_400, 101_408, 101_410],
    steps: &[Step::Talk(TAZINNA, 101_412), Step::Talk(EMUZZ, 101_414)],
    turnin: YANCY,
    end_text: 101_411,
    xp: 20,
};
const TOUR: [&Quest; 4] = [&WELCOME, &OLDEST, &TOURING, &TESTING];

fn at(guid: u64, npc: u64) -> CharacterState {
    character_at_for(guid, 0, content_position("seaside.json", npc))
}

fn standing(guid: u64, position: [f32; 3]) -> CharacterState {
    character_at_for(guid, 0, position)
}

fn player(fixture: &Fixture, guid: u64, active: i32) -> Gameplay {
    let mut state = fixture.game_as(guid, None).state();
    seed_active_profile(&mut state, active, &[ADVENTURER]);
    fixture.game_as(guid, Some(state))
}

fn dialogue_body(outcome: &Outcome) -> u32 {
    let packet = outcome
        .replies
        .iter()
        .find(|packet| packet.starts_with(&[26, 0, 3, 0]))
        .expect("a Talk answers 26/3");
    decode_dialogue_26_3(packet)
        .expect("the 26/3 decodes")
        .initial_dwords[0]
}

fn accept_quest(game: &mut Gameplay, guid: u64, quest: &Quest, now: &mut u64) {
    let giver = at(guid, quest.giver);
    let offered = offer(game, &giver, quest.giver, quest.id, *now);
    let card = offered
        .replies
        .iter()
        .find(|packet| has_family(packet, 49, 1, Some(quest.id)))
        .and_then(|packet| decode_quest_offer_49_1(packet).ok())
        .expect("the 49/1 card decodes");
    assert_eq!(
        [
            card.name_string_id,
            card.intro_text_string_id,
            card.description_string_id
        ],
        quest.card,
        "quest {}: 49/1 title, intro and description",
        quest.id
    );
    accept(game, &giver, quest.id, *now + 2);
    *now += 10;
}

fn do_step(game: &mut Gameplay, guid: u64, quest: &Quest, index: usize, now: &mut u64) {
    let outcome = match quest.steps[index] {
        Step::Visit(point) => game
            .on_location(&standing(guid, point), *now)
            .expect("the visit position is accepted"),
        Step::Talk(npc, body) => {
            let talked = talk(game, &at(guid, npc), npc, *now);
            assert_eq!(
                dialogue_body(&talked),
                body,
                "quest {} step {}: the NPC's 26/3 line",
                quest.id,
                index + 1
            );
            talked
        }
    };
    assert!(
        outcome.changed,
        "quest {} step {} did not advance",
        quest.id,
        index + 1
    );
    assert_eq!(counts(game, quest.id)[index], 1);
    *now += 10;
}

fn turn_in(game: &mut Gameplay, guid: u64, quest: &Quest, now: &mut u64) {
    let turnin = at(guid, quest.turnin);
    let talked = talk(game, &turnin, quest.turnin, *now);
    assert!(talked.changed, "quest {}: the final Talk", quest.id);
    assert_eq!(pending_turnin(game), Some(quest.id));
    let end = release_end_screen(
        game,
        &turnin,
        quest.id,
        quest.turnin,
        quest.end_text,
        *now + 2,
    );
    assert_eq!(end_screen_reward(&end), (COINS, quest.xp));
    finish_quest(
        game,
        &turnin,
        quest.id,
        ADVENTURER,
        COINS,
        quest.xp,
        *now + 4,
    );
    *now += 10;
}

fn run(game: &mut Gameplay, guid: u64, quest: &Quest, now: &mut u64) {
    accept_quest(game, guid, quest, now);
    for index in 0..quest.steps.len() {
        do_step(game, guid, quest, index, now);
    }
    turn_in(game, guid, quest, now);
}

#[test]
fn seaside_tour_runs_in_order_and_pays_once() {
    let fixture = Fixture::minimal(CHARACTER);
    let mut game = player(&fixture, CHARACTER, ADVENTURER);
    let (coins_before, xp_before) = (coins(&game), profile_xp(&game, ADVENTURER));
    let mut now = 1_000;
    for quest in TOUR {
        run(&mut game, CHARACTER, quest, &mut now);
    }
    assert_eq!(coins(&game), coins_before + 120);
    assert_eq!(profile_xp(&game, ADVENTURER), xp_before + 100);
    for quest in TOUR {
        assert_eq!(quest_status(&game, quest.id), QuestStatus::Completed);
    }
    let cole = at(CHARACTER, COLE);
    assert_packet_changes_nothing(
        &mut game,
        &cole,
        &quest_end_reply(TESTING.id),
        now,
        "a duplicate 49/14",
    );
    assert_no_offer(&mut game, &cole, COLE, &mut now, "Cole after the tour");
}

#[test]
fn next_tour_quest_waits_for_the_last() {
    let fixture = Fixture::minimal(CHARACTER);
    let mut game = player(&fixture, CHARACTER, ADVENTURER);
    let cole = at(CHARACTER, COLE);
    let mut now = 1_000;
    assert_no_offer(&mut game, &cole, COLE, &mut now, "Cole before Welcome");
    let forged = quest_reply(OLDEST.id, true);
    assert_packet_changes_nothing(&mut game, &cole, &forged, now, "a forged 49/2");

    accept_quest(&mut game, CHARACTER, &WELCOME, &mut now);
    assert_packet_changes_nothing(&mut game, &cole, &forged, now, "a 49/2 mid-Welcome");
    for index in 0..WELCOME.steps.len() {
        do_step(&mut game, CHARACTER, &WELCOME, index, &mut now);
    }
    turn_in(&mut game, CHARACTER, &WELCOME, &mut now);

    // Cole now offers The Oldest Building, never the tour quest after it.
    offer(&mut game, &cole, COLE, OLDEST.id, now);
    assert_packet_changes_nothing(
        &mut game,
        &cole,
        &quest_reply(TOURING.id, true),
        now + 10,
        "a forged 49/2 for Touring Seaside",
    );
}

#[test]
fn visit_points_need_the_radius() {
    let fixture = Fixture::minimal(CHARACTER);
    let mut game = player(&fixture, CHARACTER, ADVENTURER);
    let mut now = 1_000;
    accept_quest(&mut game, CHARACTER, &WELCOME, &mut now);
    let east = |metres: f32| standing(CHARACTER, [GAZEBO[0] + metres, GAZEBO[1], GAZEBO[2]]);

    let mut wrong_zone = standing(CHARACTER, GAZEBO);
    wrong_zone.zone_id = 2;
    for (refused, label) in [(east(15.1), "just outside 15 m"), (wrong_zone, "zone 2")] {
        let before = saved(&game);
        let moved = game
            .on_location(&refused, now)
            .expect("a position is accepted");
        assert!(!moved.changed, "{label} credited the visit");
        assert_eq!(saved(&game), before, "{label} changed the save");
        now += 10;
    }

    let visited = game.on_location(&east(14.9), now).expect("inside 15 m");
    assert!(visited.changed);
    assert!(objective_complete(&visited, WELCOME.id, 1));
    assert_eq!(counts(&game, WELCOME.id), [1, 0, 0]);

    let before = saved(&game);
    let again = game.on_location(&standing(CHARACTER, GAZEBO), now + 10);
    assert!(!again.expect("re-entry is accepted").changed);
    assert_eq!(saved(&game), before, "re-entering counted again");
}

#[test]
fn feminine_touch_completes() {
    let fixture = Fixture::minimal(CHARACTER);
    let mut game = player(&fixture, CHARACTER, ADVENTURER);
    let (coins_before, xp_before) = (coins(&game), profile_xp(&game, ADVENTURER));
    let mut now = 1_000;
    accept_quest(&mut game, CHARACTER, &FEMININE, &mut now);
    // The goals are ordered: Emuzz before Tazinna is the wrong NPC.
    let emuzz = at(CHARACTER, EMUZZ);
    assert_no_offer(&mut game, &emuzz, EMUZZ, &mut now, "Emuzz first");
    for index in 0..FEMININE.steps.len() {
        do_step(&mut game, CHARACTER, &FEMININE, index, &mut now);
    }
    turn_in(&mut game, CHARACTER, &FEMININE, &mut now);
    assert_eq!(coins(&game), coins_before + COINS);
    assert_eq!(profile_xp(&game, ADVENTURER), xp_before + FEMININE.xp);
}

#[test]
fn progress_survives_reconnect() {
    let fixture = Fixture::minimal(CHARACTER);
    let mut game = player(&fixture, CHARACTER, ADVENTURER);
    let mut now = 1_000;
    run(&mut game, CHARACTER, &WELCOME, &mut now);
    run(&mut game, CHARACTER, &OLDEST, &mut now);
    accept_quest(&mut game, CHARACTER, &TOURING, &mut now);

    // Ordered goals: the Boardwalk before the fortune teller does not count.
    let boardwalk = game.on_location(&standing(CHARACTER, BOARDWALK), now);
    assert!(!boardwalk.expect("the Boardwalk is accepted").changed);
    do_step(&mut game, CHARACTER, &TOURING, 0, &mut now);

    let mut game = fixture.reload_serialized(&game);
    assert_eq!(quest_status(&game, TOURING.id), QuestStatus::Active);
    assert_eq!(counts(&game, TOURING.id), [1, 0, 0, 0]);
    for index in 1..TOURING.steps.len() {
        do_step(&mut game, CHARACTER, &TOURING, index, &mut now);
    }
    turn_in(&mut game, CHARACTER, &TOURING, &mut now);
}

#[test]
fn early_turn_in_is_refused() {
    let fixture = Fixture::minimal(CHARACTER);
    let mut game = player(&fixture, CHARACTER, ADVENTURER);
    let cole = at(CHARACTER, COLE);
    let mut now = 1_000;
    accept_quest(&mut game, CHARACTER, &WELCOME, &mut now);
    do_step(&mut game, CHARACTER, &WELCOME, 0, &mut now);

    // Spritzy's step is open: Cole neither credits nor opens the end screen.
    assert_no_offer(&mut game, &cole, COLE, &mut now, "Cole with a step open");
    assert_eq!(counts(&game, WELCOME.id), [1, 0, 0]);
    assert_eq!(pending_turnin(&game), None);

    // Cole's menu lists the 3000320 turn-in (`quests_accepted`); choosing it is refused.
    let before = saved(&game);
    let menu = interact(&mut game, &cole, COLE, now);
    let choice = menu_option(&menu, WELCOME.card[0]);
    if let Ok(chosen) = game.handle_in_world(&npc_select(COLE, choice), &cole, now + 1) {
        assert!(!chosen.changed, "the menu turn-in changed gameplay");
        assert!(
            !chosen
                .replies
                .iter()
                .any(|packet| has_family(packet, 49, 13, None))
        );
    }
    assert_eq!(saved(&game), before, "the menu turn-in changed the save");

    let forged = quest_end_reply(WELCOME.id);
    assert_packet_changes_nothing(&mut game, &cole, &forged, now + 10, "an early 49/14");
}

#[test]
fn xp_goes_to_adventurer_with_another_job_active() {
    let fixture = Fixture::minimal(CHARACTER);
    let mut game = player(&fixture, CHARACTER, MINER);
    let miner_before = profile_xp(&game, MINER);
    let mut now = 1_000;
    run(&mut game, CHARACTER, &FEMININE, &mut now);
    assert_eq!(active_job(&game), MINER);
    assert_eq!(profile_xp(&game, MINER), miner_before);
}

#[test]
fn two_players_are_independent() {
    let fixture = Fixture::minimal(CHARACTER);
    let mut a = player(&fixture, CHARACTER, ADVENTURER);
    let mut b = player(&fixture, OTHER, ADVENTURER);
    let mut now = 1_000;
    accept_quest(&mut a, CHARACTER, &WELCOME, &mut now);
    accept_quest(&mut b, OTHER, &WELCOME, &mut now);
    let b_before = saved(&b);
    for index in 0..WELCOME.steps.len() {
        do_step(&mut a, CHARACTER, &WELCOME, index, &mut now);
    }
    turn_in(&mut a, CHARACTER, &WELCOME, &mut now);
    assert_eq!(saved(&b), b_before, "A's run changed B");
    assert_eq!(counts(&b, WELCOME.id), [0, 0, 0]);

    // B's own run still pays B, once.
    let b_coins = coins(&b);
    for index in 0..WELCOME.steps.len() {
        do_step(&mut b, OTHER, &WELCOME, index, &mut now);
    }
    turn_in(&mut b, OTHER, &WELCOME, &mut now);
    assert_eq!(coins(&b), b_coins + COINS);
    assert_eq!(quest_status(&a, WELCOME.id), QuestStatus::Completed);
}
