//! T500: Nomi's Little Brother (`3000301`), Looking for Lavender (`3000310`) and A
//! Reliable Source (`3000311`), community zone quests imported as data only into
//! `data/content/quests/zone_sanctuary.json`, plus the three NPCs that were not talkable:
//! Hasti (a new row bound to the extra spawn `100000003016` through its `content_key`),
//! Dani and Honey (now drawn legacy spawns with client name ids).
//!
//! Every test drives the production paths in `tests/support/npc_quest_chain.rs`:
//! 26/8 -> 26/10 -> 26/6 offers (49/1, decoded with the production reader), 49/2
//! accepts, Talk steps, the 26/6 that releases the held 49/13, and the 49/14 turn-in.
//!
//! Client readers (IDA, 2014 client; no new wire): 26/3 payload reader `0x00a9ef10`
//! stores the first dword after the 4-byte header as the dialogue body, which a
//! completed talk step fills with its `dialogue_string_id`; the speaker sits at bytes
//! `12..20` (`patch_npc_actor`); 49/13 `QuestEndPacket::ReadFields` `0x00c7b990` reads the
//! actor at `6..14`, quest `14..18`, end text `18..22`, one dword, then the reward bundle
//! `0x008e7930`; AddNpc reader `0x00921a10` (`startup_world::parse_add_npc`).

#[path = "support/npc_quest_chain.rs"]
mod npc_quest_chain;

use npc_quest_chain::{
    CHARACTER, Fixture, accept, active_job, assert_no_offer, assert_packet_changes_nothing,
    character_at_for, coins, content_dir, content_position, counts, end_screen_reward,
    finish_quest, has_family, offer, pending_turnin, profile_xp, quest_end_reply, quest_reply,
    quest_status, release_end_screen, saved, seed_active_profile, talk,
};
use osfr_tools::{
    character_store::CharacterState,
    gameplay::{Gameplay, Outcome},
    npc_interaction::{decode_dialogue_26_3, decode_quest_offer_49_1},
    quest_engine::QuestStatus,
    startup_world::parse_add_npc,
    world::NpcVisibility,
};

const NOMI: u64 = 9_500_000_509;
const FELDMAN: u64 = 9_500_000_476;
const BARIN: u64 = 9_500_000_443;
const DANI: u64 = 9_500_000_467;
const HONEY: u64 = 9_000_000_107;
const SUNI: u64 = 9_500_000_715;
/// The new row this slice adds (the fork's definition 3016).
const HASTI: u64 = 9_500_001_008;
/// Hasti's runtime actor: the Legacy extra spawn whose `content_key` is `HASTI`.
const HASTI_SPAWN: u64 = 100_000_003_016;
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

const LITTLE_BROTHER: Quest = Quest {
    id: 3_000_301,
    giver: NOMI,
    card: [46_280, 46_279, 46_621],
    steps: &[(HASTI, 140_679)],
    turnin: NOMI,
    end_text: 73_239,
    coins: 20,
    xp: 20,
};
const LAVENDER: Quest = Quest {
    id: 3_000_310,
    giver: FELDMAN,
    card: [390_787, 390_786, 390_788],
    steps: &[(DANI, 390_817), (HONEY, 390_820), (BARIN, 390_822)],
    turnin: FELDMAN,
    end_text: 390_789,
    coins: 36,
    xp: 30,
};
const SOURCE: Quest = Quest {
    id: 3_000_311,
    giver: FELDMAN,
    card: [390_845, 390_844, 390_846],
    steps: &[(SUNI, 390_855)],
    turnin: FELDMAN,
    end_text: 390_847,
    coins: 24,
    xp: 20,
};

/// The actor the client clicks: Hasti is an extra spawn, every other row spawns as itself.
fn runtime(npc: u64) -> u64 {
    if npc == HASTI { HASTI_SPAWN } else { npc }
}

fn at(guid: u64, npc: u64) -> CharacterState {
    let file = if npc == SUNI {
        "shrouded_glade.json"
    } else {
        "sanctuary.json"
    };
    character_at_for(guid, 0, content_position(file, npc))
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
    let talked = talk(game, &at(guid, npc), runtime(npc), *now);
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
fn nomis_little_brother_through_the_extra_spawn() {
    let visibility = NpcVisibility::load(&content_dir()).expect("NPC visibility loads");
    assert_eq!(
        visibility.content_key(HASTI_SPAWN),
        Some(HASTI),
        "the extra spawn keys to the new Hasti row"
    );
    // The regional row is scaffolding: only the extra spawn draws Hasti.
    assert_eq!(visibility.content_key(HASTI), None);

    let fixture = Fixture::minimal(CHARACTER);
    let mut game = player(&fixture, CHARACTER, ADVENTURER);
    let (coins_before, xp_before) = (coins(&game), profile_xp(&game, ADVENTURER));
    let mut now = 1_000;
    accept_quest(&mut game, CHARACTER, &LITTLE_BROTHER, &mut now);
    // The Talk is sent to the runtime guid; the 26/3 names that actor as speaker.
    let talked = talk(&mut game, &at(CHARACTER, HASTI), HASTI_SPAWN, now);
    assert!(talked.changed, "Hasti's Talk did not credit the step");
    assert_eq!(dialogue_body(&talked), 140_679);
    let dialogue = talked
        .replies
        .iter()
        .find(|packet| packet.starts_with(&[26, 0, 3, 0]))
        .unwrap();
    assert_eq!(dialogue[12..20], HASTI_SPAWN.to_le_bytes());
    assert_eq!(counts(&game, LITTLE_BROTHER.id), [1, 0]);
    now += 10;
    turn_in(&mut game, CHARACTER, &LITTLE_BROTHER, &mut now);
    assert_eq!(coins(&game), coins_before + 20);
    assert_eq!(profile_xp(&game, ADVENTURER), xp_before + 20);
    assert_eq!(
        quest_status(&game, LITTLE_BROTHER.id),
        QuestStatus::Completed
    );
    assert_no_offer(
        &mut game,
        &at(CHARACTER, NOMI),
        NOMI,
        &mut now,
        "Nomi after",
    );
}

#[test]
fn lavender_then_reliable_source() {
    let fixture = Fixture::minimal(CHARACTER);
    let mut game = player(&fixture, CHARACTER, ADVENTURER);
    let (coins_before, xp_before) = (coins(&game), profile_xp(&game, ADVENTURER));
    let mut now = 1_000;
    run(&mut game, CHARACTER, &LAVENDER, &mut now);
    // Feldman now offers the follow-up; Suni is the step, Feldman the hand-in.
    run(&mut game, CHARACTER, &SOURCE, &mut now);
    assert_eq!(coins(&game), coins_before + 60);
    assert_eq!(profile_xp(&game, ADVENTURER), xp_before + 50);
    for quest in [&LAVENDER, &SOURCE] {
        assert_eq!(quest_status(&game, quest.id), QuestStatus::Completed);
    }
    let feldman = at(CHARACTER, FELDMAN);
    assert_packet_changes_nothing(
        &mut game,
        &feldman,
        &quest_end_reply(SOURCE.id),
        now,
        "a duplicate 49/14",
    );
    assert_no_offer(&mut game, &feldman, FELDMAN, &mut now, "Feldman after both");
}

#[test]
fn reliable_source_waits_for_lavender() {
    let fixture = Fixture::minimal(CHARACTER);
    let mut game = player(&fixture, CHARACTER, ADVENTURER);
    let feldman = at(CHARACTER, FELDMAN);
    let suni = at(CHARACTER, SUNI);
    let forged = quest_reply(SOURCE.id, true);
    let mut now = 1_000;
    // Suni is no giver, and a forged 49/2 for the follow-up is refused, before and
    // while Lavender is open.
    assert_no_offer(&mut game, &suni, SUNI, &mut now, "Suni first");
    assert_packet_changes_nothing(&mut game, &feldman, &forged, now, "a forged 49/2");
    accept_quest(&mut game, CHARACTER, &LAVENDER, &mut now);
    assert_packet_changes_nothing(&mut game, &feldman, &forged, now, "a 49/2 mid-Lavender");
    // Open steps: Feldman offers nothing, and Suni's Talk credits nothing.
    assert_no_offer(&mut game, &feldman, FELDMAN, &mut now, "Feldman mid-quest");
    assert_no_offer(&mut game, &suni, SUNI, &mut now, "Suni mid-Lavender");
    for index in 0..LAVENDER.steps.len() {
        do_step(&mut game, CHARACTER, &LAVENDER, index, &mut now);
    }
    turn_in(&mut game, CHARACTER, &LAVENDER, &mut now);
    // Only now does Feldman offer A Reliable Source.
    offer(&mut game, &feldman, FELDMAN, SOURCE.id, now);
}

#[test]
fn early_and_wrong_turn_ins_are_refused() {
    let fixture = Fixture::minimal(CHARACTER);
    let mut game = player(&fixture, CHARACTER, ADVENTURER);
    let mut now = 1_000;
    accept_quest(&mut game, CHARACTER, &LAVENDER, &mut now);

    // The goals are ordered: Honey before Dani is the wrong NPC and credits nothing.
    let honey = at(CHARACTER, HONEY);
    assert_no_offer(&mut game, &honey, HONEY, &mut now, "Honey first");
    assert_eq!(counts(&game, LAVENDER.id), [0, 0, 0, 0]);
    do_step(&mut game, CHARACTER, &LAVENDER, 0, &mut now);

    // Feldman is the turn-in but not the next goal: a forged 49/14 neither credits nor pays.
    let feldman = at(CHARACTER, FELDMAN);
    let forged = quest_end_reply(LAVENDER.id);
    assert_packet_changes_nothing(&mut game, &feldman, &forged, now, "an early 49/14");
    // A 49/14 for a quest never accepted is refused too.
    let never = quest_end_reply(LITTLE_BROTHER.id);
    let nomi = at(CHARACTER, NOMI);
    assert_packet_changes_nothing(&mut game, &nomi, &never, now + 10, "a 49/14 never accepted");
}

#[test]
fn hasti_dani_and_honey_are_drawn_with_client_names() {
    let mut visibility = NpcVisibility::load(&content_dir()).expect("NPC visibility loads");
    // (actor, position, model, name id): the extra spawn keeps its transform; Dani and
    // Honey take the Sanctuary-main spawns, with the locale ids the client resolves.
    for (actor, position, model, name, key) in [
        (
            HASTI_SPAWN,
            [-2252.73, -15.94524, 564.0553],
            242,
            46_273,
            HASTI,
        ),
        (DANI, [-1284.0914, -14.196148, 263.81052], 384, 14_701, DANI),
        (HONEY, [-1503.994, -34.46316, 377.915], 377, 7_666, HONEY),
    ] {
        assert!(
            visibility
                .teleport_targets(0)
                .iter()
                .any(|target| target.guid == actor && target.position == position),
            "the production spawn list holds {actor} at its spawn"
        );
        let added = visibility
            .update(0, position)
            .expect("a player at the spawn gets the draw set")
            .iter()
            .filter_map(|packet| parse_add_npc(packet).ok())
            .find(|npc| npc.actor_id == actor)
            .unwrap_or_else(|| panic!("AddNpc for {actor}"));
        assert_eq!(added.position[..3], position);
        assert_eq!((added.model_id, added.name_string_id), (model, name));
        assert_eq!(visibility.content_key(actor), Some(key));
    }
}

#[test]
fn progress_survives_reconnect() {
    let fixture = Fixture::minimal(CHARACTER);
    let mut game = player(&fixture, CHARACTER, ADVENTURER);
    let mut now = 1_000;
    accept_quest(&mut game, CHARACTER, &LAVENDER, &mut now);
    for index in 0..2 {
        do_step(&mut game, CHARACTER, &LAVENDER, index, &mut now);
    }

    let mut game = fixture.reload_serialized(&game);
    assert_eq!(quest_status(&game, LAVENDER.id), QuestStatus::Active);
    assert_eq!(counts(&game, LAVENDER.id), [1, 1, 0, 0]);
    do_step(&mut game, CHARACTER, &LAVENDER, 2, &mut now);
    turn_in(&mut game, CHARACTER, &LAVENDER, &mut now);

    // A finished quest stays finished across a reconnect and the follow-up is offered.
    let mut game = fixture.reload_serialized(&game);
    assert_eq!(quest_status(&game, LAVENDER.id), QuestStatus::Completed);
    accept_quest(&mut game, CHARACTER, &SOURCE, &mut now);
}

#[test]
fn xp_goes_to_adventurer_with_another_job_active() {
    let fixture = Fixture::minimal(CHARACTER);
    let mut game = player(&fixture, CHARACTER, MINER);
    let miner_before = profile_xp(&game, MINER);
    let mut now = 1_000;
    run(&mut game, CHARACTER, &LAVENDER, &mut now);
    assert_eq!(active_job(&game), MINER);
    assert_eq!(profile_xp(&game, MINER), miner_before);
}

#[test]
fn two_players_are_independent() {
    let fixture = Fixture::minimal(CHARACTER);
    let mut a = player(&fixture, CHARACTER, ADVENTURER);
    let mut b = player(&fixture, OTHER, ADVENTURER);
    let mut now = 1_000;
    accept_quest(&mut a, CHARACTER, &LAVENDER, &mut now);
    accept_quest(&mut b, OTHER, &LAVENDER, &mut now);
    let b_before = saved(&b);
    for index in 0..LAVENDER.steps.len() {
        do_step(&mut a, CHARACTER, &LAVENDER, index, &mut now);
    }
    turn_in(&mut a, CHARACTER, &LAVENDER, &mut now);
    assert_eq!(saved(&b), b_before, "A's run changed B");
    assert_eq!(counts(&b, LAVENDER.id), [0, 0, 0, 0]);

    // B's own run still pays B, once.
    let b_coins = coins(&b);
    for index in 0..LAVENDER.steps.len() {
        do_step(&mut b, OTHER, &LAVENDER, index, &mut now);
    }
    turn_in(&mut b, OTHER, &LAVENDER, &mut now);
    assert_eq!(coins(&b), b_coins + LAVENDER.coins);
    assert_eq!(quest_status(&a, LAVENDER.id), QuestStatus::Completed);
}
