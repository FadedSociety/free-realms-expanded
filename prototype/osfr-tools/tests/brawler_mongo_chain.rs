//! T252: the Cobblestone Brawler start -- Carlos Brazenfist, Mongo and the Brawling Ring.
//!
//! Content-only slice: the playable battle is field battle 12400 and the two opening quests are
//! `data/content/quests/brawler.json` records 3030004/3030005. These tests drive the production
//! in-world router (`Gameplay::handle_in_world` -> `gameplay/npc.rs` / `gameplay/field_battle.rs`
//! -> `EncounterEngine` -> `QuestEngine`) with the same packets the gateway accepts; they never
//! call an engine method that the client cannot reach.
//!
//! Client shapes used here (pinned 2014 client):
//! * `26/8` interaction request, constructor `0x008b6bc0`; `26/9` menu body; `26/10`
//!   interaction select, constructor `0x00a36f80`; `26/6` dialogue response, constructor
//!   `0x009f5890`.
//! * `41/114` battle details (reader `0x009bdd70`), `41/107` ready, `41/108` Go (constructor
//!   `0x008b6e70`, sender `0x00938f00`), `41/109` leave (`0x008a31a0`), `26/42` results close.
//! * `49/1` QuestInfo reader `0x00c7bb60`, `49/2` accept constructor `0x00a86400`, `49/3`
//!   QuestAdd reader `0x00c7bd10`, `49/8` objective activated, `49/13` end screen
//!   `0x00c7b990`/`0x00c7bbd0`, `49/14` reply `0x00a864c0`.
//! * `35/2` AddNpc constructor `0x0091cac0`, body reader `0x00921a10`.

#[path = "support/npc_quest_chain.rs"]
mod chain;
#[path = "common/combat_scale.rs"]
mod combat_scale;

use chain::{
    QUEST_CHOICE, TALK_LABEL, accept, assert_packet_changes_nothing, character_at_for, coins,
    content_dir, counts, dialogue_choice, finish_quest, has_family, interact, menu_option,
    npc_interact, npc_select, pending_turnin, profile_xp, quest_reply, quest_status,
    release_end_screen, saved, select_job, send, talk,
};
use osfr_tools::{
    character_store::CharacterState,
    chat,
    encounter::{EncounterEngine, EncounterPhase},
    gameplay::{Gameplay, GameplayState},
    inventory::Inventory,
    quest_engine::QuestStatus,
    startup_world::parse_add_npc,
};
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    sync::{OnceLock, atomic::AtomicU64},
};

const CHARACTER: u64 = 252_001;
const OTHER: u64 = 252_002;
const CARLOS: u64 = 9_000_000_042;
const GATE: u64 = 100_000_124_000;
const MONGO: u64 = 100_000_124_001;
/// Mongo's authored (old-scale) health; the loader scales it by `enemy_health_scale`.
const MONGO_AUTHORED_HEALTH: u32 = 900;
const BATTLE: u32 = 12_400;
const BRAWLER: i32 = 43;
const FIRST: u32 = 3_030_004;
const SECOND: u32 = 3_030_005;
const CLAIM: u32 = 3_030_006;
const GATE_MODEL: i32 = 3126;
const MONGO_MODEL: i32 = 168;
const TITLE_ID: i32 = 24_276;
const MONGO_NAME_ID: i32 = 20_482;
const GLOVES: i32 = 75_855;
const MYSTERY_PACK: i32 = 10_482;
const POTION: i32 = 11_002;
const BATTLE_COINS: u32 = 19;
const QUEST_COINS: u32 = 5;
const BATTLE_XP: u32 = 50;
const QUEST_XP: u32 = 25;
const FIRST_END_TEXT: u32 = 140_533;
const SECOND_END_TEXT: u32 = 94_285;
const ATTACK_CHOICE: i32 = 103;
const ARENA_ZONE: u32 = 9600;
const ARENA_ARRIVAL: [f32; 3] = [136.0, -0.5, 155.0];
const GATE_POSITION: [f32; 3] = [-1908.92576, -39.4, 410.69792];

static NEXT_NOW: AtomicU64 = AtomicU64::new(1_000);
static RUNTIME_INVENTORY: OnceLock<Inventory> = OnceLock::new();

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn gameplay_content() -> PathBuf {
    manifest_dir().join("../../../lab/reports/round70-native-20261016T061000Z/content")
}

/// A fresh gameplay session over the shipped opening content and the bundled NPC, battle and
/// quest content, bound exactly as `Gateway::load_gameplay` does for a real login.
fn load_game(saved: Option<GameplayState>) -> Gameplay {
    load_game_for(CHARACTER, saved)
}

/// A session bound to its own character id, so two players keep separate quest ledgers.
fn load_game_for(character_id: u64, saved: Option<GameplayState>) -> Gameplay {
    let mut game = Gameplay::load(&gameplay_content(), saved).expect("opening content loads");
    game.bind_npc_quest_catalogue(&content_dir(), character_id)
        .expect("split quest content loads");
    game
}

fn read_json(path: &Path) -> Value {
    serde_json::from_slice(
        &fs::read(path).unwrap_or_else(|error| panic!("{}: {error}", path.display())),
    )
    .unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

fn battles() -> Vec<Value> {
    read_json(&content_dir().join("field_battles.json"))["battles"]
        .as_array()
        .unwrap()
        .clone()
}

fn battle_row(id: u32) -> Value {
    battles()
        .into_iter()
        .find(|row| row["id"].as_u64() == Some(u64::from(id)))
        .unwrap_or_else(|| panic!("battle {id} is not authored"))
}

fn position(value: &Value) -> [f32; 3] {
    let points = value.as_array().unwrap();
    [
        points[0].as_f64().unwrap() as f32,
        points[1].as_f64().unwrap() as f32,
        points[2].as_f64().unwrap() as f32,
    ]
}

fn carlos_position() -> [f32; 3] {
    chain::content_position("cobblestone_village.json", CARLOS)
}

fn now() -> u64 {
    NEXT_NOW.fetch_add(10_000, std::sync::atomic::Ordering::Relaxed)
}

fn switch_to_brawler(game: &mut Gameplay, character: &CharacterState, at: u64) {
    let switched = send(game, character, &select_job(BRAWLER), at);
    assert!(switched.changed, "the Brawler class selection must take");
}

/// A Brawler in the open world at `position`, after the production world-entry pass.
fn brawler_at(position: [f32; 3]) -> (Gameplay, CharacterState, u64) {
    let mut game = load_game(None);
    let character = character_at_for(CHARACTER, 0, position);
    let at = now();
    switch_to_brawler(&mut game, &character, at);
    game.initialize_world(&character, at + 100)
        .expect("world entry initializes");
    (game, character, at + 200)
}

/// Carlos's Talk offers C1 as the quest dialogue row, and 49/2 accepts it; the follow-up Talk
/// completes step 1. Both calls go through the ordinary 26/8 -> 26/10 -> 26/6 packets.
fn accept_first(game: &mut Gameplay, character: &CharacterState, at: &mut u64) {
    *at += 100;
    let menu = interact(game, character, CARLOS, *at);
    let choice = menu_option(&menu, TALK_LABEL);
    *at += 100;
    let dialogue = send(game, character, &npc_select(CARLOS, choice), *at);
    assert!(
        dialogue.replies.iter().any(|packet| {
            packet.starts_with(&[26, 0, 3, 0])
                && packet
                    .windows(4)
                    .any(|field| field == 140_531_u32.to_le_bytes())
        }),
        "Carlos's dialogue must carry the C1 quest row"
    );
    *at += 100;
    let offered = send(game, character, &dialogue_choice(QUEST_CHOICE), *at);
    assert!(
        offered
            .replies
            .iter()
            .any(|packet| has_family(packet, 49, 1, Some(FIRST))),
        "Carlos must offer C1 to a Brawler"
    );
    *at += 100;
    accept(game, character, FIRST, *at);
    *at += 100;
    talk(game, character, CARLOS, *at);
    assert_eq!(counts(game, FIRST), vec![1, 0], "step 1 completes");
}

/// 26/8 on the Brawling Ring gate: the start screen (41/114) plus its ready header (41/107).
fn open_gate(game: &mut Gameplay, character: &CharacterState, at: u64) -> Vec<u8> {
    let screen = interact(game, character, GATE, at);
    let details = screen
        .replies
        .iter()
        .find(|packet| packet.starts_with(&[41, 0, 114, 0]))
        .cloned()
        .unwrap_or_else(|| panic!("the gate did not open the battle start screen"));
    assert!(
        screen
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[41, 0, 107, 0])),
        "the start screen must be ready (41/107)"
    );
    assert_eq!(
        i32::from_le_bytes(details[4..8].try_into().unwrap()),
        BATTLE as i32,
        "the details packet names battle {BATTLE}"
    );
    details
}

/// 41/108 Go, copying the encounter and launch instance out of the details reply.
fn go(details: &[u8]) -> Vec<u8> {
    let mut packet = vec![41, 0, 108, 0];
    packet.extend(&details[4..12]);
    packet.extend(0_u64.to_le_bytes());
    packet
}

fn leave(instance: i32) -> Vec<u8> {
    let mut packet = vec![41, 0, 109, 0];
    packet.extend(0_i32.to_le_bytes());
    packet.extend(instance.to_le_bytes());
    packet
}

fn play(activity: u32) -> Vec<u8> {
    let mut packet = vec![167, 0, 1, 2];
    packet.extend(activity.to_le_bytes());
    packet.extend(0_u32.to_le_bytes());
    packet
}

/// Go into the arena then run the production zone-ready pass that spawns the roster.
fn enter_arena(
    game: &mut Gameplay,
    character: &mut CharacterState,
    details: &[u8],
    at: &mut u64,
) -> i32 {
    *at += 100;
    let zoned = send(game, character, &go(details), *at);
    let zone = zoned.zoning.expect("Go zones into the arena");
    assert_eq!(zone.zone_id, ARENA_ZONE);
    assert_eq!(zone.name, "sg_random_encounter_clearing");
    let instance = i32::from_le_bytes(details[8..12].try_into().unwrap());
    *at += 100;
    character.zone_id = zone.zone_id;
    character.position = ARENA_ARRIVAL;
    let loaded = game
        .initialize_world(character, *at)
        .expect("arena zone load");
    assert!(
        loaded.replies.iter().any(|packet| {
            packet.starts_with(&[35, 0, 2, 0]) && packet[4..12] == MONGO.to_le_bytes()
        }),
        "the arena spawns Mongo"
    );
    instance
}

fn active_phase(game: &Gameplay) -> Option<EncounterPhase> {
    game.state()
        .encounter
        .and_then(|state| state.active)
        .map(|active| active.phase)
}

fn active_instance(game: &Gameplay) -> Option<i32> {
    game.state()
        .encounter
        .and_then(|state| state.active)
        .map(|active| active.instance_id)
}

fn mongo_health(game: &Gameplay) -> Option<u32> {
    game.state()
        .encounter
        .and_then(|state| state.active)
        .and_then(|active| active.enemies.into_iter().find(|enemy| enemy.guid == MONGO))
        .map(|enemy| enemy.health)
}

fn mongo_position(game: &Gameplay) -> [f32; 3] {
    game.state()
        .encounter
        .and_then(|state| state.active)
        .and_then(|active| active.enemies.into_iter().find(|enemy| enemy.guid == MONGO))
        .map(|enemy| enemy.position)
        .expect("Mongo is in the arena")
}

/// Defeat the boss with the ordinary 26/10 Attack choice, inside melee reach.
fn win_mongo(game: &mut Gameplay, character: &mut CharacterState, at: &mut u64) {
    character.position = mongo_position(game);
    *at += 100;
    let menu = interact(game, character, MONGO, *at);
    assert!(
        menu.replies
            .iter()
            .any(|packet| packet.starts_with(&[26, 0, 9, 0])),
        "Mongo must expose the Attack menu inside its reach"
    );
    for _ in 0..40 {
        if active_phase(game) != Some(EncounterPhase::Fighting) {
            break;
        }
        *at += 700;
        let _ = send(game, character, &npc_select(MONGO, ATTACK_CHOICE), *at);
    }
    assert_eq!(
        active_phase(game),
        Some(EncounterPhase::Won),
        "ordinary attacks must knock Mongo out"
    );
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Economy {
    coins: u32,
    gloves: u32,
    packs: u32,
    potions: u32,
    brawler_xp: u32,
}

fn count_item(game: &Gameplay, definition_id: i32) -> u32 {
    game.state()
        .inventory
        .items
        .iter()
        .filter(|item| item.definition_id == definition_id && item.rental.is_none())
        .map(|item| item.count)
        .sum()
}

fn economy(game: &Gameplay) -> Economy {
    Economy {
        coins: game.state().inventory.coins,
        gloves: count_item(game, GLOVES),
        packs: count_item(game, MYSTERY_PACK),
        potions: count_item(game, POTION),
        brawler_xp: profile_xp(game, BRAWLER),
    }
}

fn prize_total(economy: &Economy) -> u32 {
    economy.gloves + economy.packs + economy.potions
}

/// Gate -> Go -> Mongo -> results Close -> world. The won run is paid and closed once.
fn finish_battle(game: &mut Gameplay, character: &mut CharacterState, at: &mut u64) {
    let details = open_gate(game, character, *at);
    enter_arena(game, character, &details, at);
    win_mongo(game, character, at);
    *at += 100;
    let closed = send(game, character, &[26, 0, 42, 0], *at);
    assert!(
        closed.zoning.is_some(),
        "results Close returns to the world"
    );
    character.zone_id = 0;
    character.position = carlos_position();
    *at += 100;
    game.initialize_world(character, *at).expect("world return");
}

/// Carlos's pending end screen for a completed quest, released by the 26/6 dialogue row.
fn open_end_screen(
    game: &mut Gameplay,
    character: &CharacterState,
    quest_id: u32,
    end_text: u32,
    at: &mut u64,
) -> Vec<u8> {
    *at += 100;
    let talked = talk(game, character, CARLOS, *at);
    assert!(
        talked
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[26, 0, 3, 0])),
        "the turn-in Talk opens the dialogue"
    );
    *at += 1;
    let end = release_end_screen(game, character, quest_id, CARLOS, end_text, *at);
    assert_eq!(pending_turnin(game), Some(quest_id));
    end
}

fn pay_first(game: &mut Gameplay, character: &CharacterState, at: &mut u64) {
    let end = open_end_screen(game, character, FIRST, FIRST_END_TEXT, at);
    // 49/13 reward bundle: the coin dword then the displayed XP total (native 0x008e7930).
    assert_eq!(
        i32::from_le_bytes(end[27..31].try_into().unwrap()),
        QUEST_COINS as i32
    );
    assert_eq!(
        i32::from_le_bytes(end[31..35].try_into().unwrap()),
        QUEST_XP as i32
    );
    *at += 100;
    finish_quest(game, character, FIRST, BRAWLER, QUEST_COINS, QUEST_XP, *at);
}

#[test]
fn carlos_offers_bringing_back_brazenfist_only_to_brawlers() {
    let (mut game, mut character, mut at) = brawler_at(carlos_position());

    // 26/8 -> 26/9, then the 26/10 Talk option, then the 26/6 quest row.
    let menu = interact(&mut game, &character, CARLOS, at);
    let choice = menu_option(&menu, TALK_LABEL);
    at += 100;
    let dialogue = send(&mut game, &character, &npc_select(CARLOS, choice), at);
    assert!(
        dialogue
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[26, 0, 3, 0])),
        "Carlos must open the 26/3 dialogue"
    );
    at += 100;
    let offered = send(&mut game, &character, &dialogue_choice(QUEST_CHOICE), at);
    let offers: Vec<u32> = offered
        .replies
        .iter()
        .filter(|packet| packet.starts_with(&[49, 0, 1, 0]))
        .map(|packet| u32::from_le_bytes(packet[6..10].try_into().unwrap()))
        .collect();
    assert_eq!(offers, [FIRST], "a Brawler is offered C1, exactly once");
    assert_eq!(quest_status(&game, FIRST), QuestStatus::Offered);

    // 49/2 accepts; a split quest runs its steps in order, so only step 1 activates.
    at += 100;
    let accepted = accept(&mut game, &character, FIRST, at);
    let activated: Vec<(u32, u32, u32)> = accepted
        .replies
        .iter()
        .filter(|packet| packet.starts_with(&[49, 0, 8, 0]))
        .map(|packet| {
            (
                u32::from_le_bytes(packet[6..10].try_into().unwrap()),
                u32::from_le_bytes(packet[10..14].try_into().unwrap()),
                u32::from_le_bytes(packet[14..18].try_into().unwrap()),
            )
        })
        .collect();
    assert_eq!(
        activated,
        [(FIRST, 1, 1)],
        "objective 1 (Talk to Carlos) opens alone"
    );
    assert_eq!(counts(&game, FIRST), vec![0, 0]);

    // A second Talk completes step 1 and opens step 2 (the battle).
    at += 100;
    let talked = talk(&mut game, &character, CARLOS, at);
    assert!(talked.changed, "the Carlos Talk advances C1 step 1");
    let activated: Vec<u32> = talked
        .replies
        .iter()
        .filter(|packet| packet.starts_with(&[49, 0, 8, 0]))
        .map(|packet| u32::from_le_bytes(packet[10..14].try_into().unwrap()))
        .collect();
    assert_eq!(activated, [2], "step 2 opens only after step 1");
    assert_eq!(counts(&game, FIRST), vec![1, 0]);

    // The exact payout is the quest's 5 coins / 25 Brawler XP, paid once at 49/14.
    let initial = economy(&game);
    finish_battle(&mut game, &mut character, &mut at);
    let after_run = economy(&game);
    assert_eq!(counts(&game, FIRST), vec![1, 1]);
    assert_eq!(after_run.coins, initial.coins + BATTLE_COINS);
    assert_eq!(after_run.brawler_xp, initial.brawler_xp + BATTLE_XP);
    assert_eq!(prize_total(&after_run), prize_total(&initial) + 1);
    pay_first(&mut game, &character, &mut at);
    assert_eq!(
        economy(&game).coins,
        initial.coins + BATTLE_COINS + QUEST_COINS
    );
    assert_eq!(
        economy(&game).brawler_xp,
        initial.brawler_xp + BATTLE_XP + QUEST_XP
    );

    // A non-Brawler is never offered C1: Carlos's dialogue row names C3's title instead and a
    // forged C1 accept changes nothing.
    let mut other = load_game(None);
    let outsider = character_at_for(OTHER, 0, carlos_position());
    let mut at = now();
    other
        .initialize_world(&outsider, at)
        .expect("world entry initializes");
    at += 100;
    let menu = interact(&mut other, &outsider, CARLOS, at);
    let choice = menu_option(&menu, TALK_LABEL);
    at += 100;
    let dialogue = send(&mut other, &outsider, &npc_select(CARLOS, choice), at);
    let row = dialogue
        .replies
        .iter()
        .find(|packet| packet.starts_with(&[26, 0, 3, 0]))
        .expect("the non-Brawler dialogue opens");
    assert!(
        !row.windows(4)
            .any(|field| field == 140_531_u32.to_le_bytes()),
        "C1's title row must not appear for a non-Brawler"
    );
    at += 100;
    let choice = send(&mut other, &outsider, &dialogue_choice(QUEST_CHOICE), at);
    assert!(
        !choice
            .replies
            .iter()
            .any(|packet| has_family(packet, 49, 1, Some(FIRST))),
        "a non-Brawler is never offered C1"
    );
    let before = saved(&other);
    at += 100;
    let forged = other
        .handle_in_world(&quest_reply(FIRST, true), &outsider, at)
        .expect("a forged accept is a clean refusal");
    assert!(!forged.changed, "the forged accept changed gameplay");
    assert_eq!(saved(&other), before, "the forged accept changed the save");
    assert_eq!(
        other.state().npc_quests.unwrap().quests.get(&FIRST),
        None,
        "the forged accept must not create C1 progress"
    );
    assert_eq!(quest_status(&other, CLAIM), QuestStatus::Offered);
}

#[test]
fn the_brawling_ring_gate_starts_the_mongo_battle() {
    let (mut game, mut character, mut at) = brawler_at(GATE_POSITION);

    // A wrong actor and an out-of-range gate request open nothing and change nothing.
    let before = saved(&game);
    let wrong = interact(&mut game, &character, CARLOS, at);
    assert!(
        !wrong
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[41, 0, 114, 0])),
        "another actor must not open the battle start screen"
    );
    assert_eq!(saved(&game), before, "the wrong actor changed the save");
    let mut away = character.clone();
    away.position = [GATE_POSITION[0] + 50.0, GATE_POSITION[1], GATE_POSITION[2]];
    at += 100;
    let far = interact(&mut game, &away, GATE, at);
    assert!(
        !far.replies
            .iter()
            .any(|packet| packet.starts_with(&[41, 0, 114, 0])),
        "an out-of-range gate interaction opens nothing"
    );
    assert_eq!(
        saved(&game),
        before,
        "the out-of-range interaction changed the save"
    );

    // The real gate route: 26/8 -> 41/114 + 41/107, then 41/108 Go.
    at += 100;
    let details = open_gate(&mut game, &character, at);
    assert_eq!(
        i32::from_le_bytes(details[4..8].try_into().unwrap()),
        BATTLE as i32
    );
    let mut at = at;
    let first_instance = enter_arena(&mut game, &mut character, &details, &mut at);
    assert_eq!(active_phase(&game), Some(EncounterPhase::Fighting));
    assert_eq!(active_instance(&game), Some(first_instance));

    // Busy: with the encounter active (Occupation::Encounter) the gate refuses a second start.
    let mut back = character.clone();
    back.zone_id = 0;
    back.position = GATE_POSITION;
    let busy_before = saved(&game);
    at += 100;
    let busy = interact(&mut game, &back, GATE, at);
    assert!(
        !busy
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[41, 0, 114, 0])),
        "a busy player must not open a second battle screen"
    );
    assert_eq!(
        saved(&game),
        busy_before,
        "the busy refusal changed the save"
    );
    // The shared activity-portal gate refuses the same battle while busy.
    at += 100;
    let busy_play = send(&mut game, &back, &play(BATTLE), at);
    assert!(busy_play.zoning.is_none(), "a busy Play must not zone");
    assert_eq!(saved(&game), busy_before, "the busy Play changed the save");

    // Leaving the arena pays nothing and returns to the world.
    at += 100;
    let left = send(&mut game, &back, &leave(first_instance), at);
    assert!(left.zoning.is_some(), "41/109 returns to the world");
    assert!(active_phase(&game).is_none());
    character.zone_id = 0;
    character.position = GATE_POSITION;
    at += 100;
    game.initialize_world(&character, at).expect("world return");

    // A Go from outside the marker radius is inert and drops the open screen.
    at += 100;
    let details = open_gate(&mut game, &character, at);
    let mut far = character.clone();
    far.position = [GATE_POSITION[0] + 50.0, GATE_POSITION[1], GATE_POSITION[2]];
    at += 100;
    let out_of_reach = send(&mut game, &far, &go(&details), at);
    assert!(
        out_of_reach.zoning.is_none(),
        "an out-of-reach Go must not zone"
    );
    assert!(!out_of_reach.changed);

    // Re-open and Go: a second run is a fresh instance (no reused lifecycle id).
    at += 100;
    let details = open_gate(&mut game, &character, at);
    let second_instance = enter_arena(&mut game, &mut character, &details, &mut at);
    assert_ne!(
        second_instance, first_instance,
        "each run gets its own instance id"
    );
    at += 100;
    send(&mut game, &character, &leave(second_instance), at);

    // Leaving paid nothing; the arena released the player and the wallet is untouched.
    assert!(active_phase(&game).is_none());

    // The portal publishes the same battle: Play 167/1 starts battle 12400 once.
    character.zone_id = 0;
    character.position = GATE_POSITION;
    at += 100;
    let started = send(&mut game, &character, &play(BATTLE), at);
    assert!(
        started.zoning.is_some(),
        "the activity portal starts battle 12400"
    );
    assert!(active_instance(&game).is_some());
    let instance = active_instance(&game).unwrap();
    assert_eq!(
        game.state().encounter.unwrap().active.unwrap().encounter_id,
        BATTLE
    );
    at += 100;
    send(&mut game, &character, &leave(instance), at);
    assert!(active_phase(&game).is_none());
}

#[test]
fn the_mongo_win_completes_bringing_back_brazenfist_once() {
    let (mut game, mut character, mut at) = brawler_at(carlos_position());
    accept_first(&mut game, &character, &mut at);
    let initial = economy(&game);

    let details = open_gate(&mut game, &character, at);
    enter_arena(&mut game, &mut character, &details, &mut at);
    at += 100;
    win_mongo(&mut game, &mut character, &mut at);
    let won = economy(&game);
    assert_eq!(won.coins, initial.coins + BATTLE_COINS, "19 coins once");
    assert_eq!(won.brawler_xp, initial.brawler_xp + BATTLE_XP, "50 XP once");
    assert_eq!(
        prize_total(&won),
        prize_total(&initial) + 1,
        "the wheel pays once"
    );
    assert_eq!(
        counts(&game, FIRST),
        vec![1, 1],
        "the win credits the dungeon objective before the results close"
    );

    at += 100;
    let closed = send(&mut game, &character, &[26, 0, 42, 0], at);
    assert!(closed.zoning.is_some());
    assert_eq!(
        counts(&game, FIRST),
        vec![1, 1],
        "the results Close does not credit C1 again"
    );
    assert_eq!(pending_turnin(&game), None);
    assert_eq!(
        economy(&game),
        won,
        "the results Close pays no second reward"
    );

    // A replayed results Close pays nothing.
    let after_close = saved(&game);
    at += 100;
    let duplicate = game
        .handle_in_world(&[26, 0, 42, 0], &character, at)
        .expect("a duplicate Close is a clean refusal");
    assert!(!duplicate.changed);
    assert_eq!(saved(&game), after_close);

    // The quest's own job pays even when the player has switched jobs before the hand-in. The
    // fixture unlocks the Warrior line (its starter outfit exists) so the class window accepts
    // the switch; the reward row still names profile 43.
    character.zone_id = 0;
    character.position = carlos_position();
    at += 100;
    game.initialize_world(&character, at).expect("world return");
    let mut state = game.state();
    state.profiles.as_mut().unwrap().unlocked.insert(32);
    drop(game);
    let mut game = load_game(Some(state));
    at += 100;
    game.initialize_world(&character, at)
        .expect("reloaded world");
    at += 100;
    let warrior_before = profile_xp(&game, 32);
    let switched = send(&mut game, &character, &select_job(32), at);
    assert!(switched.changed, "the Warrior class selection must take");
    assert_eq!(game.state().profiles.unwrap().active, 32);
    let before_turnin = economy(&game);
    pay_first(&mut game, &character, &mut at);
    let paid = economy(&game);
    assert_eq!(paid.coins, before_turnin.coins + QUEST_COINS);
    assert_eq!(
        paid.brawler_xp,
        before_turnin.brawler_xp + QUEST_XP,
        "the reward XP names Brawler, not the active job"
    );
    assert_eq!(
        profile_xp(&game, 32),
        warrior_before,
        "Warrior gains nothing"
    );
    assert_eq!(
        prize_total(&paid),
        prize_total(&before_turnin),
        "no extra wheel spin"
    );
    assert_eq!(quest_status(&game, FIRST), QuestStatus::Completed);
    assert_eq!(pending_turnin(&game), None);

    // A replayed 49/14 pays nothing.
    at += 100;
    assert_packet_changes_nothing(
        &mut game,
        &character,
        &chain::quest_end_reply(FIRST),
        at,
        "replayed C1 turn-in",
    );
    assert_eq!(economy(&game), paid);
}

#[test]
fn honor_lost_requires_the_first_quest_and_reuses_the_fight() {
    let (mut game, mut character, mut at) = brawler_at(carlos_position());

    // C2 is not offered while C1 is unfinished; Carlos offers C3 instead.
    let menu = interact(&mut game, &character, CARLOS, at);
    let choice = menu_option(&menu, TALK_LABEL);
    at += 100;
    let dialogue = send(&mut game, &character, &npc_select(CARLOS, choice), at);
    assert!(
        dialogue
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[26, 0, 3, 0]))
    );
    at += 100;
    let offered = send(&mut game, &character, &dialogue_choice(QUEST_CHOICE), at);
    let offers: Vec<u32> = offered
        .replies
        .iter()
        .filter(|packet| packet.starts_with(&[49, 0, 1, 0]))
        .map(|packet| u32::from_le_bytes(packet[6..10].try_into().unwrap()))
        .collect();
    assert_eq!(offers, [FIRST], "only C1 is on offer before it is done");
    assert!(
        !offered
            .replies
            .iter()
            .any(|packet| has_family(packet, 49, 1, Some(SECOND)))
    );
    at += 100;
    accept(&mut game, &character, FIRST, at);
    at += 100;
    talk(&mut game, &character, CARLOS, at);
    finish_battle(&mut game, &mut character, &mut at);
    let coins_before = coins(&game);
    let xp_before = profile_xp(&game, BRAWLER);
    pay_first(&mut game, &character, &mut at);
    assert_eq!(coins(&game), coins_before + QUEST_COINS);
    assert_eq!(profile_xp(&game, BRAWLER), xp_before + QUEST_XP);

    // C2 is offered now, and its first step is the repeated Mongo fight.
    character.position = carlos_position();
    at += 100;
    let menu = interact(&mut game, &character, CARLOS, at);
    let choice = menu_option(&menu, TALK_LABEL);
    at += 100;
    let dialogue = send(&mut game, &character, &npc_select(CARLOS, choice), at);
    assert!(
        dialogue
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[26, 0, 3, 0]))
    );
    at += 100;
    let offered = send(&mut game, &character, &dialogue_choice(QUEST_CHOICE), at);
    let offers: Vec<u32> = offered
        .replies
        .iter()
        .filter(|packet| packet.starts_with(&[49, 0, 1, 0]))
        .map(|packet| u32::from_le_bytes(packet[6..10].try_into().unwrap()))
        .collect();
    assert_eq!(offers, [SECOND], "C2 opens once C1 is completed");
    at += 100;
    let accepted = accept(&mut game, &character, SECOND, at);
    let activated: Vec<u32> = accepted
        .replies
        .iter()
        .filter(|packet| packet.starts_with(&[49, 0, 8, 0]))
        .map(|packet| u32::from_le_bytes(packet[10..14].try_into().unwrap()))
        .collect();
    assert_eq!(activated, [1], "C2 starts on the Mongo fight");
    assert_eq!(counts(&game, SECOND), vec![0, 0]);

    let before_second_run = economy(&game);
    finish_battle(&mut game, &mut character, &mut at);
    let after_second_run = economy(&game);
    assert_eq!(
        counts(&game, SECOND),
        vec![1, 0],
        "the repeat fight advances C2 once"
    );
    assert_eq!(coins(&game), before_second_run.coins + BATTLE_COINS);
    assert_eq!(
        profile_xp(&game, BRAWLER),
        before_second_run.brawler_xp + BATTLE_XP
    );
    assert_eq!(
        quest_status(&game, FIRST),
        QuestStatus::Completed,
        "the repeat fight does not reopen C1"
    );
    assert_eq!(counts(&game, FIRST), vec![1, 1]);

    // Hand in to Carlos: C2's Talk step opens and pays once; C1 is not paid again.
    let end = open_end_screen(&mut game, &character, SECOND, SECOND_END_TEXT, &mut at);
    assert_eq!(
        i32::from_le_bytes(end[27..31].try_into().unwrap()),
        QUEST_COINS as i32
    );
    at += 100;
    finish_quest(
        &mut game,
        &character,
        SECOND,
        BRAWLER,
        QUEST_COINS,
        QUEST_XP,
        at,
    );
    assert_eq!(
        coins(&game),
        before_second_run.coins + BATTLE_COINS + QUEST_COINS
    );
    assert_eq!(
        profile_xp(&game, BRAWLER),
        before_second_run.brawler_xp + BATTLE_XP + QUEST_XP
    );
    assert_eq!(
        after_second_run.brawler_xp + QUEST_XP,
        profile_xp(&game, BRAWLER)
    );
    assert_eq!(quest_status(&game, SECOND), QuestStatus::Completed);

    // Replaying C2's turn-in pays nothing and C1 stays completed exactly once.
    at += 100;
    assert_packet_changes_nothing(
        &mut game,
        &character,
        &chain::quest_end_reply(SECOND),
        at,
        "replayed C2 turn-in",
    );
    at += 100;
    assert_packet_changes_nothing(
        &mut game,
        &character,
        &chain::quest_end_reply(FIRST),
        at,
        "replayed C1 turn-in after C2",
    );
}

#[test]
fn refusals_and_replay_change_nothing() {
    let (mut game, mut character, mut at) = brawler_at(carlos_position());

    // Forged accept: C1 is not offered yet, so 49/2 is refused.
    at += 100;
    assert_packet_changes_nothing(
        &mut game,
        &character,
        &quest_reply(FIRST, true),
        at,
        "forged C1 accept",
    );

    // Unfinished objective: the turn-in arrives before any battle, so nothing is owed.
    at += 100;
    assert_packet_changes_nothing(
        &mut game,
        &character,
        &chain::quest_end_reply(FIRST),
        at,
        "forged C1 turn-in",
    );

    // Out-of-range gate interaction: refused without consuming anything.
    at += 100;
    assert_packet_changes_nothing(
        &mut game,
        &character,
        &npc_interact(GATE),
        at,
        "out-of-range gate interaction",
    );

    // Wrong NPC: with C1's battle step open, another Cobblestone NPC's Talk must not advance
    // it and must leave the whole save byte-identical.
    accept_first(&mut game, &character, &mut at);
    let jones = chain::content_position("cobblestone_village.json", 9_000_000_121);
    let mut elsewhere = character.clone();
    elsewhere.position = jones;
    let before_wrong_npc = saved(&game);
    at += 100;
    let menu = interact(&mut game, &elsewhere, 9_000_000_121, at);
    let choice = menu_option(&menu, TALK_LABEL);
    at += 100;
    let wrong = send(
        &mut game,
        &elsewhere,
        &npc_select(9_000_000_121, choice),
        at,
    );
    assert!(!wrong.changed, "another NPC's Talk changed gameplay");
    assert_eq!(
        saved(&game),
        before_wrong_npc,
        "another NPC's Talk changed the save"
    );
    assert_eq!(counts(&game, FIRST), vec![1, 0], "another NPC advanced C1");

    // Unfinished objective, now with step 2 open: the turn-in is owed nothing.
    at += 100;
    assert_packet_changes_nothing(
        &mut game,
        &character,
        &chain::quest_end_reply(FIRST),
        at,
        "turn-in before the Mongo win",
    );

    // Complete C1, then replay the 49/13 release and 49/14 after the one payment.
    finish_battle(&mut game, &mut character, &mut at);
    pay_first(&mut game, &character, &mut at);
    let paid = saved(&game);
    at += 100;
    assert_packet_changes_nothing(
        &mut game,
        &character,
        &chain::quest_end_reply(FIRST),
        at,
        "replayed C1 turn-in",
    );
    at += 100;
    let replay_screen = game
        .handle_in_world(&quest_reply(FIRST, true), &character, at)
        .expect("a replayed accept is a clean refusal");
    assert!(!replay_screen.changed);
    assert_eq!(
        saved(&game),
        paid,
        "a replayed accept changed the completed save"
    );

    // A non-Brawler session: the same packets change nothing.
    let mut other = load_game(None);
    let outsider = character_at_for(OTHER, 0, carlos_position());
    let mut at = now();
    other
        .initialize_world(&outsider, at)
        .expect("world entry initializes");
    let fresh = saved(&other);
    at += 100;
    assert_packet_changes_nothing(
        &mut other,
        &outsider,
        &quest_reply(FIRST, true),
        at,
        "non-Brawler accept",
    );
    at += 100;
    assert_packet_changes_nothing(
        &mut other,
        &outsider,
        &chain::quest_end_reply(FIRST),
        at,
        "non-Brawler turn-in",
    );
    at += 100;
    let _ = interact(&mut other, &outsider, CARLOS, at);
    let before_unoffered = saved(&other);
    at += 100;
    assert!(
        other
            .handle_in_world(&npc_select(CARLOS, i32::MAX), &outsider, at)
            .is_ok_and(|out| out.replies.is_empty() && !out.changed),
        "an option outside the open menu is refused"
    );
    assert_eq!(saved(&other), before_unoffered);
    assert_eq!(saved(&other), fresh);
    // The refused choice must not consume the open NPC menu: its Talk row still works.
    at += 100;
    let talked = other
        .handle_in_world(&npc_select(CARLOS, 1), &outsider, at)
        .expect("the open menu still accepts Talk after a refused choice");
    assert!(
        talked
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[26, 0, 3, 0])),
        "a refused choice consumed the open NPC menu"
    );
    assert_eq!(saved(&other), fresh);

    // A malformed turn-in is a per-request error, not a state change.
    at += 100;
    assert!(
        other
            .handle_in_world(&[49, 0, 14, 0], &outsider, at)
            .is_err(),
        "a truncated 49/14 must be refused"
    );
    assert_eq!(
        saved(&other),
        fresh,
        "the malformed packet changed the save"
    );
    assert_eq!(
        other.state().inventory.coins,
        coins(&other),
        "the other session is intact"
    );
}

#[test]
fn the_battle_and_quest_content_load_with_real_targets() {
    // The real loader accepts the Brawling Ring. Battle ids, not the
    // repository-wide count, are the contract: battle order is not semantic and
    // other content slices add rows (T282 adds 12340).
    let mut engine = EncounterEngine::load(&gameplay_content(), None).expect("encounter content");
    engine
        .load_field_battles(&content_dir().join("field_battles.json"))
        .expect("field battles load");
    let spawners = engine.battle_spawners();
    assert!(
        spawners.len() >= 7,
        "the shipped battles and the Brawling Ring all load"
    );
    let gate = spawners
        .iter()
        .find(|spawner| spawner.0 == GATE)
        .expect("the gate spawner is loaded");
    assert_eq!(gate.1, "Brawling Ring");
    // `world::add_static_actor` leaves `NpcSpawn.npc_name` empty and `build_add_npc` serializes
    // that field, so the client nameplate has to come from NameStringId: the recovered battle
    // title ("King of the Ring!") is the marker's locale name.
    assert_eq!(gate.2, TITLE_ID);
    assert_eq!(gate.3, GATE_MODEL);
    assert_eq!(gate.4, GATE_POSITION);
    assert!(gate.5 > 0, "the crossed-swords nameplate is visible");

    let inventory = RUNTIME_INVENTORY.get_or_init(|| {
        Inventory::load(&gameplay_content(), None).expect("the runtime item catalogue loads")
    });
    assert!(
        engine.battle_prizes_available(BATTLE, |item, quantity| {
            inventory.has_definition(item) && quantity > 0
        }),
        "every battle 12400 prize resolves in the runtime catalogue"
    );
    for prize in [GLOVES, MYSTERY_PACK, POTION] {
        assert!(
            inventory.has_definition(prize),
            "prize {prize} is not grantable"
        );
    }

    // The gate and Mongo model rows are real client assets.
    let models = fs::read_to_string(
        manifest_dir().join("../../../lab/working/OSFR.Client/OSFR Client/Resources/Models.txt"),
    )
    .unwrap();
    assert!(
        models
            .lines()
            .any(|line| line.starts_with("3126^hsg_wooden_gate_01.adr^")),
        "the gate model row exists"
    );
    assert!(
        models
            .lines()
            .any(|line| line.starts_with("168^troll_shaman.adr^")),
        "the troll model row exists"
    );
    let catalog =
        fs::read_to_string(manifest_dir().join("../../../lab/extracted/client-assets/catalog.tsv"))
            .unwrap();
    assert!(catalog.contains("\thsg_wooden_gate_01.adr\t"));
    assert!(catalog.contains("\ttroll_shaman.adr\t"));

    // The wire text ids resolve through Global.Text in the shipped locale report.
    let locale: BTreeMap<u64, String> =
        read_json(&manifest_dir().join("../../../lab/reports/locale-id-to-text.json"))
            .as_array()
            .unwrap()
            .iter()
            .map(|row| {
                (
                    row["id"].as_u64().unwrap(),
                    row["text"].as_str().unwrap().to_owned(),
                )
            })
            .collect();
    assert_eq!(locale[&(TITLE_ID as u64)], "King of the Ring!");
    assert_eq!(
        locale[&24_277],
        "Mongo is the toughest troll around! Defeat Mongo to put him in his place!"
    );
    assert_eq!(locale[&24_340], "Knock out Mongo!");
    assert_eq!(locale[&(MONGO_NAME_ID as u64)], "Mongo");
    assert_eq!(locale[&6666], "Battle Item Mystery Pack");
    assert_eq!(locale[&422_909], "Small Health Potion");
    assert!(
        locale[&132_863].contains("Saved by the Bell Brawler Gloves"),
        "the glove name id resolves"
    );

    // The arena is the existing zone; no new scene.
    let row = battle_row(BATTLE);
    assert_eq!(row["difficulty"], 1);
    assert_eq!(row["required_combat_level"], 0);
    assert_eq!(row["reward_coins"], BATTLE_COINS);
    assert_eq!(
        row["zone"],
        battle_row(96)["zone"],
        "the arena row is the template"
    );
    assert_eq!(row["zone"]["zone_id"], ARENA_ZONE);
    assert_eq!(row["enemies"].as_array().unwrap().len(), 1);
    let mongo = &row["enemies"][0];
    assert_eq!(mongo["guid"], MONGO);
    assert_eq!(mongo["name_id"], MONGO_NAME_ID);
    assert_eq!(mongo["model_id"], MONGO_MODEL);
    assert_eq!(mongo["boss"], true);
    assert_eq!(position(&mongo["position"]), [136.0, -0.6, 175.0]);

    // No pre-existing battle or Brawler record was removed or duplicated.
    // Other content slices add rows, so pin this slice's seven battles by id
    // instead of asserting the repository-wide set.
    let rows = battles();
    for id in [96, 12301, 12302, 12303, 12310, 12311, BATTLE] {
        assert_eq!(
            rows.iter()
                .filter(|row| row["id"].as_u64() == Some(u64::from(id)))
                .count(),
            1,
            "battle {id} disappeared or was duplicated"
        );
    }
    for id in [96, 12301, 12302, 12303, 12310, 12311] {
        let row = battle_row(id);
        let pinned = match id {
            96 => (42056, 42059, 42071, 25, 7, 100_000_096_000u64),
            12301 => (75452, 76503, 76505, 25, 3, 100_001_230_100),
            12302 => (42054, 42068, 42066, 25, 3, 100_001_230_200),
            12303 => (426971, 426974, 426977, 0, 4, 100_000_004_750),
            12310 => (93274, 116757, 116824, 25, 3, 100_001_231_000),
            _ => (141039, 382801, 141394, 25, 3, 100_001_231_100),
        };
        assert_eq!(
            (
                row["title_id"].as_i64().unwrap() as i32,
                row["description_id"].as_i64().unwrap() as i32,
                row["objective_id"].as_i64().unwrap() as i32,
                row["reward_coins"].as_u64().unwrap() as u32,
                row["enemies"].as_array().unwrap().len(),
                row["spawner"]["guid"].as_u64().unwrap(),
            ),
            pinned,
            "battle {id} changed"
        );
    }

    // The quest records: unique ids, a real giver, resolvable steps and provenance.
    let quests = read_json(&content_dir().join("quests/brawler.json"));
    let records = quests["records"].as_array().unwrap();
    assert_eq!(records.len(), 43);
    let mut seen = BTreeSet::new();
    for record in records {
        assert!(
            seen.insert(record["id"].as_u64().unwrap()),
            "duplicate quest id"
        );
    }
    let npcs = read_json(&content_dir().join("npcs/cobblestone_village.json"));
    assert!(
        npcs["records"]
            .as_array()
            .unwrap()
            .iter()
            .any(|npc| npc["npc_id"].as_u64() == Some(CARLOS)),
        "the C1/C2 giver is a real NPC record"
    );
    for (id, objective_kinds, encounter_step, end_text) in [
        (
            FIRST,
            ["talk_to_npc", "complete_dungeon"],
            1usize,
            FIRST_END_TEXT,
        ),
        (
            SECOND,
            ["complete_dungeon", "talk_to_npc"],
            0usize,
            SECOND_END_TEXT,
        ),
    ] {
        let record = records
            .iter()
            .find(|record| record["id"].as_u64() == Some(u64::from(id)))
            .expect("the quest record exists");
        assert_eq!(record["enabled"], true);
        assert_eq!(record["job"], BRAWLER);
        assert_eq!(record["giver_npc_id"].as_u64(), Some(CARLOS));
        assert_eq!(
            record["end_text_string_id"].as_u64(),
            Some(u64::from(end_text))
        );
        let objectives = record["objectives"].as_array().unwrap();
        assert_eq!(objectives.len(), 2);
        for (index, objective) in objectives.iter().enumerate() {
            assert_eq!(objective["kind"], objective_kinds[index]);
            assert_eq!(objective["id"].as_u64(), Some(index as u64 + 1));
            assert!(
                !objective["source"].as_array().unwrap().is_empty(),
                "quest {id} objective {index} has no provenance"
            );
        }
        assert_eq!(
            objectives[encounter_step]["encounter_id"].as_u64(),
            Some(u64::from(BATTLE))
        );
        assert_eq!(
            objectives[1 - encounter_step]["npc_id"].as_u64(),
            Some(CARLOS)
        );
        let rewards = record["rewards"].as_array().unwrap();
        assert_eq!(rewards.len(), 1);
        assert_eq!(rewards[0]["coins"], QUEST_COINS);
        assert_eq!(rewards[0]["items"].as_array().unwrap().len(), 0);
        assert_eq!(rewards[0]["job_xp"]["43"], QUEST_XP);
        assert!(
            !record["source"].as_array().unwrap().is_empty(),
            "quest {id} has no provenance"
        );
        for source in record["source"]
            .as_array()
            .unwrap()
            .iter()
            .chain(
                objectives
                    .iter()
                    .flat_map(|goal| goal["source"].as_array().unwrap().iter()),
            )
            .chain(rewards[0]["source"].as_array().unwrap().iter())
        {
            let evidence = source["evidence"].as_str().unwrap();
            assert!(!evidence.trim().is_empty(), "blank evidence on quest {id}");
            if source["kind"] == "reconstructed" {
                assert!(
                    evidence.starts_with("(R)"),
                    "reconstructed evidence must be marked (R): {evidence}"
                );
            }
        }
    }
    let first = records
        .iter()
        .find(|record| record["id"].as_u64() == Some(u64::from(FIRST)))
        .unwrap();
    assert_eq!(first["prerequisite_quest_ids"].as_array().unwrap().len(), 0);
    let second = records
        .iter()
        .find(|record| record["id"].as_u64() == Some(u64::from(SECOND)))
        .unwrap();
    assert_eq!(second["prerequisite_quest_ids"], serde_json::json!([FIRST]));
    // T101's playtest entry stays prerequisite-free and the later records keep their shape.
    let claim = records
        .iter()
        .find(|record| record["id"].as_u64() == Some(u64::from(CLAIM)))
        .unwrap();
    assert_eq!(claim["enabled"], true);
    assert_eq!(claim["prerequisite_quest_ids"].as_array().unwrap().len(), 0);
    assert_eq!(claim["objectives"][0]["kind"], "talk_to_npc");
    assert_eq!(
        claim["objectives"][0]["npc_id"].as_u64(),
        Some(9_000_000_121)
    );
    for (id, giver, prerequisite) in [
        (3_030_007u32, 9_000_000_121u64, Some(CLAIM)),
        (3_030_008, 9_000_000_121, Some(3_030_007)),
        (3_030_009, 9_000_000_446, Some(3_030_008)),
    ] {
        let record = records
            .iter()
            .find(|record| record["id"].as_u64() == Some(u64::from(id)))
            .unwrap();
        assert_eq!(record["giver_npc_id"].as_u64(), Some(giver));
        assert_eq!(
            record["prerequisite_quest_ids"].as_array().unwrap(),
            &prerequisite
                .map(|id| vec![serde_json::json!(id)])
                .unwrap_or_default()
        );
    }
    // T331 enabled C6 with its one Talk objective, Brawler gate and reward; T252's
    // C1/C2 rows above are still the only other records this slice's pin covers.
    let c6 = records
        .iter()
        .find(|record| record["id"].as_u64() == Some(3_030_009))
        .unwrap();
    assert_eq!(c6["enabled"], true, "T331 enables Given to Guard");
    assert_eq!(c6["job"], BRAWLER);
    assert_eq!(c6["objectives"].as_array().unwrap().len(), 1);
    assert_eq!(c6["objectives"][0]["npc_id"].as_u64(), Some(9_000_000_428));
}

#[test]
fn the_fight_survives_a_reconnect_mid_battle() {
    // Close the won run, then drop the session before the hand-in.
    let (mut game, mut character, mut at) = brawler_at(carlos_position());
    accept_first(&mut game, &character, &mut at);
    let before = economy(&game);
    finish_battle(&mut game, &mut character, &mut at);
    let won = economy(&game);
    assert_eq!(won.coins, before.coins + BATTLE_COINS);
    assert_eq!(won.brawler_xp, before.brawler_xp + BATTLE_XP);
    assert_eq!(prize_total(&won), prize_total(&before) + 1);
    assert_eq!(counts(&game, FIRST), vec![1, 1]);

    // Reconnect: the production loader rebuilds the session from the saved gameplay state.
    let saved_state = game.state();
    drop(game);
    let mut game = load_game(Some(saved_state));
    assert!(
        game.state().encounter.unwrap().active.is_none(),
        "reconnect releases the connection-local arena"
    );
    assert_eq!(
        economy(&game),
        won,
        "the wheel reward survives the reconnect"
    );
    assert_eq!(
        counts(&game, FIRST),
        vec![1, 1],
        "the completed objective survives"
    );
    assert_eq!(quest_status(&game, FIRST), QuestStatus::Active);
    assert_eq!(pending_turnin(&game), None);
    character.zone_id = 0;
    character.position = carlos_position();
    at += 100;
    game.initialize_world(&character, at)
        .expect("reconnected world");
    pay_first(&mut game, &character, &mut at);
    assert_eq!(economy(&game).coins, won.coins + QUEST_COINS);
    assert_eq!(economy(&game).brawler_xp, won.brawler_xp + QUEST_XP);
    at += 100;
    assert_packet_changes_nothing(
        &mut game,
        &character,
        &chain::quest_end_reply(FIRST),
        at,
        "replayed turn-in after reconnect",
    );

    // A disconnect between the killing blow and the results Close keeps the loot and the
    // objective credit: the win transition reported the completion, so the dropped run is the
    // only thing the reconnect loses.
    let (mut game, mut character, mut at) = brawler_at(carlos_position());
    accept_first(&mut game, &character, &mut at);
    let before = economy(&game);
    let details = open_gate(&mut game, &character, at);
    enter_arena(&mut game, &mut character, &details, &mut at);
    at += 100;
    win_mongo(&mut game, &mut character, &mut at);
    let paid = economy(&game);
    assert_eq!(paid.coins, before.coins + BATTLE_COINS);
    assert_eq!(
        counts(&game, FIRST),
        vec![1, 1],
        "the win already credited the dungeon objective"
    );
    let mut reloaded = load_game(Some(game.state()));
    assert_eq!(
        counts(&reloaded, FIRST),
        vec![1, 1],
        "the win credit survives the dropped run"
    );
    assert_eq!(
        economy(&reloaded),
        paid,
        "the wheel reward survives the drop and is not paid again"
    );
    assert!(reloaded.state().encounter.unwrap().active.is_none());

    // The credited objective is still handed in once after the reconnect.
    character.zone_id = 0;
    character.position = carlos_position();
    at += 100;
    reloaded
        .initialize_world(&character, at)
        .expect("reconnected world");
    at += 100;
    pay_first(&mut reloaded, &character, &mut at);
    assert_eq!(reloaded.state().inventory.coins, paid.coins + QUEST_COINS);
    assert_eq!(profile_xp(&reloaded, BRAWLER), paid.brawler_xp + QUEST_XP);
}

#[test]
fn the_gate_spawner_is_visible_to_a_second_session() {
    let mut a = load_game(None);
    let mut b = load_game(None);
    let first = character_at_for(CHARACTER, 0, GATE_POSITION);
    let second = character_at_for(OTHER, 0, GATE_POSITION);
    let mut at = now();
    switch_to_brawler(&mut a, &first, at);
    switch_to_brawler(&mut b, &second, at);
    at += 100;
    a.initialize_world(&first, at).expect("first world entry");
    b.initialize_world(&second, at).expect("second world entry");

    let gate_packet = |packets: &[Vec<u8>]| -> Vec<u8> {
        packets
            .iter()
            .find(|packet| {
                packet.starts_with(&[35, 0, 2, 0]) && packet[4..12] == GATE.to_le_bytes()
            })
            .cloned()
            .unwrap_or_else(|| panic!("the gate must be spawned through 35/2"))
    };
    let (_, first_view) = a
        .shared_npc_view(&first, &[], &BTreeSet::new())
        .expect("first shared view");
    let (_, second_view) = b
        .shared_npc_view(&second, &[], &BTreeSet::new())
        .expect("second shared view");
    let gate_first = gate_packet(&first_view);
    let gate_second = gate_packet(&second_view);
    assert_eq!(
        gate_first, gate_second,
        "the static gate actor is identical for both sessions (no per-player state)"
    );
    let spawn = parse_add_npc(&gate_first).expect("the gate AddNpc body parses");
    assert_eq!(spawn.actor_id, GATE);
    assert_eq!(
        spawn.name_string_id, TITLE_ID,
        "the nameplate locale id is the recovered battle title"
    );
    assert_eq!(spawn.model_id, GATE_MODEL);
    assert_eq!(spawn.position[0], GATE_POSITION[0]);
    assert_eq!(spawn.position[1], GATE_POSITION[1]);
    assert_eq!(spawn.position[2], GATE_POSITION[2]);
    assert_eq!(spawn.nameplate_image_id, 11_970);
    for view in [&first_view, &second_view] {
        assert!(
            !view.iter().any(|packet| {
                packet.starts_with(&[35, 0, 2, 0]) && packet[4..12] == MONGO.to_le_bytes()
            }),
            "Mongo must never be spawned into the public world"
        );
    }

    // Each session's arena is private: A's run spawns Mongo only into A's stream.
    let mut character = first.clone();
    let details = open_gate(&mut a, &character, at + 100);
    let _ = enter_arena(&mut a, &mut character, &details, &mut at);
    let (_, again) = b
        .shared_npc_view(&second, &[], &BTreeSet::new())
        .expect("second shared view after A's run");
    assert!(
        !again.iter().any(|packet| {
            packet.starts_with(&[35, 0, 2, 0]) && packet[4..12] == MONGO.to_le_bytes()
        }),
        "A's arena Mongo is not aliased into B's shared view"
    );
    assert_eq!(
        mongo_health(&a),
        Some(combat_scale::enemy_health(MONGO_AUTHORED_HEALTH))
    );
    assert_eq!(
        a.state()
            .encounter
            .unwrap()
            .active
            .unwrap()
            .enemies
            .iter()
            .filter(|enemy| enemy.guid == MONGO)
            .count(),
        1
    );
}

#[test]
fn a_failed_quest_event_for_one_player_does_not_stop_the_other() {
    let (mut a, mut character, mut at) = brawler_at(carlos_position());
    let mut b = load_game(None);
    let other = character_at_for(OTHER, 0, carlos_position());
    let mut other_at = now();
    switch_to_brawler(&mut b, &other, other_at);
    b.initialize_world(&other, other_at)
        .expect("second session world entry");

    // A completes C1's steps so the turn-in is pending; B has an open C1 offer of its own.
    accept_first(&mut a, &character, &mut at);
    finish_battle(&mut a, &mut character, &mut at);
    let opened = open_end_screen(&mut a, &character, FIRST, FIRST_END_TEXT, &mut at);
    assert!(!opened.is_empty());
    other_at += 100;
    let menu = interact(&mut b, &other, CARLOS, other_at);
    let choice = menu_option(&menu, TALK_LABEL);
    other_at += 100;
    let dialogue = send(&mut b, &other, &npc_select(CARLOS, choice), other_at);
    assert!(
        dialogue
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[26, 0, 3, 0]))
    );
    other_at += 100;
    let offered = send(&mut b, &other, &dialogue_choice(QUEST_CHOICE), other_at);
    assert!(
        offered
            .replies
            .iter()
            .any(|packet| has_family(packet, 49, 1, Some(FIRST)))
    );
    assert_eq!(quest_status(&b, FIRST), QuestStatus::Offered);

    // A's malformed quest event is a per-session error: B's pending offer and the world tick
    // keep working, and A is still usable afterwards.
    at += 100;
    assert!(
        a.handle_in_world(&[49, 0, 14, 0], &character, at).is_err(),
        "A's truncated 49/14 is refused"
    );
    other_at += 100;
    let accepted = accept(&mut b, &other, FIRST, other_at);
    assert!(accepted.changed);
    assert_eq!(quest_status(&b, FIRST), QuestStatus::Active);
    other_at += 100;
    assert!(
        b.tick(&other, other_at).is_ok(),
        "B's world tick survives A's refusal"
    );
    at += 100;
    assert!(
        a.tick(&character, at).is_ok(),
        "no quest-event error escapes A's tick"
    );

    // A's refused turn-in changes nothing; its legitimate turn-in still pays.
    at += 100;
    assert_packet_changes_nothing(
        &mut a,
        &character,
        &chain::quest_end_reply(SECOND),
        at,
        "wrong pending turn-in",
    );
    let before = economy(&a);
    at += 100;
    finish_quest(
        &mut a,
        &character,
        FIRST,
        BRAWLER,
        QUEST_COINS,
        QUEST_XP,
        at,
    );
    assert_eq!(economy(&a).coins, before.coins + QUEST_COINS);
    assert_eq!(
        b.state().inventory.coins,
        coins(&b),
        "B's wallet is untouched by A's payouts"
    );

    // B still uses the gate and starts its own Mongo fight.
    other_at += 100;
    let mut fighter = other.clone();
    let details = open_gate(&mut b, &fighter, other_at);
    let _ = enter_arena(&mut b, &mut fighter, &details, &mut other_at);
    assert_eq!(active_phase(&b), Some(EncounterPhase::Fighting));
    assert_eq!(
        mongo_health(&b),
        Some(combat_scale::enemy_health(MONGO_AUTHORED_HEALTH))
    );
    assert!(a.state().encounter.unwrap().active.is_none());
}

/// Mongo's default reach/attack schedule: stand still at the boss and let the arena
/// tick until the player is knocked out.
fn lose_mongo(game: &mut Gameplay, character: &mut CharacterState, at: &mut u64) {
    character.position = mongo_position(game);
    for _ in 0..400 {
        *at += 1_000;
        game.tick(character, *at).expect("the arena tick runs");
        if active_phase(game) == Some(EncounterPhase::Lost) {
            return;
        }
    }
    panic!("Mongo must knock the idle player out");
}

#[test]
fn a_loss_or_retry_does_not_credit_the_completion() {
    let (mut game, mut character, mut at) = brawler_at(carlos_position());
    accept_first(&mut game, &character, &mut at);
    let details = open_gate(&mut game, &character, at);
    enter_arena(&mut game, &mut character, &details, &mut at);
    let before = economy(&game);
    let quests_before = game.state().npc_quests.clone();
    let inventory_before = game.state().inventory.clone();
    assert_eq!(counts(&game, FIRST), vec![1, 0]);

    lose_mongo(&mut game, &mut character, &mut at);
    assert_eq!(active_phase(&game), Some(EncounterPhase::Lost));
    assert_eq!(
        game.state().npc_quests,
        quests_before,
        "the loss credited C1"
    );
    assert_eq!(
        game.state().inventory,
        inventory_before,
        "the loss paid a reward"
    );
    assert_eq!(counts(&game, FIRST), vec![1, 0], "the loss credited C1");

    // The lost run is retried (the ordinary `realm retry` route for a battle without a
    // guide); the retry must not credit or pay either.
    at += 100;
    let retried = send(
        &mut game,
        &character,
        &chat::local_request("realm retry"),
        at,
    );
    let zone = retried.zoning.expect("Retry restarts the arena");
    assert_eq!(zone.zone_id, ARENA_ZONE);
    assert_eq!(active_phase(&game), Some(EncounterPhase::Loading));
    assert_eq!(
        game.state().npc_quests,
        quests_before,
        "the retry credited C1"
    );
    assert_eq!(
        game.state().inventory,
        inventory_before,
        "the retry paid a reward"
    );
    assert_eq!(economy(&game), before, "the retry paid a battle reward");
    assert_eq!(counts(&game, FIRST), vec![1, 0]);

    character.zone_id = zone.zone_id;
    character.position = zone.position;
    at += 100;
    game.initialize_world(&character, at)
        .expect("the retried arena loads");
    assert_eq!(active_phase(&game), Some(EncounterPhase::Fighting));

    at += 100;
    win_mongo(&mut game, &mut character, &mut at);
    assert_eq!(
        counts(&game, FIRST),
        vec![1, 1],
        "the retried win credits the objective once"
    );
    let paid = economy(&game);
    assert_eq!(paid.coins, before.coins + BATTLE_COINS, "the win pays once");

    at += 100;
    assert!(
        send(&mut game, &character, &[26, 0, 42, 0], at)
            .zoning
            .is_some()
    );
    assert_eq!(
        counts(&game, FIRST),
        vec![1, 1],
        "the close credits the retried win again"
    );
    assert_eq!(economy(&game), paid, "the close pays the win again");
}

#[test]
fn two_players_win_separately_and_credit_only_themselves() {
    let (mut a, mut a_character, mut a_at) = brawler_at(carlos_position());
    accept_first(&mut a, &a_character, &mut a_at);

    let mut b = load_game_for(OTHER, None);
    let mut b_character = character_at_for(OTHER, 0, carlos_position());
    let mut b_at = now();
    switch_to_brawler(&mut b, &b_character, b_at);
    b_at += 100;
    b.initialize_world(&b_character, b_at)
        .expect("second world entry");
    accept_first(&mut b, &b_character, &mut b_at);

    // A's win credits A's active objective only.
    let a_before_win = economy(&a);
    let b_before = saved(&b);
    let details = open_gate(&mut a, &a_character, a_at);
    enter_arena(&mut a, &mut a_character, &details, &mut a_at);
    a_at += 100;
    win_mongo(&mut a, &mut a_character, &mut a_at);
    assert_eq!(counts(&a, FIRST), vec![1, 1], "A's win credits A");
    assert_eq!(counts(&b, FIRST), vec![1, 0], "A's win credits B");
    assert_eq!(saved(&b), b_before, "A's win changed B's session");
    let a_paid = economy(&a);
    assert_eq!(
        a_paid.coins,
        a_before_win.coins + BATTLE_COINS,
        "A's win pays A once"
    );
    a_at += 100;
    assert!(
        send(&mut a, &a_character, &[26, 0, 42, 0], a_at)
            .zoning
            .is_some()
    );

    // B's own run credits B and leaves A alone.
    let a_after_win = saved(&a);
    let b_before_run = economy(&b);
    let details = open_gate(&mut b, &b_character, b_at);
    enter_arena(&mut b, &mut b_character, &details, &mut b_at);
    b_at += 100;
    win_mongo(&mut b, &mut b_character, &mut b_at);
    assert_eq!(counts(&b, FIRST), vec![1, 1], "B's win credits B");
    assert_eq!(counts(&a, FIRST), vec![1, 1], "B's win changed A's credit");
    assert_eq!(saved(&a), a_after_win, "B's win changed A's session");
    assert_eq!(economy(&a), a_paid, "A's battle was paid by A's win only");
    assert_eq!(
        economy(&b).coins,
        b_before_run.coins + BATTLE_COINS,
        "B's own win pays B once"
    );
}

#[test]
fn a_replayed_close_or_results_packet_does_not_double_count() {
    let (mut game, mut character, mut at) = brawler_at(carlos_position());
    accept_first(&mut game, &character, &mut at);
    let details = open_gate(&mut game, &character, at);
    enter_arena(&mut game, &mut character, &details, &mut at);
    at += 100;
    win_mongo(&mut game, &mut character, &mut at);
    let paid = economy(&game);
    assert_eq!(counts(&game, FIRST), vec![1, 1]);
    let credited = saved(&game);

    // A stale start-screen frame while the results are open changes nothing.
    for _ in 0..2 {
        at += 100;
        let replay = game
            .handle_in_world(&details, &character, at)
            .expect("a stale 41/114 frame is a clean refusal");
        assert!(!replay.changed, "the stale frame changed the won run");
        assert!(replay.replies.is_empty(), "the stale frame sent a packet");
    }
    assert_eq!(
        counts(&game, FIRST),
        vec![1, 1],
        "the stale frame double-counted C1"
    );
    assert_eq!(saved(&game), credited, "the stale frame changed the save");

    // The results Close once, then every replay of it (or of the details frame)
    // is inert: no second counter and no second payment.
    at += 100;
    assert!(
        send(&mut game, &character, &[26, 0, 42, 0], at)
            .zoning
            .is_some()
    );
    let closed = saved(&game);
    for packet in [[26_u8, 0, 42, 0].as_slice(), details.as_slice()] {
        for _ in 0..2 {
            at += 100;
            let replay = game
                .handle_in_world(packet, &character, at)
                .expect("a replayed results frame is a clean refusal");
            assert!(!replay.changed, "the replay changed gameplay");
            assert!(replay.replies.is_empty(), "the replay sent a packet");
            assert!(replay.zoning.is_none(), "the replay zoned the player");
        }
        assert_eq!(saved(&game), closed, "the replay changed the closed save");
        assert_eq!(
            counts(&game, FIRST),
            vec![1, 1],
            "the replay double-counted C1"
        );
        assert_eq!(economy(&game), paid, "the replay paid a reward again");
    }
}
