//! T194: Archer entry chapter, Path of the Archer (3010003) and Oath of Honor
//! (3010006). The chapter is content-only; these tests drive the production NPC
//! packet router (`Gameplay::handle_in_world` -> `gameplay/npc.rs` ->
//! `QuestEngine`) instead of calling engine internals.
//!
//! Request/reply layouts used here:
//! - 26/8 NPC interact: constructor `008b6bc0`; body is u64 target + u8 [0].
//! - 26/10 menu select: constructor `00a36f80`; body is u64 target + i32 choice.
//! - 26/6 dialogue response: reader `009f5890`; body is i32 response row
//!   (301 is the quest row).
//! - 49/2 quest reply: reader `00a86400` / sender `00a97d70`; u32 quest + u8 accept.
//! - 49/14 end reply: constructor `00a864c0` / sender `00a97ef0`; u32 quest.
//! 49/1 offers and 49/13 end screens are parsed only for their quest id, at
//! 6..10 and 14..18 respectively, matching the existing route tests.
//!
//! The three recruit talk targets are existing unnamed Legacy Greenwood archer
//! actors (spawn GUIDs 100000003468-3470) bound to the recruit content keys in
//! `npc_spawns_extra.json`; their regional rows are unanchored scaffolding and
//! deliberately do not spawn a duplicate.

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

const CHARACTER: u64 = 194_017;
const LUGABOW: u64 = 9_000_000_387;
const BOLLYLOG: u64 = 9_500_000_289;
const HRUGAGU: u64 = 9_500_000_309;
const BRUGADUG: u64 = 9_500_000_290;
const HILFORN: u64 = 9_000_000_106;
const JONES: u64 = 9_000_000_121;
const BARTLE: u64 = 9_000_000_021;
const PATH: u32 = 3_010_003;
const OATH: u32 = 3_010_006;
const ARCHER: i32 = 35;
const BRAWLER: i32 = 43;
/// `job_trainers.json` row for Lugabow (profile 35).
const LEARN_LABEL: u32 = 412_187;
/// Generic Talk option label in the reconstructed 26/9 menu.
const TALK_LABEL: u32 = 3_300;
/// First response row of the quest dialogue (client 26/6).
const QUEST_CHOICE: i32 = 301;
const PATH_COINS: u32 = 5;
const PATH_XP: u32 = 15;
const OATH_COINS: u32 = 10;
const OATH_XP: u32 = 20;
const GREENWOOD: u32 = 6;
const COBBLESTONE: u32 = 3;

static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(0);

fn content_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content")
}

fn reference_content() -> PathBuf {
    std::env::var_os("OSFR_ARCHER_ENTRY_CONTENT")
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

/// The opening fixture supplies profile/inventory content; the repository's
/// `job_trainers.json`, `npcs/`, and split quests are bound from `data/content`
/// exactly as the bundled play package does.
struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        let source = reference_content();
        assert!(
            source.join("profiles.json").is_file() && source.join("inventory.json").is_file(),
            "the archer chapter needs the opening content fixture at {}",
            source.display()
        );
        let directory = std::env::temp_dir().join(format!(
            "osfr-archer-entry-{}-{}",
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

fn content_position(file: &str, npc_id: u64) -> [f32; 3] {
    let document = read_json(&content_dir().join("npcs").join(file));
    let row = document["records"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["npc_id"].as_u64() == Some(npc_id))
        .unwrap_or_else(|| panic!("NPC {npc_id} is defined in {file}"));
    serde_json::from_value(row["position"].clone()).expect("NPC has a position")
}

/// One runtime actor bound to a quest content key. The binding is the only way a
/// client click on an extra-spawn actor reaches the recruit's content row.
fn recruit_spawn(content_key: u64) -> (u64, [f32; 3]) {
    let document = read_json(&content_dir().join("npc_spawns_extra.json"));
    let rows: Vec<&Value> = document["records"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|row| row["content_key"].as_u64() == Some(content_key))
        .collect();
    assert_eq!(
        rows.len(),
        1,
        "content key {content_key} must bind exactly one runtime actor"
    );
    let row = rows[0];
    (
        row["spawn_guid"].as_u64().expect("bound spawn GUID"),
        serde_json::from_value(row["position"].clone()).expect("bound spawn position"),
    )
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

fn has_family(packet: &[u8], family: u16, subtype: u16, id: Option<u32>) -> bool {
    if packet.len() < 6
        || packet[..2] != family.to_le_bytes()
        || packet[2..4] != subtype.to_le_bytes()
    {
        return false;
    }
    let Some(id) = id else { return true };
    // 49/13 carries the turn-in speaker at 6..14 and the reply quest id at 14..18;
    // the other family/subtype pairs used here carry their id at 6..10.
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
    let offered = send(game, character, &dialogue_choice(QUEST_CHOICE), now + 2);
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
/// runtime actor the client clicked, not the content key.
fn release_end_screen(
    game: &mut Gameplay,
    character: &CharacterState,
    quest_id: u32,
    expected_actor: u64,
    now: u64,
) -> Vec<u8> {
    let released = send(game, character, &dialogue_choice(QUEST_CHOICE), now);
    let end = released
        .replies
        .iter()
        .find(|packet| has_family(packet, 49, 13, Some(quest_id)))
        .cloned()
        .unwrap_or_else(|| panic!("quest {quest_id} did not release its 49/13 end screen"));
    assert_eq!(
        u64::from_le_bytes(end[6..14].try_into().unwrap()),
        expected_actor,
        "the end screen must name the clicked runtime actor"
    );
    end
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
    let xp_before = archer_xp(game);
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
    assert_eq!(archer_xp(game), xp_before + expected_xp);
    assert_eq!(quest_status(game, quest_id), QuestStatus::Completed);
    assert_eq!(pending_turnin(game), None);
}

fn learn_archer(game: &mut Gameplay, character: &CharacterState, now: &mut u64) {
    *now += 10;
    let menu = interact(game, character, LUGABOW, *now);
    let learn = menu_option(&menu, LEARN_LABEL);
    *now += 10;
    let learned = send(game, character, &npc_select(LUGABOW, learn), *now);
    assert!(learned.changed, "Lugabow's Learn choice must commit");
    assert!(
        game.state()
            .profiles
            .as_ref()
            .unwrap()
            .unlocked
            .contains(&ARCHER),
        "Lugabow must unlock Archer"
    );
    *now += 10;
    let switched = send(game, character, &select_job(ARCHER), *now);
    assert!(
        switched.changed,
        "Archer selection must activate; its starter outfit must be complete"
    );
    assert_eq!(
        game.state().profiles.as_ref().unwrap().active,
        ARCHER,
        "the job gate needs Archer active"
    );
}

fn activate_brawler(game: &mut Gameplay, character: &CharacterState, now: &mut u64) {
    if game.state().profiles.as_ref().unwrap().active == BRAWLER {
        return;
    }
    *now += 10;
    let switched = send(game, character, &select_job(BRAWLER), *now);
    assert!(switched.changed, "Brawler must activate");
    assert_eq!(game.state().profiles.as_ref().unwrap().active, BRAWLER);
}

fn start_path(game: &mut Gameplay, character: &CharacterState, now: &mut u64) {
    *now += 10;
    offer(game, character, LUGABOW, PATH, *now);
    *now += 10;
    accept(game, character, PATH, *now);
}

fn talk_recruit(game: &mut Gameplay, content_key: u64, now: &mut u64) -> (Outcome, u64, [f32; 3]) {
    let (runtime_guid, position) = recruit_spawn(content_key);
    let character = character_at(GREENWOOD, position);
    *now += 10;
    let outcome = talk(game, &character, runtime_guid, *now);
    (outcome, runtime_guid, position)
}

fn complete_path_via_recruits(game: &mut Gameplay, now: &mut u64) {
    assert_eq!(quest_status(game, PATH), QuestStatus::Active);
    let before = counts(game, PATH);
    if before[0] == 0 {
        let (first, _, _) = talk_recruit(game, BOLLYLOG, now);
        assert!(first.changed);
        assert_eq!(counts(game, PATH), vec![1, 0, 0]);
        assert_eq!(pending_turnin(game), None);
    }
    if before[1] == 0 {
        let (second, _, _) = talk_recruit(game, HRUGAGU, now);
        assert!(second.changed);
        assert_eq!(counts(game, PATH), vec![1, 1, 0]);
        assert_eq!(pending_turnin(game), None);
    }

    let (third, runtime_guid, position) = talk_recruit(game, BRUGADUG, now);
    assert!(third.changed);
    assert_eq!(counts(game, PATH), vec![1, 1, 1]);
    assert_eq!(pending_turnin(game), Some(PATH));
    assert!(
        !third
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[49, 0, 13, 0])),
        "the turn-in screen waits for the 26/6 dialogue choice"
    );

    let character = character_at(GREENWOOD, position);
    *now += 10;
    let _ = release_end_screen(game, &character, PATH, runtime_guid, *now);
    *now += 10;
    finish_quest(game, &character, PATH, PATH_COINS, PATH_XP, *now);
}

fn complete_oath_via_hilforn(game: &mut Gameplay, now: &mut u64) {
    let position = content_position("greenwood_forest.json", HILFORN);
    let character = character_at(GREENWOOD, position);
    *now += 10;
    let reported = talk(game, &character, HILFORN, *now);
    assert!(reported.changed);
    assert_eq!(counts(game, OATH), vec![1]);
    assert_eq!(pending_turnin(game), Some(OATH));
    assert!(
        !reported
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[49, 0, 13, 0])),
        "the turn-in screen waits for the 26/6 dialogue choice"
    );

    *now += 10;
    let _ = release_end_screen(game, &character, OATH, HILFORN, *now);
    *now += 10;
    finish_quest(game, &character, OATH, OATH_COINS, OATH_XP, *now);
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

fn archer_xp(game: &Gameplay) -> u32 {
    profile_xp(game, ARCHER)
}

fn saved(game: &Gameplay) -> Vec<u8> {
    serde_json::to_vec(&game.state()).unwrap()
}

fn assert_no_offer(game: &mut Gameplay, character: &CharacterState, target: u64, now: &mut u64) {
    let before = saved(game);
    *now += 10;
    let dialogue = talk(game, character, target, *now);
    assert!(
        !dialogue.changed,
        "a Talk with nothing eligible must not change gameplay"
    );
    *now += 10;
    let closed = send(game, character, &dialogue_choice(QUEST_CHOICE), *now);
    assert!(
        !closed
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[49, 0, 1, 0])),
        "no quest may be offered"
    );
    assert_eq!(saved(game), before, "the refusal changed the save bytes");
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
    assert_eq!(saved(game), before, "{label} changed the save bytes");
}

#[test]
fn lugabow_offers_path_only_to_archers() {
    let fixture = Fixture::new();
    let mut game = fixture.game(None);
    let mut now = 1_000;
    let lugabow = content_position("greenwood_forest.json", LUGABOW);
    let character = character_at(GREENWOOD, lugabow);

    activate_brawler(&mut game, &character, &mut now);
    assert_no_offer(&mut game, &character, LUGABOW, &mut now);

    learn_archer(&mut game, &character, &mut now);
    offer(&mut game, &character, LUGABOW, PATH, now);
    assert!(
        !game
            .state()
            .npc_quests
            .as_ref()
            .unwrap()
            .quests
            .contains_key(&OATH),
        "Oath of Honor stays locked behind Path of the Archer"
    );
}

#[test]
fn path_chapter_completes_in_order_and_pays_once() {
    let fixture = Fixture::new();
    let mut game = fixture.game(None);
    let mut now = 2_000;
    let lugabow = content_position("greenwood_forest.json", LUGABOW);
    let character = character_at(GREENWOOD, lugabow);
    learn_archer(&mut game, &character, &mut now);
    start_path(&mut game, &character, &mut now);
    let coins_before = coins(&game);
    let xp_before = archer_xp(&game);

    // Ordered: the second recruit cannot count before the first.
    let before_wrong_order = saved(&game);
    let (out_of_order, _, _) = talk_recruit(&mut game, HRUGAGU, &mut now);
    assert!(!out_of_order.changed);
    assert_eq!(counts(&game, PATH), vec![0, 0, 0]);
    assert_eq!(saved(&game), before_wrong_order);

    let (first, _, _) = talk_recruit(&mut game, BOLLYLOG, &mut now);
    assert!(first.changed);
    assert_eq!(counts(&game, PATH), vec![1, 0, 0]);
    assert_eq!(pending_turnin(&game), None);

    let (second, _, _) = talk_recruit(&mut game, HRUGAGU, &mut now);
    assert!(second.changed);
    assert_eq!(counts(&game, PATH), vec![1, 1, 0]);
    assert_eq!(pending_turnin(&game), None);

    let (third, runtime_guid, position) = talk_recruit(&mut game, BRUGADUG, &mut now);
    assert!(third.changed);
    assert_eq!(counts(&game, PATH), vec![1, 1, 1]);
    assert_eq!(
        pending_turnin(&game),
        Some(PATH),
        "only the third talk completes all three goals"
    );
    assert!(
        !third
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[49, 0, 13, 0])),
        "the end screen is held for the 26/6 dialogue choice"
    );
    assert_eq!(coins(&game), coins_before, "no reward before the turn-in");
    assert_eq!(archer_xp(&game), xp_before);

    let character = character_at(GREENWOOD, position);
    now += 10;
    let _ = release_end_screen(&mut game, &character, PATH, runtime_guid, now);
    now += 10;
    finish_quest(&mut game, &character, PATH, PATH_COINS, PATH_XP, now);
    assert_eq!(coins(&game), coins_before + PATH_COINS);
    assert_eq!(archer_xp(&game), xp_before + PATH_XP);

    now += 10;
    assert_packet_changes_nothing(
        &mut game,
        &character,
        &quest_end_reply(PATH),
        now,
        "duplicate turn-in",
    );
    assert_eq!(coins(&game), coins_before + PATH_COINS);
    assert_eq!(archer_xp(&game), xp_before + PATH_XP);
}

#[test]
fn oath_chapter_requires_path_and_talks_to_hilforn() {
    let fixture = Fixture::new();
    let mut game = fixture.game(None);
    let mut now = 3_000;
    let lugabow = content_position("greenwood_forest.json", LUGABOW);
    let character = character_at(GREENWOOD, lugabow);
    learn_archer(&mut game, &character, &mut now);
    start_path(&mut game, &character, &mut now);

    // Active Path blocks Oath: the prerequisite is not completed yet.
    assert_no_offer(&mut game, &character, LUGABOW, &mut now);
    assert!(
        !game
            .state()
            .npc_quests
            .as_ref()
            .unwrap()
            .quests
            .contains_key(&OATH)
    );

    complete_path_via_recruits(&mut game, &mut now);
    assert_eq!(quest_status(&game, PATH), QuestStatus::Completed);
    let coins_after_path = coins(&game);
    let xp_after_path = archer_xp(&game);

    let character = character_at(GREENWOOD, lugabow);
    now += 10;
    offer(&mut game, &character, LUGABOW, OATH, now);
    now += 10;
    accept(&mut game, &character, OATH, now);
    complete_oath_via_hilforn(&mut game, &mut now);

    assert_eq!(coins(&game), coins_after_path + OATH_COINS);
    assert_eq!(archer_xp(&game), xp_after_path + OATH_XP);
    assert_eq!(quest_status(&game, OATH), QuestStatus::Completed);
    // Completed Path is not offered again.
    assert_no_offer(&mut game, &character, LUGABOW, &mut now);
}

#[test]
fn wrong_npc_and_replay_do_not_advance_or_pay() {
    let fixture = Fixture::new();
    let mut game = fixture.game(None);
    let mut now = 4_000;
    let lugabow = content_position("greenwood_forest.json", LUGABOW);
    let character = character_at(GREENWOOD, lugabow);
    learn_archer(&mut game, &character, &mut now);
    start_path(&mut game, &character, &mut now);
    let coins_before = coins(&game);
    let xp_before = archer_xp(&game);

    for (npc_id, zone, file) in [
        (JONES, COBBLESTONE, "cobblestone_village.json"),
        (BARTLE, COBBLESTONE, "cobblestone_village.json"),
    ] {
        let position = content_position(file, npc_id);
        let wrong_character = character_at(zone, position);
        let before = saved(&game);
        let dialogue = talk(&mut game, &wrong_character, npc_id, now);
        now += 10;
        let closed = send(
            &mut game,
            &wrong_character,
            &dialogue_choice(QUEST_CHOICE),
            now,
        );
        now += 10;
        assert!(!dialogue.changed);
        assert!(
            !closed
                .replies
                .iter()
                .any(|packet| packet.starts_with(&[49, 0, 1, 0]))
        );
        assert_eq!(counts(&game, PATH), vec![0, 0, 0]);
        assert_eq!(saved(&game), before, "wrong NPC {npc_id} changed the save");
        assert_eq!(coins(&game), coins_before);
        assert_eq!(archer_xp(&game), xp_before);
    }

    // First objective, then replay the same Talk: ordered progress does not move.
    let (first, first_guid, first_position) = talk_recruit(&mut game, BOLLYLOG, &mut now);
    assert!(first.changed);
    assert_eq!(counts(&game, PATH), vec![1, 0, 0]);
    let after_first = saved(&game);
    let replay_character = character_at(GREENWOOD, first_position);
    let replayed = talk(&mut game, &replay_character, first_guid, now + 1);
    now += 10;
    assert!(!replayed.changed);
    assert_eq!(counts(&game, PATH), vec![1, 0, 0]);
    assert_eq!(
        saved(&game),
        after_first,
        "replaying a completed objective changed the save"
    );

    complete_path_via_recruits(&mut game, &mut now);
    assert_eq!(coins(&game), coins_before + PATH_COINS);
    assert_eq!(archer_xp(&game), xp_before + PATH_XP);

    // A completed quest is not advanced or paid again by any old packet.
    let completed_bytes = saved(&game);
    assert_packet_changes_nothing(
        &mut game,
        &character,
        &quest_end_reply(PATH),
        now + 1,
        "replayed turn-in after completion",
    );
    assert_packet_changes_nothing(
        &mut game,
        &character,
        &quest_reply(PATH, true),
        now + 2,
        "replayed accept after completion",
    );
    let bollylog_character = character_at(GREENWOOD, first_position);
    let after_completion = talk(&mut game, &bollylog_character, first_guid, now + 3);
    assert!(!after_completion.changed);
    assert_eq!(saved(&game), completed_bytes);
    assert_eq!(coins(&game), coins_before + PATH_COINS);
    assert_eq!(archer_xp(&game), xp_before + PATH_XP);
}

#[test]
fn relog_keeps_completed_history_and_no_duplicate_reward() {
    let fixture = Fixture::new();
    let mut game = fixture.game(None);
    let mut now = 5_000;
    let lugabow = content_position("greenwood_forest.json", LUGABOW);
    let character = character_at(GREENWOOD, lugabow);
    learn_archer(&mut game, &character, &mut now);
    let starting_coins = coins(&game);
    let starting_xp = archer_xp(&game);
    start_path(&mut game, &character, &mut now);

    // Disconnect mid-activity after one objective.
    let (first, _, _) = talk_recruit(&mut game, BOLLYLOG, &mut now);
    assert!(first.changed);
    let mid_bytes = saved(&game);
    game = fixture.game(Some(game.state()));
    assert_eq!(
        saved(&game),
        mid_bytes,
        "reload changed the active quest state"
    );
    assert_eq!(quest_status(&game, PATH), QuestStatus::Active);
    assert_eq!(counts(&game, PATH), vec![1, 0, 0]);

    complete_path_via_recruits(&mut game, &mut now);
    let character = character_at(GREENWOOD, lugabow);
    now += 10;
    offer(&mut game, &character, LUGABOW, OATH, now);
    now += 10;
    accept(&mut game, &character, OATH, now);
    complete_oath_via_hilforn(&mut game, &mut now);

    let final_bytes = saved(&game);
    let final_coins = coins(&game);
    let final_xp = archer_xp(&game);
    assert_eq!(final_coins, starting_coins + PATH_COINS + OATH_COINS);
    assert_eq!(final_xp, starting_xp + PATH_XP + OATH_XP);

    // Reload the finished chapter: both quests stay in completed history.
    game = fixture.game(Some(game.state()));
    assert_eq!(
        saved(&game),
        final_bytes,
        "reload changed completed history"
    );
    for quest_id in [PATH, OATH] {
        assert_eq!(quest_status(&game, quest_id), QuestStatus::Completed);
    }
    assert_eq!(pending_turnin(&game), None);
    assert_eq!(coins(&game), final_coins);
    assert_eq!(archer_xp(&game), final_xp);

    // Neither quest is offered again.
    assert_no_offer(&mut game, &character, LUGABOW, &mut now);
    // Replaying the old packets pays nothing and changes no save byte.
    for (index, quest_id) in [PATH, OATH].into_iter().enumerate() {
        let offset = u64::try_from(index).unwrap();
        assert_packet_changes_nothing(
            &mut game,
            &character,
            &quest_end_reply(quest_id),
            now + 10 + offset,
            "replayed turn-in after relog",
        );
        assert_packet_changes_nothing(
            &mut game,
            &character,
            &quest_reply(quest_id, true),
            now + 20 + offset,
            "replayed accept after relog",
        );
    }
    assert_eq!(coins(&game), final_coins);
    assert_eq!(archer_xp(&game), final_xp);
}

#[test]
fn non_archer_is_refused_with_no_state_change() {
    let fixture = Fixture::new();
    let mut game = fixture.game(None);
    let mut now = 6_000;
    let lugabow = content_position("greenwood_forest.json", LUGABOW);
    let character = character_at(GREENWOOD, lugabow);
    activate_brawler(&mut game, &character, &mut now);

    let before = saved(&game);
    let coins_before = coins(&game);
    let xp_before = archer_xp(&game);
    let dialogue = talk(&mut game, &character, LUGABOW, now + 1);
    now += 10;
    let closed = send(&mut game, &character, &dialogue_choice(QUEST_CHOICE), now);
    assert!(!dialogue.changed);
    assert!(
        !closed
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[49, 0, 1, 0])),
        "a Brawler must not receive Path of the Archer"
    );
    assert_eq!(
        saved(&game),
        before,
        "the non-Archer refusal changed the save"
    );
    assert_eq!(coins(&game), coins_before);
    assert_eq!(archer_xp(&game), xp_before);
    assert!(
        !game
            .state()
            .npc_quests
            .as_ref()
            .unwrap()
            .quests
            .contains_key(&PATH)
    );
    assert_eq!(pending_turnin(&game), None);
}

#[test]
fn path_reward_xp_follows_the_archer_profile_after_a_job_switch() {
    let fixture = Fixture::new();
    let mut game = fixture.game(None);
    let mut now = 7_000;
    let lugabow = content_position("greenwood_forest.json", LUGABOW);
    let character = character_at(GREENWOOD, lugabow);
    learn_archer(&mut game, &character, &mut now);
    let archer_xp_before = archer_xp(&game);
    start_path(&mut game, &character, &mut now);

    // The accepted quest survives a job switch; its reward map still names 35.
    now += 10;
    let switched = send(&mut game, &character, &select_job(BRAWLER), now);
    assert!(switched.changed);
    assert_eq!(game.state().profiles.as_ref().unwrap().active, BRAWLER);
    let brawler_xp_before = profile_xp(&game, BRAWLER);
    complete_path_via_recruits(&mut game, &mut now);
    assert_eq!(archer_xp(&game), archer_xp_before + PATH_XP);
    assert_eq!(profile_xp(&game, BRAWLER), brawler_xp_before);
}

#[test]
fn two_archers_count_the_same_recruit_independently() {
    let fixture = Fixture::new();
    let mut first = fixture.game(None);
    let mut second = fixture.game(None);
    let mut now = 8_000;
    let lugabow = content_position("greenwood_forest.json", LUGABOW);
    let character = character_at(GREENWOOD, lugabow);
    learn_archer(&mut first, &character, &mut now);
    learn_archer(&mut second, &character, &mut now);
    start_path(&mut first, &character, &mut now);
    start_path(&mut second, &character, &mut now);

    for game in [&mut first, &mut second] {
        let (runtime_guid, position) = recruit_spawn(BOLLYLOG);
        let at_recruit = character_at(GREENWOOD, position);
        let outcome = talk(game, &at_recruit, runtime_guid, now);
        assert!(outcome.changed);
        assert_eq!(counts(game, PATH), vec![1, 0, 0]);
    }
    assert_eq!(counts(&first, PATH), vec![1, 0, 0]);
    assert_eq!(counts(&second, PATH), vec![1, 0, 0]);
}

#[test]
fn enabled_archer_chapter_records_are_complete_and_others_stay_disabled() {
    let document = read_json(&content_dir().join("quests/archer.json"));
    let records = document["records"].as_array().unwrap();
    let enabled: Vec<&Value> = records
        .iter()
        .filter(|record| record["enabled"].as_bool().unwrap_or(false))
        .collect();
    let enabled_ids: Vec<u64> = enabled
        .iter()
        .map(|record| record["id"].as_u64().unwrap())
        .collect();
    assert_eq!(enabled_ids, vec![u64::from(PATH), u64::from(OATH)]);

    for record in &enabled {
        assert_eq!(record["giver_npc_id"].as_u64(), Some(LUGABOW));
        assert_eq!(record["job"].as_i64(), Some(i64::from(ARCHER)));
        let objectives = record["objectives"].as_array().unwrap();
        assert!(!objectives.is_empty(), "an enabled quest needs objectives");
        assert!(
            objectives.iter().all(|goal| goal["source"]
                .as_array()
                .is_some_and(|source| !source.is_empty())),
            "every objective needs provenance"
        );
        let rewards = record["rewards"].as_array().unwrap();
        assert!(!rewards.is_empty(), "every enabled Archer record pays");
        assert!(
            rewards.iter().all(|reward| reward["source"]
                .as_array()
                .is_some_and(|source| !source.is_empty())),
            "every reward needs provenance"
        );
    }

    let path = records
        .iter()
        .find(|record| record["id"].as_u64() == Some(u64::from(PATH)))
        .unwrap();
    assert_eq!(path["title_string_id"].as_u64(), Some(22_075));
    assert_eq!(path["prerequisite_quest_ids"], serde_json::json!([]));
    let path_targets: Vec<u64> = path["objectives"]
        .as_array()
        .unwrap()
        .iter()
        .map(|goal| goal["npc_id"].as_u64().unwrap())
        .collect();
    assert_eq!(path_targets, vec![BOLLYLOG, HRUGAGU, BRUGADUG]);
    assert_eq!(
        path["rewards"][0]["coins"].as_u64(),
        Some(u64::from(PATH_COINS))
    );
    assert_eq!(
        path["rewards"][0]["job_xp"]["35"].as_u64(),
        Some(u64::from(PATH_XP))
    );
    assert_eq!(path["rewards"][0]["items"], serde_json::json!([]));

    let oath = records
        .iter()
        .find(|record| record["id"].as_u64() == Some(u64::from(OATH)))
        .unwrap();
    assert_eq!(oath["title_string_id"].as_u64(), Some(77_036));
    assert_eq!(oath["prerequisite_quest_ids"], serde_json::json!([PATH]));
    assert_eq!(oath["objectives"][0]["npc_id"].as_u64(), Some(HILFORN));
    assert_eq!(
        oath["rewards"][0]["coins"].as_u64(),
        Some(u64::from(OATH_COINS))
    );
    assert_eq!(
        oath["rewards"][0]["job_xp"]["35"].as_u64(),
        Some(u64::from(OATH_XP))
    );
    assert_eq!(oath["rewards"][0]["items"], serde_json::json!([]));

    // The chapter deliberately leaves the minigame/field-enemy records disabled.
    for quest_id in [3_010_004_u64, 3_010_005, 3_010_007] {
        let record = records
            .iter()
            .find(|record| record["id"].as_u64() == Some(quest_id))
            .unwrap();
        assert_eq!(record["enabled"].as_bool(), Some(false));
    }

    // Lugabow is a data-backed spawned trainer and quest giver.
    let npcs = read_json(&content_dir().join("npcs/greenwood_forest.json"));
    let lugabow_row = npcs["records"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["npc_id"].as_u64() == Some(LUGABOW))
        .unwrap();
    assert_eq!(lugabow_row["placement"].as_str(), Some("legacy_data"));
    assert_eq!(lugabow_row["zone_id"].as_u64(), Some(u64::from(GREENWOOD)));
    assert!(
        lugabow_row["roles"]
            .as_array()
            .unwrap()
            .iter()
            .any(|role| role == "quest_giver")
    );
    assert!(lugabow_row["position"].is_array());

    // The Oath objective's report target is a data-backed spawned quest giver.
    let hilforn_row = npcs["records"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["npc_id"].as_u64() == Some(HILFORN))
        .unwrap();
    assert_eq!(hilforn_row["placement"].as_str(), Some("legacy_data"));
    assert_eq!(hilforn_row["zone_id"].as_u64(), Some(u64::from(GREENWOOD)));
    assert!(
        hilforn_row["roles"]
            .as_array()
            .unwrap()
            .iter()
            .any(|role| role == "quest_giver")
    );

    // Each enabled talk target has a regional quest-giver row aligned to exactly
    // one existing runtime actor, so the client can click the recruit it sees.
    let extra = read_json(&content_dir().join("npc_spawns_extra.json"));
    for (content_key, legacy_id, locale_id) in [
        (BOLLYLOG, 3_468_u64, 22_168_u64),
        (HRUGAGU, 3_469, 22_169),
        (BRUGADUG, 3_470, 22_170),
    ] {
        let (runtime_guid, runtime_position) = recruit_spawn(content_key);
        assert!(runtime_guid > 1_000_000_000, "unique runtime actor id");
        let recruit = npcs["records"]
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row["npc_id"].as_u64() == Some(content_key))
            .unwrap();
        assert_eq!(recruit["name_string_id"].as_u64(), Some(locale_id));
        assert_eq!(recruit["zone_id"].as_u64(), Some(u64::from(GREENWOOD)));
        assert_eq!(recruit["placement"].as_str(), Some("unanchored"));
        let position: [f32; 3] = serde_json::from_value(recruit["position"].clone()).unwrap();
        assert_eq!(
            position, runtime_position,
            "the range anchor must match the bound runtime actor"
        );
        assert!(
            recruit["roles"]
                .as_array()
                .unwrap()
                .iter()
                .any(|role| role == "quest_giver")
        );
        let bound = extra["records"]
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row["spawn_guid"].as_u64() == Some(runtime_guid))
            .unwrap();
        assert_eq!(bound["legacy_npc_id"].as_u64(), Some(legacy_id));
    }
}
