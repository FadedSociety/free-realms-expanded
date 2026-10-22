//! T407: Postman (profile 4) is learnable from a drawn trainer, and every job an
//! enabled quest pays or gates has a learn path.
//!
//! Before T407 the only enabled Postman quest, "Postman: Trail of Mail" (3130032),
//! paid 5 Postman XP no player could receive: `job_trainers.json` had no profile-4
//! row, so the locked-profile reward gate (`rewards::locked_job_profile`) refused
//! every turn-in. The fix is one content row (Flitter in Sanctuary, menu label
//! 412445 "Become a Postman"); no production Rust changed.
//!
//! Every packet goes through `Gameplay::handle_in_world`, the router the gateway
//! session calls. Client wire evidence, unchanged by this slice: 26/8 interact
//! `0x008b6bc0`; 26/9 menu `0x00aa2360 -> 0x00aa2220 -> 0x00aa1600 -> 0x00aa04d0`
//! (option count, then seven dwords per option read by `0x00a9dca0`, the label
//! third); 26/10 select `0x00a36f80`; 26/13 job select, applied by
//! `ProcessProfileSwap` `0x00bfb620` only with Chest and Legs; 26/6 `0x009f5890`;
//! 49/1 `0x00c7bb60`; 49/2 `0x00a86400`; 49/13 `0x00c7b990`; 49/14 `0x00a864c0`.

#[path = "common/job_playtest.rs"]
mod job_playtest;
#[path = "support/npc_quest_chain.rs"]
mod npc_quest_chain;

use npc_quest_chain::{
    CHARACTER, Fixture, accept, active_job, assert_packet_changes_nothing, character_at_for,
    content_dir, content_position, end_screen_reward, finish_quest, interact, menu_option,
    npc_interact, npc_select, offer, profile_xp, quest_end_reply, read_json, release_end_screen,
    saved, select_job, send, talk_step,
};
use osfr_tools::{
    character_store::CharacterState,
    gameplay::{Gameplay, Outcome},
    profiles::Profiles,
    quest_engine::{CharacterEligibility, QuestEngine},
};
use std::{collections::BTreeSet, fs};

/// Flitter, `sanctuary.json`: the profile-4 row of `job_trainers.json`.
const TRAINER: u64 = 9_000_000_091;
const BECOME_A_POSTMAN: u32 = 412_445;
const POSTMAN: i32 = 4;
const ADVENTURER: i32 = 1;
/// The opening save's other job; unlike Adventurer it wears a Chest/Legs outfit.
const BRAWLER: i32 = 43;
const QUEST: u32 = 3_130_032;
const GIVER: u64 = 9_000_000_173;
const TARGET: u64 = 9_000_000_172;
const END_TEXT: u32 = 141_831;
/// The play package's Postman Chest and Legs starter pieces.
const CHEST_AND_LEGS: [i32; 2] = [12_827, 13_197];

fn at(npc: u64, file: &str) -> CharacterState {
    character_at_for(CHARACTER, 0, content_position(file, npc))
}

fn unlocked(game: &Gameplay) -> BTreeSet<i32> {
    game.state().profiles.expect("profiles").unlocked
}

fn owned(game: &Gameplay, definition: i32) -> u32 {
    let items = game.state().inventory.items;
    items
        .iter()
        .filter(|item| item.definition_id == definition)
        .map(|item| item.count)
        .sum()
}

fn said(outcome: &Outcome, text: &str) -> bool {
    outcome
        .replies
        .iter()
        .any(|reply| String::from_utf8_lossy(reply).contains(text))
}

/// A refusal may answer with a chat line or an error; either way the save is unchanged.
fn assert_refused(game: &mut Gameplay, who: &CharacterState, packet: &[u8], now: u64, label: &str) {
    let before = saved(game);
    let outcome = game.handle_in_world(packet, who, now);
    assert!(!outcome.is_ok_and(|outcome| outcome.changed), "{label}");
    assert_eq!(saved(game), before, "{label} changed the save");
}

#[test]
fn a_new_player_learns_postman_at_flitter_and_is_paid_for_trail_of_mail() {
    let fixture = Fixture::new();
    job_playtest::install_starter_outfits(fixture.path());
    let mut game = fixture.game(None);
    assert!(
        !unlocked(&game).contains(&POSTMAN),
        "the opening save lacks Postman"
    );
    let trainer = at(TRAINER, "sanctuary.json");
    let [x, y, z] = trainer.position;
    let far = character_at_for(CHARACTER, 0, [x + 500.0, y, z]);
    let mut now = 1_000;

    assert_refused(&mut game, &far, &npc_interact(TRAINER), now, "far 26/8");
    assert_refused(
        &mut game,
        &trainer,
        &npc_select(TRAINER, 1),
        now,
        "26/10, no menu",
    );
    let menu = interact(&mut game, &trainer, TRAINER, now);
    let choice = menu_option(&menu, BECOME_A_POSTMAN);
    let forged = npc_select(TRAINER, choice + 50);
    assert_refused(&mut game, &trainer, &forged, now + 1, "unoffered 26/10");
    assert_refused(&mut game, &trainer, &forged[..12], now + 1, "short 26/10");
    assert!(!unlocked(&game).contains(&POSTMAN));

    // The refusals left the open menu usable: its Learn choice unlocks Postman.
    let learned = send(&mut game, &trainer, &npc_select(TRAINER, choice), now + 2);
    assert!(learned.changed && said(&learned, "You learned the Postman job"));
    assert!(unlocked(&game).contains(&POSTMAN));
    assert_eq!(
        active_job(&game),
        ADVENTURER,
        "learning does not switch jobs"
    );
    for piece in CHEST_AND_LEGS {
        assert_eq!(owned(&game, piece), 1, "starter piece {piece} granted once");
    }

    // The starter outfit passes the client's Chest/Legs gate, so 26/13 activates
    // Postman. Switching back proves the reward follows the quest's job map.
    now += 10;
    let switched = send(&mut game, &trainer, &select_job(POSTMAN), now);
    assert!(
        switched
            .replies
            .iter()
            .any(|r| r.starts_with(&[38, 0, 21, 0]))
    );
    assert_eq!(active_job(&game), POSTMAN);
    send(&mut game, &trainer, &select_job(BRAWLER), now + 1);
    assert_eq!(active_job(&game), BRAWLER);

    let giver = at(GIVER, "shrouded_glade.json");
    let target = at(TARGET, "wugachug.json");
    now += 10;
    offer(&mut game, &giver, GIVER, QUEST, now);
    accept(&mut game, &giver, QUEST, now + 10);
    talk_step(
        &mut game,
        &target,
        TARGET,
        now + 20,
        "Postman: Trail of Mail",
    );
    let end = release_end_screen(&mut game, &target, QUEST, TARGET, END_TEXT, now + 30);
    assert_eq!(end_screen_reward(&end), (5, 5));
    let items = game.state().inventory.items;
    finish_quest(&mut game, &target, QUEST, POSTMAN, 5, 5, now + 40);
    assert_eq!(
        game.state().inventory.items,
        items,
        "the quest grants no item"
    );
    assert_eq!(profile_xp(&game, POSTMAN), 5);
    let replay = quest_end_reply(QUEST);
    assert_packet_changes_nothing(&mut game, &target, &replay, now + 50, "replayed 49/14");

    // Relog keeps the job, outfit, wallet and completed quest; the trainer then
    // refuses a second lesson without granting the outfit again.
    let paid = saved(&game);
    let mut game = fixture.reload_serialized(&game);
    assert_eq!(saved(&game), paid, "relog changed the save");
    let menu = interact(&mut game, &trainer, TRAINER, now + 60);
    let choice = menu_option(&menu, BECOME_A_POSTMAN);
    let again = send(&mut game, &trainer, &npc_select(TRAINER, choice), now + 61);
    assert!(!again.changed && said(&again, "already know"));
    assert_eq!(saved(&game), paid, "a second lesson changed the save");
}

/// A quest that pays (`rewards[].job_xp`) or gates (`job`) a profile no player can
/// unlock is dead content: the reward gate refuses its turn-in forever. A profile
/// is learnable when the opening save unlocks it or a trainer row teaches it after
/// a learnable prerequisite.
#[test]
fn every_job_paid_or_gated_by_an_enabled_quest_has_a_learn_path() {
    let fixture = Fixture::new();
    let profiles = Profiles::load(fixture.path(), None).unwrap();
    let trainers = profiles.load_trainers(&content_dir()).unwrap();
    let mut learnable = profiles.state().unlocked.clone();
    while let Some(row) = trainers.iter().find(|row| {
        !learnable.contains(&row.profile)
            && row
                .requires
                .as_ref()
                .is_none_or(|need| learnable.contains(&need.profile))
    }) {
        learnable.insert(row.profile);
    }

    let engine = QuestEngine::from_split_content(
        &content_dir(),
        CHARACTER,
        CharacterEligibility::default(),
        None,
    )
    .unwrap();
    let mut needed = BTreeSet::new();
    for entry in fs::read_dir(content_dir().join("quests")).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_none_or(|ext| ext != "json") {
            continue;
        }
        for quest in read_json(&path)["records"].as_array().unwrap() {
            let id = quest["id"].as_u64().unwrap();
            if !engine.is_enabled(u32::try_from(id).unwrap()) {
                continue;
            }
            let paid = quest["rewards"].as_array().unwrap().iter();
            let paid = paid.flat_map(|reward| reward["job_xp"].as_object().into_iter().flatten());
            let paid = paid.map(|(profile, _)| profile.parse::<i64>().unwrap());
            for profile in quest["job"].as_i64().into_iter().chain(paid) {
                needed.insert((i32::try_from(profile).unwrap(), id));
            }
        }
    }
    assert!(
        needed.contains(&(POSTMAN, u64::from(QUEST))),
        "the audit must see Trail of Mail paying Postman"
    );
    let dead: Vec<_> = needed
        .iter()
        .filter(|(profile, _)| !learnable.contains(profile))
        .collect();
    assert!(
        dead.is_empty(),
        "(profile, quest) with no learn path: {dead:?}"
    );
}
