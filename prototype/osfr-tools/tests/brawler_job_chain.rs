use gateway_reference::Reference;
use osfr_tools::{
    character_store::CharacterState,
    gameplay::{Gameplay, GameplayState},
    gateway::Gateway,
    npc_interaction::NpcInteraction,
    quest_engine::{CharacterEligibility, QuestDenial, QuestEngine, QuestProgress, QuestStatus},
    startup_world,
    world::NpcVisibility,
};
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

#[path = "common/gateway_reference.rs"]
mod gateway_reference;
#[path = "support/npc_quest_chain.rs"]
mod npc_quest_chain;
#[path = "support/terrain.rs"]
mod terrain;

use npc_quest_chain::{active_job, profile_xp, saved, seed_active_profile};

const CHARACTER: u64 = 101;
const SECOND_CHARACTER: u64 = 102;
const CLIENT_VERSION: &str = "1.910.1.530630";
const CARLOS: u64 = 9_000_000_042;
const JONES: u64 = 9_000_000_121;
const SABASTIAN: u64 = 9_000_000_446;
const PETRA: u64 = 9_000_000_428;
const REFERRAL_TO_JONES: u32 = 3_030_006;
const HOOLIGAN_FIGHT: u32 = 3_030_007;
const SEARCH_BLACKSPORE: u32 = 3_030_008;
const GIVEN_TO_GUARD: u32 = 3_030_009;
const HOOLIGAN_DEFINITION: u32 = 930_050;
const QUEST_REWARD_COINS: u32 = 5;
const QUEST_REWARD_JOB_XP: u32 = 25;
const BRAWLER: i32 = 43;
const ADVENTURER: i32 = 1;
const PETRA_MODEL: i32 = 356;
const PETRA_NAME_STRING_ID: i32 = 72_450;
const PETRA_LEGACY_NPC_ID: u64 = 1465;
const ATTACK_CHOICE: i32 = 103;
const QUEST_CHOICE: i32 = 301;

fn content_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content")
}

fn gameplay_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../lab/references/opening-anvil-placement-v1/content")
}

fn character_at(position: [f32; 3]) -> CharacterState {
    CharacterState {
        guid: CHARACTER,
        zone_id: 0,
        position,
        rotation: [1.0, 0.0, 0.0],
    }
}

fn load_game_for(
    character_id: u64,
    saved: Option<osfr_tools::gameplay::GameplayState>,
) -> Gameplay {
    let mut game = Gameplay::load(&gameplay_dir(), saved).expect("gameplay content loads");
    game.bind_npc_quest_catalogue(&content_dir(), character_id)
        .expect("split quest content loads");
    game
}

fn load_game(saved: Option<osfr_tools::gameplay::GameplayState>) -> Gameplay {
    load_game_for(CHARACTER, saved)
}

fn fresh_game_for(character_id: u64) -> (Gameplay, CharacterState) {
    let mut game = load_game_for(character_id, None);
    let character = CharacterState {
        guid: character_id,
        zone_id: 0,
        position: [0.0; 3],
        rotation: [1.0, 0.0, 0.0],
    };
    let selected = game
        .handle_in_world(&select_job(BRAWLER), &character, 1)
        .expect("Brawler job selection is valid");
    assert!(
        selected.changed,
        "Brawler selection must change the profile"
    );
    (game, character)
}

fn fresh_game() -> (Gameplay, CharacterState) {
    fresh_game_for(CHARACTER)
}

fn npc_position(file: &str, npc_id: u64) -> [f32; 3] {
    let document: Value =
        serde_json::from_slice(&fs::read(content_dir().join("npcs").join(file)).unwrap()).unwrap();
    let npc = document["records"]
        .as_array()
        .unwrap()
        .iter()
        .find(|record| record["npc_id"].as_u64() == Some(npc_id))
        .expect("chapter NPC exists");
    serde_json::from_value(npc["position"].clone()).expect("chapter NPC has a spawn position")
}

fn field_hooligans() -> Vec<(u64, [f32; 3])> {
    let document: Value =
        serde_json::from_slice(&fs::read(content_dir().join("field_enemies.json")).unwrap())
            .unwrap();
    let enemies: Vec<_> = document["enemies"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|enemy| {
            enemy["definition_id"].as_u64() == Some(u64::from(HOOLIGAN_DEFINITION))
                && enemy["quest_ids"].as_array().is_some_and(|ids| {
                    ids.iter()
                        .any(|id| id.as_u64() == Some(u64::from(HOOLIGAN_FIGHT)))
                })
        })
        .map(|enemy| {
            (
                enemy["guid"].as_u64().unwrap(),
                serde_json::from_value(enemy["position"].clone()).unwrap(),
            )
        })
        .collect();
    assert_eq!(enemies.len(), 4, "the chapter owns four quest-gated spawns");
    enemies
}

/// The captured opening inventory carries only the Brawler outfit, so an ordinary 26/13 switch
/// refuses every other job. Seed the saved active profile with the shared helper when direct
/// play cannot reach a job switch; the offer gate reads this production save.
fn with_active_job(game: &Gameplay, character_id: u64, profile: i32) -> Gameplay {
    let mut state = game.state();
    seed_active_profile(&mut state, profile, &[]);
    load_game_for(character_id, Some(state))
}

/// T331's drawable Petra row in `npc_spawns_extra.json` (spawn_guid/content_key 9000000428).
fn petra_spawn_row() -> Value {
    let document: Value =
        serde_json::from_slice(&fs::read(content_dir().join("npc_spawns_extra.json")).unwrap())
            .unwrap();
    let rows: Vec<_> = document["records"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|row| row["spawn_guid"].as_u64() == Some(PETRA))
        .cloned()
        .collect();
    assert_eq!(rows.len(), 1, "exactly one extra spawn carries Petra");
    rows.into_iter().next().unwrap()
}

fn petra_spawn_position() -> [f32; 3] {
    serde_json::from_value(petra_spawn_row()["position"].clone())
        .expect("Petra's drawable spawn has a position")
}

/// Stand on Petra's authored content row, inside the 12 m NPC interaction radius of the drawable
/// actor 3 m east of it (the row itself must not move; T331 rows the spawn delta in its source).
fn at_petra(character: &CharacterState) -> CharacterState {
    CharacterState {
        position: npc_position("blackspore_swamp.json", PETRA),
        ..character.clone()
    }
}

/// Complete C3-C5 and accept C6 at Sabastian through the production menus.
fn complete_chapter_and_accept_c6(game: &mut Gameplay, character: &CharacterState, now: &mut u64) {
    complete_chapter(game, character, now);
    let sabastian = npc_position("blackspore_swamp.json", SABASTIAN);
    accept_quest(game, character, SABASTIAN, sabastian, GIVEN_TO_GUARD, now);
}

fn eligibility() -> CharacterEligibility {
    CharacterEligibility {
        player_level: 20,
        job_levels: BTreeMap::from([("brawler".into(), 20)]),
        member: true,
        nonmember_quest_limit: 30,
        member_quest_limit: 30,
    }
}

/// The first spawned field enemy whose defeat definition is not C4's `930050`.
/// An empty `quest_ids` list is production's "always present" field rule
/// (`EncounterEngine` load), so this actor is live without starting a quest.
fn ungated_enemy_other_than_hooligans() -> (u64, [f32; 3], u32) {
    let document: Value =
        serde_json::from_slice(&fs::read(content_dir().join("field_enemies.json")).unwrap())
            .unwrap();
    let enemy = document["enemies"]
        .as_array()
        .unwrap()
        .iter()
        .find(|enemy| {
            enemy["quest_ids"]
                .as_array()
                .is_some_and(|ids| ids.is_empty())
                && enemy["definition_id"].as_u64() != Some(u64::from(HOOLIGAN_DEFINITION))
        })
        .expect("content has an always-present non-Hooligan field enemy");
    (
        enemy["guid"].as_u64().unwrap(),
        serde_json::from_value(enemy["position"].clone()).expect("field enemy has a position"),
        u32::try_from(enemy["definition_id"].as_u64().unwrap())
            .expect("field enemy definition fits the quest event"),
    )
}

fn assert_refusal_changes_nothing(
    game: &mut Gameplay,
    character: &CharacterState,
    packet: &[u8],
    now: u64,
    label: &str,
) {
    let before = saved(game);
    let outcome = game
        .handle_in_world(packet, character, now)
        .unwrap_or_else(|error| panic!("{label} should be a clean refusal: {error}"));
    assert!(!outcome.changed, "{label} changed gameplay");
    assert_eq!(saved(game), before, "{label} changed the saved state");
}

fn has_family(packet: &[u8], family: u16, subtype: u16, id: Option<u32>) -> bool {
    if packet.len() < 6
        || packet[..2] != family.to_le_bytes()
        || packet[2..4] != subtype.to_le_bytes()
    {
        return false;
    }
    let Some(id) = id else { return true };
    // 49/13 carries the turn-in actor at 6..14 and the reply quest id at 14..18;
    // every other used family/subtype carries its id at 6..10. Client
    // QuestEndPacket::ReadFields 0x00c7b990 / Unserialize 0x00c7bbd0.
    let offset = if (family, subtype) == (49, 13) { 14 } else { 6 };
    packet
        .get(offset..offset + 4)
        .is_some_and(|bytes| u32::from_le_bytes(bytes.try_into().unwrap()) == id)
}

fn gateway_login() -> Vec<u8> {
    let mut bytes = 1u16.to_le_bytes().to_vec();
    for (index, text) in ["f170-test-ticket", CLIENT_VERSION, ""].iter().enumerate() {
        bytes.extend((text.len() as u32).to_le_bytes());
        bytes.extend(text.as_bytes());
        if index == 0 {
            bytes.extend(CHARACTER.to_le_bytes());
        }
    }
    bytes
}

fn wrap(inner: &[u8]) -> Vec<u8> {
    let mut bytes = vec![5, 0, 1];
    bytes.extend((inner.len() as u32).to_le_bytes());
    bytes.extend(inner);
    bytes
}

fn application_body(packet: &[u8]) -> Option<&[u8]> {
    if packet.len() < 7 || packet[..3] != [5, 0, 1] {
        return None;
    }
    let length = u32::from_le_bytes(packet[3..7].try_into().unwrap()) as usize;
    (packet.len() == length + 7).then_some(&packet[7..])
}

/// Drive the real gateway world-entry sequence for a saved pending turn-in and prove the
/// directed 35/2 spawn is queued before the 49/13 end screen that resolves its actor.
fn gateway_reconnect_orders_spawn_before_screen(
    saved: &GameplayState,
    character: &CharacterState,
    speaker: u64,
    now: u64,
) {
    let content = gameplay_dir();
    let reference = Reference::new(CLIENT_VERSION, Some(content.as_path()));
    let mut gateway = Gateway::load_gameplay(
        &reference.directory,
        character.clone(),
        &content,
        Some(saved.clone()),
    )
    .expect("gateway loads the saved chapter");
    gateway.handle_at(&gateway_login(), now).unwrap();
    gateway.handle_at(&wrap(&[13, 0]), now + 1).unwrap();
    let loaded = gateway
        .handle_at(&wrap(&[10, 0]), now + 2)
        .expect("reconnect reaches the loaded event");
    let replies: Vec<_> = loaded
        .replies
        .iter()
        .filter_map(|packet| application_body(packet))
        .collect();
    let spawn = replies
        .iter()
        .position(|packet| {
            packet.starts_with(&[35, 0, 2, 0])
                && packet.len() >= 12
                && u64::from_le_bytes(packet[4..12].try_into().unwrap()) == speaker
        })
        .expect("reconnect spawns the turn-in speaker");
    let screen = replies
        .iter()
        .position(|packet| has_family(packet, 49, 13, Some(HOOLIGAN_FIGHT)))
        .expect("reconnect reopens the pending end screen");
    assert!(
        spawn < screen,
        "the 49/13 screen must follow the speaker's 35/2 spawn"
    );
    let end = replies[screen];
    assert_eq!(
        u64::from_le_bytes(end[6..14].try_into().unwrap()),
        speaker,
        "the recovered end screen carries the runtime speaker"
    );
    assert_eq!(
        u32::from_le_bytes(end[14..18].try_into().unwrap()),
        HOOLIGAN_FIGHT,
        "the third dword remains the quest id that 49/14 sends back"
    );
}

fn interact_npc(game: &mut Gameplay, character: &CharacterState, npc_id: u64, now: u64) -> Vec<u8> {
    let menu = game
        .handle_in_world(&npc_interact(npc_id), character, now)
        .expect("NPC interaction is valid");
    menu.replies
        .into_iter()
        .find(|packet| packet.starts_with(&[26, 0, 9, 0]))
        .unwrap_or_else(|| panic!("NPC {npc_id} did not open its menu"))
}

fn talk(
    game: &mut Gameplay,
    character: &CharacterState,
    npc_id: u64,
    now: u64,
) -> osfr_tools::gameplay::Outcome {
    let _ = interact_npc(game, character, npc_id, now);
    let dialogue = game
        .handle_in_world(&npc_select(npc_id, 1), character, now + 1)
        .expect("NPC Talk selection is valid");
    assert!(
        dialogue
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[26, 0, 3, 0])),
        "NPC {npc_id} did not open dialogue"
    );
    dialogue
}

fn turn_in_option(menu: &[u8], title_string_id: u32) -> i32 {
    let count = i32::from_le_bytes(menu[13..17].try_into().unwrap());
    assert!(count > 0);
    (0..count)
        .find_map(|index| {
            let offset = 17 + usize::try_from(index).unwrap() * 28;
            let id = i32::from_le_bytes(menu[offset..offset + 4].try_into().unwrap());
            let string_id = u32::from_le_bytes(menu[offset + 8..offset + 12].try_into().unwrap());
            (string_id == title_string_id).then_some(id)
        })
        .expect("native menu contains the quest turn-in option")
}

fn choose_quest_row(
    game: &mut Gameplay,
    character: &CharacterState,
    now: u64,
) -> osfr_tools::gameplay::Outcome {
    game.handle_in_world(&dialogue_choice(QUEST_CHOICE), character, now)
        .expect("quest dialogue choice is valid")
}

/// T252 enabled C1 (`3030004`, Brawler-gated) ahead of C3 in Carlos's quest list. This T101
/// chapter keeps its own C3-C5 flow by accepting C1 and completing only its Talk step: Carlos
/// then offers C3 exactly as before, and C1 stays on its Mongo step without touching any reward
/// or combat assertion below.
fn open_carlos_line(game: &mut Gameplay, character: &CharacterState, now: &mut u64) {
    const FIRST: u32 = 3_030_004;
    let mut at = character.clone();
    at.position = npc_position("cobblestone_village.json", CARLOS);
    *now += 100;
    talk(game, &at, CARLOS, *now);
    *now += 100;
    let offered = choose_quest_row(game, &at, *now);
    assert!(
        offered
            .replies
            .iter()
            .any(|packet| has_family(packet, 49, 1, Some(FIRST))),
        "T252's C1 must be Carlos's first offer to a Brawler"
    );
    *now += 100;
    let accepted = game
        .handle_in_world(&quest_reply(FIRST, true), &at, *now)
        .expect("C1 accept is valid");
    assert!(
        accepted
            .replies
            .iter()
            .any(|packet| has_family(packet, 49, 3, None))
    );
    *now += 100;
    // The follow-up Talk completes C1's Talk step and opens its Mongo step.
    talk(game, &at, CARLOS, *now);
}

fn accept_quest(
    game: &mut Gameplay,
    character: &CharacterState,
    npc_id: u64,
    position: [f32; 3],
    quest_id: u32,
    now: &mut u64,
) {
    let mut character = character.clone();
    character.position = position;
    *now += 100;
    talk(game, &character, npc_id, *now);
    *now += 100;
    let offered = choose_quest_row(game, &character, *now);
    assert!(
        offered
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[49, 0, 1, 0])),
        "quest {quest_id} was not offered"
    );
    *now += 100;
    let accepted = game
        .handle_in_world(&quest_reply(quest_id, true), &character, *now)
        .expect("quest accept is valid");
    assert!(
        accepted
            .replies
            .iter()
            .any(|packet| has_family(packet, 49, 3, None)),
        "quest {quest_id} did not send QuestAdd"
    );
    assert_eq!(
        game.state().npc_quests.unwrap().quests[&quest_id].status,
        QuestStatus::Active
    );
}

fn open_end_screen(
    game: &mut Gameplay,
    character: &CharacterState,
    npc_id: u64,
    quest_id: u32,
    now: &mut u64,
) -> Vec<u8> {
    *now += 100;
    talk(game, character, npc_id, *now);
    *now += 100;
    let chosen = choose_quest_row(game, character, *now);
    let end_screens: Vec<_> = chosen
        .replies
        .into_iter()
        .filter(|packet| has_family(packet, 49, 13, Some(quest_id)))
        .collect();
    assert_eq!(
        end_screens.len(),
        1,
        "one Talk must queue exactly one 49/13 end screen"
    );
    let end = end_screens.into_iter().next().unwrap();
    assert_eq!(
        game.state().npc_quests.unwrap().pending_turnin,
        Some(quest_id)
    );
    assert_eq!(
        u64::from_le_bytes(end[6..14].try_into().unwrap()),
        npc_id,
        "the end screen names the NPC that opened it"
    );
    assert_eq!(
        u32::from_le_bytes(end[14..18].try_into().unwrap()),
        quest_id,
        "the third dword is the quest id 49/14 sends back"
    );
    end
}

fn finish_turnin(game: &mut Gameplay, character: &CharacterState, quest_id: u32, now: &mut u64) {
    let before = game.state().inventory.coins;
    *now += 100;
    let completed = game
        .handle_in_world(&quest_end_reply(quest_id), character, *now)
        .expect("quest turn-in is valid");
    assert!(completed.changed);
    assert!(
        completed
            .replies
            .iter()
            .any(|packet| has_family(packet, 49, 4, Some(quest_id))),
        "quest {quest_id} did not complete"
    );
    let state = game.state();
    assert_eq!(
        state.inventory.coins,
        before + QUEST_REWARD_COINS,
        "quest {quest_id} paid the wrong amount"
    );
    let quests = state.npc_quests.unwrap();
    assert_eq!(quests.pending_turnin, None);
    assert_eq!(quests.quests[&quest_id].status, QuestStatus::Completed);
}

fn assert_pending_screen_recovery(
    game: &mut Gameplay,
    character: &CharacterState,
    npc_id: u64,
    quest_id: u32,
    title_string_id: u32,
    now: &mut u64,
) {
    let before = saved(game);
    let menu = interact_npc(game, character, npc_id, *now);
    let option = turn_in_option(&menu, title_string_id);
    *now += 1;
    let selected = game
        .handle_in_world(&npc_select(npc_id, option), character, *now)
        .expect("menu turn-in recovery is valid");
    let by_menu = selected
        .replies
        .iter()
        .find(|packet| has_family(packet, 49, 13, Some(quest_id)))
        .expect("the menu turn-in reopens the pending end screen");
    assert_eq!(
        u64::from_le_bytes(by_menu[6..14].try_into().unwrap()),
        npc_id,
        "the menu-reopened end screen carries the actual turn-in actor"
    );
    assert_eq!(
        saved(game),
        before,
        "the menu turn-in changed the saved state"
    );

    let reopened = open_end_screen(game, character, npc_id, quest_id, now);
    assert_eq!(
        u64::from_le_bytes(reopened[6..14].try_into().unwrap()),
        npc_id,
        "the reopened end screen carries the actual turn-in actor"
    );
    assert_eq!(
        saved(game),
        before,
        "reopening pending quest {quest_id} changed the saved state"
    );

    let mut reconnected = load_game(Some(game.state()));
    assert_eq!(
        saved(&reconnected),
        before,
        "loading the saved state for pending quest {quest_id} changed it"
    );
    *now += 100;
    let initialized = reconnected
        .initialize_world(character, *now)
        .expect("reconnected gameplay initializes");
    assert!(
        !initialized
            .replies
            .iter()
            .any(|packet| has_family(packet, 49, 13, Some(quest_id))),
        "login synchronization must not open the pending end screen before the speaker spawn"
    );
    let restored = reconnected
        .update_npc_visibility(character)
        .expect("reconnected NPC visibility updates");
    let restored = restored
        .iter()
        .find(|packet| has_family(packet, 49, 13, Some(quest_id)))
        .unwrap_or_else(|| panic!("reconnect must reopen pending quest {quest_id}"));
    assert_eq!(
        u64::from_le_bytes(restored[6..14].try_into().unwrap()),
        npc_id,
        "the recovered end screen carries the actual turn-in actor"
    );
    assert_eq!(
        saved(&reconnected),
        before,
        "reconnect changed the saved state for pending quest {quest_id}"
    );
    *game = reconnected;
}

fn kill_hooligans(game: &mut Gameplay, character: &CharacterState, now: &mut u64) {
    // A far-away location lets the production tick publish quest-gated spawns
    // without advancing any actor before the test reaches it.
    let mut character = character.clone();
    character.position = [10_000.0, 0.0, 10_000.0];
    *now += 100;
    game.tick(&character, *now)
        .expect("field quests synchronize");

    let hooligans = field_hooligans();
    for (guid, position) in hooligans.iter().copied() {
        character.position = position;
        *now += 100;
        let menu = game
            .handle_in_world(&npc_interact(guid), &character, *now)
            .expect("field enemy interaction is valid");
        assert!(
            menu.replies
                .iter()
                .any(|packet| packet.starts_with(&[26, 0, 9, 0])),
            "field enemy {guid} did not expose its Attack menu"
        );
        let before = quest_count(game, HOOLIGAN_FIGHT);
        for _ in 0..5 {
            *now += 700;
            game.handle_in_world(&npc_select(guid, ATTACK_CHOICE), &character, *now)
                .expect("field basic attack is valid");
            if quest_count(game, HOOLIGAN_FIGHT) > before {
                break;
            }
        }
        assert_eq!(
            quest_count(game, HOOLIGAN_FIGHT),
            before + 1,
            "actual combat did not advance the kill objective for {guid}"
        );
        // Replay the lethal request after the cooldown: the enemy is now dead, so the
        // production request path must not double-credit or touch the saved state.
        let after_kill = saved(game);
        *now += 700;
        let replayed = game
            .handle_in_world(&npc_select(guid, ATTACK_CHOICE), &character, *now)
            .expect("a replayed lethal attack is a clean refusal");
        assert!(
            !replayed.changed,
            "a replayed lethal attack changed gameplay"
        );
        assert_eq!(
            saved(game),
            after_kill,
            "a replayed lethal attack changed the saved state"
        );
        assert_eq!(quest_count(game, HOOLIGAN_FIGHT), before + 1);
    }
    assert_eq!(quest_count(game, HOOLIGAN_FIGHT), 4);

    // The authored 30 s respawn makes a fifth live 930050 kill possible. With C4's
    // objective already full, that live kill must not raise the count or change the
    // saved state; a double-credit or missing cap would fail here.
    let mut observer = character.clone();
    observer.position = [10_000.0, 0.0, 10_000.0];
    *now += 30_100;
    game.tick(&observer, *now)
        .expect("the field respawn tick is valid");
    let (respawned, position) = field_hooligans()
        .into_iter()
        .next()
        .expect("the chapter owns four field spawns");
    observer.position = position;
    *now += 100;
    interact_npc(game, &observer, respawned, *now);
    let before_live_kill = saved(game);
    let mut killed = false;
    for _ in 0..5 {
        *now += 700;
        let out = game
            .handle_in_world(&npc_select(respawned, ATTACK_CHOICE), &observer, *now)
            .expect("the respawned Hooligan attack is valid");
        if out.replies.iter().any(|packet| {
            packet.starts_with(&[35, 0, 3, 0]) && packet[6..14] == respawned.to_le_bytes()
        }) {
            killed = true;
            break;
        }
    }
    assert!(killed, "the respawned Hooligan did not die");
    assert_eq!(
        quest_count(game, HOOLIGAN_FIGHT),
        4,
        "a live respawned kill exceeded C4's required count"
    );
    assert_eq!(
        saved(game),
        before_live_kill,
        "a live respawned kill changed the completed objective"
    );
}

fn quest_count(game: &Gameplay, quest_id: u32) -> u32 {
    game.state().npc_quests.unwrap().quests[&quest_id].counts[0]
}

fn select_job(profile_id: i32) -> Vec<u8> {
    let mut packet = vec![26, 0, 13, 0];
    packet.extend(profile_id.to_le_bytes());
    packet
}

// Client constructors/readers for the packets below (IDA native map):
// 26/8 CommandPacketInteractRequest 0x008b6bc0; 26/10
// CommandPacketInteractionSelect 0x00a36f80; 26/6 PacketDialogResponse 0x009f5890;
// 49/2 QuestReplyPacket 0x00a86400; 49/13 QuestEndPacket::ReadFields 0x00c7b990 /
// Unserialize 0x00c7bbd0; 49/14 QuestEndReplyPacket 0x00a864c0; 35/2 AddNpc
// reader 0x00921a10.
fn npc_interact(npc_id: u64) -> Vec<u8> {
    // 26/8 CommandPacketInteractRequest (ctor 0x008b6bc0).
    let mut packet = vec![26, 0, 8, 0];
    packet.extend(npc_id.to_le_bytes());
    packet.push(0);
    packet
}

fn npc_select(npc_id: u64, choice: i32) -> Vec<u8> {
    // 26/10 CommandPacketInteractionSelect (ctor 0x00a36f80).
    let mut packet = vec![26, 0, 10, 0];
    packet.extend(npc_id.to_le_bytes());
    packet.extend(choice.to_le_bytes());
    packet
}

fn dialogue_choice(row: i32) -> Vec<u8> {
    // 26/6 PacketDialogResponse (ctor 0x009f5890), one i32 row.
    let mut packet = vec![26, 0, 6, 0];
    packet.extend(row.to_le_bytes());
    packet
}

fn quest_reply(quest_id: u32, accept: bool) -> Vec<u8> {
    // 49/2 QuestReplyPacket (ctor 0x00a86400): quest id then the accept byte.
    let mut packet = vec![49, 0, 2, 0, 0, 0];
    packet.extend(quest_id.to_le_bytes());
    packet.push(u8::from(accept));
    packet
}

fn quest_end_reply(quest_id: u32) -> Vec<u8> {
    // 49/14 QuestEndReplyPacket (ctor 0x00a864c0): one quest id.
    let mut packet = vec![49, 0, 14, 0, 0, 0];
    packet.extend(quest_id.to_le_bytes());
    packet
}

/// Complete C3-C5 through the production NPC menu, dialogue, offer, field combat and
/// 49/13-49/14 turn-in paths, returning the wallet before the chapter.
fn complete_chapter(game: &mut Gameplay, character: &CharacterState, now: &mut u64) -> u32 {
    let initial = game.state().inventory.coins;
    let carlos = npc_position("cobblestone_village.json", CARLOS);
    let jones = npc_position("cobblestone_village.json", JONES);
    let sabastian = npc_position("blackspore_swamp.json", SABASTIAN);
    let mut at = character.clone();

    at.position = carlos;
    open_carlos_line(game, character, now);
    accept_quest(game, &at, CARLOS, carlos, REFERRAL_TO_JONES, now);
    at.position = jones;
    let _ = open_end_screen(game, &at, JONES, REFERRAL_TO_JONES, now);
    finish_turnin(game, &at, REFERRAL_TO_JONES, now);

    accept_quest(game, &at, JONES, jones, HOOLIGAN_FIGHT, now);
    kill_hooligans(game, &at, now);
    at.position = jones;
    let _ = open_end_screen(game, &at, JONES, HOOLIGAN_FIGHT, now);
    finish_turnin(game, &at, HOOLIGAN_FIGHT, now);

    accept_quest(game, &at, JONES, jones, SEARCH_BLACKSPORE, now);
    at.position = sabastian;
    let _ = open_end_screen(game, &at, SABASTIAN, SEARCH_BLACKSPORE, now);
    finish_turnin(game, &at, SEARCH_BLACKSPORE, now);
    initial
}

#[test]
fn chapter_drives_npc_and_combat_paths_and_restores_pending_turnin() {
    let (mut game, mut character) = fresh_game();
    let mut now = 1_000;
    let carlos = npc_position("cobblestone_village.json", CARLOS);
    let jones = npc_position("cobblestone_village.json", JONES);
    let sabastian = npc_position("blackspore_swamp.json", SABASTIAN);
    let initial_coins = game.state().inventory.coins;

    character.position = carlos;
    let spawned = game
        .update_npc_visibility(&character)
        .expect("Carlos visibility updates");
    let carlos_adds: Vec<_> = spawned
        .iter()
        .filter_map(|packet| startup_world::parse_add_npc(packet).ok())
        .filter(|npc| npc.actor_id == CARLOS)
        .collect();
    assert_eq!(
        carlos_adds.len(),
        1,
        "the bound extra spawn must be the only Carlos creation; a regional copy emits two 35/2s"
    );
    assert_eq!(
        carlos_adds[0].model_id, 964,
        "Carlos keeps the extra-spawn model"
    );
    assert_eq!(
        carlos_adds[0].name_string_id, 20_671,
        "Carlos keeps the recovered locale name id"
    );
    open_carlos_line(&mut game, &character, &mut now);
    accept_quest(
        &mut game,
        &character,
        CARLOS,
        carlos,
        REFERRAL_TO_JONES,
        &mut now,
    );
    character.position = jones;
    let _ = open_end_screen(&mut game, &character, JONES, REFERRAL_TO_JONES, &mut now);
    assert_pending_screen_recovery(
        &mut game,
        &character,
        JONES,
        REFERRAL_TO_JONES,
        94_295,
        &mut now,
    );
    // Another NPC must neither reopen nor consume the pending screen. C3 is given by
    // Carlos but completed at Jones; the saved opener is Jones, not the formal giver.
    character.position = carlos;
    let before_wrong_actor = saved(&game);
    let wrong_actor = talk(&mut game, &character, CARLOS, now + 1);
    assert!(!wrong_actor.changed);
    assert!(
        !wrong_actor
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[49, 0, 13, 0])),
        "a different NPC must not reopen the pending end screen"
    );
    // The 49/13 rides the dialog choice, so the choice must not release it either.
    let refused_choice = game
        .handle_in_world(&dialogue_choice(QUEST_CHOICE), &character, now + 2)
        .expect("the wrong actor's dialog choice is valid");
    assert!(
        !refused_choice
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[49, 0, 13, 0])),
        "a different NPC must not queue the pending end screen"
    );
    assert_eq!(
        saved(&game),
        before_wrong_actor,
        "a different NPC changed the pending turn-in"
    );
    character.position = jones;
    // C3's exact five-coin payout is asserted by `finish_turnin`; the review of
    // c65b2b2 found the old test only checked the chapter total.
    finish_turnin(&mut game, &character, REFERRAL_TO_JONES, &mut now);
    assert_refusal_changes_nothing(
        &mut game,
        &character,
        &quest_end_reply(REFERRAL_TO_JONES),
        now + 1,
        "duplicate C3 turn-in",
    );
    now += 1;

    accept_quest(
        &mut game,
        &character,
        JONES,
        jones,
        HOOLIGAN_FIGHT,
        &mut now,
    );
    kill_hooligans(&mut game, &character, &mut now);

    character.position = jones;
    let end = open_end_screen(&mut game, &character, JONES, HOOLIGAN_FIGHT, &mut now);
    assert_eq!(
        u64::from_le_bytes(end[6..14].try_into().unwrap()),
        JONES,
        "the live end screen carries the Talk actor"
    );
    let before_reconnect = game.state().inventory.coins;
    let before_reopen = saved(&game);
    let reopened = open_end_screen(&mut game, &character, JONES, HOOLIGAN_FIGHT, &mut now);
    assert_eq!(
        u64::from_le_bytes(reopened[6..14].try_into().unwrap()),
        JONES,
        "talking again reopens the same end screen without another state change"
    );
    assert_eq!(game.state().inventory.coins, before_reconnect);
    assert_eq!(saved(&game), before_reopen);
    let restored_state = game.state();
    drop(game);

    // The real gateway must queue Jones's 35/2 before the 49/13 that resolves him.
    gateway_reconnect_orders_spawn_before_screen(&restored_state, &character, JONES, now + 50);

    // Disconnect after the end screen opened and before its 49/14 confirmation.
    let mut game = load_game(Some(restored_state));
    character.position = jones;
    let initialized = game
        .initialize_world(&character, now + 100)
        .expect("reconnected gameplay initializes");
    assert!(
        !initialized
            .replies
            .iter()
            .any(|packet| has_family(packet, 49, 13, Some(HOOLIGAN_FIGHT))),
        "login synchronization must not open the end screen before the speaker spawn"
    );
    let restored = game
        .update_npc_visibility(&character)
        .expect("reconnected NPC visibility updates");
    let spawn = restored
        .iter()
        .position(|packet| {
            packet.starts_with(&[35, 0, 2, 0])
                && packet.len() >= 12
                && u64::from_le_bytes(packet[4..12].try_into().unwrap()) == JONES
        })
        .expect("reconnect must spawn the turn-in speaker");
    let screen = restored
        .iter()
        .position(|packet| has_family(packet, 49, 13, Some(HOOLIGAN_FIGHT)))
        .expect("reconnect must reopen the pending end screen");
    assert!(
        spawn < screen,
        "the recovered end screen must follow the speaker spawn"
    );
    let restored_end = &restored[screen];
    assert_eq!(
        u64::from_le_bytes(restored_end[6..14].try_into().unwrap()),
        JONES,
        "the recovered end screen carries the turn-in actor for client sub_C7B990"
    );
    assert_eq!(
        u32::from_le_bytes(restored_end[14..18].try_into().unwrap()),
        HOOLIGAN_FIGHT,
        "the third dword remains the quest id that 49/14 sends back"
    );
    let before_refusal = saved(&game);
    now += 200;
    assert_refusal_changes_nothing(
        &mut game,
        &character,
        &quest_end_reply(SEARCH_BLACKSPORE),
        now,
        "wrong pending turn-in",
    );
    assert_eq!(saved(&game), before_refusal);
    now += 100;
    let paid = game
        .handle_in_world(&quest_end_reply(HOOLIGAN_FIGHT), &character, now)
        .expect("recovered turn-in is valid");
    assert!(paid.changed);
    assert_eq!(
        game.state().inventory.coins,
        before_reconnect + QUEST_REWARD_COINS
    );
    assert_eq!(
        game.state().npc_quests.unwrap().quests[&HOOLIGAN_FIGHT].status,
        QuestStatus::Completed
    );
    assert_eq!(game.state().npc_quests.unwrap().pending_turnin, None);
    now += 100;
    let coins_after_c4 = game.state().inventory.coins;
    assert_refusal_changes_nothing(
        &mut game,
        &character,
        &quest_end_reply(HOOLIGAN_FIGHT),
        now,
        "duplicate reward before reload",
    );
    game = load_game(Some(game.state()));
    character.position = jones;
    now += 100;
    game.initialize_world(&character, now)
        .expect("completed chapter reload initializes");
    now += 100;
    assert_refusal_changes_nothing(
        &mut game,
        &character,
        &quest_end_reply(HOOLIGAN_FIGHT),
        now,
        "duplicate reward after reload",
    );
    assert_eq!(
        game.state().inventory.coins,
        coins_after_c4,
        "a replayed turn-in after reload paid again"
    );

    accept_quest(
        &mut game,
        &character,
        JONES,
        jones,
        SEARCH_BLACKSPORE,
        &mut now,
    );
    character.position = sabastian;
    let _ = open_end_screen(
        &mut game,
        &character,
        SABASTIAN,
        SEARCH_BLACKSPORE,
        &mut now,
    );
    assert_pending_screen_recovery(
        &mut game,
        &character,
        SABASTIAN,
        SEARCH_BLACKSPORE,
        103_354,
        &mut now,
    );
    finish_turnin(&mut game, &character, SEARCH_BLACKSPORE, &mut now);

    assert_eq!(
        game.state().inventory.coins,
        initial_coins + 3 * QUEST_REWARD_COINS
    );
    let quests = game.state().npc_quests.unwrap();
    for quest_id in [REFERRAL_TO_JONES, HOOLIGAN_FIGHT, SEARCH_BLACKSPORE] {
        assert_eq!(
            quests.quests[&quest_id].status,
            QuestStatus::Completed,
            "quest {quest_id} did not reach the final outcome"
        );
    }

    // Completed chapter state must survive serialization and a fresh gameplay load.
    let reloaded = load_game(Some(game.state()));
    let reloaded = reloaded.state();
    assert_eq!(
        reloaded.inventory.coins,
        initial_coins + 3 * QUEST_REWARD_COINS
    );
    let quests = reloaded.npc_quests.unwrap();
    assert_eq!(quests.pending_turnin, None);
    for quest_id in [REFERRAL_TO_JONES, HOOLIGAN_FIGHT, SEARCH_BLACKSPORE] {
        assert_eq!(
            quests.quests[&quest_id].status,
            QuestStatus::Completed,
            "quest {quest_id} was not restored as completed"
        );
    }
}

#[test]
fn unoffered_menu_choice_changes_nothing_and_keeps_the_menu_open() {
    let (mut game, _) = fresh_game();
    let character = character_at(npc_position("cobblestone_village.json", CARLOS));
    game.update_npc_visibility(&character)
        .expect("Carlos visibility updates");
    interact_npc(&mut game, &character, CARLOS, 1_000);
    let before = saved(&game);
    let refused = game.handle_in_world(&npc_select(CARLOS, i32::MAX), &character, 1_001);
    assert!(
        refused.is_ok_and(|out| out.replies.is_empty() && !out.changed),
        "an unoffered menu choice is refused without ending the session (T836)"
    );
    assert_eq!(
        saved(&game),
        before,
        "a refused menu choice changes nothing"
    );
    let talked = game
        .handle_in_world(&npc_select(CARLOS, 1), &character, 1_002)
        .expect("the open menu still accepts Talk");
    assert!(
        talked
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[26, 0, 3, 0])),
        "a refused choice must not consume the open menu"
    );
}

#[test]
fn shared_npc_view_replays_pending_end_screen_after_the_actor_spawn() {
    // Production fans NPC AddNpc packets out from the shared tick, not from
    // `update_npc_visibility`, so the replay must ride that same batch.
    let (game, _) = fresh_game();
    let mut state = game.state();
    let quests = state.npc_quests.as_mut().unwrap();
    quests.quests.insert(
        REFERRAL_TO_JONES,
        QuestProgress {
            status: QuestStatus::Active,
            counts: vec![1],
        },
    );
    quests.pending_turnin = Some(REFERRAL_TO_JONES);
    quests.pending_turnin_actor = Some(JONES);
    let mut game = load_game(Some(state));
    game.take_shared_fields()
        .expect("the shared field runtime is available");
    let character = character_at(npc_position("cobblestone_village.json", JONES));
    let known = BTreeSet::new();
    let (visible, packets) = game
        .shared_npc_view(&character, &[], &known)
        .expect("shared NPC view builds");
    let actor_spawn = packets
        .iter()
        .position(|packet| {
            packet.starts_with(&[35, 0, 2, 0])
                && startup_world::parse_add_npc(packet).is_ok_and(|npc| npc.actor_id == JONES)
        })
        .expect("the turn-in actor is spawned");
    let end_screen = packets
        .iter()
        .position(|packet| packet.starts_with(&[49, 0, 13, 0]))
        .expect("the shared tick reopens the pending end screen");
    assert!(
        actor_spawn < end_screen,
        "the shared replay must follow the actor AddNpc"
    );
    assert_eq!(
        u64::from_le_bytes(packets[end_screen][6..14].try_into().unwrap()),
        JONES,
        "the shared replay names the actor that opened the screen"
    );
    assert_eq!(
        u32::from_le_bytes(packets[end_screen][14..18].try_into().unwrap()),
        REFERRAL_TO_JONES
    );
    let (_, again) = game
        .shared_npc_view(&character, &[], &visible)
        .expect("known actors do not replay");
    assert!(
        !again
            .iter()
            .any(|packet| packet.starts_with(&[49, 0, 13, 0])),
        "the replay fires once per visibility transition"
    );
}

#[test]
fn disabled_starting_quests_refuse_without_changing_state() {
    let mut engine =
        QuestEngine::from_split_content(&content_dir(), CHARACTER, eligibility(), None).unwrap();
    let before = engine.state().clone();
    // The still-hollow starting records refuse, and their provenance must not claim objectives
    // the server does not run (F135 review finding 4). T252 enabled C1/C2, so C1 now offers and
    // C2 is gated on C1's completion instead. T331 enabled C6, so it left this list: without C5
    // the engine refuses it on the prerequisite, and the wrong NPC is a wrong giver.
    for quest_id in [3_030_001, 3_030_002, 3_030_003, 3_030_010] {
        let refused = engine.offer(quest_id, CARLOS);
        assert_eq!(refused.denial, Some(QuestDenial::Unavailable));
        assert!(!refused.changed);
        assert!(refused.outbound.is_empty());
        assert_eq!(engine.state(), &before);
    }
    let early = engine.offer(GIVEN_TO_GUARD, SABASTIAN);
    assert_eq!(early.denial, Some(QuestDenial::Prerequisite));
    assert!(!early.changed);
    assert!(early.outbound.is_empty());
    assert_eq!(engine.state(), &before);
    let wrong_giver = engine.offer(GIVEN_TO_GUARD, CARLOS);
    assert_eq!(wrong_giver.denial, Some(QuestDenial::WrongGiver));
    assert!(!wrong_giver.changed);
    assert!(wrong_giver.outbound.is_empty());
    assert_eq!(engine.state(), &before);
    let gated = engine.offer(3_030_005, CARLOS);
    assert_eq!(gated.denial, Some(QuestDenial::Prerequisite));
    assert!(!gated.changed);
    assert!(gated.outbound.is_empty());
    assert_eq!(engine.state(), &before);
    let offered = engine.offer(3_030_004, CARLOS);
    assert_eq!(
        offered.denial, None,
        "T252's C1 is a real, offerable record"
    );
    assert!(offered.changed);
    assert_eq!(
        engine.state().quests[&3_030_004].status,
        QuestStatus::Offered
    );
}

#[test]
fn carlos_to_jones_to_blackspore_completes_in_order() {
    let (mut game, character) = fresh_game();
    let mut now = 5_000;
    let initial = complete_chapter(&mut game, &character, &mut now);

    let state = game.state();
    assert_eq!(state.inventory.coins, initial + 3 * QUEST_REWARD_COINS);
    let quests = state
        .npc_quests
        .as_ref()
        .expect("chapter quest state exists");
    for quest_id in [REFERRAL_TO_JONES, HOOLIGAN_FIGHT, SEARCH_BLACKSPORE] {
        assert_eq!(
            quests.quests[&quest_id].status,
            QuestStatus::Completed,
            "quest {quest_id} did not complete"
        );
    }
    assert_eq!(quests.pending_turnin, None);
}

#[test]
fn wrong_npc_and_wrong_enemy_do_not_advance() {
    // Carlos gave C3 but is not its Jones turn-in step. C1 is on its Mongo step (T252) and C2
    // waits on C1's completion, so Carlos has nothing left to offer: this is a genuine refusal
    // rather than another NPC's legitimate quest choice.
    let (mut game, character) = fresh_game();
    let mut now = 6_000;
    let carlos = npc_position("cobblestone_village.json", CARLOS);
    let jones = npc_position("cobblestone_village.json", JONES);
    let mut at = character.clone();

    at.position = carlos;
    open_carlos_line(&mut game, &at, &mut now);
    accept_quest(&mut game, &at, CARLOS, carlos, REFERRAL_TO_JONES, &mut now);

    // The wrong NPC's menu, Talk and dialogue row must leave the whole saved state
    // byte-identical, not merely leave C3's counts alone.
    let before_wrong_npc = saved(&game);
    now += 100;
    let _ = interact_npc(&mut game, &at, CARLOS, now);
    assert_eq!(
        saved(&game),
        before_wrong_npc,
        "opening the wrong NPC menu changed the saved state"
    );
    now += 100;
    assert_refusal_changes_nothing(
        &mut game,
        &at,
        &npc_select(CARLOS, 1),
        now,
        "wrong NPC Talk",
    );
    now += 100;
    assert_refusal_changes_nothing(
        &mut game,
        &at,
        &dialogue_choice(QUEST_CHOICE),
        now,
        "wrong NPC dialogue row",
    );
    assert_eq!(
        saved(&game),
        before_wrong_npc,
        "the wrong NPC exchange changed the saved state"
    );
    let state = game.state();
    let quests = state.npc_quests.as_ref().unwrap();
    assert_eq!(
        quests.quests[&REFERRAL_TO_JONES].counts,
        vec![0],
        "another NPC advanced the Jones talk step"
    );
    assert_eq!(
        quests.quests[&REFERRAL_TO_JONES].status,
        QuestStatus::Active
    );
    assert_eq!(quests.pending_turnin, None);

    // Complete C3 and accept C4 so the four-kill step is active.
    at.position = jones;
    let _ = open_end_screen(&mut game, &at, JONES, REFERRAL_TO_JONES, &mut now);
    finish_turnin(&mut game, &at, REFERRAL_TO_JONES, &mut now);
    accept_quest(&mut game, &at, JONES, jones, HOOLIGAN_FIGHT, &mut now);

    // A spawned enemy with another defeat definition dies through the same production
    // menu/attack/defeat-forwarding path. Its kill must not advance C4, and the whole
    // saved state must stay byte-identical.
    let (wrong, position, definition) = ungated_enemy_other_than_hooligans();
    at.position = position;
    let before_wrong_enemy = saved(&game);
    now += 100;
    interact_npc(&mut game, &at, wrong, now);
    let mut defeated = false;
    for _ in 0..5 {
        now += 700;
        let killed = game
            .handle_in_world(&npc_select(wrong, ATTACK_CHOICE), &at, now)
            .expect("field basic attack is valid");
        if killed.replies.iter().any(|packet| {
            packet.starts_with(&[35, 0, 3, 0]) && packet[6..14] == wrong.to_le_bytes()
        }) {
            defeated = true;
            break;
        }
    }
    assert!(
        defeated,
        "the wrong enemy did not die, so no defeat reached the quest engine"
    );
    assert_eq!(
        quest_count(&game, HOOLIGAN_FIGHT),
        0,
        "a {definition} kill advanced the 930050 objective"
    );
    let state = game.state();
    let quests = state.npc_quests.as_ref().unwrap();
    assert_eq!(quests.quests[&HOOLIGAN_FIGHT].status, QuestStatus::Active);
    assert_eq!(quests.pending_turnin, None);
    assert_eq!(
        saved(&game),
        before_wrong_enemy,
        "killing another enemy changed the saved state"
    );
}

#[test]
fn chapter_rewards_are_paid_once() {
    let (mut game, character) = fresh_game();
    let mut now = 7_000;
    let initial = complete_chapter(&mut game, &character, &mut now);
    let total = initial + 3 * QUEST_REWARD_COINS;
    assert_eq!(game.state().inventory.coins, total);

    for quest_id in [REFERRAL_TO_JONES, HOOLIGAN_FIGHT, SEARCH_BLACKSPORE] {
        assert_refusal_changes_nothing(
            &mut game,
            &character,
            &quest_end_reply(quest_id),
            now + 1,
            "duplicate chapter turn-in",
        );
        now += 2;
    }
    assert_refusal_changes_nothing(
        &mut game,
        &character,
        &quest_reply(HOOLIGAN_FIGHT, true),
        now + 1,
        "duplicate chapter accept",
    );
    assert_eq!(game.state().inventory.coins, total);
}

#[test]
fn offer_gates_and_save_reload_survive() {
    let (mut game, character) = fresh_game();
    let mut now = 8_000;
    let initial = complete_chapter(&mut game, &character, &mut now);

    let mut reloaded = load_game(Some(game.state()));
    let state = reloaded.state();
    assert_eq!(state.inventory.coins, initial + 3 * QUEST_REWARD_COINS);
    let quests = state
        .npc_quests
        .as_ref()
        .expect("completed quest state persists");
    assert_eq!(quests.pending_turnin, None);
    for quest_id in [REFERRAL_TO_JONES, HOOLIGAN_FIGHT, SEARCH_BLACKSPORE] {
        assert_eq!(
            quests.quests[&quest_id].status,
            QuestStatus::Completed,
            "quest {quest_id} history did not survive the save"
        );
    }

    // The disabled C1/C2 rows must stay unofferable through the real Carlos menu:
    // Talk plus the dialogue row is the production path that would emit 49/1.
    let carlos = npc_position("cobblestone_village.json", CARLOS);
    let mut at = character.clone();
    at.position = carlos;
    let before_menu = saved(&reloaded);
    now += 100;
    let dialogue = talk(&mut reloaded, &at, CARLOS, now);
    assert!(
        !dialogue
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[49, 0, 1, 0])),
        "the completed chapter reopened a quest offer"
    );
    now += 100;
    let choice = choose_quest_row(&mut reloaded, &at, now);
    for quest_id in [3_030_004, 3_030_005] {
        assert!(
            !choice
                .replies
                .iter()
                .any(|packet| has_family(packet, 49, 1, Some(quest_id))),
            "disabled quest {quest_id} was offered through Carlos's menu"
        );
    }
    assert_eq!(
        saved(&reloaded),
        before_menu,
        "an unofferable Carlos Talk changed the completed chapter"
    );
}

/// T331: C6 is the Cobblestone line's Brawler hand-off. Only an active, unlocked Brawler that
/// has finished C5 is offered it, and the offer is the ordinary 26/8 -> 26/10 -> 26/6 -> 49/1
/// path at Sabastian.
#[test]
fn sabastian_offers_given_to_guard_after_honor_lost() {
    let (mut game, character) = fresh_game();
    let mut now = 20_000;
    let sabastian = npc_position("blackspore_swamp.json", SABASTIAN);
    let mut at = character.clone();
    at.position = sabastian;

    // A Brawler who has not completed C5 gets no offer: the prerequisite is real.
    let before = saved(&game);
    now += 100;
    let _ = talk(&mut game, &at, SABASTIAN, now);
    now += 100;
    let early = choose_quest_row(&mut game, &at, now);
    assert!(
        !early
            .replies
            .iter()
            .any(|packet| has_family(packet, 49, 1, Some(GIVEN_TO_GUARD))),
        "Given to Guard was offered before Bad Business completed"
    );
    assert_eq!(saved(&game), before, "the early Talk changed the save");

    complete_chapter(&mut game, &character, &mut now);
    at.position = sabastian;

    // A non-Brawler is not offered the Brawler hand-off. The captured opening inventory has no
    // Adventurer outfit, so the active profile is seeded in the save instead of a 26/13 switch.
    let mut adventurer_game = with_active_job(&game, CHARACTER, ADVENTURER);
    assert_eq!(active_job(&adventurer_game), ADVENTURER);
    let before_adventurer = saved(&adventurer_game);
    now += 100;
    let _ = talk(&mut adventurer_game, &at, SABASTIAN, now);
    now += 100;
    let adventurer = choose_quest_row(&mut adventurer_game, &at, now);
    assert!(
        !adventurer
            .replies
            .iter()
            .any(|packet| has_family(packet, 49, 1, Some(GIVEN_TO_GUARD))),
        "a non-Brawler was offered the Brawler hand-off"
    );
    assert_eq!(
        saved(&adventurer_game),
        before_adventurer,
        "a non-Brawler changed the save"
    );

    // The active, unlocked Brawler gets it through the real menu, dialogue and reply path.
    now += 100;
    let menu = interact_npc(&mut game, &at, SABASTIAN, now);
    assert!(
        menu.starts_with(&[26, 0, 9, 0]),
        "Sabastian did not open his 26/9 menu"
    );
    now += 100;
    let dialogue = game
        .handle_in_world(&npc_select(SABASTIAN, 1), &at, now)
        .expect("Sabastian's Talk selection is valid");
    assert!(
        dialogue
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[26, 0, 3, 0])),
        "Sabastian's Talk did not open dialogue"
    );
    now += 100;
    let offered = choose_quest_row(&mut game, &at, now);
    assert!(
        offered
            .replies
            .iter()
            .any(|packet| has_family(packet, 49, 1, Some(GIVEN_TO_GUARD))),
        "Sabastian did not offer Given to Guard"
    );

    // A second Talk while the offer window is open changes nothing and reoffers nothing.
    let before_second = saved(&game);
    now += 100;
    let _ = talk(&mut game, &at, SABASTIAN, now);
    now += 100;
    let again = choose_quest_row(&mut game, &at, now);
    assert!(
        !again
            .replies
            .iter()
            .any(|packet| has_family(packet, 49, 1, Some(GIVEN_TO_GUARD))),
        "a second Talk reoffered the pending quest"
    );
    assert_eq!(
        saved(&game),
        before_second,
        "a second Talk changed the pending offer"
    );

    now += 100;
    let accepted = game
        .handle_in_world(&quest_reply(GIVEN_TO_GUARD, true), &at, now)
        .expect("the C6 accept is valid");
    assert!(
        accepted
            .replies
            .iter()
            .any(|packet| has_family(packet, 49, 3, None)),
        "the C6 accept did not send QuestAdd"
    );
    assert_eq!(
        game.state().npc_quests.unwrap().quests[&GIVEN_TO_GUARD].status,
        QuestStatus::Active
    );
}

/// T331: the C6 delivery is the real Petra actor: the Talk completes the only objective, the
/// 49/13 names the drawn actor, and 49/14 pays exactly five coins and 25 Brawler XP once.
#[test]
fn delivering_the_offer_to_petra_completes_and_pays_once() {
    let (mut game, character) = fresh_game();
    let mut now = 22_000;
    complete_chapter_and_accept_c6(&mut game, &character, &mut now);
    let at = at_petra(&character);

    // A player standing on the drawn actor's spawn point -- not only on the authored content
    // row `at_petra` uses -- can click her: 26/8 must pass the production 12 m reach
    // (`npc_interaction::check_range`) and open her 26/9 menu. `interact_npc` panics otherwise.
    let mut at_spawn = character.clone();
    at_spawn.position = petra_spawn_position();
    now += 100;
    interact_npc(&mut game, &at_spawn, PETRA, now);

    let coins_before = game.state().inventory.coins;
    let inventory_before = game.state().inventory.clone();
    let xp_before = profile_xp(&game, BRAWLER);
    // The 26/8 click resolves the drawable actor's content key before anything else, so a
    // missing or unbound Petra spawn fails here (canary 1).
    let end = open_end_screen(&mut game, &at, PETRA, GIVEN_TO_GUARD, &mut now);
    let drawn = game
        .update_npc_visibility(&at)
        .expect("Petra visibility updates");
    let petra = drawn
        .iter()
        .filter_map(|packet| startup_world::parse_add_npc(packet).ok())
        .find(|npc| npc.actor_id == PETRA)
        .expect("the drawn Petra actor must exist at her spawn");
    assert_eq!(petra.model_id, PETRA_MODEL);
    assert_eq!(petra.name_string_id, PETRA_NAME_STRING_ID);
    assert_eq!(
        u64::from_le_bytes(end[6..14].try_into().unwrap()),
        PETRA,
        "the delivered offer's end screen names the drawn Petra actor"
    );
    finish_turnin(&mut game, &at, GIVEN_TO_GUARD, &mut now);

    let state = game.state();
    assert_eq!(
        state.inventory.coins,
        coins_before + QUEST_REWARD_COINS,
        "C6 must pay exactly five coins"
    );
    assert_eq!(
        profile_xp(&game, BRAWLER),
        xp_before + QUEST_REWARD_JOB_XP,
        "C6 must pay 25 Brawler XP"
    );
    assert_eq!(
        state.inventory.items, inventory_before.items,
        "C6 must not grant an item"
    );
    assert_eq!(
        state.inventory.equipment, inventory_before.equipment,
        "C6 must not grant equipment"
    );
    let quests = state.npc_quests.as_ref().unwrap();
    assert_eq!(
        quests.quests[&GIVEN_TO_GUARD].status,
        QuestStatus::Completed
    );
    assert_eq!(quests.pending_turnin, None);

    // A replayed 49/14 and a replayed 49/2 pay nothing.
    let paid = saved(&game);
    now += 100;
    assert_refusal_changes_nothing(
        &mut game,
        &at,
        &quest_end_reply(GIVEN_TO_GUARD),
        now,
        "replayed C6 turn-in",
    );
    now += 100;
    assert_refusal_changes_nothing(
        &mut game,
        &at,
        &quest_reply(GIVEN_TO_GUARD, true),
        now,
        "replayed C6 accept",
    );
    assert_eq!(saved(&game), paid);
    assert_eq!(
        game.state().inventory.coins,
        coins_before + QUEST_REWARD_COINS
    );
}

/// T331 edge case 1: the explicit `{"43": 25}` map pays profile 43 even after a job switch.
#[test]
fn the_reward_goes_to_profile_43_after_a_job_switch() {
    let (mut game, character) = fresh_game();
    let mut now = 24_000;
    complete_chapter_and_accept_c6(&mut game, &character, &mut now);
    let at = at_petra(&character);

    // A real 26/13 switch needs another job's outfit; the captured opening inventory only has
    // the Brawler set, so the active profile changes through the saved state (T202's `seeded`).
    game = with_active_job(&game, CHARACTER, ADVENTURER);
    assert_eq!(active_job(&game), ADVENTURER);
    let adventurer_xp = profile_xp(&game, ADVENTURER);
    let brawler_xp = profile_xp(&game, BRAWLER);

    let _ = open_end_screen(&mut game, &at, PETRA, GIVEN_TO_GUARD, &mut now);
    finish_turnin(&mut game, &at, GIVEN_TO_GUARD, &mut now);

    assert_eq!(
        profile_xp(&game, BRAWLER),
        brawler_xp + QUEST_REWARD_JOB_XP,
        "profile 43 must receive the configured XP"
    );
    assert_eq!(
        profile_xp(&game, ADVENTURER),
        adventurer_xp,
        "the active job must receive no XP"
    );
    assert_eq!(
        active_job(&game),
        ADVENTURER,
        "the turn-in must not switch jobs"
    );
}

/// T331 test 4: wrong target, forged replies, the near-cap wallet and a replay each leave the
/// whole saved `GameplayState` equal; nothing pays or completes on a refusal.
#[test]
fn refusals_wrong_target_and_replay_change_nothing() {
    let (mut game, character) = fresh_game();
    let mut now = 26_000;
    complete_chapter(&mut game, &character, &mut now);
    let sabastian = npc_position("blackspore_swamp.json", SABASTIAN);
    let carlos = npc_position("cobblestone_village.json", CARLOS);
    let spawn = petra_spawn_position();
    let mut at = character.clone();

    // A forged accept before any offer changes nothing.
    at.position = sabastian;
    now += 100;
    assert_refusal_changes_nothing(
        &mut game,
        &at,
        &quest_reply(GIVEN_TO_GUARD, true),
        now,
        "forged C6 accept without an offer",
    );

    // Carlos is not C6's target: his Talk and dialogue row change nothing.
    at.position = carlos;
    let before_carlos = saved(&game);
    now += 100;
    let _ = talk(&mut game, &at, CARLOS, now);
    now += 100;
    let _ = choose_quest_row(&mut game, &at, now);
    assert_eq!(
        saved(&game),
        before_carlos,
        "a wrong-NPC Talk advanced the C6 delivery"
    );

    // An out-of-range 26/8 at Petra is refused before anything changes.
    let far = CharacterState {
        position: [spawn[0] + 400.0, spawn[1], spawn[2]],
        ..character.clone()
    };
    now += 100;
    let before_far = saved(&game);
    assert!(
        game.handle_in_world(&npc_interact(PETRA), &far, now)
            .is_ok_and(|out| out.replies.is_empty() && !out.changed),
        "a far interact must be refused"
    );
    assert_eq!(
        saved(&game),
        before_far,
        "the far interact changed the save"
    );

    // The legitimate accept, then replies with no completed objective.
    at.position = sabastian;
    accept_quest(
        &mut game,
        &character,
        SABASTIAN,
        sabastian,
        GIVEN_TO_GUARD,
        &mut now,
    );
    now += 100;
    assert_refusal_changes_nothing(
        &mut game,
        &at,
        &quest_end_reply(GIVEN_TO_GUARD),
        now,
        "C6 turn-in with no end screen",
    );
    now += 100;
    assert_refusal_changes_nothing(
        &mut game,
        &at,
        &quest_end_reply(SEARCH_BLACKSPORE),
        now,
        "wrong quest turn-in",
    );

    // Complete the delivery at the drawable Petra actor and hold her end screen.
    let petra_at = at_petra(&character);
    let _ = game
        .update_npc_visibility(&petra_at)
        .expect("Petra visibility updates");
    let _ = open_end_screen(&mut game, &petra_at, PETRA, GIVEN_TO_GUARD, &mut now);
    let pending = saved(&game);
    now += 100;
    assert_refusal_changes_nothing(
        &mut game,
        &petra_at,
        &quest_end_reply(SEARCH_BLACKSPORE),
        now,
        "wrong quest with the C6 screen open",
    );
    assert_eq!(
        game.state().npc_quests.as_ref().unwrap().pending_turnin,
        Some(GIVEN_TO_GUARD)
    );
    assert_eq!(saved(&game), pending);

    // Near cap: the checked coin grant refuses for this player without paying anything.
    let wallet = game.state().inventory.coins;
    let mut capped = game.state();
    capped.inventory.coins = i32::MAX as u32 - 2;
    let mut capped = load_game(Some(capped));
    let capped_before = saved(&capped);
    let capped_xp = profile_xp(&capped, BRAWLER);
    let capped_items = capped.state().inventory.items.clone();
    now += 100;
    // T382: a per-player denial, not a session-ending error or a panic.
    let refused = capped
        .handle_in_world(&quest_end_reply(GIVEN_TO_GUARD), &petra_at, now)
        .expect("a near-cap wallet is a refusal, not an error");
    assert!(
        !refused.changed,
        "a near-cap wallet must refuse the turn-in"
    );
    assert_eq!(
        saved(&capped),
        capped_before,
        "the near-cap refusal changed the save"
    );
    assert_eq!(
        capped.state().npc_quests.as_ref().unwrap().pending_turnin,
        Some(GIVEN_TO_GUARD),
        "the near-cap refusal consumed the end screen"
    );
    assert_eq!(capped.state().inventory.coins, i32::MAX as u32 - 2);
    assert_eq!(capped.state().inventory.items, capped_items);
    assert_eq!(profile_xp(&capped, BRAWLER), capped_xp);

    // The same save with the wallet restored pays exactly once.
    let mut restored = capped.state();
    restored.inventory.coins = wallet;
    let mut game = load_game(Some(restored));
    now += 100;
    let paid = game
        .handle_in_world(&quest_end_reply(GIVEN_TO_GUARD), &petra_at, now)
        .expect("the restored wallet pays the turn-in");
    assert!(paid.changed);
    assert_eq!(game.state().inventory.coins, wallet + QUEST_REWARD_COINS);
    assert_eq!(profile_xp(&game, BRAWLER), capped_xp + QUEST_REWARD_JOB_XP);
    assert_eq!(
        game.state().npc_quests.as_ref().unwrap().quests[&GIVEN_TO_GUARD].status,
        QuestStatus::Completed
    );
    let paid = saved(&game);
    for (packet, label) in [
        (quest_end_reply(GIVEN_TO_GUARD), "replayed C6 turn-in"),
        (quest_reply(GIVEN_TO_GUARD, true), "replayed C6 accept"),
    ] {
        now += 100;
        assert_refusal_changes_nothing(&mut game, &petra_at, &packet, now, label);
    }
    assert_eq!(saved(&game), paid);
}

/// T331 edge case 2: the accepted state, the counter and the pending end screen are all in the
/// character save; the grant key pays once across a reload.
#[test]
fn progress_survives_a_reload() {
    let (mut game, character) = fresh_game();
    let mut now = 30_000;
    complete_chapter_and_accept_c6(&mut game, &character, &mut now);
    let accepted = saved(&game);

    // Reload after accept: the quest is active with its counter at zero and no offer pending.
    game = load_game(Some(game.state()));
    assert_eq!(saved(&game), accepted, "the accept save changed on load");
    let accepted_state = game.state();
    let quests = accepted_state.npc_quests.as_ref().unwrap();
    assert_eq!(quests.quests[&GIVEN_TO_GUARD].status, QuestStatus::Active);
    assert_eq!(quests.quests[&GIVEN_TO_GUARD].counts, vec![0]);
    assert_eq!(quests.pending_turnin, None);

    // Reload after the Petra Talk: the completed counter and the pending end screen persist.
    let at = at_petra(&character);
    let _ = open_end_screen(&mut game, &at, PETRA, GIVEN_TO_GUARD, &mut now);
    let pending = saved(&game);
    game = load_game(Some(game.state()));
    assert_eq!(saved(&game), pending, "the pending save changed on load");
    let pending_state = game.state();
    let quests = pending_state.npc_quests.as_ref().unwrap();
    assert_eq!(quests.quests[&GIVEN_TO_GUARD].counts, vec![1]);
    assert_eq!(quests.pending_turnin, Some(GIVEN_TO_GUARD));

    let coins_before = game.state().inventory.coins;
    let xp_before = profile_xp(&game, BRAWLER);
    finish_turnin(&mut game, &at, GIVEN_TO_GUARD, &mut now);
    let paid = saved(&game);

    // Reload after payment: Completed, the coins and the XP persist; a replay pays nothing.
    game = load_game(Some(game.state()));
    assert_eq!(saved(&game), paid, "the paid save changed on load");
    let state = game.state();
    assert_eq!(
        state.npc_quests.as_ref().unwrap().quests[&GIVEN_TO_GUARD].status,
        QuestStatus::Completed
    );
    assert_eq!(state.npc_quests.as_ref().unwrap().pending_turnin, None);
    assert_eq!(state.inventory.coins, coins_before + QUEST_REWARD_COINS);
    assert_eq!(profile_xp(&game, BRAWLER), xp_before + QUEST_REWARD_JOB_XP);
    now += 100;
    assert_refusal_changes_nothing(
        &mut game,
        &at,
        &quest_end_reply(GIVEN_TO_GUARD),
        now,
        "replayed C6 turn-in after relog",
    );
    assert_eq!(saved(&game), paid);
}

/// T331 edge case 5: two Brawlers own their offer, counter and reward; one player's refusal
/// changes neither the other player nor the first player's held end screen.
#[test]
fn two_brawlers_progress_independently() {
    let (mut first, first_character) = fresh_game_for(CHARACTER);
    let (mut second, second_character) = fresh_game_for(SECOND_CHARACTER);
    let mut now = 34_000;
    complete_chapter_and_accept_c6(&mut first, &first_character, &mut now);
    complete_chapter_and_accept_c6(&mut second, &second_character, &mut now);
    let first_at = at_petra(&first_character);
    let second_at = at_petra(&second_character);

    // The first player's Talk and held screen leave the second player untouched.
    let second_before = saved(&second);
    let _ = first
        .update_npc_visibility(&first_at)
        .expect("first Petra visibility updates");
    let _ = open_end_screen(&mut first, &first_at, PETRA, GIVEN_TO_GUARD, &mut now);
    assert_eq!(
        saved(&second),
        second_before,
        "the first player's Talk changed the second player"
    );

    // A near-cap wallet refuses the first player's turn-in and changes nothing.
    let first_wallet = first.state().inventory.coins;
    let mut capped = first.state();
    capped.inventory.coins = i32::MAX as u32 - 2;
    first = load_game_for(CHARACTER, Some(capped));
    let capped_before = saved(&first);
    now += 100;
    assert!(
        !first
            .handle_in_world(&quest_end_reply(GIVEN_TO_GUARD), &first_at, now)
            .expect("a near-cap wallet is a refusal, not an error")
            .changed,
        "the near-cap wallet must refuse the first player's turn-in"
    );
    assert_eq!(saved(&first), capped_before);
    assert_eq!(saved(&second), second_before);

    // The second player turns in normally and is paid once.
    let _ = second
        .update_npc_visibility(&second_at)
        .expect("second Petra visibility updates");
    let _ = open_end_screen(&mut second, &second_at, PETRA, GIVEN_TO_GUARD, &mut now);
    let second_coins = second.state().inventory.coins;
    let second_xp = profile_xp(&second, BRAWLER);
    finish_turnin(&mut second, &second_at, GIVEN_TO_GUARD, &mut now);
    assert_eq!(
        second.state().inventory.coins,
        second_coins + QUEST_REWARD_COINS
    );
    assert_eq!(
        profile_xp(&second, BRAWLER),
        second_xp + QUEST_REWARD_JOB_XP
    );
    assert_eq!(
        second.state().npc_quests.as_ref().unwrap().quests[&GIVEN_TO_GUARD].status,
        QuestStatus::Completed
    );
    assert_eq!(
        saved(&first),
        capped_before,
        "the second player's payment changed the first player"
    );

    // The first player's own save retries and pays once after the wallet is restored.
    let mut restored = first.state();
    restored.inventory.coins = first_wallet;
    first = load_game_for(CHARACTER, Some(restored));
    let first_xp = profile_xp(&first, BRAWLER);
    now += 100;
    let paid = first
        .handle_in_world(&quest_end_reply(GIVEN_TO_GUARD), &first_at, now)
        .expect("the first player's restored wallet pays");
    assert!(paid.changed);
    assert_eq!(
        first.state().inventory.coins,
        first_wallet + QUEST_REWARD_COINS
    );
    assert_eq!(profile_xp(&first, BRAWLER), first_xp + QUEST_REWARD_JOB_XP);
    assert_eq!(
        first.state().npc_quests.as_ref().unwrap().character_id,
        CHARACTER
    );
    assert_eq!(
        second.state().npc_quests.as_ref().unwrap().character_id,
        SECOND_CHARACTER
    );
    // Each session still holds only its own paid quest.
    let second_paid = saved(&second);
    now += 100;
    assert_refusal_changes_nothing(
        &mut first,
        &first_at,
        &quest_end_reply(GIVEN_TO_GUARD),
        now,
        "the first player's replay",
    );
    assert_eq!(saved(&second), second_paid);
}

/// T331 test 7: one unique drawable Petra row reaches the production 35/2 path for two
/// independent sessions, stands on standable ground, and resolves Sabastian, Petra and C6.
#[test]
fn the_spawn_and_content_are_reachable() {
    let row = petra_spawn_row();
    assert_eq!(row["content_key"].as_u64(), Some(PETRA));
    assert_eq!(row["legacy_npc_id"].as_u64(), Some(PETRA_LEGACY_NPC_ID));
    assert_eq!(row["model_id"].as_i64(), Some(i64::from(PETRA_MODEL)));
    assert_eq!(
        row["model_file_name"].as_str(),
        Some("fairy_f_freestyle_01.agr")
    );
    assert_eq!(row["name"].as_str(), Some("Petra"));
    assert_eq!(
        row["name_string_id"].as_i64(),
        Some(i64::from(PETRA_NAME_STRING_ID))
    );
    assert_eq!(row["heading"].as_f64(), Some(0.0));
    assert!(row["capture_match"].is_null());
    let spawn = petra_spawn_position();
    assert!(spawn.iter().all(|axis| axis.is_finite()));

    // Unique ids: no second row aliases Petra's actor id or content key.
    let document: Value =
        serde_json::from_slice(&fs::read(content_dir().join("npc_spawns_extra.json")).unwrap())
            .unwrap();
    let rows = document["records"].as_array().unwrap();
    assert_eq!(
        rows.iter()
            .filter(|row| row["spawn_guid"].as_u64() == Some(PETRA))
            .count(),
        1,
        "Petra's actor id is unique"
    );
    assert_eq!(
        rows.iter()
            .filter(|row| row["content_key"].as_u64() == Some(PETRA))
            .count(),
        1,
        "Petra's content key is unique"
    );

    // The production draw set binds the actor to her content key, and two independent
    // sessions receive the same 35/2 actor name/model/position.
    let visibility = NpcVisibility::load(&content_dir()).expect("NPC visibility loads");
    assert_eq!(visibility.content_key(PETRA), Some(PETRA));
    let mut drawn = Vec::new();
    for character_id in [CHARACTER, SECOND_CHARACTER] {
        let mut game = load_game_for(character_id, None);
        let character = CharacterState {
            guid: character_id,
            ..character_at(spawn)
        };
        let packets = game
            .update_npc_visibility(&character)
            .expect("NPC visibility updates");
        let actor = packets
            .iter()
            .filter_map(|packet| startup_world::parse_add_npc(packet).ok())
            .find(|npc| npc.actor_id == PETRA)
            .unwrap_or_else(|| panic!("session {character_id} did not receive Petra's 35/2"));
        assert_eq!(actor.model_id, PETRA_MODEL);
        assert_eq!(actor.name_string_id, PETRA_NAME_STRING_ID);
        for axis in 0..3 {
            assert!(
                (actor.position[axis] - spawn[axis]).abs() <= 0.001,
                "session {character_id}: Petra's axis {axis} differs"
            );
        }
        drawn.push((
            actor.actor_id,
            actor.model_id,
            actor.name_string_id,
            actor.position,
        ));
    }
    assert_eq!(drawn[0], drawn[1], "both sessions must draw the same Petra");

    // The authored content row is unchanged and reaches the drawable actor's click range.
    let interaction = NpcInteraction::load(
        &content_dir().join("npcs"),
        &content_dir().join("stores.json"),
    )
    .expect("NPC interaction content loads");
    let (petra_zone, petra_position) = interaction
        .location(PETRA)
        .expect("Petra is a positioned content NPC");
    let (sabastian_zone, sabastian_position) = interaction
        .location(SABASTIAN)
        .expect("Sabastian is a positioned content NPC");
    assert_eq!(petra_zone, 1);
    assert_eq!(sabastian_zone, 1);
    for (axis, expected) in [-2141.724403_f32, -32.948, 1045.102653].iter().enumerate() {
        assert!(
            (petra_position[axis] - expected).abs() <= 0.001,
            "T331 must not move Petra's content row"
        );
    }
    for (axis, expected) in [-1998.09_f32, -27.43718, 1156.538].iter().enumerate() {
        assert!(
            (sabastian_position[axis] - expected).abs() <= 0.001,
            "T331 must not move Sabastian's content row"
        );
    }
    let delta =
        ((spawn[0] - petra_position[0]).powi(2) + (spawn[2] - petra_position[2]).powi(2)).sqrt();
    assert!(
        delta <= 5.0,
        "the drawable spawn must stay inside the sampled 5 m authoring disc ({delta} m)"
    );

    // The spawn stands on ground and its full 5 m disc is standable (<= 25 degrees).
    let terrain = terrain::load();
    let grid = terrain.grid(0).expect("zone 0 terrain is loaded");
    let ground = grid
        .height_at(spawn[0], spawn[2])
        .expect("the spawn is on the terrain grid");
    assert!(
        (spawn[1] - ground).abs() <= 0.1,
        "spawn y {} is {:.4} m from the sampled ground {ground}",
        spawn[1],
        spawn[1] - ground
    );
    let disc = terrain::disc_points(spawn[0], spawn[2]);
    assert_eq!(disc.len(), 105);
    let mut max_slope = 0.0_f32;
    for (x, z) in &disc {
        assert!(
            grid.height_at(*x, *z).is_some(),
            "disc sample ({x}, {z}) has no ground"
        );
        let slope = terrain::slope_degrees(grid, *x, *z);
        max_slope = max_slope.max(slope);
    }
    for angle in (0..360).step_by(15) {
        let radians = (angle as f32).to_radians();
        let (x, z) = (
            spawn[0] + 5.0 * radians.cos(),
            spawn[2] + 5.0 * radians.sin(),
        );
        assert!(
            grid.height_at(x, z).is_some(),
            "ring sample {angle} degrees has no ground"
        );
    }
    assert!(
        max_slope <= 25.0,
        "the 5 m authoring disc reaches {max_slope:.2} degrees"
    );

    // Sabastian, Petra and C6 resolve through the production quest catalogue.
    let engine = QuestEngine::from_split_content(&content_dir(), CHARACTER, eligibility(), None)
        .expect("the split quest catalogue loads");
    assert!(engine.is_enabled(GIVEN_TO_GUARD));
    assert_eq!(engine.giver_npc_id(GIVEN_TO_GUARD), Some(SABASTIAN));
    assert_eq!(engine.title_string_id(GIVEN_TO_GUARD), Some(130_857));
    assert!(
        engine
            .quest_ids_for_giver(SABASTIAN)
            .any(|quest_id| quest_id == GIVEN_TO_GUARD),
        "Sabastian's catalogue lists C6"
    );
    let end = engine
        .quest_end_packet(GIVEN_TO_GUARD, PETRA)
        .expect("C6 encodes its end screen");
    assert_eq!(u64::from_le_bytes(end[6..14].try_into().unwrap()), PETRA);
    assert_eq!(
        u32::from_le_bytes(end[14..18].try_into().unwrap()),
        GIVEN_TO_GUARD
    );
    assert_eq!(
        i32::from_le_bytes(end[27..31].try_into().unwrap()),
        i32::try_from(QUEST_REWARD_COINS).unwrap(),
        "the end screen displays the five-coin reward"
    );
    assert_eq!(
        i32::from_le_bytes(end[31..35].try_into().unwrap()),
        i32::try_from(QUEST_REWARD_JOB_XP).unwrap(),
        "the end screen displays the 25 Brawler XP reward"
    );
}
