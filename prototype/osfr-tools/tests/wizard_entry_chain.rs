//! T242: Fizzlesticks' first Wizard chapter, "Wizard: How to be a Wizard" (3160003) and
//! "Wizard: I Can't Read This!" (3160004). The chapter is content-only; these tests drive
//! the production request path (`Gameplay::handle_in_world` -> `gameplay/npc.rs` ->
//! `QuestEngine`) plus the production position event (`Gameplay::on_location`, the exact
//! call `gateway.rs` makes after movement validation).
//!
//! Request/reply layouts used here:
//! - 26/8 NPC interact: constructor `008b6bc0`; body is u64 target + u8 [0].
//! - 26/10 menu select: constructor `00a36f80`; body is u64 target + i32 choice.
//! - 26/6 dialogue response: reader `009f5890`; body is i32 response row (301 is the quest row).
//! - 26/13 job select: i32 profile id.
//! - 49/1 QuestInfo: reader `00c7bb60` -> `00c7b7a0` (six dwords: quest id, intro, card title,
//!   name, description, an unused field), decoded with the production
//!   `decode_quest_offer_49_1` rather than by hand.
//! - 49/2 quest reply: constructor `00a86400` / sender `00a97d70`; i32 quest + u8 accept.
//! - 49/3 QuestAdd: reader `00c7bd10`.
//! - 49/13 QuestEnd: writer/reader `00c7bbd0`/`00c7b990`; actor u64 at 6..14, quest i32 at
//!   14..18, end-text locale at 18..22, animation at 22..26, reward bundle from 26.
//! - 49/14 end reply: constructor `00a864c0`; i32 quest.
//! 35/2 AddNpc (constructor `0091cac0`) draws Fizzlesticks and Nurble from their `legacy_data`
//! regional rows; `enabled_wizard_chapter_records_are_complete_and_drawable` pins that.
//!
//! The chapter fixture is the same opening play package the other chapter tests use
//! (`lab/references/opening-anvil-placement-v1/content`); the repository's own
//! `data/content` supplies the quest, NPC and trainer catalogues, exactly as the bundled play
//! package does. The Wizard save is seeded the way the NINJA chain seeds a Ninja save: the
//! fixture's `profiles.json` has no Wizard starter outfit, so the 26/13 switch to Wizard is
//! not used here; the chapter asks only that Wizard is the active job.

use osfr_tools::{
    character_store::CharacterState,
    gameplay::{Gameplay, GameplayState, Outcome},
    npc_interaction::{QuestOfferPacket49_1, decode_quest_offer_49_1},
    profiles::ProfileState,
    quest_engine::QuestStatus,
};
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

const CHARACTER: u64 = 242_017;
const SECOND_CHARACTER: u64 = 242_018;
const FIZZLESTICKS: u64 = 9_000_000_324;
const NURBLE: u64 = 9_000_000_415;
/// Drawable Robgoblin Camp NPC whose ground spot is the spell-book search area.
const SQUEE: u64 = 9_500_000_949;
const BOOK: u32 = 3_160_003;
const GLASSES: u32 = 3_160_004;
/// "Sticky Spell Book" (the Nurble cleaning errand) stays disabled in this slice.
const STICKY: u32 = 3_160_005;
const BOOK_TITLE: u32 = 20_859;
const BOOK_DESCRIPTION: u32 = 20_860;
const BOOK_INTRO: u32 = 24_947;
const BOOK_END_TEXT: u32 = 9_797;
const GLASSES_TITLE: u32 = 22_188;
const GLASSES_DESCRIPTION: u32 = 22_189;
const GLASSES_INTRO: u32 = 22_190;
const GLASSES_END_TEXT: u32 = 38_882;
const REWARD_COINS: u32 = 5;
const REWARD_XP: u32 = 25;
const WIZARD: i32 = 12;
const BRAWLER: i32 = 43;
/// The open continent is world zone 0; regional zone ids are content labels.
const OPEN_WORLD: u32 = 0;
/// Generic Talk option label in the reconstructed 26/9 menu (`npc_interaction.rs`).
const TALK_LABEL: u32 = 3_300;
/// First response row of the quest dialogue (client 26/6).
const QUEST_CHOICE: i32 = 301;

static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(0);

fn content_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content")
}

fn reference_content() -> PathBuf {
    std::env::var_os("OSFR_WIZARD_ENTRY_CONTENT")
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

/// One temporary play package. The fixture supplies profile/inventory content; the
/// repository's `job_trainers.json`, `npcs/` and split quests are bound from `data/content`.
struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        let source = reference_content();
        assert!(
            source.join("profiles.json").is_file() && source.join("inventory.json").is_file(),
            "the wizard chapter needs the opening content fixture at {}",
            source.display()
        );
        let directory = std::env::temp_dir().join(format!(
            "osfr-wizard-entry-{}-{}",
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
        self.game_for(state, CHARACTER)
    }

    fn game_for(&self, state: Option<GameplayState>, character_id: u64) -> Gameplay {
        let mut game = Gameplay::load(&self.0, state).expect("opening fixture loads");
        game.bind_npc_quest_catalogue(&content_dir(), character_id)
            .expect("split quest content loads");
        game
    }

    /// A saved Wizard: profile 12 unlocked and active at rank 1 (no experience). The client's
    /// level-20 Medic unlock is the trainer row's prerequisite, not a quest gate.
    fn wizard_state(&self) -> GameplayState {
        let mut state = Gameplay::load(&self.0, None)
            .expect("opening fixture loads")
            .state();
        state.profiles = Some(ProfileState {
            version: 1,
            active: WIZARD,
            unlocked: BTreeSet::from([1, WIZARD, BRAWLER]),
            experience: BTreeMap::new(),
        });
        state
    }

    /// A Wizard job the player has unlocked but is not playing.
    fn unlocked_wizard_state(&self) -> GameplayState {
        let mut state = self.wizard_state();
        state.profiles = Some(ProfileState {
            version: 1,
            active: BRAWLER,
            unlocked: BTreeSet::from([1, WIZARD, BRAWLER]),
            experience: BTreeMap::new(),
        });
        state
    }

    fn wizard_game(&self) -> Gameplay {
        self.game(Some(self.wizard_state()))
    }

    /// Reconnect: round-trip the saved state through JSON, then load it again.
    fn reconnect(&self, game: &Gameplay) -> Gameplay {
        let bytes = serde_json::to_vec(&game.state()).unwrap();
        let state: GameplayState = serde_json::from_slice(&bytes).unwrap();
        self.game(Some(state))
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

/// The player standing in front of `npc_id` in the open continent.
fn avatar_at(npc_id: u64) -> CharacterState {
    character_at(OPEN_WORLD, content_position("wilds.json", npc_id))
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

/// Talk through the client's quest row and assert the 49/1 offer, decoded with the
/// production reader.
fn offer(
    game: &mut Gameplay,
    character: &CharacterState,
    target: u64,
    quest_id: u32,
    now: u64,
) -> (Outcome, QuestOfferPacket49_1) {
    let dialogue = talk(game, character, target, now);
    assert!(
        dialogue
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[26, 0, 3, 0])),
        "Talk must open the 26/3 dialogue"
    );
    let offered = send(game, character, &dialogue_choice(QUEST_CHOICE), now + 2);
    let offers: Vec<&Vec<u8>> = offered
        .replies
        .iter()
        .filter(|packet| packet.starts_with(&[49, 0, 1, 0]))
        .collect();
    assert_eq!(offers.len(), 1, "one quest choice offers exactly one quest");
    assert!(
        has_family(offers[0], 49, 1, Some(quest_id)),
        "quest {quest_id} was not offered"
    );
    let decoded = decode_quest_offer_49_1(offers[0]).expect("49/1 decodes");
    assert_eq!(decoded.quest_id, quest_id);
    assert_eq!(quest_status(game, quest_id), QuestStatus::Offered);
    (offered, decoded)
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
    assert!(
        accepted
            .replies
            .iter()
            .any(|packet| has_family(packet, 49, 8, None)),
        "quest {quest_id} did not activate its first objective"
    );
    assert_eq!(quest_status(game, quest_id), QuestStatus::Active);
    accepted
}

/// `gateway.rs` sends the validated pose to `Gameplay::on_location`; that is the production
/// event a `visit_position` objective consumes.
fn visit(game: &mut Gameplay, position: [f32; 3], now: u64) -> Outcome {
    game.on_location(&character_at(OPEN_WORLD, position), now)
        .expect("location event")
}

fn assert_visit_changes_nothing(game: &mut Gameplay, position: [f32; 3], now: u64, label: &str) {
    let before = saved(game);
    let outcome = visit(game, position, now);
    assert!(!outcome.changed, "{label} changed gameplay");
    assert_eq!(saved(game), before, "{label} changed the save bytes");
}

/// 26/6 releases the 49/13 held for the quest choice. `QuestEndPacket::ReadFields`
/// (`00c7b990`) reads actor 6..14, quest 14..18, end text 18..22, animation 22..26; the
/// reward bundle starts at 26 (`encode_quest_reward_bundle`): success byte, coins i32,
/// stars i32, ..., display source guid, player guid, icon, name, rows.
fn release_end_screen(
    game: &mut Gameplay,
    character: &CharacterState,
    quest_id: u32,
    expected_actor: u64,
    expected_end_text: u32,
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
    assert_eq!(
        u32::from_le_bytes(end[18..22].try_into().unwrap()),
        expected_end_text,
        "the end screen must carry the quest's debrief text"
    );
    assert_eq!(
        u32::from_le_bytes(end[22..26].try_into().unwrap()),
        0,
        "the animation dword is the (R) default"
    );
    assert_eq!(end[26], 1, "the reward bundle success byte");
    assert_eq!(
        i32::from_le_bytes(end[27..31].try_into().unwrap()),
        REWARD_COINS as i32,
        "the 49/13 reward bundle carries the coin row"
    );
    assert_eq!(
        i32::from_le_bytes(end[31..35].try_into().unwrap()),
        REWARD_XP as i32,
        "the 49/13 reward bundle carries the job-XP row"
    );
    assert_eq!(
        u32::from_le_bytes(end[83..87].try_into().unwrap()),
        quest_title(quest_id),
        "the reward display names the quest title"
    );
    assert_eq!(
        u64::from_le_bytes(end[63..71].try_into().unwrap()),
        FIZZLESTICKS,
        "the reward display source is the quest giver"
    );
    assert_eq!(
        u64::from_le_bytes(end[71..79].try_into().unwrap()),
        CHARACTER,
        "the reward display names the character"
    );
    end
}

fn finish_quest(game: &mut Gameplay, character: &CharacterState, quest_id: u32, now: u64) {
    let coins_before = coins(game);
    let xp_before = profile_xp(game, WIZARD);
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
    assert_eq!(coins(game), coins_before + REWARD_COINS);
    assert_eq!(profile_xp(game, WIZARD), xp_before + REWARD_XP);
    assert_eq!(quest_status(game, quest_id), QuestStatus::Completed);
    assert_eq!(pending_turnin(game), None);
}

fn quest_title(quest_id: u32) -> u32 {
    match quest_id {
        BOOK => BOOK_TITLE,
        GLASSES => GLASSES_TITLE,
        other => panic!("quest {other} has no title in this chapter"),
    }
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

fn has_quest(game: &Gameplay, quest_id: u32) -> bool {
    game.state()
        .npc_quests
        .as_ref()
        .unwrap()
        .quests
        .contains_key(&quest_id)
}

fn pending_offer(game: &Gameplay) -> Option<u32> {
    game.state().npc_quests.as_ref().unwrap().pending_offer
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

fn saved(game: &Gameplay) -> Vec<u8> {
    serde_json::to_vec(&game.state()).unwrap()
}

fn assert_no_offer(game: &mut Gameplay, character: &CharacterState, target: u64, now: &mut u64) {
    let before = saved(game);
    *now += 10;
    let menu = interact(game, character, target, *now);
    let choice = menu_option(&menu, TALK_LABEL);
    *now += 1;
    let dialogue = send(game, character, &npc_select(target, choice), *now);
    assert!(
        !dialogue.changed,
        "a Talk with nothing eligible must not change gameplay"
    );
    // The refusal must leave the NPC menu open: the same 26/9 option still starts a Talk.
    *now += 1;
    let again = send(game, character, &npc_select(target, choice), *now);
    assert!(
        again
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[26, 0, 3, 0])),
        "a refused Talk consumed the open NPC menu"
    );
    *now += 8;
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
fn fizzlesticks_offers_the_first_wizard_chapter_to_an_eligible_wizard() {
    let fixture = Fixture::new();
    let character = avatar_at(FIZZLESTICKS);
    let mut now = 1_000;

    // An Adventurer sees no Wizard chapter; the open menu survives the refusal.
    let mut game = fixture.game(None);
    assert_no_offer(&mut game, &character, FIZZLESTICKS, &mut now);

    // A character on another job sees no Wizard chapter, and the offer gate does not
    // depend on the menu being freshly opened.
    now += 10;
    let switched = send(&mut game, &character, &select_job(BRAWLER), now);
    assert!(switched.changed);
    assert_eq!(game.state().profiles.as_ref().unwrap().active, BRAWLER);
    assert_no_offer(&mut game, &character, FIZZLESTICKS, &mut now);

    // A Wizard the player has unlocked but is not playing is refused by the same job gate.
    let mut inactive = fixture.game(Some(fixture.unlocked_wizard_state()));
    assert_no_offer(&mut inactive, &character, FIZZLESTICKS, &mut now);

    // The active Wizard sees the first chapter, and only the first chapter.
    let mut game = fixture.wizard_game();
    let (_, offer) = offer(&mut game, &character, FIZZLESTICKS, BOOK, now);
    assert_eq!(offer.intro_text_string_id, BOOK_INTRO);
    assert_eq!(offer.field_2, BOOK_TITLE);
    assert_eq!(offer.name_string_id, BOOK_TITLE);
    assert_eq!(offer.description_string_id, BOOK_DESCRIPTION);
    assert_eq!(offer.field_5, BOOK_TITLE);
    assert!(!offer.members_only);
    assert_eq!(offer.reward_bundle_body.len(), 69);
    // The offer card's bundle is the deliberate empty 69-byte placeholder: the exact coins
    // and job XP are asserted on the 49/13 turn-in screen, which is where they are shown.
    assert!(offer.reward_bundle_body.iter().all(|byte| *byte == 0));
    assert_eq!(pending_offer(&game), Some(BOOK));
    assert_eq!(counts(&game, BOOK), vec![0, 0]);
    assert!(!has_quest(&game, GLASSES));

    // A second Talk while the offer is open starts nothing new and changes nothing.
    assert_no_offer(&mut game, &character, FIZZLESTICKS, &mut now);
    assert_eq!(pending_offer(&game), Some(BOOK));

    // Accepting pays nothing yet.
    let coins_before = coins(&game);
    let xp_before = profile_xp(&game, WIZARD);
    now += 10;
    accept(&mut game, &character, BOOK, now);
    assert_eq!(pending_offer(&game), None);
    assert_eq!(counts(&game, BOOK), vec![0, 0]);
    assert_eq!(coins(&game), coins_before);
    assert_eq!(profile_xp(&game, WIZARD), xp_before);
    // The second quest stays gated behind the first.
    assert!(!has_quest(&game, GLASSES));
    assert_no_offer(&mut game, &character, FIZZLESTICKS, &mut now);
}

#[test]
fn the_spell_book_and_glasses_steps_advance_and_turn_in_once() {
    let fixture = Fixture::new();
    let character = avatar_at(FIZZLESTICKS);
    let book_search = content_position("wilds.json", SQUEE);
    let glasses_search = content_position("wilds.json", NURBLE);
    let mut now = 2_000;
    let mut game = fixture.wizard_game();

    now += 10;
    offer(&mut game, &character, FIZZLESTICKS, BOOK, now);
    now += 10;
    accept(&mut game, &character, BOOK, now);
    assert_eq!(counts(&game, BOOK), vec![0, 0]);

    // The other chapter's search area is out of range for this step.
    assert_visit_changes_nothing(&mut game, glasses_search, now + 1, "out-of-range visit");
    assert_eq!(counts(&game, BOOK), vec![0, 0]);

    // The camp search area completes the first ordered step only.
    now += 10;
    let searched = visit(&mut game, book_search, now);
    assert!(searched.changed);
    assert_eq!(counts(&game, BOOK), vec![1, 0]);
    assert_eq!(pending_turnin(&game), None);
    assert!(
        !searched
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[49, 0, 13, 0])),
        "the search is not the turn-in"
    );
    // A duplicate visit changes nothing.
    assert_visit_changes_nothing(&mut game, book_search, now + 1, "duplicate visit");

    // A talk to the wrong Robgoblin Camp NPC changes nothing.
    now += 10;
    let wrong = talk(&mut game, &avatar_at(NURBLE), NURBLE, now);
    assert!(!wrong.changed, "a talk to the wrong NPC changed gameplay");
    assert_eq!(counts(&game, BOOK), vec![1, 0]);

    // Returning to Fizzlesticks completes the ordered step and holds the end screen.
    let coins_before = coins(&game);
    let xp_before = profile_xp(&game, WIZARD);
    now += 10;
    let returned = talk(&mut game, &character, FIZZLESTICKS, now);
    assert!(returned.changed);
    assert_eq!(counts(&game, BOOK), vec![1, 1]);
    assert_eq!(pending_turnin(&game), Some(BOOK));
    assert!(
        !returned
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[49, 0, 13, 0])),
        "the end screen is held for the 26/6 dialogue choice"
    );
    assert_eq!(coins(&game), coins_before, "no reward before the turn-in");
    assert_eq!(profile_xp(&game, WIZARD), xp_before);

    now += 10;
    release_end_screen(
        &mut game,
        &character,
        BOOK,
        FIZZLESTICKS,
        BOOK_END_TEXT,
        now,
    );
    now += 10;
    finish_quest(&mut game, &character, BOOK, now);
    assert_eq!(coins(&game), coins_before + REWARD_COINS);
    assert_eq!(profile_xp(&game, WIZARD), xp_before + REWARD_XP);

    // No duplicate payment on a replayed 49/14.
    now += 10;
    assert_packet_changes_nothing(
        &mut game,
        &character,
        &quest_end_reply(BOOK),
        now,
        "duplicate turn-in",
    );
    assert_eq!(coins(&game), coins_before + REWARD_COINS);

    // The second chapter opens only now: the next quest choice offers "I Can't Read This!"
    // and never re-offers the completed first record.
    now += 10;
    let (_, offer) = offer(&mut game, &character, FIZZLESTICKS, GLASSES, now);
    assert_eq!(offer.intro_text_string_id, GLASSES_INTRO);
    assert_eq!(offer.name_string_id, GLASSES_TITLE);
    assert_eq!(offer.description_string_id, GLASSES_DESCRIPTION);
    assert_eq!(offer.field_2, GLASSES_TITLE);
    now += 10;
    accept(&mut game, &character, GLASSES, now);
    assert_eq!(counts(&game, GLASSES), vec![0, 0]);

    // The spell-book search area is out of range for the junkpile step.
    assert_visit_changes_nothing(&mut game, book_search, now + 1, "wrong-area visit");
    assert_eq!(counts(&game, GLASSES), vec![0, 0]);

    now += 10;
    let searched = visit(&mut game, glasses_search, now);
    assert!(searched.changed);
    assert_eq!(counts(&game, GLASSES), vec![1, 0]);
    assert_visit_changes_nothing(
        &mut game,
        glasses_search,
        now + 1,
        "duplicate junkpile visit",
    );

    now += 10;
    let returned = talk(&mut game, &character, FIZZLESTICKS, now);
    assert!(returned.changed);
    assert_eq!(counts(&game, GLASSES), vec![1, 1]);
    assert_eq!(pending_turnin(&game), Some(GLASSES));

    let coins_before = coins(&game);
    let xp_before = profile_xp(&game, WIZARD);
    now += 10;
    release_end_screen(
        &mut game,
        &character,
        GLASSES,
        FIZZLESTICKS,
        GLASSES_END_TEXT,
        now,
    );
    now += 10;
    finish_quest(&mut game, &character, GLASSES, now);
    assert_eq!(coins(&game), coins_before + REWARD_COINS);
    assert_eq!(profile_xp(&game, WIZARD), xp_before + REWARD_XP);
    now += 10;
    assert_packet_changes_nothing(
        &mut game,
        &character,
        &quest_end_reply(GLASSES),
        now,
        "duplicate glasses turn-in",
    );
    assert_eq!(coins(&game), coins_before + REWARD_COINS);

    // The Nurble cleaning errand stays disabled and is never offered.
    assert!(!has_quest(&game, STICKY));
    assert_no_offer(&mut game, &character, FIZZLESTICKS, &mut now);
}

#[test]
fn forged_or_missing_objective_events_change_nothing() {
    let fixture = Fixture::new();
    let character = avatar_at(FIZZLESTICKS);
    let book_search = content_position("wilds.json", SQUEE);
    let glasses_search = content_position("wilds.json", NURBLE);
    let far_away = [book_search[0] + 60.0, book_search[1], book_search[2]];
    let mut now = 3_000;
    let mut game = fixture.wizard_game();
    let coins_before = coins(&game);
    let xp_before = profile_xp(&game, WIZARD);

    now += 10;
    offer(&mut game, &character, FIZZLESTICKS, BOOK, now);
    // A forged accept of the still-gated second quest must not consume the open offer.
    let before = saved(&game);
    let forged = send(&mut game, &character, &quest_reply(GLASSES, true), now + 1);
    assert!(!forged.changed);
    assert_eq!(
        saved(&game),
        before,
        "the forged accept changed the save bytes"
    );
    assert_eq!(pending_offer(&game), Some(BOOK));
    assert!(!has_quest(&game, GLASSES));

    now += 10;
    accept(&mut game, &character, BOOK, now);

    // A replayed 49/2 changes nothing.
    assert_packet_changes_nothing(
        &mut game,
        &character,
        &quest_reply(BOOK, true),
        now + 1,
        "replayed accept",
    );
    // A premature 49/14 for the active quest pays nothing.
    assert_packet_changes_nothing(
        &mut game,
        &character,
        &quest_end_reply(BOOK),
        now + 2,
        "premature turn-in",
    );
    // A forged 49/14 for the gated second quest pays nothing.
    assert_packet_changes_nothing(
        &mut game,
        &character,
        &quest_end_reply(GLASSES),
        now + 3,
        "forged second-quest turn-in",
    );
    // A visit outside every search area changes nothing.
    assert_visit_changes_nothing(&mut game, far_away, now + 4, "missing-objective visit");
    // The other chapter's search area is out of range for this step.
    assert_visit_changes_nothing(&mut game, glasses_search, now + 5, "out-of-range visit");
    // A talk to a wrong NPC changes nothing.
    let wrong = talk(&mut game, &avatar_at(NURBLE), NURBLE, now + 6);
    assert!(!wrong.changed);
    assert_eq!(counts(&game, BOOK), vec![0, 0]);
    assert_eq!(pending_turnin(&game), None);
    assert_eq!(coins(&game), coins_before);
    assert_eq!(profile_xp(&game, WIZARD), xp_before);

    // Complete the chapter properly, then replay every old packet.
    now += 10;
    assert!(visit(&mut game, book_search, now).changed);
    now += 10;
    assert!(talk(&mut game, &character, FIZZLESTICKS, now).changed);
    now += 10;
    release_end_screen(
        &mut game,
        &character,
        BOOK,
        FIZZLESTICKS,
        BOOK_END_TEXT,
        now,
    );
    now += 10;
    finish_quest(&mut game, &character, BOOK, now);
    let completed_coins = coins(&game);
    let completed_xp = profile_xp(&game, WIZARD);
    for (index, packet) in [
        quest_end_reply(BOOK),
        quest_reply(BOOK, true),
        quest_end_reply(GLASSES),
    ]
    .into_iter()
    .enumerate()
    {
        let offset = u64::try_from(index).unwrap();
        assert_packet_changes_nothing(
            &mut game,
            &character,
            &packet,
            now + 10 + offset,
            "replayed chapter packet after completion",
        );
    }
    assert_visit_changes_nothing(&mut game, book_search, now + 20, "post-completion visit");
    assert_eq!(coins(&game), completed_coins);
    assert_eq!(profile_xp(&game, WIZARD), completed_xp);
    assert_eq!(quest_status(&game, BOOK), QuestStatus::Completed);
    assert!(!has_quest(&game, GLASSES));
}

#[test]
fn a_mid_chapter_reconnect_resumes_the_saved_progress() {
    let fixture = Fixture::new();
    let character = avatar_at(FIZZLESTICKS);
    let book_search = content_position("wilds.json", SQUEE);
    let glasses_search = content_position("wilds.json", NURBLE);
    let mut now = 4_000;
    let mut game = fixture.wizard_game();
    let starting_coins = coins(&game);
    let starting_xp = profile_xp(&game, WIZARD);

    now += 10;
    offer(&mut game, &character, FIZZLESTICKS, BOOK, now);
    now += 10;
    accept(&mut game, &character, BOOK, now);
    now += 10;
    assert!(visit(&mut game, book_search, now).changed);
    assert_eq!(counts(&game, BOOK), vec![1, 0]);

    // Disconnect after the first objective.
    let mid_bytes = saved(&game);
    game = fixture.reconnect(&game);
    assert_eq!(
        saved(&game),
        mid_bytes,
        "reload changed the active quest state"
    );
    assert_eq!(quest_status(&game, BOOK), QuestStatus::Active);
    assert_eq!(counts(&game, BOOK), vec![1, 0]);
    assert_eq!(pending_turnin(&game), None);
    assert!(!has_quest(&game, GLASSES));

    // The second quest is still gated until the first completes: a forged accept changes
    // nothing, and no second quest exists in the save.
    let before = saved(&game);
    let forged = send(&mut game, &character, &quest_reply(GLASSES, true), now + 1);
    assert!(!forged.changed);
    assert_eq!(saved(&game), before, "the gated accept changed the save");
    assert!(!has_quest(&game, GLASSES));

    // Resume the chapter: the talk step and the held end screen survive too.
    now += 10;
    assert!(talk(&mut game, &character, FIZZLESTICKS, now).changed);
    assert_eq!(counts(&game, BOOK), vec![1, 1]);
    assert_eq!(pending_turnin(&game), Some(BOOK));
    let held_bytes = saved(&game);
    game = fixture.reconnect(&game);
    assert_eq!(saved(&game), held_bytes, "reload changed the held turn-in");
    assert_eq!(pending_turnin(&game), Some(BOOK));
    assert_eq!(coins(&game), starting_coins, "a disconnect pays nothing");
    assert_eq!(profile_xp(&game, WIZARD), starting_xp);

    // The relog visibility batch re-sends the saved end screen, and only after the speaker's
    // 35/2 AddNpc: `ShowQuestEndScreen` (`00a95420`) resolves the actor through the client
    // actor table and never retries.
    let packets = game
        .update_npc_visibility(&character)
        .expect("relog NPC visibility");
    let spawn = packets.iter().position(|packet| {
        osfr_tools::startup_world::parse_add_npc(packet)
            .is_ok_and(|npc| npc.actor_id == FIZZLESTICKS)
    });
    let end = packets
        .iter()
        .position(|packet| has_family(packet, 49, 13, Some(BOOK)))
        .expect("the held end screen is re-sent after relog");
    assert!(
        spawn.is_some_and(|spawn| spawn < end),
        "the speaker's 35/2 must precede its 49/13"
    );
    assert_eq!(
        u64::from_le_bytes(packets[end][6..14].try_into().unwrap()),
        FIZZLESTICKS,
        "the re-sent end screen must name the runtime speaker"
    );
    assert_eq!(
        u32::from_le_bytes(packets[end][18..22].try_into().unwrap()),
        BOOK_END_TEXT,
        "the re-sent end screen must carry the spell-book debrief text"
    );

    // The client answers the re-sent end screen directly after the reconnect.
    now += 10;
    finish_quest(&mut game, &character, BOOK, now);
    assert_eq!(coins(&game), starting_coins + REWARD_COINS);
    assert_eq!(profile_xp(&game, WIZARD), starting_xp + REWARD_XP);

    // The chain order is restored after the reconnect: the second quest opens now.
    now += 10;
    offer(&mut game, &character, FIZZLESTICKS, GLASSES, now);
    now += 10;
    accept(&mut game, &character, GLASSES, now);
    now += 10;
    assert!(visit(&mut game, glasses_search, now).changed);
    now += 10;
    assert!(talk(&mut game, &character, FIZZLESTICKS, now).changed);
    now += 10;
    release_end_screen(
        &mut game,
        &character,
        GLASSES,
        FIZZLESTICKS,
        GLASSES_END_TEXT,
        now,
    );
    now += 10;
    finish_quest(&mut game, &character, GLASSES, now);

    let final_bytes = saved(&game);
    let final_coins = coins(&game);
    let final_xp = profile_xp(&game, WIZARD);
    game = fixture.reconnect(&game);
    assert_eq!(
        saved(&game),
        final_bytes,
        "reload changed completed history"
    );
    assert_eq!(final_coins, starting_coins + 2 * REWARD_COINS);
    assert_eq!(final_xp, starting_xp + 2 * REWARD_XP);
    for quest_id in [BOOK, GLASSES] {
        assert_eq!(quest_status(&game, quest_id), QuestStatus::Completed);
    }
    assert_eq!(pending_turnin(&game), None);
}

#[test]
fn a_malformed_wizard_record_fails_at_load() {
    let fixture = Fixture::new();

    // The control: a well-formed synthetic record binds through the production path.
    let control = record(3_999_001, vec![talk_goal(1, FIZZLESTICKS)], vec![]);
    let control_dir = synthetic_content(&control);
    let mut control_game = fixture.game(None);
    control_game
        .bind_npc_quest_catalogue(&control_dir, CHARACTER)
        .expect("the control record loads");
    let _ = fs::remove_dir_all(&control_dir);

    // A zero talk target.
    assert_load_refused(
        &fixture,
        record(3_999_001, vec![talk_goal(1, 0)], vec![]),
        "zero objective target",
    );
    // An objective without provenance.
    let mut no_source = visit_goal(1, [0.0, 0.0, 0.0], 5.0);
    no_source["source"] = serde_json::json!([]);
    assert_load_refused(
        &fixture,
        record(3_999_001, vec![no_source], vec![]),
        "empty objective source",
    );
    // Overlapping objective ids inside one record.
    assert_load_refused(
        &fixture,
        record(
            3_999_001,
            vec![talk_goal(1, FIZZLESTICKS), talk_goal(1, NURBLE)],
            vec![],
        ),
        "overlapping objective ids",
    );
    // A reward outside the wire bounds.
    let mut out_of_range = reward(5);
    out_of_range["coins"] = serde_json::json!(u32::MAX);
    assert_load_refused(
        &fixture,
        record(
            3_999_001,
            vec![talk_goal(1, FIZZLESTICKS)],
            vec![out_of_range],
        ),
        "out-of-range reward",
    );
    // A job-XP row that names no known job profile.
    let mut unknown_profile = reward(5);
    unknown_profile["job_xp"] = serde_json::json!({"999": 25});
    assert_load_refused(
        &fixture,
        record(
            3_999_001,
            vec![talk_goal(1, FIZZLESTICKS)],
            vec![unknown_profile],
        ),
        "unknown job profile",
    );
}

fn talk_goal(id: u32, npc_id: u64) -> Value {
    serde_json::json!({
        "kind": "talk_to_npc",
        "id": id,
        "npc_id": npc_id,
        "source": [{"kind": "reconstructed", "evidence": "(R) T242 synthetic input"}]
    })
}

fn visit_goal(id: u32, position: [f32; 3], radius: f32) -> Value {
    serde_json::json!({
        "kind": "visit_position",
        "id": id,
        "zone_id": 0,
        "position": position,
        "radius": radius,
        "source": [{"kind": "reconstructed", "evidence": "(R) T242 synthetic input"}]
    })
}

fn reward(coins: u32) -> Value {
    serde_json::json!({
        "coins": coins,
        "items": [],
        "job_xp": {"12": 25},
        "source": [{"kind": "reconstructed", "evidence": "(R) T242 synthetic input"}]
    })
}

fn record(id: u32, objectives: Vec<Value>, rewards: Vec<Value>) -> Value {
    serde_json::json!({
        "id": id,
        "job": WIZARD,
        "title_string_id": BOOK_TITLE,
        "giver_npc_id": FIZZLESTICKS,
        "prerequisite_quest_ids": [],
        "objectives": objectives,
        "rewards": rewards,
        "source": [{"kind": "reconstructed", "evidence": "(R) T242 synthetic record"}]
    })
}

/// Write one synthetic split-content package and return its directory.
fn synthetic_content(quest: &Value) -> PathBuf {
    let id = quest["id"].as_u64().unwrap();
    let directory = std::env::temp_dir().join(format!(
        "osfr-wizard-malformed-{}-{}-{id}",
        std::process::id(),
        NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = fs::remove_dir_all(&directory);
    fs::create_dir_all(directory.join("quests")).unwrap();
    fs::create_dir_all(directory.join("npcs")).unwrap();
    fs::write(
        directory.join("quests/wizard.json"),
        serde_json::to_vec(&serde_json::json!({
            "schema_version": 1,
            "records": [quest]
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        directory.join("npcs/wilds.json"),
        serde_json::to_vec(&serde_json::json!({"schema_version": 1, "records": []})).unwrap(),
    )
    .unwrap();
    fs::write(
        directory.join("jobs.json"),
        serde_json::to_vec(&serde_json::json!({
            "schema_version": 1,
            "records": [{
                "id": "wizard",
                "name": "Wizard",
                "profile_id": WIZARD,
                "category": "combat",
                "max_level": 20,
                "members_only": false,
                "trainer_npc_ids": [],
                "quest_ids": [id],
                "abilities": [],
                "source": [{"kind": "reconstructed", "evidence": "(R) T242 synthetic job"}]
            }]
        }))
        .unwrap(),
    )
    .unwrap();
    directory
}

fn assert_load_refused(fixture: &Fixture, quest: Value, label: &str) {
    let directory = synthetic_content(&quest);
    let mut game = fixture.game(None);
    let result = game.bind_npc_quest_catalogue(&directory, CHARACTER);
    let _ = fs::remove_dir_all(&directory);
    assert!(result.is_err(), "{label} must be refused at content load");
    // The refusal left the previously bound catalogue in place and changed no save byte.
    assert_eq!(saved(&game), saved(&fixture.game(None)));
}

/// Content assertions: exactly the two chapter records are enabled, every target is a
/// drawable `legacy_data` Robgoblin Camp actor, and the search areas are anchored to real
/// spawn ground.
#[test]
fn enabled_wizard_chapter_records_are_complete_and_drawable() {
    let document = read_json(&content_dir().join("quests/wizard.json"));
    let records = document["records"].as_array().unwrap();
    let enabled: Vec<&Value> = records
        .iter()
        .filter(|record| record["enabled"].as_bool().unwrap_or(false))
        .collect();
    assert_eq!(
        enabled
            .iter()
            .map(|record| record["id"].as_u64().unwrap())
            .collect::<Vec<_>>(),
        vec![u64::from(BOOK), u64::from(GLASSES)]
    );
    for record in &enabled {
        assert_eq!(record["giver_npc_id"].as_u64(), Some(FIZZLESTICKS));
        assert_eq!(record["job"].as_i64(), Some(i64::from(WIZARD)));
        let objectives = record["objectives"].as_array().unwrap();
        assert_eq!(
            objectives.len(),
            2,
            "an enabled quest needs real objectives"
        );
        assert!(
            objectives.iter().all(|goal| goal["source"]
                .as_array()
                .is_some_and(|source| !source.is_empty())),
            "every objective needs provenance"
        );
        let rewards = record["rewards"].as_array().unwrap();
        assert!(!rewards.is_empty(), "every enabled Wizard record pays");
        assert_eq!(rewards[0]["coins"].as_u64(), Some(u64::from(REWARD_COINS)));
        assert_eq!(
            rewards[0]["job_xp"]["12"].as_u64(),
            Some(u64::from(REWARD_XP))
        );
        assert_eq!(rewards[0]["items"], serde_json::json!([]));
        assert!(rewards.iter().all(|reward| {
            reward["source"]
                .as_array()
                .is_some_and(|source| !source.is_empty())
        }));
    }

    let book = records
        .iter()
        .find(|record| record["id"].as_u64() == Some(u64::from(BOOK)))
        .unwrap();
    assert_eq!(
        book["title_string_id"].as_u64(),
        Some(u64::from(BOOK_TITLE))
    );
    assert_eq!(
        book["description_string_id"].as_u64(),
        Some(u64::from(BOOK_DESCRIPTION))
    );
    assert_eq!(
        book["intro_text_string_id"].as_u64(),
        Some(u64::from(BOOK_INTRO))
    );
    assert_eq!(
        book["end_text_string_id"].as_u64(),
        Some(u64::from(BOOK_END_TEXT))
    );
    assert_eq!(book["prerequisite_quest_ids"], serde_json::json!([]));
    assert_eq!(book["objectives"][0]["kind"], "visit_position");
    assert_eq!(book["objectives"][0]["zone_id"].as_u64(), Some(0));
    let radius = book["objectives"][0]["radius"].as_f64().unwrap();
    assert!((4.0..=8.0).contains(&radius), "the camp search radius");
    assert_eq!(book["objectives"][1]["kind"], "talk_to_npc");
    assert_eq!(book["objectives"][1]["npc_id"].as_u64(), Some(FIZZLESTICKS));

    let glasses = records
        .iter()
        .find(|record| record["id"].as_u64() == Some(u64::from(GLASSES)))
        .unwrap();
    assert_eq!(
        glasses["title_string_id"].as_u64(),
        Some(u64::from(GLASSES_TITLE))
    );
    assert_eq!(
        glasses["description_string_id"].as_u64(),
        Some(u64::from(GLASSES_DESCRIPTION))
    );
    assert_eq!(
        glasses["intro_text_string_id"].as_u64(),
        Some(u64::from(GLASSES_INTRO))
    );
    assert_eq!(
        glasses["end_text_string_id"].as_u64(),
        Some(u64::from(GLASSES_END_TEXT))
    );
    assert_eq!(glasses["prerequisite_quest_ids"], serde_json::json!([BOOK]));
    assert_eq!(glasses["objectives"][0]["kind"], "visit_position");
    assert_eq!(glasses["objectives"][1]["kind"], "talk_to_npc");
    assert_eq!(
        glasses["objectives"][1]["npc_id"].as_u64(),
        Some(FIZZLESTICKS)
    );

    // The two search areas are anchored to the ground of drawable camp NPCs, and stay
    // outside the 12 m Fizzlesticks talk ring so neither step completes on acceptance.
    let npcs = read_json(&content_dir().join("npcs/wilds.json"));
    let regional = |content_key: u64| -> &Value {
        let rows: Vec<&Value> = npcs["records"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|row| row["npc_id"].as_u64() == Some(content_key))
            .collect();
        assert_eq!(
            rows.len(),
            1,
            "NPC {content_key} must have one regional row"
        );
        rows[0]
    };
    for (content_key, role) in [(FIZZLESTICKS, "trainer"), (NURBLE, "quest_giver")] {
        let row = regional(content_key);
        assert_eq!(row["placement"].as_str(), Some("legacy_data"));
        assert_eq!(row["zone_id"].as_u64(), Some(16));
        assert!(
            row["roles"]
                .as_array()
                .unwrap()
                .iter()
                .any(|value| value == role),
            "NPC {content_key} must be a drawable {role}"
        );
    }
    let fizzlesticks_ground = content_position("wilds.json", FIZZLESTICKS);
    let distance = |a: [f32; 3], b: [f32; 3]| -> f32 {
        (a.iter().zip(b).map(|(x, y)| (x - y) * (x - y)).sum::<f32>()).sqrt()
    };
    for (record, anchor, name) in [(book, SQUEE, "spell-book"), (glasses, NURBLE, "glasses")] {
        let point: [f32; 3] =
            serde_json::from_value(record["objectives"][0]["position"].clone()).unwrap();
        let radius = record["objectives"][0]["radius"].as_f64().unwrap() as f32;
        let anchor_ground = content_position("wilds.json", anchor);
        assert!(
            distance(point, anchor_ground) < 0.001,
            "the {name} search area must sit on the {anchor} spawn ground"
        );
        assert!(
            distance(point, fizzlesticks_ground) > 12.0 + radius,
            "the {name} search area must be outside the talk ring plus its radius"
        );
    }

    // The Nurble cleaning errand stays disabled.
    let sticky = records
        .iter()
        .find(|record| record["id"].as_u64() == Some(u64::from(STICKY)))
        .unwrap();
    assert_eq!(sticky["enabled"].as_bool(), Some(false));
    assert_eq!(sticky["objectives"], serde_json::json!([]));
}

#[test]
fn two_wizards_progress_independently() {
    let fixture = Fixture::new();
    let first_character = avatar_at(FIZZLESTICKS);
    let second_character = CharacterState {
        guid: SECOND_CHARACTER,
        ..first_character.clone()
    };
    let book_search = content_position("wilds.json", SQUEE);
    let mut now = 5_000;
    let mut first = fixture.wizard_game();
    let mut second = fixture.game_for(Some(fixture.wizard_state()), SECOND_CHARACTER);

    for (game, character) in [
        (&mut first, &first_character),
        (&mut second, &second_character),
    ] {
        now += 10;
        offer(game, character, FIZZLESTICKS, BOOK, now);
        now += 10;
        accept(game, character, BOOK, now);
    }
    for game in [&first, &second] {
        assert_eq!(counts(game, BOOK), vec![0, 0]);
    }

    // One player's search never enters the other's state.
    now += 10;
    assert!(visit(&mut first, book_search, now).changed);
    assert_eq!(counts(&first, BOOK), vec![1, 0]);
    assert_eq!(counts(&second, BOOK), vec![0, 0]);
    assert_eq!(pending_offer(&second), None);

    // The second player's own search and turn-in pay only that character.
    now += 10;
    assert!(visit(&mut second, book_search, now).changed);
    assert_eq!(counts(&second, BOOK), vec![1, 0]);
    let second_coins = coins(&second);
    assert!(
        talk(&mut second, &second_character, FIZZLESTICKS, now + 1).changed,
        "the second Wizard completes the same search step independently"
    );
    assert_eq!(counts(&second, BOOK), vec![1, 1]);
    assert_eq!(counts(&first, BOOK), vec![1, 0]);
    assert_eq!(pending_turnin(&second), Some(BOOK));
    assert_eq!(pending_turnin(&first), None);
    assert_eq!(coins(&second), second_coins, "no reward before the turn-in");
}
