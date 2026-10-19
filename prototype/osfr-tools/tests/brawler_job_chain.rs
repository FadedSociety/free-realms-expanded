use osfr_tools::{
    character_store::CharacterState,
    gameplay::{Gameplay, GameplayState},
    gateway::Gateway,
    npc_interaction::NpcInteraction,
    quest_engine::{
        CharacterEligibility, QuestDenial, QuestEngine, QuestEvent, QuestProgress, QuestStatus,
    },
    startup_world,
};
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

const CHARACTER: u64 = 101;
const CARLOS: u64 = 9_000_000_042;
const JONES: u64 = 9_000_000_121;
const SABASTIAN: u64 = 9_000_000_446;
const REFERRAL_TO_JONES: u32 = 3_030_006;
const HOOLIGAN_FIGHT: u32 = 3_030_007;
const SEARCH_BLACKSPORE: u32 = 3_030_008;
const HOOLIGAN_DEFINITION: u32 = 930_050;
const QUEST_REWARD_COINS: u32 = 5;
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

fn load_game(saved: Option<osfr_tools::gameplay::GameplayState>) -> Gameplay {
    let mut game = Gameplay::load(&gameplay_dir(), saved).expect("gameplay content loads");
    game.bind_npc_quest_catalogue(&content_dir(), CHARACTER)
        .expect("split quest content loads");
    game
}

fn fresh_game() -> (Gameplay, CharacterState) {
    let mut game = load_game(None);
    let character = character_at([0.0; 3]);
    let selected = game
        .handle_in_world(&select_job(43), &character, 1)
        .expect("Brawler job selection is valid");
    assert!(
        selected.changed,
        "Brawler selection must change the profile"
    );
    (game, character)
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

fn saved(game: &Gameplay) -> Vec<u8> {
    serde_json::to_vec(&game.state()).unwrap()
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

fn different_enemy_definition() -> u32 {
    let document: Value =
        serde_json::from_slice(&fs::read(content_dir().join("field_enemies.json")).unwrap())
            .unwrap();
    document["enemies"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|enemy| enemy["definition_id"].as_u64())
        .find(|definition| *definition != u64::from(HOOLIGAN_DEFINITION))
        .and_then(|definition| u32::try_from(definition).ok())
        .expect("content has another field enemy definition")
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
    for (index, text) in ["f170-test-ticket", "1.910.1.530630", ""]
        .iter()
        .enumerate()
    {
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
    let reference = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../lab/references/rust-gateway-reference");
    // The gateway reference is a private lab fixture. Skip this regression when it is
    // absent so an ordinary checkout still runs the rest of the chapter suite; the
    // shared-tick regression covers the replay without it.
    if !reference.join("manifest.json").is_file() {
        eprintln!(
            "skipping the gateway reconnect ordering regression: private reference {} is absent",
            reference.display()
        );
        return;
    }
    let mut gateway = Gateway::load_gameplay(
        &reference,
        character.clone(),
        &gameplay_dir(),
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
    }
    assert_eq!(quest_count(game, HOOLIGAN_FIGHT), 4);

    // A replayed hit against the last defeated enemy must not advance the objective.
    let last = field_hooligans()
        .last()
        .expect("the chapter owns four field spawns")
        .0;
    let before_replay = saved(game);
    *now += 700;
    let replayed = game
        .handle_in_world(&targeted_basic_attack(last), &character, *now)
        .expect("a replayed attack is a clean refusal");
    assert!(!replayed.changed);
    assert_eq!(
        saved(game),
        before_replay,
        "a replayed hit on a defeated enemy changes no quest state"
    );
    assert_eq!(quest_count(game, HOOLIGAN_FIGHT), 4);
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
// 36/10 AbilityPacketClientRequestStartAbility 0x00a2e0e0; 49/2 QuestReplyPacket
// 0x00a86400; 49/13 QuestEndPacket::ReadFields 0x00c7b990 / Unserialize 0x00c7bbd0;
// 49/14 QuestEndReplyPacket 0x00a864c0; 35/2 AddNpc reader 0x00921a10.
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

fn targeted_basic_attack(target: u64) -> Vec<u8> {
    // 36/10 AbilityPacketClientRequestStartAbility (ctor 0x00a2e0e0).
    [
        vec![36, 0, 10, 0],
        1_i32.to_le_bytes().to_vec(),
        0_i32.to_le_bytes().to_vec(),
        0_i32.to_le_bytes().to_vec(),
        target.to_le_bytes().to_vec(),
    ]
    .concat()
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
    assert!(refused.is_err(), "an unoffered menu choice is refused");
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
    // C2 and the later Highroad records stay empty/disabled; their provenance must not
    // claim objectives the server does not run (F135 review finding 4).
    for quest_id in [3_030_004, 3_030_005, 3_030_009, 3_030_010] {
        let refused = engine.offer(quest_id, CARLOS);
        assert_eq!(refused.denial, Some(QuestDenial::Unavailable));
        assert!(!refused.changed);
        assert!(refused.outbound.is_empty());
        assert_eq!(engine.state(), &before);
    }
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
    const OTHER_NPC: u64 = 9_000_000_189;
    let (mut game, character) = fresh_game();
    let mut now = 6_000;
    let carlos = npc_position("cobblestone_village.json", CARLOS);
    let jones = npc_position("cobblestone_village.json", JONES);
    let other = npc_position("cobblestone_village.json", OTHER_NPC);
    let mut at = character.clone();

    at.position = carlos;
    accept_quest(&mut game, &at, CARLOS, carlos, REFERRAL_TO_JONES, &mut now);

    // A different NPC's Talk leaves the Jones referral step untouched.
    at.position = other;
    let counts = game.state().npc_quests.as_ref().unwrap().quests[&REFERRAL_TO_JONES]
        .counts
        .clone();
    now += 100;
    let _ = interact_npc(&mut game, &at, OTHER_NPC, now);
    now += 100;
    game.handle_in_world(&npc_select(OTHER_NPC, 1), &at, now)
        .expect("the wrong NPC Talk is a clean request");
    let state = game.state();
    let quests = state.npc_quests.as_ref().unwrap();
    assert_eq!(
        quests.quests[&REFERRAL_TO_JONES].counts, counts,
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

    // A kill of another definition cannot satisfy the four 930050 Hooligans.
    let mut engine = QuestEngine::from_split_content(
        &content_dir(),
        CHARACTER,
        eligibility(),
        Some(game.state().npc_quests.clone().unwrap()),
    )
    .expect("the active C4 state loads into the quest engine");
    let before = engine.progress(HOOLIGAN_FIGHT).unwrap().clone();
    let wrong = engine
        .on_event(QuestEvent::KillActorKind {
            actor_kind_id: different_enemy_definition(),
            count: 1,
        })
        .unwrap();
    assert!(!wrong.changed, "another enemy advanced the kill step");
    assert_eq!(engine.progress(HOOLIGAN_FIGHT).unwrap(), &before);
    assert_eq!(before.counts, vec![0]);
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
    now += 2;
    assert_eq!(game.state().inventory.coins, total);

    // Replayed damage on a defeated Hooligan, from its own position, adds nothing.
    let (last, position) = *field_hooligans().last().expect("four quest-gated spawns");
    let mut at = character.clone();
    at.position = position;
    let before = saved(&game);
    now += 700;
    let replayed = game
        .handle_in_world(&targeted_basic_attack(last), &at, now)
        .expect("a replayed hit is a clean refusal");
    assert!(!replayed.changed, "replayed damage advanced chapter state");
    assert_eq!(saved(&game), before, "replayed damage changed the save");
    assert_eq!(game.state().inventory.coins, total);
}

#[test]
fn offer_gates_and_save_reload_survive() {
    let (mut game, character) = fresh_game();
    let mut now = 8_000;
    let initial = complete_chapter(&mut game, &character, &mut now);

    let reloaded = load_game(Some(game.state()));
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

    let mut engine = QuestEngine::from_split_content(
        &content_dir(),
        CHARACTER,
        eligibility(),
        Some(quests.clone()),
    )
    .expect("the completed chapter reloads into the quest engine");
    let before = engine.state().clone();
    for quest_id in [3_030_004, 3_030_005] {
        let refused = engine.offer(quest_id, CARLOS);
        assert_eq!(
            refused.denial,
            Some(QuestDenial::Unavailable),
            "disabled quest {quest_id} was offerable"
        );
        assert!(!refused.changed);
        assert!(refused.outbound.is_empty());
        assert_eq!(engine.state(), &before);
    }
    let interaction = NpcInteraction::load(
        &content_dir().join("npcs"),
        &content_dir().join("stores.json"),
    )
    .expect("NPC content loads");
    assert_eq!(
        interaction.offerable_quest_for_talk(CARLOS, &engine),
        None,
        "a disabled C1/C2 quest remains offerable after the chapter"
    );
}
