use osfr_tools::{
    character_store::CharacterState,
    gameplay::{Gameplay, GameplayState},
    npc_interaction::{NpcInteractRequest, QUEST_CHOICE_RESPONSE, decode_quest_offer_49_1},
    quest_engine::{CharacterEligibility, QuestDenial, QuestEngine, QuestStatus},
    startup_world::{NpcSpawn, parse_add_npc},
};
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

const CHARACTER: u64 = 101;
const CARLOS_KEY: u64 = 9_000_000_042;
const JONES: u64 = 9_000_000_121;
const SABASTIAN: u64 = 9_000_000_446;
const REFERRAL_TO_JONES: u32 = 3_030_006;
const HOOLIGAN_FIGHT: u32 = 3_030_007;
const SEARCH_BLACKSPORE: u32 = 3_030_008;
const HOOLIGAN_DEFINITION: u32 = 930_050;
const CHAIN_REWARD_COINS: u32 = 5;
const COMBAT_PROFILE: i32 = 43;
const TALK_OPTION_STRING_ID: u32 = 3300;

fn content_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content")
}

fn reference() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../lab/references/rust-opening-cooking-original-assets")
}

fn character_at(position: [f32; 3]) -> CharacterState {
    CharacterState {
        guid: CHARACTER,
        zone_id: 0,
        position,
        rotation: [1.0, 0.0, 0.0],
    }
}

fn game() -> Gameplay {
    let reference = reference();
    assert!(
        reference.join("opening.json").is_file(),
        "brawler chapter requires the private opening fixture at {}",
        reference.display()
    );
    let mut game = Gameplay::load(&reference, None).unwrap();
    game.bind_npc_quest_catalogue(&content_dir(), CHARACTER)
        .unwrap();
    let switched = game
        .handle_in_world(&select_job(COMBAT_PROFILE), &character_at([0.0; 3]), 100)
        .unwrap();
    assert!(switched.changed);
    assert_eq!(game.active_profile(), Some(COMBAT_PROFILE));
    game
}

fn reload(game: Gameplay) -> Gameplay {
    let encoded = serde_json::to_vec(&game.state()).unwrap();
    let state: GameplayState = serde_json::from_slice(&encoded).unwrap();
    let mut reloaded = Gameplay::load(&reference(), Some(state)).unwrap();
    reloaded
        .bind_npc_quest_catalogue(&content_dir(), CHARACTER)
        .unwrap();
    reloaded
}

fn carlos_spawn_record() -> Value {
    let document: Value =
        serde_json::from_slice(&fs::read(content_dir().join("npc_spawns_extra.json")).unwrap())
            .unwrap();
    document["records"]
        .as_array()
        .unwrap()
        .iter()
        .find(|record| record["content_key"].as_u64() == Some(CARLOS_KEY))
        .expect("Carlos has a bound extra spawn")
        .clone()
}

fn spawn_position(record: &Value) -> [f32; 3] {
    serde_json::from_value(record["position"].clone()).unwrap()
}

fn npc_position(file: &str, npc_id: u64) -> [f32; 3] {
    let document: Value =
        serde_json::from_slice(&fs::read(content_dir().join("npcs").join(file)).unwrap()).unwrap();
    let record = document["records"]
        .as_array()
        .unwrap()
        .iter()
        .find(|record| record["npc_id"].as_u64() == Some(npc_id))
        .expect("required NPC exists");
    serde_json::from_value(record["position"].clone()).unwrap()
}

fn visible_add(game: &mut Gameplay, character: &CharacterState, actor: u64) -> NpcSpawn {
    let packets = game.update_npc_visibility(character).unwrap();
    packets
        .iter()
        .filter(|packet| packet.starts_with(&[35, 0, 2, 0]))
        .filter_map(|packet| parse_add_npc(packet).ok())
        .find(|spawn| spawn.actor_id == actor)
        .expect("actor sent through 35/2")
}

fn interact_packet(actor: u64) -> Vec<u8> {
    NpcInteractRequest {
        target: actor,
        trailing_flag: false,
    }
    .encode()
}

fn select_packet(actor: u64, choice: i32) -> Vec<u8> {
    let mut packet = vec![26, 0, 10, 0];
    packet.extend(actor.to_le_bytes());
    packet.extend(choice.to_le_bytes());
    packet
}

fn quest_choice_packet() -> Vec<u8> {
    let mut packet = vec![26, 0, 6, 0];
    packet.extend(QUEST_CHOICE_RESPONSE.to_le_bytes());
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

fn targeted_basic_attack(target: u64) -> Vec<u8> {
    [
        vec![36, 0, 10, 0],
        1_i32.to_le_bytes().to_vec(),
        0_i32.to_le_bytes().to_vec(),
        0_i32.to_le_bytes().to_vec(),
        target.to_le_bytes().to_vec(),
    ]
    .concat()
}

fn open_menu(game: &mut Gameplay, character: &CharacterState, actor: u64, now: u64) -> Vec<u8> {
    let opened = game
        .handle_in_world(&interact_packet(actor), character, now)
        .unwrap();
    let menu = opened
        .replies
        .iter()
        .find(|packet| packet.starts_with(&[26, 0, 9, 0]))
        .expect("NPC interaction returns the native menu");
    assert_eq!(
        u64::from_le_bytes(menu[4..12].try_into().unwrap()),
        actor,
        "menu targets the runtime actor"
    );
    menu.clone()
}

fn talk_option(menu: &[u8]) -> i32 {
    let count = i32::from_le_bytes(menu[13..17].try_into().unwrap());
    assert!(count > 0);
    (0..count)
        .find_map(|index| {
            let offset = 17 + usize::try_from(index).unwrap() * 28;
            let id = i32::from_le_bytes(menu[offset..offset + 4].try_into().unwrap());
            let string_id = u32::from_le_bytes(menu[offset + 8..offset + 12].try_into().unwrap());
            (string_id == TALK_OPTION_STRING_ID).then_some(id)
        })
        .expect("native menu contains Talk")
}

fn select_talk(
    game: &mut Gameplay,
    character: &CharacterState,
    actor: u64,
    now: u64,
) -> Vec<Vec<u8>> {
    let menu = open_menu(game, character, actor, now);
    let talk = game
        .handle_in_world(
            &select_packet(actor, talk_option(&menu)),
            character,
            now + 1,
        )
        .unwrap();
    assert!(
        talk.replies
            .iter()
            .any(|packet| packet.starts_with(&[26, 0, 3, 0])),
        "Talk opens the native dialogue"
    );
    talk.replies
}

fn offer_and_accept(
    game: &mut Gameplay,
    character: &CharacterState,
    actor: u64,
    quest_id: u32,
    now: u64,
) {
    select_talk(game, character, actor, now);
    let offered = game
        .handle_in_world(&quest_choice_packet(), character, now + 2)
        .unwrap();
    let packet = offered
        .replies
        .iter()
        .find(|packet| packet.starts_with(&[49, 0, 1, 0]))
        .expect("quest choice opens 49/1");
    assert_eq!(decode_quest_offer_49_1(packet).unwrap().quest_id, quest_id);

    let accepted = game
        .handle_in_world(&quest_reply(quest_id, true), character, now + 3)
        .unwrap();
    assert!(accepted.changed);
    assert_eq!(
        game.state().npc_quests.as_ref().unwrap().quests[&quest_id].status,
        QuestStatus::Active
    );
}

fn finish_turnin(
    game: &mut Gameplay,
    character: &CharacterState,
    actor: u64,
    quest_id: u32,
    now: u64,
) {
    select_talk(game, character, actor, now);
    let ended = game
        .handle_in_world(&quest_choice_packet(), character, now + 2)
        .unwrap();
    assert!(
        ended.replies.iter().any(|packet| {
            packet.starts_with(&[49, 0, 13, 0, 0, 0])
                && u32::from_le_bytes(packet[14..18].try_into().unwrap()) == quest_id
        }),
        "completed objective opens the native quest end screen"
    );
    let completed = game
        .handle_in_world(&quest_end_reply(quest_id), character, now + 3)
        .unwrap();
    assert!(completed.changed);
    assert_eq!(
        game.state().npc_quests.as_ref().unwrap().quests[&quest_id].status,
        QuestStatus::Completed
    );
}

fn quest_count(game: &Gameplay, quest_id: u32) -> u32 {
    game.state().npc_quests.as_ref().unwrap().quests[&quest_id].counts[0]
}

fn hooligan_guids() -> Vec<u64> {
    let document: Value =
        serde_json::from_slice(&fs::read(content_dir().join("field_enemies.json")).unwrap())
            .unwrap();
    let mut guids: Vec<u64> = document["enemies"]
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
        .filter_map(|enemy| enemy["guid"].as_u64())
        .collect();
    guids.sort_unstable();
    assert_eq!(guids.len(), 4);
    guids
}

fn known_hooligans(game: &Gameplay, character: &CharacterState, guids: &[u64]) -> BTreeSet<u64> {
    let wanted: BTreeSet<_> = guids.iter().copied().collect();
    game.known_actors(character)
        .into_iter()
        .map(|target| target.guid)
        .filter(|guid| wanted.contains(guid))
        .collect()
}

fn actor_position(game: &Gameplay, character: &CharacterState, actor: u64) -> [f32; 3] {
    game.known_actors(character)
        .into_iter()
        .find(|target| target.guid == actor)
        .expect("live actor is known")
        .position
}

fn kill_hooligan(
    game: &mut Gameplay,
    guide: &CharacterState,
    actor: u64,
    now: &mut u64,
    guids: &[u64],
) {
    let position = actor_position(game, guide, actor);
    let attacker = character_at(position);
    let mut killed = false;
    for _ in 0..6 {
        *now += 700;
        game.handle_in_world(&targeted_basic_attack(actor), &attacker, *now)
            .unwrap();
        if !known_hooligans(game, guide, guids).contains(&actor) {
            killed = true;
            break;
        }
    }
    assert!(killed, "server damage defeats Hooligan {actor}");
}

fn start_hooligan_quest(game: &mut Gameplay, now: &mut u64) {
    let carlos = character_at(spawn_position(&carlos_spawn_record()));
    visible_add(game, &carlos, CARLOS_KEY);
    offer_and_accept(game, &carlos, CARLOS_KEY, REFERRAL_TO_JONES, *now);

    let jones = character_at(npc_position("cobblestone_village.json", JONES));
    game.update_npc_visibility(&jones).unwrap();
    finish_turnin(game, &jones, JONES, REFERRAL_TO_JONES, *now + 10);
    offer_and_accept(game, &jones, JONES, HOOLIGAN_FIGHT, *now + 20);
    *now += 30;
}

#[test]
fn carlos_spawns_and_offers_the_chapter_through_npc_interaction() {
    let carlos = carlos_spawn_record();
    let character = character_at(spawn_position(&carlos));
    let mut game = game();
    let spawn = visible_add(&mut game, &character, CARLOS_KEY);
    assert_eq!(spawn.model_id, 964);
    assert_eq!(spawn.name_string_id, 20_671);
    assert_eq!(spawn.position[..3], spawn_position(&carlos));

    let menu = open_menu(&mut game, &character, CARLOS_KEY, 1_000);
    let before = serde_json::to_vec(&game.state()).unwrap();
    assert!(
        game.handle_in_world(&select_packet(CARLOS_KEY, i32::MAX), &character, 1_000)
            .is_err(),
        "an unoffered menu choice is refused"
    );
    assert_eq!(
        serde_json::to_vec(&game.state()).unwrap(),
        before,
        "a refused menu choice changes nothing"
    );
    let option = talk_option(&menu);
    let talked = game
        .handle_in_world(&select_packet(CARLOS_KEY, option), &character, 1_001)
        .unwrap();
    assert!(
        talked
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[26, 0, 3, 0]))
    );
    let offered = game
        .handle_in_world(&quest_choice_packet(), &character, 1_002)
        .unwrap();
    let packet = offered
        .replies
        .iter()
        .find(|packet| packet.starts_with(&[49, 0, 1, 0]))
        .expect("Carlos offers the chapter");
    assert_eq!(
        decode_quest_offer_49_1(packet).unwrap().quest_id,
        REFERRAL_TO_JONES
    );
    let accepted = game
        .handle_in_world(&quest_reply(REFERRAL_TO_JONES, true), &character, 1_003)
        .unwrap();
    assert!(accepted.changed);
    assert_eq!(
        game.state().npc_quests.as_ref().unwrap().quests[&REFERRAL_TO_JONES].status,
        QuestStatus::Active
    );
}

#[test]
fn real_hooligan_defeats_progress_reload_and_pay_the_chapter_once() {
    let mut now = 2_000;
    let mut game = game();
    start_hooligan_quest(&mut game, &mut now);

    let jones = character_at(npc_position("cobblestone_village.json", JONES));
    let guids = hooligan_guids();
    assert!(
        known_hooligans(&game, &jones, &guids).is_empty(),
        "quest enemies stay hidden before the defeat objective is active"
    );
    now += 100;
    game.tick(&jones, now).unwrap();
    assert_eq!(
        known_hooligans(&game, &jones, &guids),
        guids.iter().copied().collect(),
        "the active defeat objective exposes exactly four Hooligans"
    );
    let mut last_position = [0.0; 3];
    for (index, guid) in guids.iter().copied().enumerate() {
        last_position = actor_position(&game, &jones, guid);
        kill_hooligan(&mut game, &jones, guid, &mut now, &guids);
        assert_eq!(quest_count(&game, HOOLIGAN_FIGHT), index as u32 + 1);
    }

    let last = guids.last().copied().unwrap();
    let before = serde_json::to_vec(&game.state()).unwrap();
    now += 700;
    let replay = game
        .handle_in_world(
            &targeted_basic_attack(last),
            &character_at(last_position),
            now,
        )
        .unwrap();
    assert!(!replay.changed);
    assert_eq!(
        serde_json::to_vec(&game.state()).unwrap(),
        before,
        "a replayed hit on a defeated enemy changes no quest state"
    );
    assert_eq!(quest_count(&game, HOOLIGAN_FIGHT), 4);

    game.update_npc_visibility(&jones).unwrap();
    let coins_before = game.state().inventory.coins;
    finish_turnin(&mut game, &jones, JONES, HOOLIGAN_FIGHT, now + 10);
    assert_eq!(
        game.state().inventory.coins,
        coins_before + CHAIN_REWARD_COINS
    );
    now += 20;
    game.tick(&jones, now).unwrap();
    assert!(
        known_hooligans(&game, &jones, &guids).is_empty(),
        "Hooligans leave after their quest completes"
    );
    offer_and_accept(&mut game, &jones, JONES, SEARCH_BLACKSPORE, now + 20);

    let sabastian = character_at(npc_position("blackspore_swamp.json", SABASTIAN));
    game.update_npc_visibility(&sabastian).unwrap();
    let coins_before = game.state().inventory.coins;
    finish_turnin(
        &mut game,
        &sabastian,
        SABASTIAN,
        SEARCH_BLACKSPORE,
        now + 30,
    );
    assert_eq!(
        game.state().inventory.coins,
        coins_before + CHAIN_REWARD_COINS
    );

    for quest_id in [REFERRAL_TO_JONES, HOOLIGAN_FIGHT, SEARCH_BLACKSPORE] {
        assert_eq!(
            game.state().npc_quests.as_ref().unwrap().quests[&quest_id].status,
            QuestStatus::Completed
        );
    }
    let paid = game.state().inventory.coins;
    let saved = reload(game);
    assert_eq!(saved.state().inventory.coins, paid);
    for quest_id in [REFERRAL_TO_JONES, HOOLIGAN_FIGHT, SEARCH_BLACKSPORE] {
        assert_eq!(
            saved.state().npc_quests.as_ref().unwrap().quests[&quest_id].status,
            QuestStatus::Completed
        );
    }
}

#[test]
fn disabled_starting_quests_refuse_without_changing_state() {
    let mut engine = QuestEngine::from_split_content(
        &content_dir(),
        CHARACTER,
        CharacterEligibility {
            player_level: 20,
            job_levels: BTreeMap::from([("brawler".into(), 20)]),
            member: true,
            nonmember_quest_limit: 30,
            member_quest_limit: 30,
        },
        None,
    )
    .unwrap();
    let before = engine.state().clone();
    for quest_id in [3_030_004, 3_030_005] {
        let refused = engine.offer(quest_id, CARLOS_KEY);
        assert_eq!(refused.denial, Some(QuestDenial::Unavailable));
        assert!(!refused.changed);
        assert!(refused.outbound.is_empty());
        assert_eq!(engine.state(), &before);
    }
}
