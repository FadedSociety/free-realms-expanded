//! T202: the playable job quest lines pay their quest's job XP.
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
//! by `008e7930`; its second dword after the coin count is the displayed XP total.
//! Job XP is delivered by the same 38/14 progress packet (`008d86e0`, six i32:
//! profile, rank, progress, 0, 0, 0) and 38/18 rank packet (`008d8910`, profile and
//! rank) that `rewards::experience_packets` already writes.

use osfr_tools::{
    character_store::CharacterState,
    gameplay::{Gameplay, GameplayState, Outcome},
    npc_interaction::NpcInteraction,
    profiles::ProfileState,
    quest_engine::{CharacterEligibility, QuestEngine, QuestStatus},
};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

const CHARACTER: u64 = 202_017;
const ADVENTURER: i32 = 1;
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

const INTRODUCE_YOURSELF: u32 = 3_000_020;
const BURDEN_OF_FAME: u32 = 3_080_001;
const RANGER_REQUISITION: u32 = 3_050_011;
const NINJA_PATH: u32 = 3_110_001;

const TALK_LABEL: u32 = 3_300;
const QUEST_CHOICE: i32 = 301;

static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(0);

fn content_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content")
}

fn reference_content() -> PathBuf {
    std::env::var_os("OSFR_QUEST_JOB_XP_CONTENT")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../../lab/references/opening-anvil-placement-v1/content")
        })
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

/// Opening profile/inventory content; the repository's split quest and NPC content is
/// bound exactly as the bundled play package binds it (`bind_npc_quest_catalogue`).
struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        let source = reference_content();
        assert!(
            source.join("profiles.json").is_file() && source.join("inventory.json").is_file(),
            "the job-XP chapter needs the opening content fixture at {}",
            source.display()
        );
        let directory = std::env::temp_dir().join(format!(
            "osfr-quest-job-xp-{}-{}",
            std::process::id(),
            NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed)
        ));
        copy_dir(&source, &directory);
        // A new character starts with only the Brawler outfit. The capture also holds the
        // full per-profile default outfit; promote it to the fixture's initial state so the
        // production `ProcessProfileSwap` gate can wear the Kart Driver and Ninja outfits.
        let inventory_path = directory.join("inventory.json");
        let mut manifest: Value =
            serde_json::from_slice(&fs::read(&inventory_path).unwrap()).unwrap();
        manifest["initial_state"] = manifest["default_state"].clone();
        fs::write(&inventory_path, serde_json::to_vec(&manifest).unwrap()).unwrap();
        Self(directory)
    }

    fn load(&self, state: GameplayState, character_id: u64) -> Gameplay {
        let mut game = Gameplay::load(&self.0, Some(state)).expect("saved opening fixture loads");
        game.bind_npc_quest_catalogue(&content_dir(), character_id)
            .expect("split quest content loads");
        game
    }

    fn fresh(&self, active: i32, unlocked: &[i32]) -> Gameplay {
        self.fresh_as(active, unlocked, CHARACTER)
    }

    fn fresh_as(&self, active: i32, unlocked: &[i32], character_id: u64) -> Gameplay {
        let mut state = Gameplay::load(&self.0, None)
            .expect("opening fixture loads")
            .state();
        state.profiles = Some(ProfileState {
            version: 1,
            active,
            unlocked: unlocked.iter().copied().collect(),
            experience: BTreeMap::new(),
        });
        self.load(state, character_id)
    }

    fn reload(&self, game: &Gameplay) -> Gameplay {
        self.load(game.state(), CHARACTER)
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn spawn_location(npc_id: u64) -> (u32, [f32; 3]) {
    let content = content_dir();
    NpcInteraction::load(&content.join("npcs"), &content.join("stores.json"))
        .expect("NPC content loads")
        .location(npc_id)
        .unwrap_or_else(|| panic!("NPC {npc_id} is a positioned content spawn"))
}

fn character_at(npc_id: u64) -> CharacterState {
    let (zone_id, position) = spawn_location(npc_id);
    CharacterState {
        guid: CHARACTER,
        zone_id,
        position,
        rotation: [1.0, 0.0, 0.0],
    }
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

fn offer(game: &mut Gameplay, character: &CharacterState, giver: u64, quest_id: u32, now: u64) {
    let dialogue = talk(game, character, giver, now);
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
}

fn accept(game: &mut Gameplay, character: &CharacterState, quest_id: u32, now: u64) {
    let accepted = send(game, character, &quest_reply(quest_id, true), now);
    assert!(
        accepted
            .replies
            .iter()
            .any(|packet| has_family(packet, 49, 3, Some(quest_id))),
        "quest {quest_id} did not send QuestAdd"
    );
    assert_eq!(quest_status(game, quest_id), QuestStatus::Active);
}

/// Complete one ordered Talk objective. A final target opens the pending end screen,
/// which the client releases with its 26/6 dialogue choice.
fn talk_step(game: &mut Gameplay, character: &CharacterState, target: u64, now: u64, label: &str) {
    let talked = talk(game, character, target, now);
    assert!(talked.changed, "{label} did not advance its objective");
}

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
    profile: i32,
    expected_xp: u32,
    now: u64,
) {
    let coins_before = coins(game);
    let xp_before = profile_xp(game, profile);
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
    assert_eq!(profile_xp(game, profile), xp_before + expected_xp);
    assert_eq!(quest_status(game, quest_id), QuestStatus::Completed);
    assert_eq!(pending_turnin(game), None);
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

fn saved(game: &Gameplay) -> Vec<u8> {
    serde_json::to_vec(&game.state()).unwrap()
}

/// A refusal must leave coins, items, profile XP, quest counters and the save bytes equal.
fn assert_no_change(
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

fn assert_no_offer_at(game: &mut Gameplay, target: u64, now: &mut u64, label: &str) {
    let character = character_at(target);
    assert_no_offer(game, &character, target, now, label);
}

fn introduce_yourself(game: &mut Gameplay, now: &mut u64) {
    let merigold = character_at(MERIGOLD);
    let raina = character_at(RAINA);
    *now += 10;
    offer(game, &merigold, MERIGOLD, INTRODUCE_YOURSELF, *now);
    *now += 10;
    accept(game, &merigold, INTRODUCE_YOURSELF, *now);
    *now += 10;
    talk_step(game, &raina, RAINA, *now, "Introduce Yourself");
    assert_eq!(counts(game, INTRODUCE_YOURSELF), [1]);
    assert_eq!(pending_turnin(game), Some(INTRODUCE_YOURSELF));
    *now += 10;
    release_end_screen(game, &raina, INTRODUCE_YOURSELF, RAINA, *now);
    *now += 10;
    finish_quest(game, &raina, INTRODUCE_YOURSELF, 5, ADVENTURER, 15, *now);
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
    release_end_screen(game, &raina, BURDEN_OF_FAME, RAINA, *now);
    *now += 10;
    finish_quest(game, &raina, BURDEN_OF_FAME, 5, KART_DRIVER, 25, *now);
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
    release_end_screen(game, &ricardo, RANGER_REQUISITION, RICARDO, *now);
    *now += 10;
    finish_quest(game, &ricardo, RANGER_REQUISITION, 0, CHEF, 20, *now);
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
    release_end_screen(game, &grosh, NINJA_PATH, GROSH, *now);
    *now += 10;
    finish_quest(game, &grosh, NINJA_PATH, 0, NINJA, 15, *now);
}

#[test]
fn each_enabled_quest_pays_its_configured_job_xp_once() {
    let fixture = Fixture::new();
    let mut game = fixture.fresh(ADVENTURER, &[1, 2, 43, 45, 48]);
    let mut now = 1_000;
    let raina = character_at(RAINA);
    let auguste = character_at(AUGUSTE);
    let ty = character_at(TY);

    introduce_yourself(&mut game, &mut now);
    assert_eq!(profile_xp(&game, ADVENTURER), 15);

    now += 10;
    switch_job(&mut game, &raina, KART_DRIVER, now);
    burden_of_fame(&mut game, &mut now);
    assert_eq!(profile_xp(&game, KART_DRIVER), 25);

    now += 10;
    switch_job(&mut game, &auguste, CHEF, now);
    ranger_requisition(&mut game, &mut now);
    assert_eq!(profile_xp(&game, CHEF), 20);

    now += 10;
    switch_job(&mut game, &ty, NINJA, now);
    ninja_path(&mut game, &mut now);
    assert_eq!(profile_xp(&game, NINJA), 15);

    assert_eq!(profile_xp(&game, BRAWLER), 0, "no other job is paid");
    assert_eq!(coins(&game), 10, "the existing coin rows are unchanged");

    let finished = saved(&game);
    let finished_coins = coins(&game);
    for (quest_id, profile) in [
        (INTRODUCE_YOURSELF, ADVENTURER),
        (BURDEN_OF_FAME, KART_DRIVER),
        (RANGER_REQUISITION, CHEF),
        (NINJA_PATH, NINJA),
    ] {
        let before = profile_xp(&game, profile);
        now += 1;
        assert_no_change(
            &mut game,
            &raina,
            &quest_end_reply(quest_id),
            now,
            "duplicate turn-in",
        );
        assert_eq!(profile_xp(&game, profile), before);
    }
    assert_eq!(
        saved(&game),
        finished,
        "a duplicate turn-in changed the save"
    );
    assert_eq!(coins(&game), finished_coins);
}

#[test]
fn xp_follows_the_quest_job_not_the_active_job() {
    let fixture = Fixture::new();
    let mut game = fixture.fresh(NINJA, &[1, 2, 43]);
    let mut now = 2_000;
    let ty = character_at(TY);
    let grosh = character_at(GROSH);

    offer(&mut game, &ty, TY, NINJA_PATH, now);
    now += 10;
    accept(&mut game, &ty, NINJA_PATH, now);
    now += 10;
    talk_step(&mut game, &grosh, GROSH, now, "Ninja: The Ninja Path");
    now += 10;
    release_end_screen(&mut game, &grosh, NINJA_PATH, GROSH, now);

    // The engine does not recheck the offer-time job at turn-in; the explicit content
    // map still pays profile 2 after the player switched to Brawler.
    now += 10;
    let brawler_before = profile_xp(&game, BRAWLER);
    switch_job(&mut game, &grosh, BRAWLER, now);
    now += 10;
    finish_quest(&mut game, &grosh, NINJA_PATH, 0, NINJA, 15, now);

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
    let fixture = Fixture::new();
    let mut game = fixture.fresh(ADVENTURER, &[1, 43]);
    let mut now = 3_000;
    let raina = character_at(RAINA);

    // Qualify the Kart prerequisite as the default Adventurer, then switch to Brawler.
    introduce_yourself(&mut game, &mut now);
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

    assert_no_offer_at(&mut game, AUGUSTE, &mut now, "Brawler at the Chef giver");
    assert_no_offer_at(&mut game, TY, &mut now, "Brawler at the Ninja giver");
    assert_no_offer_at(
        &mut game,
        RAINA,
        &mut now,
        "Brawler at the Kart giver with the prerequisite complete",
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

#[test]
fn refused_turn_in_changes_nothing() {
    let fixture = Fixture::new();
    let mut game = fixture.fresh(CHEF, &[1, 45]);
    let mut now = 4_000;
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
    assert_no_change(
        &mut game,
        &ricardo,
        &quest_end_reply(RANGER_REQUISITION),
        now,
        "turn-in before the objective",
    );
    // Wrong quest while the real turn-in is not pending either.
    now += 10;
    assert_no_change(
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
            .is_err(),
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
    release_end_screen(&mut game, &ricardo, RANGER_REQUISITION, RICARDO, now);
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
            .is_err(),
        "a far menu selection must be refused"
    );
    assert_eq!(saved(&game), held);

    now += 10;
    finish_quest(&mut game, &ricardo, RANGER_REQUISITION, 0, CHEF, 20, now);
    let finished = saved(&game);
    let finished_coins = coins(&game);
    let finished_xp = profile_xp(&game, CHEF);
    now += 10;
    assert_no_change(
        &mut game,
        &ricardo,
        &quest_end_reply(RANGER_REQUISITION),
        now,
        "replayed 49/14",
    );
    now += 10;
    assert_no_change(
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
    let fixture = Fixture::new();
    let mut game = fixture.fresh(NINJA, &[1, 2, 43]);
    let mut now = 5_000;
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
    finish_quest(&mut game, &grosh, NINJA_PATH, 0, NINJA, 15, now);
    let finished = saved(&game);

    game = fixture.reload(&game);
    assert_eq!(saved(&game), finished, "reload changed the paid save");
    assert_eq!(profile_xp(&game, NINJA), 15, "the paid XP survives reload");
    assert_eq!(quest_status(&game, NINJA_PATH), QuestStatus::Completed);
    assert_eq!(pending_turnin(&game), None);

    // The completed quest is not offered again, and old packets pay nothing.
    assert_no_offer_at(&mut game, TY, &mut now, "completed Ninja quest after relog");
    now += 10;
    assert_no_change(
        &mut game,
        &grosh,
        &quest_end_reply(NINJA_PATH),
        now,
        "replayed turn-in after relog",
    );
    now += 10;
    assert_no_change(
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
    let fixture = Fixture::new();
    let mut first = fixture.fresh_as(ADVENTURER, &[1], 202_101);
    let mut second = fixture.fresh_as(ADVENTURER, &[1], 202_102);
    let mut now = 6_000;

    introduce_yourself(&mut first, &mut now);
    introduce_yourself(&mut second, &mut now);

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

fn copy_split_content(source: &Path, target: &Path) {
    fs::create_dir_all(target.join("quests")).unwrap();
    copy_dir(&source.join("quests"), &target.join("quests"));
    copy_dir(&source.join("npcs"), &target.join("npcs"));
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
    let fixture = Fixture::new();
    let scratch = std::env::temp_dir().join(format!(
        "osfr-quest-job-xp-load-{}-{}",
        std::process::id(),
        NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed)
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

    let mut game = Gameplay::load(&fixture.0, None).expect("opening fixture loads");
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
    let expected: [(&str, u32, u64, &[u32], u32, Option<i32>, i32, u32); 12] = [
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
    ];

    let mut listed = BTreeSet::new();
    let has_t202 = |source: &Value| {
        source.as_array().is_some_and(|tags| {
            tags.iter().any(|tag| {
                tag["evidence"]
                    .as_str()
                    .is_some_and(|text| text.starts_with("(R) T202:"))
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
                has_t202(&record["source"]),
                "{file}: quest {quest_id} carries the T202 (R) job-gate decision"
            );
        }
        assert!(
            has_t202(&reward["source"]),
            "{file}: quest {quest_id} reward carries the T202 (R) decision"
        );
    }

    // Only the listed IDs gained XP or a job gate in these four files. The disabled
    // C1 record 3000009 already carried its Brawler gate before this slice.
    for file in [
        "adventurer.json",
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
            for reward in record["rewards"].as_array().into_iter().flatten() {
                assert_eq!(
                    reward["job_xp"],
                    json!({}),
                    "{file}: quest {id} gained unexpected job XP"
                );
            }
            if record.get("job").is_none() {
                continue;
            }
            assert_eq!(
                (file, id, record["job"].as_i64()),
                ("adventurer.json", 3_000_009, Some(i64::from(BRAWLER))),
                "{file}: quest {id} gained an unexpected job gate"
            );
        }
    }
}
