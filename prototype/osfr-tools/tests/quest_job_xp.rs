//! T202/F249: the playable job quest lines pay their quest's job XP.
//!
//! The change is content-only for the reward rows and job gates; these tests drive the
//! production in-world router (`Gameplay::handle_in_world` -> `gameplay/npc.rs` ->
//! `QuestEngine`) with the same 26/8, 26/10, 26/6, 49/2 and 49/14 packets the gateway
//! accepts. They do not call `QuestOutcome` or `finish_turnin` directly.
//!
//! Client request/reply layouts used here (verified in the client constructors and
//! field readers):
//! - 26/8 NPC interact: constructor `008b6bc0`; u64 target + u8 [0].
//! - 26/10 menu select: constructor `00a36f80`; u64 target + i32 choice.
//! - 26/6 dialogue response: constructor `009f5890`, sender `009f6740`; i32 row
//!   (301 is the quest row).
//! - 49/2 quest reply: constructor `00a86400`, sender `00a97d70`; u32 quest + u8 accept.
//! - 49/14 end reply: constructor `00a864c0` / sender `00a97ef0`; u32 quest.
//! 49/1 offers are read for their quest id at 6..10 (`QuestInfoPacket` reader
//! `00c7bb60`); 49/13 end screens name the speaker at 6..14 and the quest at
//! 14..18 (`QuestEndPacket::ReadFields` `00c7b990`, resolved by `00984960` in
//! `ShowQuestEndScreen` `00a95420`). The 49/13 reward bundle is the body consumed
//! by `008e7930`; its coin dword is at packet offset 27..31 and its displayed XP
//! total at 31..35. Job XP is delivered by the same 38/14 progress packet
//! (`008d86e0`, six i32: profile, rank, progress, 0, 0, 0) and 38/18 rank packet
//! (`008d8910`, profile and rank) that `rewards::experience_packets` already
//! writes.

#[path = "support/npc_quest_chain.rs"]
mod npc_quest_chain;

use npc_quest_chain::{
    Fixture, accept, active_job, assert_no_offer, assert_packet_changes_nothing,
    character_at_for as character_in_zone, coins, content_dir, copy_tree, counts, finish_quest,
    interact, menu_option, npc_interact, npc_select, offer, pending_turnin, profile_xp,
    quest_abandon, quest_end_reply, quest_reply, quest_status, saved, seed_active_profile,
    seed_quest, select_job, send, talk, talk_step,
};
use osfr_tools::{
    character_store::CharacterState,
    gameplay::Gameplay,
    npc_interaction::NpcInteraction,
    profiles::ProfileState,
    quest_engine::{CharacterEligibility, QuestEngine, QuestStatus},
};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

const CHARACTER: u64 = 202_017;
const ADVENTURER: i32 = 1;
const MINER: i32 = 14;
const NINJA: i32 = 2;
const BRAWLER: i32 = 43;
const CHEF: i32 = 45;
const KART_DRIVER: i32 = 48;

const MERIGOLD: u64 = 9_000_000_402;
const RAINA: u64 = 9_000_000_177;
const CHAD: u64 = 9_500_000_735;
const MUSTANG: u64 = 9_500_000_759;
const BURT: u64 = 9_500_000_731;
const AUGUSTE: u64 = 9_000_000_019;
const RICARDO: u64 = 9_000_000_178;
const TY: u64 = 9_000_000_223;
const GROSH: u64 = 9_000_000_335;
const CLEETUS: u64 = 9_000_000_050;
/// T279: the Royal Palace giver of "Chapter 1: The Missing Sisters".
const GEOFFREY: u64 = 9_000_000_330;
/// T277: the Greenwood Forest Royal Courier of the independent "Local Hero" entry.
const ROYAL_COURIER: u64 = 9_000_000_444;

const INTRODUCE_YOURSELF: u32 = 3_000_020;
const BURDEN_OF_FAME: u32 = 3_080_001;
const RANGER_REQUISITION: u32 = 3_050_011;
const NINJA_PATH: u32 = 3_110_001;
/// T542's Jobs-window record: Ty is its giver, its one talk target and the turn-in.
const BECOME_A_NINJA: u32 = 3_110_026;

/// The 49/13 end text (`end_text_string_id`) of each quest, from `data/content/quests`.
const INTRODUCE_END_TEXT: u32 = 0;
const BURDEN_END_TEXT: u32 = 104_108;
const RANGER_END_TEXT: u32 = 104_136;
const NINJA_END_TEXT: u32 = 9_731;

const SIMONE: u64 = 9_000_000_467;

/// `job_trainers.json` row for Simone's Chef lesson (profile 45).
const LEARN_CHEF_LABEL: u32 = 413_021;

static NEXT_SCRATCH: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

fn spawn_location(npc_id: u64) -> (u32, [f32; 3]) {
    let content = content_dir();
    NpcInteraction::load(&content.join("npcs"), &content.join("stores.json"))
        .expect("NPC content loads")
        .location(npc_id)
        .unwrap_or_else(|| panic!("NPC {npc_id} is a positioned content spawn"))
}

fn character_at(npc_id: u64) -> CharacterState {
    character_at_for(CHARACTER, npc_id)
}

fn character_at_for(guid: u64, npc_id: u64) -> CharacterState {
    let (zone_id, position) = spawn_location(npc_id);
    character_in_zone(guid, zone_id, position)
}

/// 26/6 releases the 49/13 the final Talk held. These rows carry no display rows, so the
/// client-reader bundle (`008e7930`) is exactly 99 bytes and the stars dword at `31..35`
/// must be the quest's configured job XP; the shared helper checks the actor and end text.
fn release_end_screen(
    game: &mut Gameplay,
    character: &CharacterState,
    quest_id: u32,
    expected_actor: u64,
    expected_end_text: u32,
    expected_xp: u32,
    now: u64,
) -> Vec<u8> {
    let end = npc_quest_chain::release_end_screen(
        game,
        character,
        quest_id,
        expected_actor,
        expected_end_text,
        now,
    );
    assert_eq!(
        end.len(),
        99,
        "quest {quest_id}: the populated reward bundle keeps the client-reader length"
    );
    assert_eq!(
        i32::from_le_bytes(end[31..35].try_into().unwrap()),
        i32::try_from(expected_xp).unwrap(),
        "quest {quest_id}: the end screen displays its configured job XP"
    );
    end
}

/// Seed a profile snapshot for the cases direct play cannot reach: the captured opening
/// inventory has no Kart Driver or Ninja starter outfit. The positive Chef gate is
/// exercised through the real trainer and job switch instead.
fn seeded(fixture: &Fixture, active: i32, unlocked: &[i32]) -> Gameplay {
    let mut state = fixture.game(None).state();
    seed_active_profile(&mut state, active, unlocked);
    fixture.game(Some(state))
}

fn switch_job(game: &mut Gameplay, character: &CharacterState, profile: i32, now: u64) {
    let switched = send(game, character, &select_job(profile), now);
    assert!(
        switched.changed,
        "selecting profile {profile} must commit: {:?}",
        switched
            .replies
            .iter()
            .map(|packet| String::from_utf8_lossy(packet))
            .collect::<Vec<_>>()
    );
    assert_eq!(active_job(game), profile);
}

fn introduce_yourself(game: &mut Gameplay, character: &CharacterState, now: &mut u64) {
    let merigold = CharacterState {
        guid: character.guid,
        ..character_at(MERIGOLD)
    };
    *now += 10;
    offer(game, &merigold, MERIGOLD, INTRODUCE_YOURSELF, *now);
    *now += 10;
    accept(game, &merigold, INTRODUCE_YOURSELF, *now);
    *now += 10;
    talk_step(game, character, RAINA, *now, "Introduce Yourself");
    assert_eq!(counts(game, INTRODUCE_YOURSELF), [1]);
    assert_eq!(pending_turnin(game), Some(INTRODUCE_YOURSELF));
    *now += 10;
    release_end_screen(
        game,
        character,
        INTRODUCE_YOURSELF,
        RAINA,
        INTRODUCE_END_TEXT,
        15,
        *now,
    );
    *now += 10;
    finish_quest(game, character, INTRODUCE_YOURSELF, ADVENTURER, 5, 15, *now);
}

fn burden_of_fame(game: &mut Gameplay, now: &mut u64) {
    let raina = character_at(RAINA);
    let chad = character_at(CHAD);
    let mustang = character_at(MUSTANG);
    let burt = character_at(BURT);
    *now += 10;
    offer(game, &raina, RAINA, BURDEN_OF_FAME, *now);
    *now += 10;
    accept(game, &raina, BURDEN_OF_FAME, *now);
    for (target, character) in [
        (CHAD, &chad),
        (MUSTANG, &mustang),
        (BURT, &burt),
        (RAINA, &raina),
    ] {
        *now += 10;
        talk_step(game, character, target, *now, "The Burden of Fame");
    }
    assert_eq!(counts(game, BURDEN_OF_FAME), [1, 1, 1, 1]);
    assert_eq!(pending_turnin(game), Some(BURDEN_OF_FAME));
    *now += 10;
    release_end_screen(
        game,
        &raina,
        BURDEN_OF_FAME,
        RAINA,
        BURDEN_END_TEXT,
        25,
        *now,
    );
    *now += 10;
    finish_quest(game, &raina, BURDEN_OF_FAME, KART_DRIVER, 5, 25, *now);
}

fn ranger_requisition(game: &mut Gameplay, now: &mut u64) {
    let auguste = character_at(AUGUSTE);
    let ricardo = character_at(RICARDO);
    *now += 10;
    offer(game, &auguste, AUGUSTE, RANGER_REQUISITION, *now);
    *now += 10;
    accept(game, &auguste, RANGER_REQUISITION, *now);
    *now += 10;
    talk_step(game, &ricardo, RICARDO, *now, "Chef: Ranger Requisition");
    assert_eq!(counts(game, RANGER_REQUISITION), [1]);
    assert_eq!(pending_turnin(game), Some(RANGER_REQUISITION));
    *now += 10;
    release_end_screen(
        game,
        &ricardo,
        RANGER_REQUISITION,
        RICARDO,
        RANGER_END_TEXT,
        20,
        *now,
    );
    *now += 10;
    finish_quest(game, &ricardo, RANGER_REQUISITION, CHEF, 0, 20, *now);
}

fn ninja_path(game: &mut Gameplay, now: &mut u64) {
    let ty = character_at(TY);
    let grosh = character_at(GROSH);
    *now += 10;
    offer(game, &ty, TY, NINJA_PATH, *now);
    *now += 10;
    accept(game, &ty, NINJA_PATH, *now);
    *now += 10;
    talk_step(game, &grosh, GROSH, *now, "Ninja: The Ninja Path");
    assert_eq!(counts(game, NINJA_PATH), [1]);
    assert_eq!(pending_turnin(game), Some(NINJA_PATH));
    *now += 10;
    release_end_screen(game, &grosh, NINJA_PATH, GROSH, NINJA_END_TEXT, 15, *now);
    *now += 10;
    finish_quest(game, &grosh, NINJA_PATH, NINJA, 0, 15, *now);
}

#[test]
fn adventurer_line_pays_its_job_xp_once() {
    let fixture = Fixture::for_character(CHARACTER);
    let mut game = fixture.game(None);
    let mut now = 1_000;
    let raina = character_at(RAINA);

    introduce_yourself(&mut game, &raina, &mut now);
    assert_eq!(profile_xp(&game, ADVENTURER), 15);
    assert_eq!(coins(&game), 5, "the existing coin row is unchanged");

    let finished = saved(&game);
    now += 10;
    assert_packet_changes_nothing(
        &mut game,
        &raina,
        &quest_end_reply(INTRODUCE_YOURSELF),
        now,
        "duplicate turn-in",
    );
    assert_eq!(profile_xp(&game, ADVENTURER), 15);
    assert_eq!(
        saved(&game),
        finished,
        "a duplicate turn-in changed the save"
    );
}

#[test]
fn chef_trainer_unlock_opens_the_job_gate_and_pays() {
    let fixture = Fixture::for_character(CHARACTER);
    let mut game = fixture.game(None);
    let mut now = 2_000;
    let simone = character_at(SIMONE);
    let auguste = character_at(AUGUSTE);

    // Chef is not unlocked at character creation: the line is not offered.
    assert_no_offer(&mut game, &auguste, AUGUSTE, &mut now, "Chef before unlock");

    // The real trainer path unlocks Chef; the job list switch makes it active.
    now += 10;
    let menu = interact(&mut game, &simone, SIMONE, now);
    let learn = menu_option(&menu, LEARN_CHEF_LABEL);
    now += 10;
    let learned = send(&mut game, &simone, &npc_select(SIMONE, learn), now);
    assert!(learned.changed, "Simone's Chef lesson must commit");
    assert!(
        game.state()
            .profiles
            .as_ref()
            .unwrap()
            .unlocked
            .contains(&CHEF),
        "the trainer must unlock Chef"
    );
    // Unlocked is not enough: the gate reads the active job.
    assert_no_offer(
        &mut game,
        &auguste,
        AUGUSTE,
        &mut now,
        "Chef unlocked but inactive",
    );
    switch_job(&mut game, &simone, CHEF, now + 1);

    ranger_requisition(&mut game, &mut now);
    assert_eq!(profile_xp(&game, CHEF), 20);
    assert_eq!(active_job(&game), CHEF, "the turn-in must not switch jobs");
}

#[test]
fn kart_line_pays_its_job_xp_once() {
    let fixture = Fixture::for_character(CHARACTER);
    let mut game = seeded(&fixture, KART_DRIVER, &[1, KART_DRIVER]);
    let mut now = 3_000;

    let raina = character_at(RAINA);
    introduce_yourself(&mut game, &raina, &mut now);
    burden_of_fame(&mut game, &mut now);
    assert_eq!(profile_xp(&game, KART_DRIVER), 25);
    assert_eq!(profile_xp(&game, ADVENTURER), 15);
    assert_eq!(coins(&game), 10);

    let finished = saved(&game);
    now += 10;
    assert_packet_changes_nothing(
        &mut game,
        &raina,
        &quest_end_reply(BURDEN_OF_FAME),
        now,
        "duplicate Kart turn-in",
    );
    assert_eq!(profile_xp(&game, KART_DRIVER), 25);
    assert_eq!(saved(&game), finished);
}

#[test]
fn ninja_line_pays_its_job_xp_once() {
    let fixture = Fixture::for_character(CHARACTER);
    let mut game = seeded(&fixture, NINJA, &[1, NINJA]);
    let mut now = 4_000;

    ninja_path(&mut game, &mut now);
    assert_eq!(profile_xp(&game, NINJA), 15);
    assert_eq!(coins(&game), 0, "the zero-coin Ninja row pays XP only");
}

#[test]
fn each_enabled_quest_pays_its_configured_job_xp_once() {
    adventurer_line_pays_its_job_xp_once();
    chef_trainer_unlock_opens_the_job_gate_and_pays();
    kart_line_pays_its_job_xp_once();
    ninja_line_pays_its_job_xp_once();
}

#[test]
fn xp_follows_the_quest_job_not_the_active_job() {
    let fixture = Fixture::for_character(CHARACTER);
    let mut game = seeded(&fixture, NINJA, &[1, NINJA, BRAWLER]);
    let mut now = 5_000;
    let ty = character_at(TY);
    let grosh = character_at(GROSH);

    offer(&mut game, &ty, TY, NINJA_PATH, now);
    now += 10;
    accept(&mut game, &ty, NINJA_PATH, now);
    now += 10;
    talk_step(&mut game, &grosh, GROSH, now, "Ninja: The Ninja Path");
    now += 10;
    release_end_screen(
        &mut game,
        &grosh,
        NINJA_PATH,
        GROSH,
        NINJA_END_TEXT,
        15,
        now,
    );

    // The engine does not recheck the offer-time job at turn-in; the explicit content
    // map still pays profile 2 after the player switched to Brawler.
    now += 10;
    let brawler_before = profile_xp(&game, BRAWLER);
    switch_job(&mut game, &grosh, BRAWLER, now);
    now += 10;
    finish_quest(&mut game, &grosh, NINJA_PATH, NINJA, 0, 15, now);

    assert_eq!(
        active_job(&game),
        BRAWLER,
        "the turn-in must not switch jobs"
    );
    assert_eq!(profile_xp(&game, NINJA), 15, "profile 2 receives the XP");
    assert_eq!(
        profile_xp(&game, BRAWLER),
        brawler_before,
        "the active job receives no XP"
    );
}

#[test]
fn job_locked_quest_is_not_offered() {
    let fixture = Fixture::for_character(CHARACTER);
    let mut game = seeded(&fixture, ADVENTURER, &[1, BRAWLER]);
    // T542: Ty's menu also offers the ungated Become a Ninja; complete it in the saved
    // state so his Talk is the no-offer case the locked-job gate needs.
    let mut state = game.state();
    seed_quest(&mut state, BECOME_A_NINJA, QuestStatus::Completed, &[1]);
    game = fixture.game_as(CHARACTER, Some(state));
    let mut now = 6_000;
    let raina = character_at(RAINA);

    // Qualify the Kart prerequisite as the default Adventurer, then switch to Brawler.
    introduce_yourself(&mut game, &raina, &mut now);
    now += 10;
    switch_job(&mut game, &raina, BRAWLER, now);
    assert!(
        !game
            .state()
            .profiles
            .as_ref()
            .unwrap()
            .unlocked
            .contains(&CHEF)
    );

    assert_no_offer(
        &mut game,
        &character_at(AUGUSTE),
        AUGUSTE,
        &mut now,
        "Brawler at Chef",
    );
    assert_no_offer(
        &mut game,
        &character_at(TY),
        TY,
        &mut now,
        "Brawler at Ninja",
    );
    assert_no_offer(
        &mut game,
        &raina,
        RAINA,
        &mut now,
        "Brawler at Kart with the prerequisite complete",
    );
    let state = game.state();
    let quests = &state.npc_quests.as_ref().unwrap().quests;
    for quest_id in [RANGER_REQUISITION, BURDEN_OF_FAME, NINJA_PATH] {
        assert!(
            !quests.contains_key(&quest_id),
            "a locked job received quest {quest_id}"
        );
    }
}

/// A pre-T202 save accepted these lines without a job gate. Its reward now names a
/// locked profile, so the turn-in must refuse without a grant and release the pending
/// end screen instead of raising a hard error that blocks both retry and abandon.
#[test]
fn locked_profile_turnin_is_refused_cleanly() {
    let fixture = Fixture::for_character(CHARACTER);
    let mut game = seeded(&fixture, CHEF, &[1, CHEF]);
    let mut now = 7_000;
    let auguste = character_at(AUGUSTE);
    let ricardo = character_at(RICARDO);

    offer(&mut game, &auguste, AUGUSTE, RANGER_REQUISITION, now);
    now += 10;
    accept(&mut game, &auguste, RANGER_REQUISITION, now);
    now += 10;
    talk_step(
        &mut game,
        &ricardo,
        RICARDO,
        now,
        "Chef: Ranger Requisition",
    );
    assert_eq!(pending_turnin(&game), Some(RANGER_REQUISITION));

    // Rewrite the saved profile snapshot as a pre-T202 Brawler save: the quest stays
    // active and pending, but Chef is no longer unlocked.
    let mut legacy = game.state();
    legacy.profiles = Some(ProfileState {
        version: 1,
        active: BRAWLER,
        unlocked: BTreeSet::from([1, BRAWLER]),
        experience: BTreeMap::new(),
    });
    let mut game = fixture.game(Some(legacy));
    let inventory_before = game.state().inventory;
    now += 10;
    let refused = send(
        &mut game,
        &ricardo,
        &quest_end_reply(RANGER_REQUISITION),
        now,
    );

    assert!(
        refused.changed,
        "the released end screen must be persisted for a relog"
    );
    assert_eq!(pending_turnin(&game), None, "the end screen is released");
    assert_eq!(
        quest_status(&game, RANGER_REQUISITION),
        QuestStatus::Active,
        "a refusal does not complete the quest"
    );
    assert_eq!(game.state().inventory, inventory_before, "no loot or coins");
    assert_eq!(profile_xp(&game, CHEF), 0, "no XP is paid to a locked job");
    assert_eq!(profile_xp(&game, BRAWLER), 0);
    assert!(
        refused
            .replies
            .iter()
            .all(|packet| packet.starts_with(&[26, 0, 4, 0])),
        "a refused turn-in sends no completion or reward packet: {:?}",
        refused
            .replies
            .iter()
            .map(|packet| &packet[..packet.len().min(4)])
            .collect::<Vec<_>>()
    );

    // The released handshake survives a reconnect, and old packets stay inert.
    game = fixture.reload(&game);
    assert_eq!(pending_turnin(&game), None);
    assert_eq!(quest_status(&game, RANGER_REQUISITION), QuestStatus::Active);
    now += 10;
    assert_packet_changes_nothing(
        &mut game,
        &ricardo,
        &quest_end_reply(RANGER_REQUISITION),
        now,
        "replayed 49/14 after a locked-profile refusal",
    );

    // Not wedged: the quest can now be abandoned, which the pending screen blocked.
    now += 10;
    let abandoned = send(&mut game, &ricardo, &quest_abandon(RANGER_REQUISITION), now);
    assert!(abandoned.changed, "abandon must succeed after the refusal");
    assert_eq!(
        quest_status(&game, RANGER_REQUISITION),
        QuestStatus::Abandoned
    );
}

/// The 49/14 handler builds a synthetic profile snapshot when the server has no class
/// content; the locked-profile guard must hold on that fallback too (review blocker 1).
#[test]
fn locked_profile_turnin_refuses_without_profile_content() {
    let fixture = Fixture::for_character(CHARACTER);
    let mut game = seeded(&fixture, CHEF, &[1, CHEF]);
    let mut now = 8_000;
    let auguste = character_at(AUGUSTE);
    let ricardo = character_at(RICARDO);

    offer(&mut game, &auguste, AUGUSTE, RANGER_REQUISITION, now);
    now += 10;
    accept(&mut game, &auguste, RANGER_REQUISITION, now);
    now += 10;
    talk_step(
        &mut game,
        &ricardo,
        RICARDO,
        now,
        "Chef: Ranger Requisition",
    );
    let pending = game.state().npc_quests;

    let scratch = std::env::temp_dir().join(format!(
        "osfr-quest-job-xp-noprofile-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&scratch);
    copy_tree(fixture.path(), &scratch);
    // Strip class and opening content: the server has no profiles.json and no opening
    // campaign, so `self.profiles` stays None and 49/14 takes its synthetic fallback.
    for file in [
        "profiles.json",
        "opening.json",
        "smelting.json",
        "forging-campaign.json",
        "cooking-campaign.json",
    ] {
        let _ = fs::remove_file(scratch.join(file));
    }
    let mut state = Gameplay::load(&scratch, None)
        .expect("no-profile fixture loads")
        .state();
    state.npc_quests = pending;
    state.profiles = None;
    let mut game = Gameplay::load(&scratch, Some(state)).expect("no-profile save loads");
    game.bind_npc_quest_catalogue(&content_dir(), CHARACTER)
        .expect("split content binds without profiles");
    assert!(game.state().profiles.is_none());

    now += 10;
    let refused = send(
        &mut game,
        &ricardo,
        &quest_end_reply(RANGER_REQUISITION),
        now,
    );
    assert!(refused.changed);
    assert_eq!(pending_turnin(&game), None);
    assert_eq!(quest_status(&game, RANGER_REQUISITION), QuestStatus::Active);
    let _ = fs::remove_dir_all(&scratch);
}

#[test]
fn refused_turn_in_changes_nothing() {
    let fixture = Fixture::for_character(CHARACTER);
    let mut game = seeded(&fixture, CHEF, &[1, CHEF]);
    let mut now = 9_000;
    let auguste = character_at(AUGUSTE);
    let ricardo = character_at(RICARDO);
    let far = CharacterState {
        position: [
            auguste.position[0] + 400.0,
            auguste.position[1],
            auguste.position[2],
        ],
        ..auguste.clone()
    };

    offer(&mut game, &auguste, AUGUSTE, RANGER_REQUISITION, now);
    now += 10;
    accept(&mut game, &auguste, RANGER_REQUISITION, now);

    // Missing objective: the 49/14 has no pending screen to settle.
    now += 10;
    assert_packet_changes_nothing(
        &mut game,
        &ricardo,
        &quest_end_reply(RANGER_REQUISITION),
        now,
        "turn-in before the objective",
    );
    // Wrong quest while the real turn-in is not pending either.
    now += 10;
    assert_packet_changes_nothing(
        &mut game,
        &ricardo,
        &quest_end_reply(INTRODUCE_YOURSELF),
        now,
        "wrong quest turn-in",
    );
    // Out of range: the request is refused before the open menu is consumed.
    now += 10;
    let before_far = saved(&game);
    assert!(
        game.handle_in_world(&npc_interact(AUGUSTE), &far, now)
            .is_ok_and(|out| out.replies.is_empty() && !out.changed),
        "a far interact must be refused"
    );
    assert_eq!(saved(&game), before_far);

    // Complete the objective at Ricardo; the end screen opens there.
    now += 10;
    talk_step(
        &mut game,
        &ricardo,
        RICARDO,
        now,
        "Chef: Ranger Requisition",
    );
    assert_eq!(pending_turnin(&game), Some(RANGER_REQUISITION));
    now += 10;
    release_end_screen(
        &mut game,
        &ricardo,
        RANGER_REQUISITION,
        RICARDO,
        RANGER_END_TEXT,
        20,
        now,
    );
    let held = saved(&game);

    // The formal giver Auguste is not the actor that opened the screen: the Talk
    // reopens nothing and the pending screen stays at Ricardo.
    now += 10;
    let _ = talk(&mut game, &auguste, AUGUSTE, now);
    assert_eq!(
        saved(&game),
        held,
        "a wrong giver changed the pending turn-in"
    );
    assert_eq!(pending_turnin(&game), Some(RANGER_REQUISITION));

    // Out-of-range menu selection leaves the held screen untouched as well.
    now += 10;
    assert!(
        game.handle_in_world(&npc_select(AUGUSTE, 1), &far, now)
            .is_ok_and(|out| out.replies.is_empty() && !out.changed),
        "a far menu selection must be refused"
    );
    assert_eq!(saved(&game), held);

    now += 10;
    finish_quest(&mut game, &ricardo, RANGER_REQUISITION, CHEF, 0, 20, now);
    let finished = saved(&game);
    let finished_coins = coins(&game);
    let finished_xp = profile_xp(&game, CHEF);
    now += 10;
    assert_packet_changes_nothing(
        &mut game,
        &ricardo,
        &quest_end_reply(RANGER_REQUISITION),
        now,
        "replayed 49/14",
    );
    now += 10;
    assert_packet_changes_nothing(
        &mut game,
        &auguste,
        &quest_reply(RANGER_REQUISITION, true),
        now,
        "replayed 49/2",
    );
    assert_eq!(saved(&game), finished);
    assert_eq!(coins(&game), finished_coins);
    assert_eq!(profile_xp(&game, CHEF), finished_xp);
}

#[test]
fn xp_survives_relog_and_is_not_repaid() {
    let fixture = Fixture::for_character(CHARACTER);
    let mut game = seeded(&fixture, NINJA, &[1, NINJA, BRAWLER]);
    let mut now = 10_000;
    let ty = character_at(TY);
    let grosh = character_at(GROSH);

    offer(&mut game, &ty, TY, NINJA_PATH, now);
    now += 10;
    accept(&mut game, &ty, NINJA_PATH, now);
    now += 10;
    talk_step(&mut game, &grosh, GROSH, now, "Ninja: The Ninja Path");
    assert_eq!(pending_turnin(&game), Some(NINJA_PATH));
    assert_eq!(profile_xp(&game, NINJA), 0, "no XP before the turn-in");

    // Disconnect with the completed quest and its open end screen still pending.
    game = fixture.reload(&game);
    assert_eq!(pending_turnin(&game), Some(NINJA_PATH));
    assert_eq!(quest_status(&game, NINJA_PATH), QuestStatus::Active);
    assert_eq!(profile_xp(&game, NINJA), 0);
    // The replayed 49/13 arrives with the speaker spawn; the client's 49/14 settles
    // the pending screen without needing the session-only held packet.
    now += 10;
    finish_quest(&mut game, &grosh, NINJA_PATH, NINJA, 0, 15, now);
    let finished = saved(&game);

    game = fixture.reload(&game);
    assert_eq!(saved(&game), finished, "reload changed the paid save");
    assert_eq!(profile_xp(&game, NINJA), 15, "the paid XP survives reload");
    assert_eq!(quest_status(&game, NINJA_PATH), QuestStatus::Completed);
    assert_eq!(pending_turnin(&game), None);

    // The completed quest is not offered again, and old packets pay nothing. T542: Ty's
    // menu also offers the ungated Become a Ninja, so that one is completed here too.
    let mut state = game.state();
    seed_quest(&mut state, BECOME_A_NINJA, QuestStatus::Completed, &[1]);
    game = fixture.game_as(CHARACTER, Some(state));
    assert_no_offer(
        &mut game,
        &ty,
        TY,
        &mut now,
        "completed Ninja quest after relog",
    );
    now += 10;
    assert_packet_changes_nothing(
        &mut game,
        &grosh,
        &quest_end_reply(NINJA_PATH),
        now,
        "replayed turn-in after relog",
    );
    now += 10;
    assert_packet_changes_nothing(
        &mut game,
        &ty,
        &quest_reply(NINJA_PATH, true),
        now,
        "replayed accept after relog",
    );
    assert_eq!(profile_xp(&game, NINJA), 15);
    assert_eq!(coins(&game), 0, "the zero-coin Ninja row pays XP only");
}

#[test]
fn two_players_are_paid_independently() {
    let first_fixture = Fixture::for_character(202_101);
    let second_fixture = Fixture::for_character(202_102);
    let mut first = first_fixture.game(None);
    let mut second = second_fixture.game(None);
    let mut now = 11_000;

    let first_character = character_at_for(202_101, RAINA);
    let second_character = character_at_for(202_102, RAINA);
    introduce_yourself(&mut first, &first_character, &mut now);
    introduce_yourself(&mut second, &second_character, &mut now);

    for game in [&first, &second] {
        assert_eq!(
            quest_status(game, INTRODUCE_YOURSELF),
            QuestStatus::Completed
        );
        assert_eq!(profile_xp(game, ADVENTURER), 15);
        assert_eq!(coins(game), 5);
    }
    assert_eq!(
        first.state().npc_quests.as_ref().unwrap().character_id,
        202_101,
        "each character owns its own quest state"
    );
    assert_eq!(
        second.state().npc_quests.as_ref().unwrap().character_id,
        202_102
    );
}

/// The client reads the displayed reward XP from the 49/13 bundle (reader
/// `008e7930`: success byte at 26, coins at 27..31, stars at 31..35). Every
/// configured row must reach that dword; the profile XP alone is not enough.
#[test]
fn reward_bundle_displays_each_configured_job_xp_total() {
    let content = content_dir();
    let engine =
        QuestEngine::from_split_content(&content, 1, CharacterEligibility::default(), None)
            .expect("split quest content loads");
    let listed: [(&str, u32); 12] = [
        ("adventurer.json", 3_000_020),
        ("adventurer.json", 3_000_021),
        ("adventurer.json", 3_000_022),
        ("adventurer.json", 3_000_059),
        ("adventurer.json", 3_000_060),
        ("chef.json", 3_050_004),
        ("chef.json", 3_050_011),
        ("kart_driver.json", 3_080_001),
        ("kart_driver.json", 3_080_002),
        ("ninja.json", 3_110_001),
        ("ninja.json", 3_110_002),
        ("ninja.json", 3_110_004),
    ];
    for (file, quest_id) in listed {
        let document: Value = serde_json::from_slice(
            &fs::read(content.join("quests").join(file))
                .unwrap_or_else(|error| panic!("{file}: {error}")),
        )
        .unwrap();
        let record = document["records"]
            .as_array()
            .unwrap()
            .iter()
            .find(|record| record["id"].as_u64() == Some(u64::from(quest_id)))
            .unwrap_or_else(|| panic!("{file}: quest {quest_id} exists"));
        let rewards = record["rewards"].as_array().unwrap();
        let coins: u64 = rewards
            .iter()
            .map(|reward| reward["coins"].as_u64().unwrap())
            .sum();
        let xp: u64 = rewards
            .iter()
            .flat_map(|reward| reward["job_xp"].as_object().into_iter().flatten())
            .map(|(_, amount)| amount.as_u64().unwrap())
            .sum();
        let packet = engine
            .quest_end_packet(quest_id, 9_000_000_001)
            .unwrap_or_else(|error| panic!("quest {quest_id} end screen: {error}"));
        assert_eq!(
            packet.len(),
            99,
            "quest {quest_id}: no display rows, so the client-reader bundle is 99 bytes"
        );
        assert_eq!(
            u32::from_le_bytes(packet[14..18].try_into().unwrap()),
            quest_id
        );
        assert_eq!(
            u32::from_le_bytes(packet[27..31].try_into().unwrap()),
            coins as u32,
            "quest {quest_id}: authored coins are the bundle's coin dword"
        );
        assert_eq!(
            i32::from_le_bytes(packet[31..35].try_into().unwrap()),
            xp as i32,
            "quest {quest_id}: authored job XP is the bundle's displayed XP dword"
        );
    }
}

fn copy_split_content(source: &Path, target: &Path) {
    copy_tree(&source.join("quests"), &target.join("quests"));
    copy_tree(&source.join("npcs"), &target.join("npcs"));
    fs::copy(source.join("jobs.json"), target.join("jobs.json")).unwrap();
}

/// Overwrite the real Ninja entry quest's reward map in a temporary content copy.
fn set_ninja_entry_job_xp(directory: &Path, job_xp: Value) {
    let path = directory.join("quests/ninja.json");
    let mut document: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    let record = document["records"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|record| record["id"].as_u64() == Some(u64::from(NINJA_PATH)))
        .expect("the copied content still has the Ninja entry quest");
    record["rewards"][0]["job_xp"] = job_xp;
    fs::write(&path, serde_json::to_vec(&document).unwrap()).unwrap();
}

#[test]
fn content_overflow_is_rejected_at_load() {
    let fixture = Fixture::for_character(CHARACTER);
    let scratch = std::env::temp_dir().join(format!(
        "osfr-quest-job-xp-load-{}-{}",
        std::process::id(),
        NEXT_SCRATCH.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ));
    let eligibility = CharacterEligibility {
        member: true,
        member_quest_limit: 10,
        nonmember_quest_limit: 10,
        ..CharacterEligibility::default()
    };

    copy_split_content(&content_dir(), &scratch);
    if let Err(error) = QuestEngine::from_split_content(&scratch, 1, eligibility.clone(), None) {
        panic!("the control content must load: {error}");
    }

    // Each entry fits a wire i32; their per-grant total does not.
    set_ninja_entry_job_xp(
        &scratch,
        json!({"1": 2_000_000_000u32, "2": 2_000_000_000u32}),
    );
    let error = match QuestEngine::from_split_content(&scratch, 1, eligibility.clone(), None) {
        Ok(_) => panic!("an over-budget XP total must fail content load"),
        Err(error) => error,
    };
    assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);

    // A profile id that jobs.json does not define fails before any player exists.
    set_ninja_entry_job_xp(&scratch, json!({"9999": 15}));
    let error = match QuestEngine::from_split_content(&scratch, 1, eligibility, None) {
        Ok(_) => panic!("an unknown XP profile must fail content load"),
        Err(error) => error,
    };
    assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);

    let mut game = Gameplay::load(fixture.path(), None).expect("opening fixture loads");
    let before = saved(&game);
    let error = match game.bind_npc_quest_catalogue(&scratch, CHARACTER) {
        Ok(()) => panic!("the request path must refuse the bad content"),
        Err(error) => error,
    };
    assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
    assert!(
        game.state().npc_quests.is_none(),
        "a rejected catalogue must not create quest state"
    );
    assert_eq!(
        saved(&game),
        before,
        "a rejected catalogue changed the player"
    );

    let _ = fs::remove_dir_all(&scratch);
}

#[test]
fn configured_job_xp_and_job_gates_match_the_task_table() {
    let content = content_dir();
    // (file, quest id, giver, prerequisites, reward coins, job gate, profile, XP)
    let expected: [(&str, u32, u64, &[u32], u32, Option<i32>, i32, u32); 28] = [
        (
            "adventurer.json",
            INTRODUCE_YOURSELF,
            MERIGOLD,
            &[],
            5,
            None,
            ADVENTURER,
            15,
        ),
        (
            "adventurer.json",
            3_000_021,
            9_000_000_189,
            &[],
            5,
            None,
            ADVENTURER,
            20,
        ),
        (
            "adventurer.json",
            3_000_022,
            9_000_000_227,
            &[],
            5,
            None,
            ADVENTURER,
            15,
        ),
        (
            "adventurer.json",
            3_000_059,
            9_500_000_254,
            &[3_000_021],
            5,
            None,
            ADVENTURER,
            30,
        ),
        (
            "adventurer.json",
            3_000_060,
            9_500_000_254,
            &[3_000_059],
            5,
            None,
            ADVENTURER,
            25,
        ),
        (
            "chef.json",
            3_050_004,
            AUGUSTE,
            &[],
            5,
            Some(CHEF),
            CHEF,
            25,
        ),
        (
            "chef.json",
            RANGER_REQUISITION,
            AUGUSTE,
            &[],
            0,
            Some(CHEF),
            CHEF,
            20,
        ),
        (
            "kart_driver.json",
            BURDEN_OF_FAME,
            RAINA,
            &[INTRODUCE_YOURSELF],
            5,
            Some(KART_DRIVER),
            KART_DRIVER,
            25,
        ),
        (
            "kart_driver.json",
            3_080_002,
            MUSTANG,
            &[BURDEN_OF_FAME],
            5,
            Some(KART_DRIVER),
            KART_DRIVER,
            30,
        ),
        // T726: the Seaside Board of Tourism Sponsorship (Petal Oceanmist, 9500000613);
        // 5 coins / 30 Kart Driver XP repeat 3080002's rung (no source gives a reward).
        // F745 keeps the row as the decision pin the final scan's allowlist needs.
        (
            "kart_driver.json",
            3_080_019,
            9_500_000_613,
            &[],
            5,
            Some(KART_DRIVER),
            KART_DRIVER,
            30,
        ),
        // T732: two Wildwood round-3 quests. Each is a one-Talk referral; the
        // predecessor stubs are still disabled, so neither gains a prerequisite.
        (
            "kart_driver.json",
            3_080_007,
            9_500_000_164,
            &[],
            5,
            Some(KART_DRIVER),
            KART_DRIVER,
            15,
        ),
        (
            "kart_driver.json",
            3_080_009,
            9_500_000_872,
            &[],
            5,
            Some(KART_DRIVER),
            KART_DRIVER,
            15,
        ),
        ("ninja.json", NINJA_PATH, TY, &[], 0, Some(NINJA), NINJA, 15),
        (
            "ninja.json",
            3_110_002,
            9_000_000_335,
            &[NINJA_PATH],
            0,
            Some(NINJA),
            NINJA,
            25,
        ),
        (
            "ninja.json",
            3_110_004,
            TY,
            &[3_110_002],
            5,
            Some(NINJA),
            NINJA,
            20,
        ),
        // T241: the live Farnum starting pair. Fighting the Pack is a playtest entry
        // (the disabled 3000008 predecessor is dropped); Copper Collection listens to the
        // Copper Ore item definition the ordinary Snowhill node path emits.
        (
            "adventurer.json",
            3_000_009,
            CLEETUS,
            &[],
            18,
            Some(BRAWLER),
            BRAWLER,
            25,
        ),
        (
            "adventurer.json",
            3_000_016,
            CLEETUS,
            &[],
            5,
            Some(MINER),
            MINER,
            15,
        ),
        // T279: the second story beat. It is an independent playtest entry (no
        // prerequisite and no job gate); its 25 XP pays the explicit Adventurer profile.
        (
            "adventurer.json",
            3_000_029,
            GEOFFREY,
            &[],
            5,
            None,
            ADVENTURER,
            25,
        ),
        // T277: the Royal Courier's independent Local Hero entry. Like T279 it has
        // no prerequisite and no job gate; its 25 XP pays the explicit Adventurer
        // profile and its 13 coins are the guide's final popup.
        (
            "adventurer.json",
            3_000_024,
            ROYAL_COURIER,
            &[],
            13,
            None,
            ADVENTURER,
            25,
        ),
        // T330: the chapter after T279's playable 3000029. The disabled middle
        // records are skipped, so its only prerequisite is the playable chapter;
        // it keeps T277's 13-coin chapter rung and pays explicit profile 1.
        (
            "adventurer.json",
            3_000_032,
            GEOFFREY,
            &[3_000_029],
            13,
            None,
            ADVENTURER,
            25,
        ),
        // T377: the next two chapter records. The disabled E2/E3 are skipped, so E5
        // chains from the playable E4 and E6 from E5. Both keep the small-referral
        // fallback (8 coins, explicit Adventurer profile-1 XP, no item).
        (
            "adventurer.json",
            3_000_033,
            GEOFFREY,
            &[3_000_032],
            8,
            None,
            ADVENTURER,
            20,
        ),
        (
            "adventurer.json",
            3_000_034,
            GEOFFREY,
            &[3_000_033],
            8,
            None,
            ADVENTURER,
            20,
        ),
        // T252's two Cobblestone Brawler records; T331's C6 is the hand-off they lead to.
        // The table now covers every job-gated Brawler record in `brawler.json`, so the
        // final scan below can reject a new gate or XP row in that file.
        (
            "brawler.json",
            3_030_004,
            9_000_000_042,
            &[],
            5,
            Some(BRAWLER),
            BRAWLER,
            25,
        ),
        (
            "brawler.json",
            3_030_005,
            9_000_000_042,
            &[3_030_004],
            5,
            Some(BRAWLER),
            BRAWLER,
            25,
        ),
        // T331: C6 keeps the Brawler hand-off gate and the small-chapter 5-coin rung.
        (
            "brawler.json",
            3_030_009,
            9_000_000_446,
            &[3_030_008],
            5,
            Some(BRAWLER),
            BRAWLER,
            25,
        ),
        // T724: Brawler: IOU! (M8), a playtest entry behind the band gate (min_level 5 replaces the
        // disabled 3030026 prerequisite); the guide's reward cell is unknown, so the small-quest rung.
        (
            "brawler.json",
            3_030_027,
            9_000_000_009,
            &[],
            5,
            Some(BRAWLER),
            BRAWLER,
            25,
        ),
        // T334: the second Miner chapter, after T300's playable 3100002. 5 coins
        // and 15 Miner XP mirror the first rung; the guide's reward cell is unknown
        // and no item is granted.
        (
            "miner.json",
            3_100_003,
            9_000_000_217,
            &[3_100_002],
            5,
            Some(MINER),
            MINER,
            15,
        ),
        // T411: the third Miner chapter, Trixi's Rush Order after 3100003. 5 coins and
        // 20 Miner XP follow the small follow-up rate; no item is granted.
        (
            "miner.json",
            3_100_004,
            9_000_000_220,
            &[3_100_003],
            5,
            Some(MINER),
            MINER,
            20,
        ),
    ];

    let mut listed = BTreeSet::new();
    // The task decisions are accepted: T202 wrote the chapter job XP and gates, T241
    // extended the same table with the two live Farnum starting records, T279 adds
    // the independent Missing Sisters playtest entry, T277 adds Local Hero, T252's two
    // Cobblestone Brawler records and T331's hand-off are added with their own tags,
    // T334 adds the second Miner chapter behind T300's practice test, T377 adds
    // the two chapter records after E4, T411 adds the third Miner chapter, T726 adds
    // Petal Oceanmist's sponsorship quest and T732 adds the two Wildwood referrals.
    // A later task's decision tag counts only for the record that task decided, so a
    // new quest cannot satisfy the table by carrying someone else's tag.
    let has_decision = |quest_id: u32, source: &Value| {
        let own_tag = match quest_id {
            3_000_009 | 3_000_016 => "(R) T241:",
            3_000_024 => "(R) T277:",
            3_000_029 => "(R) T279:",
            3_000_032 => "(R) T330:",
            3_000_033 | 3_000_034 => "(R) T377:",
            3_030_004 | 3_030_005 => "(R) T252:",
            3_030_009 => "(R) T331:",
            3_030_027 => "(R) T724:",
            3_100_003 => "(R) T334:",
            3_100_004 => "(R) T411:",
            3_080_007 | 3_080_009 => "(R) T732:",
            3_080_019 => "(R) T726:",
            _ => "",
        };
        source.as_array().is_some_and(|tags| {
            tags.iter().any(|tag| {
                tag["evidence"].as_str().is_some_and(|text| {
                    text.starts_with("(R) T202:")
                        || (!own_tag.is_empty() && text.starts_with(own_tag))
                })
            })
        })
    };
    for (file, quest_id, giver, prerequisites, coins, job, profile, xp) in expected {
        let document: Value =
            serde_json::from_slice(&fs::read(content.join("quests").join(file)).unwrap()).unwrap();
        let record = document["records"]
            .as_array()
            .unwrap()
            .iter()
            .find(|record| record["id"].as_u64() == Some(u64::from(quest_id)))
            .unwrap_or_else(|| panic!("{file}: quest {quest_id} exists"));
        assert!(listed.insert((file, quest_id)));
        assert_ne!(
            record["enabled"],
            json!(false),
            "{file}: quest {quest_id} must stay enabled"
        );
        assert_eq!(record["giver_npc_id"].as_u64(), Some(giver));
        assert_eq!(
            record["prerequisite_quest_ids"],
            json!(prerequisites),
            "{file}: quest {quest_id} prerequisites are unchanged"
        );
        if let Some(job) = job {
            assert_eq!(
                record["job"].as_i64(),
                Some(i64::from(job)),
                "{file}: quest {quest_id} must be offered only to its job"
            );
        } else {
            assert!(
                record.get("job").is_none(),
                "{file}: quest {quest_id} must not gain a job gate"
            );
        }
        let rewards = record["rewards"].as_array().unwrap();
        assert_eq!(rewards.len(), 1, "{file}: quest {quest_id} pays one row");
        let reward = &rewards[0];
        assert_eq!(reward["coins"].as_u64(), Some(u64::from(coins)));
        assert_eq!(reward["items"], json!([]));
        assert_eq!(
            reward["job_xp"],
            json!({ profile.to_string(): xp }),
            "{file}: quest {quest_id} pays the configured profile"
        );
        assert!(
            reward["source"]
                .as_array()
                .is_some_and(|source| !source.is_empty()),
            "{file}: quest {quest_id} reward provenance"
        );
        assert!(
            record["source"]
                .as_array()
                .is_some_and(|source| source.iter().any(|tag| {
                    tag["kind"] == "reconstructed"
                        && tag["evidence"]
                            .as_str()
                            .is_some_and(|text| text.starts_with("(R)"))
                })),
            "{file}: quest {quest_id} carries an (R) source decision"
        );
        if job.is_some() {
            assert!(
                has_decision(quest_id, &record["source"]),
                "{file}: quest {quest_id} carries its (R) job-gate decision"
            );
        }
        assert!(
            has_decision(quest_id, &reward["source"]),
            "{file}: quest {quest_id} reward carries its (R) decision"
        );
    }

    // Only the listed IDs gained XP or a job gate in these five files.
    for file in [
        "adventurer.json",
        "brawler.json",
        "chef.json",
        "kart_driver.json",
        "ninja.json",
    ] {
        let document: Value =
            serde_json::from_slice(&fs::read(content.join("quests").join(file)).unwrap()).unwrap();
        for record in document["records"].as_array().unwrap() {
            let id = record["id"].as_u64().unwrap() as u32;
            if listed.contains(&(file, id)) {
                continue;
            }
            // T723/T667/T672: Chef: Reach Level 5, 10 and 20 are level gates (job gate 45,
            // coins only, no XP); tests/zone_quests_sanctuary_round3.rs,
            // tests/zone_quests_merry_vale_round2.rs and tests/zone_quests_wilds_part2.rs cover
            // them. T672 also enables Chef: Chugawhat? (3050024, job gate 45, 25 Chef XP),
            // covered by tests/zone_quests_merry_vale_round2.rs.
            if file == "chef.json" && [3_050_020, 3_050_023, 3_050_024, 3_050_031].contains(&id) {
                continue;
            }
            // T670: Chef: Delivery Derailed and Gavin Gone Missing are one-talk referrals on the
            // Ranger Requisition rung (job gate 45, 20 Chef XP, no coins);
            // tests/zone_quests_round2_chef.rs covers them.
            if file == "chef.json" && [3_050_007, 3_050_016].contains(&id) {
                continue;
            }
            // T660: Brawler: Achieve Level 5 is the same kind of level gate (job gate 43, coins
            // only, no XP); tests/zone_quests_blackspore.rs covers it.
            if (file, id) == ("brawler.json", 3_030_021) {
                continue;
            }
            // T662/T668: Ninja: Achieve Level 10 and Achieve Level 5 are level gates (job gate 2,
            // coins only, no XP); tests/zone_quests_lakeshore.rs and zone_quests_blackspore.rs
            // cover them.
            if file == "ninja.json" && [3_110_007, 3_110_013].contains(&id) {
                continue;
            }
            for reward in record["rewards"].as_array().into_iter().flatten() {
                assert_eq!(
                    reward["job_xp"],
                    json!({}),
                    "{file}: quest {id} gained unexpected job XP"
                );
            }
            assert!(
                record.get("job").is_none(),
                "{file}: quest {id} gained an unexpected job gate"
            );
        }
    }
}
