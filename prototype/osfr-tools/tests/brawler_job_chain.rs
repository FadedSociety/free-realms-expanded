use osfr_tools::{
    character_store::CharacterState,
    gameplay::Gameplay,
    quest_engine::{CharacterEligibility, QuestDenial, QuestEngine, QuestStatus},
    startup_world,
};
use serde_json::Value;
use std::{
    collections::BTreeMap,
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
    packet.len() >= 6
        && packet[..2] == family.to_le_bytes()
        && packet[2..4] == subtype.to_le_bytes()
        && id.is_none_or(|id| {
            (packet.len() >= 10 && u32::from_le_bytes(packet[6..10].try_into().unwrap()) == id)
                || (packet.len() >= 18
                    && u32::from_le_bytes(packet[14..18].try_into().unwrap()) == id)
        })
}

fn interact_npc(game: &mut Gameplay, character: &CharacterState, npc_id: u64, now: u64) {
    let menu = game
        .handle_in_world(&npc_interact(npc_id), character, now)
        .expect("NPC interaction is valid");
    assert!(
        menu.replies
            .iter()
            .any(|packet| packet.starts_with(&[26, 0, 9, 0])),
        "NPC {npc_id} did not open its menu"
    );
}

fn talk(game: &mut Gameplay, character: &CharacterState, npc_id: u64, now: u64) {
    interact_npc(game, character, npc_id, now);
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
    let end = chosen
        .replies
        .into_iter()
        .find(|packet| has_family(packet, 49, 13, Some(quest_id)))
        .expect("the completed quest opens its end screen");
    assert_eq!(
        game.state().npc_quests.unwrap().pending_turnin,
        Some(quest_id)
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

fn kill_hooligans(game: &mut Gameplay, character: &CharacterState, now: &mut u64) {
    // A far-away location lets the production tick publish quest-gated spawns
    // without advancing any actor before the test reaches it.
    let mut character = character.clone();
    character.position = [10_000.0, 0.0, 10_000.0];
    *now += 100;
    game.tick(&character, *now)
        .expect("field quests synchronize");

    for (guid, position) in field_hooligans() {
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
}

fn quest_count(game: &Gameplay, quest_id: u32) -> u32 {
    game.state().npc_quests.unwrap().quests[&quest_id].counts[0]
}

fn select_job(profile_id: i32) -> Vec<u8> {
    let mut packet = vec![26, 0, 13, 0];
    packet.extend(profile_id.to_le_bytes());
    packet
}

fn npc_interact(npc_id: u64) -> Vec<u8> {
    let mut packet = vec![26, 0, 8, 0];
    packet.extend(npc_id.to_le_bytes());
    packet.push(0);
    packet
}

fn npc_select(npc_id: u64, choice: i32) -> Vec<u8> {
    let mut packet = vec![26, 0, 10, 0];
    packet.extend(npc_id.to_le_bytes());
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
    assert!(
        spawned.iter().any(|packet| {
            packet.starts_with(&[35, 0, 2, 0])
                && startup_world::parse_add_npc(packet).is_ok_and(|npc| npc.actor_id == CARLOS)
        }),
        "the reconstructed chapter spawn must be sent to the client"
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
    finish_turnin(&mut game, &character, REFERRAL_TO_JONES, &mut now);

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

    // Disconnect after the end screen opened and before its 49/14 confirmation.
    let mut game = load_game(Some(restored_state));
    character.position = jones;
    let initialized = game
        .initialize_world(&character, now + 100)
        .expect("reconnected gameplay initializes");
    let restored_end = initialized
        .replies
        .iter()
        .find(|packet| has_family(packet, 49, 13, Some(HOOLIGAN_FIGHT)))
        .expect("reconnect must reopen the pending end screen");
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
    assert_refusal_changes_nothing(
        &mut game,
        &character,
        &quest_end_reply(HOOLIGAN_FIGHT),
        now,
        "duplicate recovered reward",
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
        let refused = engine.offer(quest_id, CARLOS);
        assert_eq!(refused.denial, Some(QuestDenial::Unavailable));
        assert!(!refused.changed);
        assert!(refused.outbound.is_empty());
        assert_eq!(engine.state(), &before);
    }
}
