//! T197: the Medic entry chapter — So You Want to be a Medic? (3090001),
//! House Call (3090003) and Take Your Vitamins (3090005).
//!
//! The chapter is content-only; these tests drive the production in-world router
//! (`Gameplay::handle_in_world` -> `gameplay/npc.rs` -> `QuestEngine`), the same
//! path the gateway calls for every in-world request, instead of engine internals.
//! Medic is learned through Nurse Naia's ordinary trainer menu and activated with
//! an ordinary 26/13 selection.
//!
//! Client request/reply layouts used here (verified in the client readers):
//! - 26/8 NPC interact: constructor `008b6bc0`; u64 target + u8 [0].
//! - 26/10 menu select: constructor `00a36f80`; u64 target + i32 choice.
//! - 26/6 dialogue response: reader `009f5890`; i32 row (301 is the quest row).
//! - 49/2 quest reply: reader `00a86400` / sender `00a97d70`; u32 quest + u8 accept.
//! - 49/14 end reply: constructor `00a864c0` / sender `00a97ef0`; u32 quest.
//! 49/1 offers are read for their quest id at 6..10 (`QuestOffer` reader
//! `00c7bb60`); 49/13 end screens name the speaker at 6..14 and the quest at
//! 14..18 (`QuestEndPacket::ReadFields` `00c7b990`, resolved by `00984960` in
//! `ShowQuestEndScreen` `00a95420`).
//!
//! Giver reachability matters here: Naia (9000000161) and Greyli Dustywings
//! (9500000488) are legacy_data spawns. Doctor Gordon (9000000073) was an
//! unanchored row that `NpcVisibility` never spawned, so this slice corrected it
//! to its Legacy built-content spawn (FabledRealms.lua:1083, Npcs.json Id=2339);
//! the content test pins that.
//!
//! The final Talk of House Call and Take Your Vitamins is at Greyli. The engine
//! opens the 49/13 at the actor whose Talk completed the last objective
//! (`QuestEngine::on_event`, named-objective-target rule), so the 49/14 is sent
//! from there; the client text has no return-to-the-giver step for either quest.
//! This is the same recovered rule the T194 Archer Oath chapter uses at Hilforn.

use osfr_tools::{
    character_store::CharacterState,
    gameplay::{Gameplay, GameplayState, Outcome},
    quest_engine::QuestStatus,
};
use serde_json::Value;
use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

const CHARACTER: u64 = 197_017;
const NAIA: u64 = 9_000_000_161;
const GORDON: u64 = 9_000_000_073;
const GREYLI: u64 = 9_500_000_488;
const MEDIC: i32 = 11;
const BRAWLER: i32 = 43;
const EXAM: u32 = 3_090_001;
const HOUSE_CALL: u32 = 3_090_003;
const VITAMINS: u32 = 3_090_005;
const EXAM_COINS: u32 = 5;
const EXAM_XP: u32 = 15;
const HOUSE_COINS: u32 = 10;
const HOUSE_XP: u32 = 20;
const VITAMIN_COINS: u32 = 15;
const VITAMIN_XP: u32 = 25;
const SANCTUARY: u32 = 10;
/// `job_trainers.json` row for Nurse Naia (profile 11).
const LEARN_LABEL: u32 = 412_422;
/// Generic Talk option in the reconstructed 26/9 menu.
const TALK_LABEL: u32 = 3_300;
/// First response row of the quest dialogue (client 26/6).
const QUEST_CHOICE: i32 = 301;
static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(0);

fn content_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content")
}

fn reference_content() -> PathBuf {
    std::env::var_os("OSFR_MEDIC_ENTRY_CONTENT")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../../lab/references/opening-anvil-placement-v1/content")
        })
}

fn read_json(path: &Path) -> Value {
    serde_json::from_slice(
        &fs::read(path).unwrap_or_else(|error| panic!("{}: {error}", path.display())),
    )
    .unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

fn copy_dir(source: &Path, destination: &Path) {
    fs::create_dir_all(destination).unwrap();
    for entry in fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        let target = destination.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_dir(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), &target).unwrap();
        }
    }
}

/// Opening profile/inventory content plus the repository's trainer content. The
/// repository's split quest and NPC content is bound the way the bundled play
/// package binds it (`bind_npc_quest_catalogue`).
struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        let source = reference_content();
        assert!(
            source.join("profiles.json").is_file() && source.join("inventory.json").is_file(),
            "the Medic chapter needs the opening content fixture at {}",
            source.display()
        );
        let directory = std::env::temp_dir().join(format!(
            "osfr-medic-entry-{}-{}",
            std::process::id(),
            NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed)
        ));
        copy_dir(&source, &directory);
        fs::copy(
            content_dir().join("job_trainers.json"),
            directory.join("job_trainers.json"),
        )
        .unwrap();
        Self(directory)
    }

    fn game(&self, state: Option<GameplayState>) -> Gameplay {
        let mut game = Gameplay::load(&self.0, state).expect("opening fixture loads");
        game.bind_npc_quest_catalogue(&content_dir(), CHARACTER)
            .expect("split quest content loads");
        game
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn character_at(zone_id: u32, position: [f32; 3]) -> CharacterState {
    CharacterState {
        guid: CHARACTER,
        zone_id,
        position,
        rotation: [1.0, 0.0, 0.0],
    }
}

fn content_position(npc_id: u64) -> [f32; 3] {
    let document = read_json(&content_dir().join("npcs/sanctuary.json"));
    let row = document["records"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["npc_id"].as_u64() == Some(npc_id))
        .unwrap_or_else(|| panic!("NPC {npc_id} is defined in sanctuary.json"));
    serde_json::from_value(row["position"].clone()).expect("NPC has a position")
}

fn npc_character(npc_id: u64) -> CharacterState {
    character_at(SANCTUARY, content_position(npc_id))
}

fn npc_interact(target: u64) -> Vec<u8> {
    let mut packet = vec![26, 0, 8, 0];
    packet.extend(target.to_le_bytes());
    packet.push(0);
    packet
}

fn npc_select(target: u64, choice: i32) -> Vec<u8> {
    let mut packet = vec![26, 0, 10, 0];
    packet.extend(target.to_le_bytes());
    packet.extend(choice.to_le_bytes());
    packet
}

fn dialogue_choice(row: i32) -> Vec<u8> {
    let mut packet = vec![26, 0, 6, 0];
    packet.extend(row.to_le_bytes());
    packet
}

fn quest_reply(quest_id: u32, accept: bool) -> Vec<u8> {
    let mut packet = vec![49, 0, 2, 0, 0, 0];
    packet.extend(quest_id.to_le_bytes());
    packet.push(u8::from(accept));
    packet
}

fn quest_end_reply(quest_id: u32) -> Vec<u8> {
    let mut packet = vec![49, 0, 14, 0, 0, 0];
    packet.extend(quest_id.to_le_bytes());
    packet
}

fn select_job(profile: i32) -> Vec<u8> {
    let mut packet = vec![26, 0, 13, 0];
    packet.extend(profile.to_le_bytes());
    packet
}

fn send(game: &mut Gameplay, character: &CharacterState, packet: &[u8], now: u64) -> Outcome {
    game.handle_in_world(packet, character, now)
        .unwrap_or_else(|error| {
            panic!(
                "packet {:?} was refused: {error}",
                &packet[..packet.len().min(4)]
            )
        })
}

fn send_error(
    game: &mut Gameplay,
    character: &CharacterState,
    packet: &[u8],
    now: u64,
    label: &str,
) {
    let before = saved(game);
    let refused = game.handle_in_world(packet, character, now);
    assert!(refused.is_err(), "{label}: the request must be refused");
    assert_eq!(saved(game), before, "{label}: a refusal changed the save");
}

fn has_family(packet: &[u8], family: u16, subtype: u16, id: Option<u32>) -> bool {
    if packet.len() < 6
        || packet[..2] != family.to_le_bytes()
        || packet[2..4] != subtype.to_le_bytes()
    {
        return false;
    }
    let Some(id) = id else { return true };
    // 49/13 carries the turn-in speaker at 6..14 and the quest id at 14..18;
    // the other pairs used here carry their id at 6..10.
    let offset = if (family, subtype) == (49, 13) { 14 } else { 6 };
    packet
        .get(offset..offset + 4)
        .is_some_and(|bytes| u32::from_le_bytes(bytes.try_into().unwrap()) == id)
}

fn menu_option(outcome: &Outcome, label: u32) -> i32 {
    let menu = outcome
        .replies
        .iter()
        .find(|packet| packet.starts_with(&[26, 0, 9, 0]))
        .expect("26/8 returns the 26/9 interaction menu");
    let count = i32::from_le_bytes(menu[13..17].try_into().unwrap());
    assert!(count > 0, "the interaction menu has no options");
    (0..count)
        .find_map(|index| {
            let offset = 17 + usize::try_from(index).unwrap() * 28;
            let id = i32::from_le_bytes(menu[offset..offset + 4].try_into().unwrap());
            let string_id = u32::from_le_bytes(menu[offset + 8..offset + 12].try_into().unwrap());
            (string_id == label).then_some(id)
        })
        .unwrap_or_else(|| panic!("menu label {label} is absent"))
}

fn interact(game: &mut Gameplay, character: &CharacterState, target: u64, now: u64) -> Outcome {
    send(game, character, &npc_interact(target), now)
}

fn talk(game: &mut Gameplay, character: &CharacterState, target: u64, now: u64) -> Outcome {
    let menu = interact(game, character, target, now);
    let choice = menu_option(&menu, TALK_LABEL);
    send(game, character, &npc_select(target, choice), now + 1)
}

fn offer(
    game: &mut Gameplay,
    character: &CharacterState,
    target: u64,
    quest_id: u32,
    now: u64,
) -> Outcome {
    let dialogue = talk(game, character, target, now);
    assert!(
        dialogue
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[26, 0, 3, 0])),
        "Talk must open the 26/3 dialogue"
    );
    let offered = send(game, character, &dialogue_choice(QUEST_CHOICE), now + 1);
    assert!(
        offered
            .replies
            .iter()
            .any(|packet| has_family(packet, 49, 1, Some(quest_id))),
        "quest {quest_id} was not offered"
    );
    assert_eq!(quest_status(game, quest_id), QuestStatus::Offered);
    offered
}

fn accept(game: &mut Gameplay, character: &CharacterState, quest_id: u32, now: u64) -> Outcome {
    let accepted = send(game, character, &quest_reply(quest_id, true), now);
    assert!(
        accepted
            .replies
            .iter()
            .any(|packet| has_family(packet, 49, 3, Some(quest_id))),
        "quest {quest_id} did not send QuestAdd"
    );
    assert_eq!(quest_status(game, quest_id), QuestStatus::Active);
    accepted
}

/// 26/6 releases the 49/13 that the final Talk held. Its speaker must be the
/// runtime actor the client clicked.
fn release_end_screen(
    game: &mut Gameplay,
    character: &CharacterState,
    quest_id: u32,
    expected_actor: u64,
    now: u64,
) {
    let released = send(game, character, &dialogue_choice(QUEST_CHOICE), now);
    let end = released
        .replies
        .iter()
        .find(|packet| has_family(packet, 49, 13, Some(quest_id)))
        .unwrap_or_else(|| panic!("quest {quest_id} did not release its 49/13 end screen"));
    assert_eq!(
        u64::from_le_bytes(end[6..14].try_into().unwrap()),
        expected_actor,
        "the end screen must name the clicked runtime actor"
    );
}

fn finish_quest(
    game: &mut Gameplay,
    character: &CharacterState,
    quest_id: u32,
    expected_coins: u32,
    expected_xp: u32,
    now: u64,
) {
    let coins_before = coins(game);
    let xp_before = medic_xp(game);
    let completed = send(game, character, &quest_end_reply(quest_id), now);
    assert!(
        completed.changed,
        "quest {quest_id} turn-in did not change state"
    );
    assert!(
        completed
            .replies
            .iter()
            .any(|packet| has_family(packet, 49, 4, Some(quest_id))),
        "quest {quest_id} did not complete"
    );
    assert_eq!(coins(game), coins_before + expected_coins);
    assert_eq!(medic_xp(game), xp_before + expected_xp);
    assert_eq!(quest_status(game, quest_id), QuestStatus::Completed);
    assert_eq!(pending_turnin(game), None);
}

fn learn_medic(game: &mut Gameplay, character: &CharacterState, now: &mut u64) {
    *now += 10;
    let active_before = active_job(game);
    let menu = interact(game, character, NAIA, *now);
    let learn = menu_option(&menu, LEARN_LABEL);
    *now += 10;
    let learned = send(game, character, &npc_select(NAIA, learn), *now);
    assert!(learned.changed, "Naia's Learn choice must commit");
    assert!(
        game.state()
            .profiles
            .as_ref()
            .unwrap()
            .unlocked
            .contains(&MEDIC),
        "Naia's trainer row must unlock Medic"
    );
    assert_eq!(
        active_job(game),
        active_before,
        "learning a job must not switch to it"
    );
    *now += 10;
    let switched = send(game, character, &select_job(MEDIC), *now);
    assert!(
        switched.changed,
        "Medic selection must activate; its starter outfit has Chest and Legs"
    );
    assert_eq!(active_job(game), MEDIC, "the job gate needs Medic active");
}

fn start_exam(game: &mut Gameplay, character: &CharacterState, now: &mut u64) {
    *now += 10;
    offer(game, character, NAIA, EXAM, *now);
    *now += 10;
    accept(game, character, EXAM, *now);
}

fn complete_exam(game: &mut Gameplay, character: &CharacterState, now: &mut u64) {
    start_exam(game, character, now);
    assert_eq!(medic_xp(game), 0);
    *now += 10;
    let talked = talk(game, character, NAIA, *now);
    assert!(talked.changed, "the exam talk must complete the objective");
    assert_eq!(counts(game, EXAM), vec![1]);
    assert_eq!(pending_turnin(game), Some(EXAM));
    assert!(
        !talked
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[49, 0, 13, 0])),
        "the end screen is held for the 26/6 dialogue choice"
    );
    *now += 10;
    release_end_screen(game, character, EXAM, NAIA, *now);
    *now += 10;
    finish_quest(game, character, EXAM, EXAM_COINS, EXAM_XP, *now);
}

fn complete_house_call(game: &mut Gameplay, now: &mut u64) {
    let naia = npc_character(NAIA);
    let greyli = npc_character(GREYLI);
    *now += 10;
    offer(game, &naia, NAIA, HOUSE_CALL, *now);
    *now += 10;
    accept(game, &naia, HOUSE_CALL, *now);
    assert_eq!(medic_xp(game), EXAM_XP);

    *now += 10;
    let talked = talk(game, &greyli, GREYLI, *now);
    assert!(
        talked.changed,
        "the House Call talk must complete the objective"
    );
    assert_eq!(counts(game, HOUSE_CALL), vec![1]);
    assert_eq!(pending_turnin(game), Some(HOUSE_CALL));
    *now += 10;
    release_end_screen(game, &greyli, HOUSE_CALL, GREYLI, *now);
    *now += 10;
    finish_quest(game, &greyli, HOUSE_CALL, HOUSE_COINS, HOUSE_XP, *now);
}

fn complete_vitamins(game: &mut Gameplay, now: &mut u64) {
    let gordon = npc_character(GORDON);
    let greyli = npc_character(GREYLI);
    *now += 10;
    offer(game, &gordon, GORDON, VITAMINS, *now);
    *now += 10;
    accept(game, &gordon, VITAMINS, *now);
    assert_eq!(medic_xp(game), EXAM_XP + HOUSE_XP);

    *now += 10;
    let talked = talk(game, &greyli, GREYLI, *now);
    assert!(
        talked.changed,
        "the vitamins talk must complete the objective"
    );
    assert_eq!(counts(game, VITAMINS), vec![1]);
    assert_eq!(pending_turnin(game), Some(VITAMINS));
    *now += 10;
    release_end_screen(game, &greyli, VITAMINS, GREYLI, *now);
    *now += 10;
    finish_quest(game, &greyli, VITAMINS, VITAMIN_COINS, VITAMIN_XP, *now);
}

fn counts(game: &Gameplay, quest_id: u32) -> Vec<u32> {
    game.state().npc_quests.as_ref().unwrap().quests[&quest_id]
        .counts
        .clone()
}

fn quest_status(game: &Gameplay, quest_id: u32) -> QuestStatus {
    game.state().npc_quests.as_ref().unwrap().quests[&quest_id]
        .status
        .clone()
}

fn pending_turnin(game: &Gameplay) -> Option<u32> {
    game.state().npc_quests.as_ref().unwrap().pending_turnin
}

fn active_job(game: &Gameplay) -> i32 {
    game.state().profiles.as_ref().unwrap().active
}

fn coins(game: &Gameplay) -> u32 {
    game.state().inventory.coins
}

fn profile_xp(game: &Gameplay, profile: i32) -> u32 {
    game.state()
        .profiles
        .as_ref()
        .unwrap()
        .experience
        .get(&profile)
        .copied()
        .unwrap_or(0)
}

fn medic_xp(game: &Gameplay) -> u32 {
    profile_xp(game, MEDIC)
}

fn saved(game: &Gameplay) -> Vec<u8> {
    serde_json::to_vec(&game.state()).unwrap()
}

/// A Talk with nothing eligible must not offer, complete or mutate anything:
/// the 26/6 choice releases no offer or end screen, and the save bytes stay equal.
fn assert_no_offer(
    game: &mut Gameplay,
    character: &CharacterState,
    target: u64,
    now: &mut u64,
    label: &str,
) {
    let before = saved(game);
    *now += 10;
    let menu = interact(game, character, target, *now);
    let choice = menu_option(&menu, TALK_LABEL);
    *now += 1;
    let dialogue = send(game, character, &npc_select(target, choice), *now);
    assert!(
        !dialogue.changed,
        "{label}: a Talk with nothing eligible changed gameplay"
    );
    assert!(
        !dialogue
            .replies
            .iter()
            .any(|packet| has_family(packet, 49, 1, None)),
        "{label}: a Talk with nothing eligible offered a quest"
    );
    *now += 1;
    let closed = send(game, character, &dialogue_choice(QUEST_CHOICE), *now);
    assert!(
        !closed
            .replies
            .iter()
            .any(|packet| has_family(packet, 49, 1, None) || has_family(packet, 49, 13, None)),
        "{label}: no quest may be offered or turned in"
    );
    assert_eq!(
        saved(game),
        before,
        "{label}: the refusal changed the save bytes"
    );
}

fn assert_packet_changes_nothing(
    game: &mut Gameplay,
    character: &CharacterState,
    packet: &[u8],
    now: u64,
    label: &str,
) {
    let before = saved(game);
    let outcome = send(game, character, packet, now);
    assert!(!outcome.changed, "{label} changed gameplay");
    assert!(
        !outcome
            .replies
            .iter()
            .any(|reply| has_family(reply, 49, 3, None) || has_family(reply, 49, 4, None)),
        "{label} completed or added a quest"
    );
    assert_eq!(saved(game), before, "{label} changed the save bytes");
}

#[test]
fn naia_offers_the_medic_exam_only_to_medics() {
    let fixture = Fixture::new();
    let mut game = fixture.game(None);
    let mut now = 1_000;
    let naia = npc_character(NAIA);

    // Brawler active: the job 11 gate must hide the exam.
    assert_ne!(active_job(&game), BRAWLER);
    let switched = send(&mut game, &naia, &select_job(BRAWLER), now);
    assert!(switched.changed, "Brawler must be selectable");
    assert_eq!(active_job(&game), BRAWLER);
    assert_no_offer(&mut game, &naia, NAIA, &mut now, "Brawler at Naia");
    assert!(
        !game
            .state()
            .npc_quests
            .as_ref()
            .unwrap()
            .quests
            .contains_key(&EXAM),
        "a Brawler must not receive the Medic exam"
    );

    // A forged 49/2 accept for an offer the client never received changes nothing.
    let forged = send(&mut game, &naia, &quest_reply(EXAM, true), now + 1);
    assert!(!forged.changed);
    assert!(
        forged
            .replies
            .iter()
            .all(|packet| !has_family(packet, 49, 3, Some(EXAM)))
    );
    assert!(
        !game
            .state()
            .npc_quests
            .as_ref()
            .unwrap()
            .quests
            .contains_key(&EXAM)
    );

    // The ordinary NPC menu teaches and activates Medic.
    now += 10;
    learn_medic(&mut game, &naia, &mut now);
    offer(&mut game, &naia, NAIA, EXAM, now + 10);

    // House Call stays behind the exam prerequisite.
    assert!(
        !game
            .state()
            .npc_quests
            .as_ref()
            .unwrap()
            .quests
            .contains_key(&HOUSE_CALL),
        "House Call must wait for the exam"
    );
}

#[test]
fn exam_and_house_call_complete_in_order() {
    let fixture = Fixture::new();
    let mut game = fixture.game(None);
    let mut now = 2_000;
    let naia = npc_character(NAIA);
    let greyli = npc_character(GREYLI);
    learn_medic(&mut game, &naia, &mut now);
    let coins_before = coins(&game);

    start_exam(&mut game, &naia, &mut now);
    assert_eq!(counts(&game, EXAM), vec![0]);
    assert_eq!(pending_turnin(&game), None);
    assert!(
        !game
            .state()
            .npc_quests
            .as_ref()
            .unwrap()
            .quests
            .contains_key(&HOUSE_CALL),
        "House Call must not be offered while the exam runs"
    );
    assert_eq!(coins(&game), coins_before, "no reward before the turn-in");

    // The exam talk then turn-in.
    now += 10;
    let talked = talk(&mut game, &naia, NAIA, now);
    assert!(talked.changed);
    assert_eq!(counts(&game, EXAM), vec![1]);
    assert_eq!(pending_turnin(&game), Some(EXAM));
    assert_eq!(coins(&game), coins_before);
    now += 10;
    release_end_screen(&mut game, &naia, EXAM, NAIA, now);
    now += 10;
    finish_quest(&mut game, &naia, EXAM, EXAM_COINS, EXAM_XP, now);
    assert_eq!(coins(&game), coins_before + EXAM_COINS);
    assert_eq!(medic_xp(&game), EXAM_XP);

    // House Call becomes offered; the objective is at Greyli.
    now += 10;
    offer(&mut game, &naia, NAIA, HOUSE_CALL, now);
    now += 10;
    accept(&mut game, &naia, HOUSE_CALL, now);
    now += 10;
    let visited = talk(&mut game, &greyli, GREYLI, now);
    assert!(visited.changed);
    assert_eq!(counts(&game, HOUSE_CALL), vec![1]);
    assert_eq!(pending_turnin(&game), Some(HOUSE_CALL));
    assert_eq!(coins(&game), coins_before + EXAM_COINS);
    now += 10;
    release_end_screen(&mut game, &greyli, HOUSE_CALL, GREYLI, now);
    now += 10;
    finish_quest(&mut game, &greyli, HOUSE_CALL, HOUSE_COINS, HOUSE_XP, now);

    assert_eq!(coins(&game), coins_before + EXAM_COINS + HOUSE_COINS);
    assert_eq!(medic_xp(&game), EXAM_XP + HOUSE_XP);

    // A duplicated 49/14 pays nothing.
    now += 10;
    assert_packet_changes_nothing(
        &mut game,
        &greyli,
        &quest_end_reply(HOUSE_CALL),
        now,
        "duplicate House Call turn-in",
    );
    assert_eq!(coins(&game), coins_before + EXAM_COINS + HOUSE_COINS);
    assert_eq!(medic_xp(&game), EXAM_XP + HOUSE_XP);
}

#[test]
fn take_your_vitamins_requires_house_call() {
    let fixture = Fixture::new();
    let mut game = fixture.game(None);
    let mut now = 3_000;
    let naia = npc_character(NAIA);
    let gordon = npc_character(GORDON);
    let greyli = npc_character(GREYLI);
    learn_medic(&mut game, &naia, &mut now);
    let coins_before = coins(&game);

    complete_exam(&mut game, &naia, &mut now);
    assert_eq!(coins(&game), coins_before + EXAM_COINS);

    // The prerequisite is missing: Gordon offers nothing.
    assert_no_offer(
        &mut game,
        &gordon,
        GORDON,
        &mut now,
        "Take Your Vitamins before House Call",
    );
    assert!(
        !game
            .state()
            .npc_quests
            .as_ref()
            .unwrap()
            .quests
            .contains_key(&VITAMINS)
    );

    complete_house_call(&mut game, &mut now);
    let coins_after_house = coins(&game);
    let xp_after_house = medic_xp(&game);
    assert_eq!(coins_after_house, coins_before + EXAM_COINS + HOUSE_COINS);

    // Naia is not the vitamins giver: her menu offers nothing for the chapter.
    assert_no_offer(
        &mut game,
        &naia,
        NAIA,
        &mut now,
        "Take Your Vitamins at the wrong giver",
    );

    // The completed House Call unlocks the vitamins at Doctor Gordon.
    now += 10;
    offer(&mut game, &gordon, GORDON, VITAMINS, now);
    now += 10;
    accept(&mut game, &gordon, VITAMINS, now);

    // The client's only recovered step is the talk at Greyli; that Talk opens the
    // end screen there and the 49/14 settles the Gordon-issued quest.
    now += 10;
    let talked = talk(&mut game, &greyli, GREYLI, now);
    assert!(talked.changed);
    assert_eq!(counts(&game, VITAMINS), vec![1]);
    assert_eq!(pending_turnin(&game), Some(VITAMINS));
    assert_eq!(
        coins(&game),
        coins_after_house,
        "no reward before the turn-in"
    );
    now += 10;
    release_end_screen(&mut game, &greyli, VITAMINS, GREYLI, now);
    now += 10;
    finish_quest(&mut game, &greyli, VITAMINS, VITAMIN_COINS, VITAMIN_XP, now);

    assert_eq!(
        coins(&game),
        coins_before + EXAM_COINS + HOUSE_COINS + VITAMIN_COINS
    );
    assert_eq!(medic_xp(&game), xp_after_house + VITAMIN_XP);

    // Paid exactly once: a replayed 49/14 changes nothing.
    now += 10;
    assert_packet_changes_nothing(
        &mut game,
        &greyli,
        &quest_end_reply(VITAMINS),
        now,
        "duplicate Take Your Vitamins turn-in",
    );
    assert_eq!(
        coins(&game),
        coins_before + EXAM_COINS + HOUSE_COINS + VITAMIN_COINS
    );
    assert_eq!(medic_xp(&game), xp_after_house + VITAMIN_XP);
}

#[test]
fn medic_xp_goes_to_medic_not_the_active_job() {
    let fixture = Fixture::new();
    let mut game = fixture.game(None);
    let mut now = 4_000;
    let naia = npc_character(NAIA);
    let gordon = npc_character(GORDON);
    let greyli = npc_character(GREYLI);
    learn_medic(&mut game, &naia, &mut now);
    complete_exam(&mut game, &naia, &mut now);
    complete_house_call(&mut game, &mut now);
    let medic_before = medic_xp(&game);
    assert_eq!(medic_before, EXAM_XP + HOUSE_XP);

    now += 10;
    offer(&mut game, &gordon, GORDON, VITAMINS, now);
    now += 10;
    accept(&mut game, &gordon, VITAMINS, now);

    // Switch away before the turn-in: the accepted quest keeps its own reward map.
    now += 10;
    let brawler_xp_before = profile_xp(&game, BRAWLER);
    let switched = send(&mut game, &greyli, &select_job(BRAWLER), now);
    assert!(switched.changed, "Brawler must be selectable again");
    assert_eq!(active_job(&game), BRAWLER);

    now += 10;
    let talked = talk(&mut game, &greyli, GREYLI, now);
    assert!(talked.changed);
    now += 10;
    release_end_screen(&mut game, &greyli, VITAMINS, GREYLI, now);
    now += 10;
    finish_quest(&mut game, &greyli, VITAMINS, VITAMIN_COINS, VITAMIN_XP, now);

    assert_eq!(
        active_job(&game),
        BRAWLER,
        "the turn-in must not switch jobs"
    );
    assert_eq!(
        medic_xp(&game),
        medic_before + VITAMIN_XP,
        "profile 11 must receive the Medic XP"
    );
    assert_eq!(
        profile_xp(&game, BRAWLER),
        brawler_xp_before,
        "the active job must not receive the Medic XP"
    );
}

#[test]
fn refusals_and_replay_change_nothing() {
    let fixture = Fixture::new();
    let mut game = fixture.game(None);
    let mut now = 5_000;
    let naia = npc_character(NAIA);
    let greyli = npc_character(GREYLI);
    learn_medic(&mut game, &naia, &mut now);

    // Wrong talk target: Greyli cannot advance the Naia exam objective.
    start_exam(&mut game, &naia, &mut now);
    let before = saved(&game);
    let wrong_npc = talk(&mut game, &greyli, GREYLI, now + 1);
    assert!(!wrong_npc.changed, "a wrong NPC must not advance the exam");
    assert_eq!(counts(&game, EXAM), vec![0]);
    assert_eq!(saved(&game), before, "the wrong NPC changed the save");

    // Forged accept for a quest offered by another giver changes nothing.
    assert_packet_changes_nothing(
        &mut game,
        &naia,
        &quest_reply(HOUSE_CALL, true),
        now + 2,
        "forged House Call accept",
    );
    assert_packet_changes_nothing(
        &mut game,
        &naia,
        &quest_reply(VITAMINS, true),
        now + 3,
        "forged Take Your Vitamins accept",
    );

    // Out of range: the menu and a formerly offered choice are both refused.
    let far = character_at(
        SANCTUARY,
        [naia.position[0] + 400.0, naia.position[1], naia.position[2]],
    );
    send_error(
        &mut game,
        &far,
        &npc_interact(NAIA),
        now + 4,
        "far interact",
    );
    now += 10;
    let menu = interact(&mut game, &naia, NAIA, now);
    let talk_choice = menu_option(&menu, TALK_LABEL);
    let offered = send(&mut game, &naia, &npc_select(NAIA, talk_choice), now + 1);
    assert!(
        offered.changed,
        "the exam objective is completed by the Naia talk"
    );
    assert_eq!(pending_turnin(&game), Some(EXAM));
    let held_bytes = saved(&game);
    send_error(
        &mut game,
        &far,
        &npc_select(NAIA, talk_choice),
        now + 2,
        "far menu select",
    );
    assert_eq!(
        saved(&game),
        held_bytes,
        "the refused choice changed the save"
    );

    // A replayed Talk at the same actor only re-sends the end screen.
    let replayed = send(&mut game, &naia, &npc_select(NAIA, talk_choice), now + 3);
    assert!(!replayed.changed, "a replayed Talk changed gameplay");
    assert_eq!(saved(&game), held_bytes, "a replayed Talk changed the save");
    assert_eq!(pending_turnin(&game), Some(EXAM));
    assert_eq!(counts(&game, EXAM), vec![1]);

    // Release and finish, then refuse every old packet again.
    now += 10;
    release_end_screen(&mut game, &naia, EXAM, NAIA, now);
    now += 10;
    finish_quest(&mut game, &naia, EXAM, EXAM_COINS, EXAM_XP, now);
    let finished = saved(&game);
    let finished_coins = coins(&game);
    let finished_xp = medic_xp(&game);
    for (label, packet) in [
        ("replayed exam end reply", quest_end_reply(EXAM)),
        ("replayed exam accept", quest_reply(EXAM, true)),
    ] {
        let outcome = send(&mut game, &naia, &packet, now + 10);
        assert!(!outcome.changed, "{label} changed gameplay");
        assert!(
            !outcome
                .replies
                .iter()
                .any(|reply| has_family(reply, 49, 3, None) || has_family(reply, 49, 4, None)),
            "{label} added or completed a quest"
        );
    }
    assert_eq!(saved(&game), finished, "old packets changed the save");
    assert_eq!(coins(&game), finished_coins);
    assert_eq!(medic_xp(&game), finished_xp);

    // A Talk at an unrelated NPC offers nothing and changes nothing.
    assert_no_offer(
        &mut game,
        &greyli,
        GREYLI,
        &mut now,
        "exam after completion",
    );

    // Naia's next chapter step is offered; the completed exam never is.
    now += 10;
    let menu = interact(&mut game, &naia, NAIA, now);
    let choice = menu_option(&menu, TALK_LABEL);
    let dialogue = send(&mut game, &naia, &npc_select(NAIA, choice), now + 1);
    assert!(
        dialogue
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[26, 0, 3, 0])),
        "Naia must open a dialogue for the next chapter step"
    );
    let offered = send(&mut game, &naia, &dialogue_choice(QUEST_CHOICE), now + 2);
    assert!(
        !offered
            .replies
            .iter()
            .any(|packet| has_family(packet, 49, 1, Some(EXAM))),
        "the completed exam must not be offered again"
    );
    assert!(
        offered
            .replies
            .iter()
            .any(|packet| has_family(packet, 49, 1, Some(HOUSE_CALL))),
        "House Call must follow the completed exam"
    );
}

#[test]
fn relog_preserves_completed_history() {
    let fixture = Fixture::new();
    let mut game = fixture.game(None);
    let mut now = 6_000;
    let naia = npc_character(NAIA);
    let gordon = npc_character(GORDON);
    learn_medic(&mut game, &naia, &mut now);
    complete_exam(&mut game, &naia, &mut now);
    complete_house_call(&mut game, &mut now);
    complete_vitamins(&mut game, &mut now);
    let final_bytes = saved(&game);
    let final_coins = coins(&game);
    let final_xp = medic_xp(&game);
    assert_eq!(
        final_coins,
        EXAM_COINS + HOUSE_COINS + VITAMIN_COINS,
        "the three chapter rewards"
    );
    assert_eq!(final_xp, EXAM_XP + HOUSE_XP + VITAMIN_XP);

    // Disconnect and reconnect with the saved game state.
    game = fixture.game(Some(game.state()));
    assert_eq!(saved(&game), final_bytes, "reload changed the save bytes");
    for quest_id in [EXAM, HOUSE_CALL, VITAMINS] {
        assert_eq!(
            quest_status(&game, quest_id),
            QuestStatus::Completed,
            "quest {quest_id} history"
        );
    }
    assert_eq!(pending_turnin(&game), None);
    assert_eq!(coins(&game), final_coins);
    assert_eq!(medic_xp(&game), final_xp);

    // None is offered again.
    assert_no_offer(
        &mut game,
        &naia,
        NAIA,
        &mut now,
        "completed exam after relog",
    );
    assert_no_offer(
        &mut game,
        &gordon,
        GORDON,
        &mut now,
        "completed vitamins after relog",
    );

    // Old packets pay nothing after the reload.
    for quest_id in [EXAM, HOUSE_CALL, VITAMINS] {
        let greyli = npc_character(GREYLI);
        assert_packet_changes_nothing(
            &mut game,
            &greyli,
            &quest_end_reply(quest_id),
            now + 1,
            "replayed turn-in after relog",
        );
        assert_packet_changes_nothing(
            &mut game,
            &greyli,
            &quest_reply(quest_id, true),
            now + 2,
            "replayed accept after relog",
        );
        now += 10;
    }
    assert_eq!(coins(&game), final_coins);
    assert_eq!(medic_xp(&game), final_xp);
}

#[test]
fn two_medics_progress_independently() {
    let fixture = Fixture::new();
    let mut first = fixture.game(None);
    let mut second = fixture.game(None);
    let mut now = 7_000;
    let naia = npc_character(NAIA);
    learn_medic(&mut first, &naia, &mut now);
    learn_medic(&mut second, &naia, &mut now);
    start_exam(&mut first, &naia, &mut now);
    start_exam(&mut second, &naia, &mut now);

    for game in [&mut first, &mut second] {
        let talked = talk(game, &naia, NAIA, now);
        assert!(talked.changed);
        assert_eq!(counts(game, EXAM), vec![1]);
        assert_eq!(pending_turnin(game), Some(EXAM));
    }
    assert_eq!(counts(&first, EXAM), vec![1]);
    assert_eq!(counts(&second, EXAM), vec![1]);
    assert_eq!(coins(&first), coins(&second));
}

#[test]
fn enabled_medic_chapter_records_are_complete_and_others_stay_disabled() {
    let document = read_json(&content_dir().join("quests/medic.json"));
    let records = document["records"].as_array().unwrap();
    let enabled: Vec<&Value> = records
        .iter()
        .filter(|record| record["enabled"].as_bool().unwrap_or(false))
        .collect();
    let enabled_ids: Vec<u64> = enabled
        .iter()
        .map(|record| record["id"].as_u64().unwrap())
        .collect();
    assert_eq!(
        enabled_ids,
        vec![u64::from(EXAM), u64::from(HOUSE_CALL), u64::from(VITAMINS)]
    );

    // Every enabled record has a resolved giver in spawned NPC content, a job
    // gate, at least one sourced objective and one sourced reward.
    let mut npc_ids = std::collections::BTreeSet::new();
    for entry in fs::read_dir(content_dir().join("npcs")).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }
        let npcs = read_json(&path);
        for row in npcs["records"].as_array().unwrap() {
            let id = row["npc_id"].as_u64().unwrap();
            npc_ids.insert(id);
            if [NAIA, GORDON, GREYLI].contains(&id) {
                assert_eq!(
                    row["placement"].as_str(),
                    Some("legacy_data"),
                    "NPC {id} must be a spawned Legacy row"
                );
                assert_eq!(row["zone_id"].as_u64(), Some(u64::from(SANCTUARY)));
                assert!(row["position"].is_array());
                assert!(
                    row["roles"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .any(|role| role == "quest_giver"),
                    "NPC {id} must be a quest giver"
                );
                assert!(
                    row["model_id"].as_u64().is_some_and(|model| model > 0),
                    "NPC {id} must have a renderable Legacy model"
                );
                assert!(
                    row["heading"].as_f64().is_some_and(f64::is_finite),
                    "NPC {id} must have a Legacy heading"
                );
                if id == GORDON {
                    assert_eq!(
                        row["model_file_name"].as_str(),
                        Some("human_m_doctor_gordon.agr")
                    );
                    assert_eq!(row["name_string_id"].as_u64(), Some(7_610));
                }
            }
        }
    }
    for record in &enabled {
        let quest_id = record["id"].as_u64().unwrap();
        let giver = record["giver_npc_id"].as_u64().unwrap();
        assert!(
            npc_ids.contains(&giver),
            "quest {quest_id}: giver {giver} is not a spawned NPC"
        );
        assert_eq!(record["job"].as_i64(), Some(i64::from(MEDIC)));
        let objectives = record["objectives"].as_array().unwrap();
        assert!(!objectives.is_empty(), "quest {quest_id} needs objectives");
        assert!(
            objectives.iter().all(|goal| goal["source"]
                .as_array()
                .is_some_and(|source| !source.is_empty())),
            "quest {quest_id}: every objective needs provenance"
        );
        let rewards = record["rewards"].as_array().unwrap();
        assert!(!rewards.is_empty(), "quest {quest_id} must pay");
        assert!(
            rewards.iter().all(|reward| reward["source"]
                .as_array()
                .is_some_and(|source| !source.is_empty())),
            "quest {quest_id}: every reward needs provenance"
        );
        assert!(
            rewards
                .iter()
                .all(|reward| reward["job_xp"]["11"].as_u64().is_some_and(|xp| xp > 0)),
            "quest {quest_id}: the reward must name Medic (profile 11) XP"
        );
        assert!(
            rewards.iter().all(|reward| reward["items"]
                .as_array()
                .is_some_and(|items| items.is_empty())),
            "quest {quest_id}: no invented item is granted"
        );
    }

    let exam = records
        .iter()
        .find(|record| record["id"].as_u64() == Some(u64::from(EXAM)))
        .unwrap();
    assert_eq!(exam["title_string_id"].as_u64(), Some(23_048));
    assert_eq!(exam["giver_npc_id"].as_u64(), Some(NAIA));
    assert_eq!(exam["prerequisite_quest_ids"], serde_json::json!([]));
    assert_eq!(exam["objectives"][0]["kind"], "talk_to_npc");
    assert_eq!(exam["objectives"][0]["npc_id"].as_u64(), Some(NAIA));
    assert_eq!(
        exam["rewards"][0]["coins"].as_u64(),
        Some(u64::from(EXAM_COINS))
    );
    assert_eq!(
        exam["rewards"][0]["job_xp"]["11"].as_u64(),
        Some(u64::from(EXAM_XP))
    );

    let house = records
        .iter()
        .find(|record| record["id"].as_u64() == Some(u64::from(HOUSE_CALL)))
        .unwrap();
    assert_eq!(house["title_string_id"].as_u64(), Some(22_989));
    assert_eq!(
        house["giver_npc_id"].as_u64(),
        Some(NAIA),
        "House Call's giver is Nurse Naia, not the unspawned 9000000419"
    );
    assert_eq!(house["prerequisite_quest_ids"], serde_json::json!([EXAM]));
    assert_eq!(house["objectives"][0]["npc_id"].as_u64(), Some(GREYLI));

    let vitamins = records
        .iter()
        .find(|record| record["id"].as_u64() == Some(u64::from(VITAMINS)))
        .unwrap();
    assert_eq!(vitamins["title_string_id"].as_u64(), Some(23_019));
    assert_eq!(vitamins["giver_npc_id"].as_u64(), Some(GORDON));
    assert_eq!(
        vitamins["prerequisite_quest_ids"],
        serde_json::json!([HOUSE_CALL])
    );
    assert_eq!(vitamins["objectives"][0]["npc_id"].as_u64(), Some(GREYLI));
    assert_eq!(
        vitamins["rewards"][0]["coins"].as_u64(),
        Some(u64::from(VITAMIN_COINS))
    );
    assert_eq!(
        vitamins["rewards"][0]["job_xp"]["11"].as_u64(),
        Some(u64::from(VITAMIN_XP))
    );

    // The Bixie Hive and collection steps stay disabled.
    for quest_id in [3_090_002_u64, 3_090_004] {
        let record = records
            .iter()
            .find(|record| record["id"].as_u64() == Some(quest_id))
            .unwrap();
        assert_eq!(record["enabled"].as_bool(), Some(false));
    }

    // Naia's trainer row teaches profile 11 through the ordinary menu.
    let trainers = read_json(&content_dir().join("job_trainers.json"));
    assert!(
        trainers["trainers"].as_array().unwrap().iter().any(|row| {
            row["npc"].as_u64() == Some(NAIA)
                && row["profile"].as_i64() == Some(i64::from(MEDIC))
                && row["label_string_id"].as_u64() == Some(u64::from(LEARN_LABEL))
        }),
        "Naia's trainer row drives the Learn menu option"
    );
}
