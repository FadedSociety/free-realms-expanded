//! T498: Who Done It? and Making Amends (Pixie Nursery, `3000332`/`3000333`), Lazy
//! Trolls and Question at the Quarry (Forest Troll Village, `3000340`/`3000341`),
//! community zone quests imported as data only into `data/content/quests/zone_wilds.json`
//! at existing Wilds `legacy_data` rows, plus the new troll Burglz row.
//!
//! Every test drives the production paths in `tests/support/npc_quest_chain.rs`:
//! 26/8 -> 26/10 -> 26/6 offers (49/1, decoded with the production reader), 49/2
//! accepts, Talk steps, the 26/6 that releases the held 49/13, and the 49/14 turn-in.
//!
//! Client readers (IDA, 2014 client; no new wire): 26/3 payload reader `0x00a9ef10`
//! stores the first dword after the 4-byte header as the dialogue body, which a
//! completed talk step fills with its `dialogue_string_id`; 49/13
//! `QuestEndPacket::ReadFields` `0x00c7b990` reads the actor at `6..14`, quest
//! `14..18`, end text `18..22`, one dword, then the reward bundle `0x008e7930`;
//! AddNpc reader `0x00921a10` (`startup_world::parse_add_npc`).

#[path = "support/npc_quest_chain.rs"]
mod npc_quest_chain;

use npc_quest_chain::{
    CHARACTER, Fixture, TALK_LABEL, accept, active_job, assert_no_offer,
    assert_packet_changes_nothing, character_at_for, coins, content_dir, content_position, counts,
    end_screen_reward, finish_quest, has_family, interact, menu_option, npc_select, offer,
    pending_turnin, profile_xp, quest_end_reply, quest_reply, quest_status, release_end_screen,
    saved, seed_active_profile, talk,
};
use osfr_tools::{
    character_store::CharacterState,
    gameplay::{Gameplay, Outcome},
    npc_interaction::{decode_dialogue_26_3, decode_quest_offer_49_1},
    quest_engine::QuestStatus,
    startup_world::parse_add_npc,
    world::NpcVisibility,
};

const BLAZING_STAR: u64 = 9_500_000_874;
const ZINNEA: u64 = 9_500_000_965;
const FUNNEL: u64 = 9_500_000_900;
const OXALIS: u64 = 9_500_000_926;
const BULB: u64 = 9_500_000_882;
const FLAX: u64 = 9_500_000_898;
const VERGOSH: u64 = 9_500_000_955;
const VOGGUN: u64 = 9_500_000_958;
const TROGGERS: u64 = 9_500_000_954;
const GROSH: u64 = 9_000_000_335;
const NOG: u64 = 9_500_000_925;
/// The new row this slice adds (the fork's definition 3120).
const BURGLZ: u64 = 9_500_001_007;
const OTHER: u64 = CHARACTER + 1;
const ADVENTURER: i32 = 1;
const MINER: i32 = 14;

/// One imported quest: giver, the ordered Talk steps before the turn-in (each with the
/// 26/3 body its NPC speaks when the step completes), and the final Talk at `turnin`.
struct Quest {
    id: u32,
    giver: u64,
    /// Title, intro and description ids the 49/1 card carries.
    card: [u32; 3],
    steps: &'static [(u64, u32)],
    turnin: u64,
    end_text: u32,
    coins: u32,
    xp: u32,
}

const WHO_DONE_IT: Quest = Quest {
    id: 3_000_332,
    giver: BLAZING_STAR,
    card: [389_951, 389_950, 389_952],
    steps: &[(ZINNEA, 389_967), (FUNNEL, 389_970), (OXALIS, 390_019)],
    turnin: OXALIS,
    end_text: 389_953,
    coins: 5,
    xp: 30,
};
const AMENDS: Quest = Quest {
    id: 3_000_333,
    giver: OXALIS,
    card: [3_787, 38_142, 38_144],
    steps: &[(BULB, 38_163)],
    turnin: FLAX,
    end_text: 2_134,
    coins: 2,
    xp: 20,
};
const LAZY_TROLLS: Quest = Quest {
    id: 3_000_340,
    giver: VERGOSH,
    card: [38_385, 38_384, 38_386],
    // Grosh reuses Voggun's body 38511 in the fork; kept.
    steps: &[
        (VOGGUN, 38_511),
        (TROGGERS, 38_402),
        (GROSH, 38_511),
        (NOG, 38_530),
    ],
    turnin: VERGOSH,
    end_text: 9_585,
    coins: 30,
    xp: 30,
};
const QUARRY: Quest = Quest {
    id: 3_000_341,
    giver: VERGOSH,
    card: [70_107, 70_102, 70_106],
    steps: &[],
    turnin: BURGLZ,
    end_text: 70_105,
    coins: 30,
    xp: 15,
};

fn at(guid: u64, npc: u64) -> CharacterState {
    character_at_for(guid, 0, content_position("wilds.json", npc))
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
    let (npc, body) = quest.steps[index];
    let talked = talk(game, &at(guid, npc), npc, *now);
    assert_eq!(
        dialogue_body(&talked),
        body,
        "quest {} step {}: the NPC's 26/3 line",
        quest.id,
        index + 1
    );
    assert!(
        talked.changed,
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
    assert_eq!(end_screen_reward(&end), (quest.coins, quest.xp));
    finish_quest(
        game,
        &turnin,
        quest.id,
        ADVENTURER,
        quest.coins,
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
fn pixie_nursery_chain_pays_once() {
    let fixture = Fixture::minimal(CHARACTER);
    let mut game = player(&fixture, CHARACTER, ADVENTURER);
    let (coins_before, xp_before) = (coins(&game), profile_xp(&game, ADVENTURER));
    let mut now = 1_000;
    run(&mut game, CHARACTER, &WHO_DONE_IT, &mut now);
    run(&mut game, CHARACTER, &AMENDS, &mut now);
    assert_eq!(coins(&game), coins_before + 7);
    assert_eq!(profile_xp(&game, ADVENTURER), xp_before + 50);
    for quest in [&WHO_DONE_IT, &AMENDS] {
        assert_eq!(quest_status(&game, quest.id), QuestStatus::Completed);
    }
    let flax = at(CHARACTER, FLAX);
    assert_packet_changes_nothing(
        &mut game,
        &flax,
        &quest_end_reply(AMENDS.id),
        now,
        "a duplicate 49/14",
    );
    assert_no_offer(&mut game, &flax, FLAX, &mut now, "Flax after Amends");
    let oxalis = at(CHARACTER, OXALIS);
    assert_no_offer(&mut game, &oxalis, OXALIS, &mut now, "Oxalis after Amends");
}

#[test]
fn lazy_trolls_then_the_quarry() {
    let fixture = Fixture::minimal(CHARACTER);
    let mut game = player(&fixture, CHARACTER, ADVENTURER);
    let (coins_before, xp_before) = (coins(&game), profile_xp(&game, ADVENTURER));
    let mut now = 1_000;
    run(&mut game, CHARACTER, &LAZY_TROLLS, &mut now);
    // Vergosh now offers the Quarry; Burglz, not Vergosh, takes the hand-in.
    run(&mut game, CHARACTER, &QUARRY, &mut now);
    assert_eq!(coins(&game), coins_before + 60);
    assert_eq!(profile_xp(&game, ADVENTURER), xp_before + 45);
    for quest in [&LAZY_TROLLS, &QUARRY] {
        assert_eq!(quest_status(&game, quest.id), QuestStatus::Completed);
    }
    let vergosh = at(CHARACTER, VERGOSH);
    assert_packet_changes_nothing(
        &mut game,
        &vergosh,
        &quest_end_reply(QUARRY.id),
        now,
        "a duplicate 49/14",
    );
    assert_no_offer(&mut game, &vergosh, VERGOSH, &mut now, "Vergosh after both");
    let burglz = at(CHARACTER, BURGLZ);
    assert_no_offer(&mut game, &burglz, BURGLZ, &mut now, "Burglz after both");
}

#[test]
fn burglz_is_drawn_as_the_troll() {
    let mut visibility = NpcVisibility::load(&content_dir()).expect("NPC visibility loads");
    let position = content_position("wilds.json", BURGLZ);
    assert_eq!(position, [-540.8602, 0.08854178, 1358.897]);
    assert!(
        visibility
            .teleport_targets(0)
            .iter()
            .any(|target| target.guid == BURGLZ && target.position == position),
        "the production spawn list holds Burglz at the fork spawn"
    );
    let added = visibility
        .update(0, position)
        .expect("a player at the spawn gets the draw set")
        .iter()
        .filter_map(|packet| parse_add_npc(packet).ok())
        .find(|npc| npc.actor_id == BURGLZ)
        .expect("AddNpc for Burglz");
    assert_eq!(added.position[..3], position);
    // troll_peasant_02.adr, and the client locale name 'Burglz'.
    assert_eq!((added.model_id, added.name_string_id), (167, 441_415));
    assert_eq!(visibility.content_key(BURGLZ), Some(BURGLZ));
}

#[test]
fn quarry_waits_for_lazy_trolls() {
    let fixture = Fixture::minimal(CHARACTER);
    let mut game = player(&fixture, CHARACTER, ADVENTURER);
    let vergosh = at(CHARACTER, VERGOSH);
    let burglz = at(CHARACTER, BURGLZ);
    let forged = quest_reply(QUARRY.id, true);
    let mut now = 1_000;
    // Burglz offers nothing, and a forged 49/2 for the Quarry is refused, before and
    // while Lazy Trolls is open.
    assert_no_offer(&mut game, &burglz, BURGLZ, &mut now, "Burglz first");
    assert_packet_changes_nothing(&mut game, &vergosh, &forged, now, "a forged 49/2");
    accept_quest(&mut game, CHARACTER, &LAZY_TROLLS, &mut now);
    assert_packet_changes_nothing(&mut game, &vergosh, &forged, now, "a 49/2 mid-Lazy Trolls");
    // Open steps: Vergosh offers nothing and the Quarry's end target is not Vergosh.
    assert_no_offer(&mut game, &vergosh, VERGOSH, &mut now, "Vergosh mid-quest");
    for index in 0..LAZY_TROLLS.steps.len() {
        do_step(&mut game, CHARACTER, &LAZY_TROLLS, index, &mut now);
    }
    turn_in(&mut game, CHARACTER, &LAZY_TROLLS, &mut now);
    assert_eq!(quest_status(&game, LAZY_TROLLS.id), QuestStatus::Completed);
    // Only now does Vergosh offer the Quarry.
    offer(&mut game, &vergosh, VERGOSH, QUARRY.id, now);
}

#[test]
fn repeated_oxalis_steps() {
    // Goals 3 and 4 of Who Done It? are both Oxalis (the fork's goal 3 has a reply
    // 390019, goal 4 is the hand-in with end text 389953). (R) One click on Oxalis
    // credits exactly one goal: the first click completes goal 3 and speaks 390019,
    // opens no end screen and pays nothing; the second click completes goal 4 and
    // holds the 49/13 for the 26/6.
    let fixture = Fixture::minimal(CHARACTER);
    let mut game = player(&fixture, CHARACTER, ADVENTURER);
    let oxalis = at(CHARACTER, OXALIS);
    let mut now = 1_000;
    accept_quest(&mut game, CHARACTER, &WHO_DONE_IT, &mut now);
    for index in 0..2 {
        do_step(&mut game, CHARACTER, &WHO_DONE_IT, index, &mut now);
    }
    let coins_before = coins(&game);

    let first = talk(&mut game, &oxalis, OXALIS, now);
    assert_eq!(dialogue_body(&first), 390_019);
    assert_eq!(counts(&game, WHO_DONE_IT.id), [1, 1, 1, 0]);
    assert_eq!(pending_turnin(&game), None);
    assert!(
        !first
            .replies
            .iter()
            .any(|packet| has_family(packet, 49, 13, None)),
        "the first click opened the end screen"
    );
    assert_eq!(coins(&game), coins_before);
    assert_eq!(quest_status(&game, WHO_DONE_IT.id), QuestStatus::Active);

    let second = talk(&mut game, &oxalis, OXALIS, now + 10);
    assert!(second.changed);
    assert_eq!(counts(&game, WHO_DONE_IT.id), [1, 1, 1, 1]);
    assert_eq!(pending_turnin(&game), Some(WHO_DONE_IT.id));
    assert_eq!(coins(&game), coins_before);
}

#[test]
fn early_and_wrong_turn_ins_are_refused() {
    let fixture = Fixture::minimal(CHARACTER);
    let mut game = player(&fixture, CHARACTER, ADVENTURER);
    let mut now = 1_000;
    accept_quest(&mut game, CHARACTER, &WHO_DONE_IT, &mut now);
    let oxalis = at(CHARACTER, OXALIS);

    // Out of order: the goals are ordered, so Funnel before Zinnea is the wrong NPC.
    let funnel = at(CHARACTER, FUNNEL);
    assert_no_offer(&mut game, &funnel, FUNNEL, &mut now, "Funnel first");
    assert_eq!(counts(&game, WHO_DONE_IT.id), [0, 0, 0, 0]);
    do_step(&mut game, CHARACTER, &WHO_DONE_IT, 0, &mut now);

    // Oxalis is the turn-in but not the next goal: a Talk or a forged 49/14 neither
    // credits nor pays.
    let before = saved(&game);
    let menu = interact(&mut game, &oxalis, OXALIS, now);
    let choice = menu_option(&menu, TALK_LABEL);
    if let Ok(chosen) = game.handle_in_world(&npc_select(OXALIS, choice), &oxalis, now + 1) {
        assert!(!chosen.changed, "Oxalis credited a goal out of order");
    }
    assert_eq!(saved(&game), before, "Oxalis out of order changed the save");
    let forged = quest_end_reply(WHO_DONE_IT.id);
    assert_packet_changes_nothing(&mut game, &oxalis, &forged, now + 10, "an early 49/14");
    // A 49/14 for a quest never accepted is refused too.
    let flax = at(CHARACTER, FLAX);
    let never = quest_end_reply(AMENDS.id);
    assert_packet_changes_nothing(&mut game, &flax, &never, now + 20, "a 49/14 never accepted");
}

#[test]
fn progress_survives_reconnect() {
    let fixture = Fixture::minimal(CHARACTER);
    let mut game = player(&fixture, CHARACTER, ADVENTURER);
    let mut now = 1_000;
    accept_quest(&mut game, CHARACTER, &LAZY_TROLLS, &mut now);
    for index in 0..2 {
        do_step(&mut game, CHARACTER, &LAZY_TROLLS, index, &mut now);
    }

    let mut game = fixture.reload_serialized(&game);
    assert_eq!(quest_status(&game, LAZY_TROLLS.id), QuestStatus::Active);
    assert_eq!(counts(&game, LAZY_TROLLS.id), [1, 1, 0, 0, 0]);
    for index in 2..LAZY_TROLLS.steps.len() {
        do_step(&mut game, CHARACTER, &LAZY_TROLLS, index, &mut now);
    }
    turn_in(&mut game, CHARACTER, &LAZY_TROLLS, &mut now);

    // A finished quest stays finished across a reconnect and is not re-offered.
    let mut game = fixture.reload_serialized(&game);
    assert_eq!(quest_status(&game, LAZY_TROLLS.id), QuestStatus::Completed);
    accept_quest(&mut game, CHARACTER, &QUARRY, &mut now);
}

#[test]
fn xp_goes_to_adventurer_with_another_job_active() {
    let fixture = Fixture::minimal(CHARACTER);
    let mut game = player(&fixture, CHARACTER, MINER);
    let miner_before = profile_xp(&game, MINER);
    let mut now = 1_000;
    run(&mut game, CHARACTER, &LAZY_TROLLS, &mut now);
    assert_eq!(active_job(&game), MINER);
    assert_eq!(profile_xp(&game, MINER), miner_before);
}

#[test]
fn two_players_are_independent() {
    let fixture = Fixture::minimal(CHARACTER);
    let mut a = player(&fixture, CHARACTER, ADVENTURER);
    let mut b = player(&fixture, OTHER, ADVENTURER);
    let mut now = 1_000;
    accept_quest(&mut a, CHARACTER, &LAZY_TROLLS, &mut now);
    accept_quest(&mut b, OTHER, &LAZY_TROLLS, &mut now);
    let b_before = saved(&b);
    for index in 0..LAZY_TROLLS.steps.len() {
        do_step(&mut a, CHARACTER, &LAZY_TROLLS, index, &mut now);
    }
    turn_in(&mut a, CHARACTER, &LAZY_TROLLS, &mut now);
    assert_eq!(saved(&b), b_before, "A's run changed B");
    assert_eq!(counts(&b, LAZY_TROLLS.id), [0, 0, 0, 0, 0]);

    // B's own run still pays B, once.
    let b_coins = coins(&b);
    for index in 0..LAZY_TROLLS.steps.len() {
        do_step(&mut b, OTHER, &LAZY_TROLLS, index, &mut now);
    }
    turn_in(&mut b, OTHER, &LAZY_TROLLS, &mut now);
    assert_eq!(coins(&b), b_coins + LAZY_TROLLS.coins);
    assert_eq!(quest_status(&a, LAZY_TROLLS.id), QuestStatus::Completed);
}
