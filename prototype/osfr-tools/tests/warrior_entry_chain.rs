//! T257: the first playable Warrior chapter, "Becoming a Warrior" (3150001) and
//! "Warrior: One With Your Weapon" (3150004), plus the drawable Morninglory and
//! Steele spawns that make the chapter reachable. T361 enables the level gate
//! "Warrior: Achieve Level 5" (3150005), so this file also pins its record shape.
//!
//! The chapter is content-only; these tests drive the production in-world router
//! (`Gameplay::handle_in_world` -> `gameplay/npc.rs` -> `QuestEngine`), the same path
//! the gateway calls for every in-world request, instead of engine internals.
//!
//! Client request/reply layouts used here (pinned in the client):
//! - 26/8 NPC interact: constructor `008b6bc0`; u64 target + u8 [0].
//! - 26/10 menu select: constructor `00a36f80`; u64 target + i32 choice.
//! - 26/6 dialogue response: constructor `009f5890`; i32 row (301 is the quest row).
//! - 26/13 job select: i32 profile id.
//! - 49/1 QuestInfo: reader `00c7bb60`; the quest id is the first body dword and the
//!   intro locale id the second (decoded by the production offer encoder).
//! - 49/2 quest reply: constructor `00a86400`; u32 quest + u8 accept.
//! - 49/3 QuestAdd: reader `00c7bd10`.
//! - 49/13 QuestEnd: writer/reader `00c7bbd0`/`00c7b990`; actor u64 at 6..14, quest
//!   i32 at 14..18, end-text locale at 18..22, animation at 22..26, reward bundle
//!   from 26 (success byte, coins i32, stars i32, ..., source/player u64s, name id).
//! - 49/14 end reply: constructor `00a864c0`; u32 quest.
//! - 35/2 AddNpc: constructor `0091cac0`, body reader `00921a10` (u64 actor id, then
//!   i32 name id, then i32 model id), parsed by the production `parse_add_npc`.
//!
//! The shared fixture and packet helpers live in `tests/support/npc_quest_chain.rs`.
//! The fixture has no Warrior starter outfit, so the Warrior session is seeded the
//! way the Wizard/Ninja chapter tests seed a job save; the trainer menu is still
//! checked through the real 26/8 -> 26/10 frame path.

#[path = "support/npc_quest_chain.rs"]
mod npc_quest_chain;

use npc_quest_chain::*;
use osfr_tools::{
    character_store::CharacterState,
    game_store,
    gameplay::{Gameplay, GameplayState},
    profiles::ProfileState,
    quest_engine::{CharacterEligibility, QuestEngine, QuestStatus},
    startup_world::parse_add_npc,
    world,
};
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

const SECOND_CHARACTER: u64 = 194_018;
const DEWEY: u64 = 9_000_000_076;
const MORNINGLORY: u64 = 9_000_000_155;
const STEELE: u64 = 9_000_000_208;
const WARRIOR: i32 = 32;
const BRAWLER: i32 = 43;
const W1: u32 = 3_150_001;
const W2: u32 = 3_150_002;
const W3: u32 = 3_150_003;
const W4: u32 = 3_150_004;
const W5: u32 = 3_150_005;
const W1_TITLE: u32 = 22_526;
const W4_TITLE: u32 = 22_580;
const W4_DESCRIPTION: u32 = 22_581;
const W4_INTRO: u32 = 22_579;
const REWARD_COINS: u32 = 5;
const REWARD_XP: u32 = 25;
const LEARN_LABEL: u32 = 412_471;
/// The open continent is world zone 0; regional zone ids are content labels. The
/// regional Snowhill NPC rows accept a zone-0 player for range checks, and the
/// shared NPC view places every extra spawn in zone 0.
const OPEN_WORLD: u32 = 0;
const MORNINGLORY_NAME: u64 = 5_102_192;
const STEELE_NAME: u64 = 434_620;
const MORNINGLORY_MODEL: u32 = 731;
const STEELE_MODEL: u32 = 391;
const MORNINGLORY_MODEL_FILE: &str = "fairy_f_morninglory.agr";
const STEELE_MODEL_FILE: &str = "fairy_m_warrior_01.agr";
/// A Brawler rank-5 character on the local 100-XP rank curve (T201's gate).
const BRAWLER_RANK_FIVE_XP: u32 = 400;

static NEXT_SAVE: AtomicU64 = AtomicU64::new(0);

fn avatar_at(npc_id: u64) -> CharacterState {
    character_at(OPEN_WORLD, content_position("snowhill.json", npc_id))
}

fn avatar_for(guid: u64, npc_id: u64) -> CharacterState {
    character_at_for(guid, OPEN_WORLD, content_position("snowhill.json", npc_id))
}

fn profile_state(
    active: i32,
    unlocked: BTreeSet<i32>,
    experience: BTreeMap<i32, u32>,
) -> ProfileState {
    ProfileState {
        version: 1,
        active,
        unlocked,
        experience,
    }
}

fn warrior_unlocks() -> BTreeSet<i32> {
    BTreeSet::from([1, WARRIOR, BRAWLER])
}

/// A saved Warrior session: profile 32 unlocked and active at rank 1.
fn warrior_state(fixture: &Fixture) -> GameplayState {
    let mut state = Gameplay::load(fixture.path(), None)
        .expect("opening fixture loads")
        .state();
    state.profiles = Some(profile_state(WARRIOR, warrior_unlocks(), BTreeMap::new()));
    state
}

/// A Warrior the player has unlocked but is not playing.
fn inactive_warrior_state(fixture: &Fixture) -> GameplayState {
    let mut state = warrior_state(fixture);
    state.profiles = Some(profile_state(BRAWLER, warrior_unlocks(), BTreeMap::new()));
    state
}

/// A Brawler high enough to see the trainer's Become a Warrior option.
fn brawler_rank_five_state(fixture: &Fixture) -> GameplayState {
    let mut state = warrior_state(fixture);
    state.profiles = Some(profile_state(
        BRAWLER,
        BTreeSet::from([1, BRAWLER]),
        BTreeMap::from([(BRAWLER, BRAWLER_RANK_FIVE_XP)]),
    ));
    state
}

fn warrior_game(fixture: &Fixture) -> Gameplay {
    fixture.game(Some(warrior_state(fixture)))
}

fn warrior_game_for(fixture: &Fixture, guid: u64) -> Gameplay {
    fixture.game_as(guid, Some(warrior_state(fixture)))
}

fn pending_offer(game: &Gameplay) -> Option<u32> {
    game.state().npc_quests.as_ref().unwrap().pending_offer
}

fn has_quest(game: &Gameplay, quest_id: u32) -> bool {
    game.state()
        .npc_quests
        .as_ref()
        .unwrap()
        .quests
        .contains_key(&quest_id)
}

fn unlocked(game: &Gameplay, profile: i32) -> bool {
    game.state()
        .profiles
        .as_ref()
        .unwrap()
        .unlocked
        .contains(&profile)
}

/// The 49/13 end screen for W1/W4: both carry the same small-quest reward. The
/// reward bundle starts at byte 26; 27..31 is the coin row, 31..35 the job-XP row,
/// 63..71 the display source GUID (the giver), 71..79 the player, 83..87 the title.
fn assert_reward_bundle(end: &[u8], quest_id: u32, player: u64) {
    let (giver, title) = match quest_id {
        W1 => (DEWEY, W1_TITLE),
        W4 => (MORNINGLORY, W4_TITLE),
        other => panic!("quest {other} has no reward in this chapter"),
    };
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
        u64::from_le_bytes(end[63..71].try_into().unwrap()),
        giver,
        "the reward display source is the quest giver"
    );
    assert_eq!(
        u64::from_le_bytes(end[71..79].try_into().unwrap()),
        player,
        "the reward display names the character"
    );
    assert_eq!(
        u32::from_le_bytes(end[83..87].try_into().unwrap()),
        title,
        "the reward display names the quest title"
    );
}

fn release_reward(
    game: &mut Gameplay,
    character: &CharacterState,
    quest_id: u32,
    expected_actor: u64,
    now: u64,
) -> Vec<u8> {
    let end = release_end_screen(game, character, quest_id, expected_actor, 0, now);
    assert_reward_bundle(&end, quest_id, character.guid);
    end
}

/// Offer, accept, hear the battle cry, release the end screen and turn in W1.
fn complete_w1(game: &mut Gameplay, character: &CharacterState, now: &mut u64) {
    *now += 10;
    offer(game, character, DEWEY, W1, *now);
    *now += 10;
    accept(game, character, W1, *now);
    finish_w1(game, character, now);
}

/// The remaining W1 steps for a quest already active.
fn finish_w1(game: &mut Gameplay, character: &CharacterState, now: &mut u64) {
    *now += 10;
    let talked = talk(game, character, DEWEY, *now);
    assert!(talked.changed, "the battle-cry talk must complete W1");
    assert_eq!(counts(game, W1), vec![1]);
    assert_eq!(pending_turnin(game), Some(W1));
    *now += 10;
    release_reward(game, character, W1, DEWEY, *now);
    *now += 10;
    finish_quest(game, character, W1, WARRIOR, REWARD_COINS, REWARD_XP, *now);
}

// ---- content helpers -------------------------------------------------------

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

fn lab_root() -> PathBuf {
    repo_root().parent().unwrap().join("lab")
}

fn spawn_rows() -> Vec<Value> {
    read_json(&content_dir().join("npc_spawns_extra.json"))["records"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|row| {
            matches!(
                row.get("content_key").and_then(Value::as_u64),
                Some(MORNINGLORY) | Some(STEELE)
            )
        })
        .cloned()
        .collect()
}

fn one_spawn_row(key: u64) -> Value {
    let rows: Vec<Value> = spawn_rows()
        .into_iter()
        .filter(|row| row["content_key"].as_u64() == Some(key))
        .collect();
    assert_eq!(
        rows.len(),
        1,
        "content key {key} must bind exactly one spawn"
    );
    rows[0].clone()
}

/// Build the production loader and return the 35/2 AddNpc for `npc_id` at its
/// Snowhill row's position.
fn drawn_npc(npc_id: u64) -> osfr_tools::startup_world::NpcSpawn {
    let position = content_position("snowhill.json", npc_id);
    let mut visibility = world::NpcVisibility::load(&content_dir()).expect("NpcVisibility loads");
    visibility
        .update(OPEN_WORLD, position)
        .expect("NPC visibility update")
        .iter()
        .filter_map(|packet| parse_add_npc(packet).ok())
        .find(|npc| npc.actor_id == npc_id)
        .unwrap_or_else(|| panic!("NPC {npc_id} must be drawn for a nearby player"))
}

// ---- tests -----------------------------------------------------------------

#[test]
fn dewey_offers_becoming_a_warrior_only_to_warriors() {
    let fixture = Fixture::new();
    let dewey = avatar_at(DEWEY);
    let mut now = 1_000;

    // A default Adventurer gets no W1 offer and no state change.
    let mut ordinary = fixture.game(None);
    assert_no_offer(
        &mut ordinary,
        &dewey,
        DEWEY,
        &mut now,
        "Adventurer at Dewey",
    );
    assert!(!has_quest(&ordinary, W1));
    assert_eq!(pending_offer(&ordinary), None);

    // An unlocked Warrior that is not the active job is refused by the job gate.
    let mut inactive = fixture.game(Some(inactive_warrior_state(&fixture)));
    assert_no_offer(&mut inactive, &dewey, DEWEY, &mut now, "inactive Warrior");
    assert!(!has_quest(&inactive, W1));
    assert!(!has_quest(&inactive, W4));

    // The trainer menu still offers "Become a Warrior" once the rank gate is met (T201).
    let mut trainer = fixture.game(Some(brawler_rank_five_state(&fixture)));
    let menu = interact(&mut trainer, &dewey, DEWEY, now + 10);
    let learn = menu_option(&menu, LEARN_LABEL);
    let learned = send(&mut trainer, &dewey, &npc_select(DEWEY, learn), now + 20);
    assert!(learned.changed, "Dewey's Learn choice must commit");
    assert!(unlocked(&trainer, WARRIOR));
    assert_eq!(
        active_job(&trainer),
        BRAWLER,
        "learning must not switch jobs"
    );
    // Unlocking without activating Warrior still cannot open the quest.
    assert_no_offer(
        &mut trainer,
        &dewey,
        DEWEY,
        &mut now,
        "learned but inactive Warrior",
    );
    assert!(!has_quest(&trainer, W1));

    // The active Warrior sees exactly one W1 offer.
    let mut game = warrior_game(&fixture);
    let offered = offer(&mut game, &dewey, DEWEY, W1, now + 10);
    assert_eq!(
        offered
            .replies
            .iter()
            .filter(|packet| packet.starts_with(&[49, 0, 1, 0]))
            .count(),
        1,
        "one quest choice offers exactly one quest"
    );
    assert_eq!(pending_offer(&game), Some(W1));
    assert_eq!(quest_status(&game, W1), QuestStatus::Offered);
    assert_eq!(counts(&game, W1), vec![0]);
    assert!(!has_quest(&game, W4), "W4 waits for W1");
    assert!(!has_quest(&game, W2) && !has_quest(&game, W3) && !has_quest(&game, W5));

    // A second Talk while the offer is open starts no second offer and changes nothing.
    let before = saved(&game);
    assert_no_offer(&mut game, &dewey, DEWEY, &mut now, "replayed W1 offer");
    assert_eq!(saved(&game), before, "the replayed offer changed the save");
    assert_eq!(pending_offer(&game), Some(W1));

    // The exact 5-coin / 25-Warrior-XP reward is carried by the end screen and
    // asserted again as a delta in the battle-cry test below.
    now += 10;
    accept(&mut game, &dewey, W1, now);
    now += 10;
    assert!(talk(&mut game, &dewey, DEWEY, now).changed);
    now += 10;
    release_reward(&mut game, &dewey, W1, DEWEY, now);
}

#[test]
fn the_battle_cry_talk_completes_w1_and_pays_warrior_xp() {
    let fixture = Fixture::new();
    let character = avatar_at(DEWEY);
    let mut now = 2_000;
    let mut game = warrior_game(&fixture);
    let coins_before = coins(&game);
    let warrior_before = profile_xp(&game, WARRIOR);
    assert_eq!(warrior_before, 0);

    now += 10;
    offer(&mut game, &character, DEWEY, W1, now);
    now += 10;
    accept(&mut game, &character, W1, now);
    assert_eq!(counts(&game, W1), vec![0]);
    assert_eq!(coins(&game), coins_before, "accepting pays nothing");

    now += 10;
    let talked = talk(&mut game, &character, DEWEY, now);
    assert!(
        talked.changed,
        "the battle-cry talk must complete the objective"
    );
    assert_eq!(counts(&game, W1), vec![1]);
    assert_eq!(pending_turnin(&game), Some(W1));
    assert!(
        !talked
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[49, 0, 13, 0])),
        "the end screen is held for the 26/6 dialogue choice"
    );
    assert_eq!(coins(&game), coins_before, "no reward before the turn-in");

    now += 10;
    release_reward(&mut game, &character, W1, DEWEY, now);

    // Switch to another unlocked job before the turn-in: the explicit reward map,
    // not the active job, decides which profile gains the XP.
    let brawler_before = profile_xp(&game, BRAWLER);
    now += 10;
    let switched = send(&mut game, &character, &select_job(BRAWLER), now);
    assert!(switched.changed, "Brawler must be selectable");
    assert_eq!(active_job(&game), BRAWLER);
    let items_before = game.state().inventory.items.clone();
    let equipment_before = game.state().inventory.equipment.clone();

    now += 10;
    finish_quest(
        &mut game,
        &character,
        W1,
        WARRIOR,
        REWARD_COINS,
        REWARD_XP,
        now,
    );
    assert_eq!(
        active_job(&game),
        BRAWLER,
        "the turn-in must not switch jobs"
    );
    assert_eq!(profile_xp(&game, WARRIOR), warrior_before + REWARD_XP);
    assert_eq!(
        profile_xp(&game, BRAWLER),
        brawler_before,
        "the active job must not receive the Warrior XP"
    );
    assert_eq!(coins(&game), coins_before + REWARD_COINS);
    assert_eq!(quest_status(&game, W1), QuestStatus::Completed);
    assert_eq!(
        game.state().inventory.items,
        items_before,
        "the turn-in must not grant an item"
    );
    assert_eq!(
        game.state().inventory.equipment,
        equipment_before,
        "the turn-in must not equip an item"
    );

    // A repeated 49/14 pays nothing.
    now += 10;
    assert_packet_changes_nothing(
        &mut game,
        &character,
        &quest_end_reply(W1),
        now,
        "repeated W1 turn-in",
    );
    assert_eq!(coins(&game), coins_before + REWARD_COINS);
    assert_eq!(profile_xp(&game, WARRIOR), warrior_before + REWARD_XP);
}

#[test]
fn morninglory_offers_w4_after_w1_and_steele_completes_it() {
    let fixture = Fixture::new();
    let dewey = avatar_at(DEWEY);
    let morninglory = avatar_at(MORNINGLORY);
    let steele = avatar_at(STEELE);
    let mut now = 3_000;
    let mut game = warrior_game(&fixture);
    let coins_before = coins(&game);
    let xp_before = profile_xp(&game, WARRIOR);

    // W4 is not offered before W1 is completed.
    assert_no_offer(
        &mut game,
        &morninglory,
        MORNINGLORY,
        &mut now,
        "W4 before W1",
    );
    assert!(!has_quest(&game, W4));

    complete_w1(&mut game, &dewey, &mut now);
    assert_eq!(coins(&game), coins_before + REWARD_COINS);
    assert_eq!(profile_xp(&game, WARRIOR), xp_before + REWARD_XP);

    // Morninglory (the new drawable spawn) offers W4 now, and not W1 again.
    now += 10;
    let offered = offer(&mut game, &morninglory, MORNINGLORY, W4, now);
    assert!(
        !offered
            .replies
            .iter()
            .any(|packet| has_family(packet, 49, 1, Some(W1))),
        "the completed W1 must not be offered again"
    );
    assert_eq!(pending_offer(&game), Some(W4));
    now += 10;
    accept(&mut game, &morninglory, W4, now);
    assert_eq!(counts(&game, W4), vec![0]);

    // The objective targets Steele, not Morninglory: a Talk at the giver does not
    // advance it.
    now += 10;
    let wrong = talk(&mut game, &morninglory, MORNINGLORY, now);
    assert!(
        !wrong.changed,
        "the giver must not complete the report step"
    );
    assert_eq!(counts(&game, W4), vec![0]);

    // Steele (the new drawable spawn) completes the report.
    now += 10;
    let reported = talk(&mut game, &steele, STEELE, now);
    assert!(reported.changed, "the Steele report must complete W4");
    assert_eq!(counts(&game, W4), vec![1]);
    assert_eq!(pending_turnin(&game), Some(W4));
    assert!(
        !reported
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[49, 0, 13, 0])),
        "the end screen is held for the 26/6 dialogue choice"
    );
    assert_eq!(
        coins(&game),
        coins_before + REWARD_COINS,
        "no reward before the W4 turn-in"
    );

    now += 10;
    release_reward(&mut game, &steele, W4, STEELE, now);
    now += 10;
    finish_quest(
        &mut game,
        &steele,
        W4,
        WARRIOR,
        REWARD_COINS,
        REWARD_XP,
        now,
    );
    assert_eq!(
        coins(&game),
        coins_before + 2 * REWARD_COINS,
        "the two chapter rewards"
    );
    assert_eq!(profile_xp(&game, WARRIOR), xp_before + 2 * REWARD_XP);
    assert_eq!(quest_status(&game, W1), QuestStatus::Completed);
    assert_eq!(quest_status(&game, W4), QuestStatus::Completed);

    // The completed chapter is not offered again.
    assert_no_offer(
        &mut game,
        &morninglory,
        MORNINGLORY,
        &mut now,
        "W4 replay after completion",
    );
}

#[test]
fn refusals_and_replay_change_nothing() {
    let fixture = Fixture::new();
    let dewey = avatar_at(DEWEY);
    let morninglory = avatar_at(MORNINGLORY);
    let steele = avatar_at(STEELE);
    let mut now = 4_000;
    let mut game = warrior_game(&fixture);
    let start = saved(&game);

    // Wrong NPC: Morninglory cannot offer or advance W1.
    let before = saved(&game);
    assert_no_offer(
        &mut game,
        &morninglory,
        MORNINGLORY,
        &mut now,
        "wrong giver for W1",
    );
    assert_eq!(saved(&game), before, "the wrong NPC changed the save");
    assert!(!has_quest(&game, W1));
    assert!(!has_quest(&game, W4));

    // Missing prerequisite: a forged 49/2 for W4 changes nothing, and so does an
    // accept for W1 that was never offered.
    assert_packet_changes_nothing(
        &mut game,
        &morninglory,
        &quest_reply(W4, true),
        now + 1,
        "forged W4 accept without W1",
    );
    assert_packet_changes_nothing(
        &mut game,
        &dewey,
        &quest_reply(W1, true),
        now + 2,
        "W1 accept without an offer",
    );
    assert_eq!(saved(&game), start, "a forged accept changed the save");

    // Start W1; a Talk at the wrong NPC (Steele) cannot complete the battle cry.
    now += 10;
    offer(&mut game, &dewey, DEWEY, W1, now);
    now += 10;
    accept(&mut game, &dewey, W1, now);
    now += 10;
    let before = saved(&game);
    let wrong = talk(&mut game, &steele, STEELE, now);
    assert!(!wrong.changed, "a wrong NPC advanced W1");
    assert_eq!(saved(&game), before, "the wrong NPC changed the save");
    assert_eq!(counts(&game, W1), vec![0]);

    // Complete and turn in W1.
    finish_w1(&mut game, &dewey, &mut now);
    let finished = saved(&game);
    let finished_coins = coins(&game);
    let finished_xp = profile_xp(&game, WARRIOR);

    // Replayed turn-ins and accepts pay nothing and change no byte.
    for (label, packet) in [
        ("duplicate W1 turn-in", quest_end_reply(W1)),
        ("replayed W1 accept", quest_reply(W1, true)),
        ("forged W4 turn-in", quest_end_reply(W4)),
        ("forged W4 accept", quest_reply(W4, true)),
    ] {
        assert_packet_changes_nothing(&mut game, &dewey, &packet, now + 10, label);
    }
    assert_eq!(saved(&game), finished, "a replay changed the save");
    assert_eq!(coins(&game), finished_coins);
    assert_eq!(profile_xp(&game, WARRIOR), finished_xp);

    // Out of range: no menu and no state change, even with a real actor id.
    let far = CharacterState {
        position: [
            dewey.position[0] + 400.0,
            dewey.position[1],
            dewey.position[2],
        ],
        ..dewey.clone()
    };
    let before = saved(&game);
    let refused = game.handle_in_world(&npc_interact(DEWEY), &far, now + 20);
    assert!(refused.is_err(), "an out-of-range 26/8 must be refused");
    assert_eq!(
        saved(&game),
        before,
        "the out-of-range menu changed the save"
    );

    // A forged runtime actor that no content spawn binds changes nothing.
    assert_packet_changes_nothing(
        &mut game,
        &dewey,
        &npc_interact(123_456_789),
        now + 21,
        "unknown runtime actor",
    );
}

#[test]
fn the_two_spawns_and_quests_load_with_real_targets() {
    // ---- the two extra spawn rows -----------------------------------------
    let rows = spawn_rows();
    assert_eq!(rows.len(), 2, "exactly the two T257 spawns are bound");
    let locale: BTreeMap<u64, String> =
        read_json(&lab_root().join("reports/locale-id-to-text.json"))
            .as_array()
            .unwrap()
            .iter()
            .map(|row| {
                (
                    row["id"].as_u64().unwrap(),
                    row["text"].as_str().unwrap().into(),
                )
            })
            .collect();
    let models: BTreeMap<u32, String> = fs::read_to_string(
        lab_root().join("references/FreeRealms-Legacy/src/Resources/Models.txt"),
    )
    .unwrap()
    .lines()
    .filter_map(|line| {
        let mut parts = line.split('^');
        Some((parts.next()?.parse().ok()?, parts.next()?.to_owned()))
    })
    .collect();
    let catalog =
        fs::read_to_string(lab_root().join("extracted/client-assets/catalog.tsv")).unwrap();
    let catalog_names: BTreeSet<String> = catalog
        .lines()
        .skip(1)
        .filter_map(|line| line.split('\t').nth(2))
        .map(str::to_ascii_lowercase)
        .collect();

    for (key, name, name_id, model_id, model_file) in [
        (
            MORNINGLORY,
            "Morninglory",
            MORNINGLORY_NAME,
            MORNINGLORY_MODEL,
            MORNINGLORY_MODEL_FILE,
        ),
        (
            STEELE,
            "Steele",
            STEELE_NAME,
            STEELE_MODEL,
            STEELE_MODEL_FILE,
        ),
    ] {
        let row = one_spawn_row(key);
        assert_eq!(row["spawn_guid"].as_u64(), Some(key));
        assert_eq!(row["name"].as_str(), Some(name));
        assert_eq!(row["name_string_id"].as_u64(), Some(name_id));
        assert_eq!(
            locale.get(&name_id).map(String::as_str),
            Some(name),
            "the AddNpc name id must resolve through Global.Text"
        );
        assert!(row["legacy_npc_id"].is_null());
        assert!(row["capture_match"].is_null());
        assert_eq!(row["model_id"].as_u64(), Some(u64::from(model_id)));
        assert_eq!(row["model_file_name"].as_str(), Some(model_file));
        assert_eq!(
            models.get(&model_id).map(String::as_str),
            Some(model_file),
            "the model id and file must match the client/Legacy model table"
        );
        assert!(
            catalog_names.contains(&model_file.to_ascii_lowercase()),
            "the model file must exist in the client catalog: {model_file}"
        );
        let position: [f32; 3] = serde_json::from_value(row["position"].clone()).unwrap();
        assert!(position.iter().all(|value| value.is_finite()));
        let snowhill = read_json(&content_dir().join("npcs/snowhill.json"));
        let regional_row = snowhill["records"]
            .as_array()
            .unwrap()
            .iter()
            .find(|npc| npc["npc_id"].as_u64() == Some(key))
            .unwrap();
        let regional: [f32; 3] = serde_json::from_value(regional_row["position"].clone()).unwrap();
        for axis in 0..3 {
            assert!(
                (position[axis] - regional[axis]).abs() < 1e-3,
                "the spawn must stand where the Snowhill row does"
            );
        }
        // T506: both stand at Sanctuary-main's placement, heading included.
        assert_eq!(regional_row["placement"], "reference_data");
        assert!(
            row["heading"]
                .as_f64()
                .is_some_and(|heading| heading != 0.0)
        );
        assert_eq!(row["heading"], regional_row["heading"]);
        let sources = row["source"].as_array().unwrap();
        assert_eq!(
            sources.len(),
            6,
            "five provenance entries and the reference"
        );
        assert!(
            sources
                .iter()
                .any(|source| source["kind"].as_str() == Some("legacy-data")),
            "every extra spawn needs the Legacy tag"
        );
        for source in sources {
            assert!(
                source["evidence"]
                    .as_str()
                    .is_some_and(|evidence| !evidence.trim().is_empty()),
                "spawn provenance must not be empty"
            );
        }

        // The production loader draws the row for a nearby player.
        let drawn = drawn_npc(key);
        assert_eq!(drawn.actor_id, key);
        assert_eq!(drawn.model_id, model_id as i32);
        assert_eq!(drawn.name_string_id, name_id as i32);
        for axis in 0..3 {
            assert!(
                (drawn.position[axis] - position[axis]).abs() < 1e-3,
                "axis {axis}: AddNpc position differs from the content row"
            );
        }
    }

    // ---- both quest records load and their givers/targets resolve ----------
    let engine = QuestEngine::from_split_content(
        &content_dir(),
        CHARACTER,
        CharacterEligibility::default(),
        None,
    )
    .expect("the split quest catalogue loads");
    assert!(engine.is_enabled(W1));
    assert!(engine.is_enabled(W4));
    assert_eq!(engine.giver_npc_id(W1), Some(DEWEY));
    assert_eq!(engine.turnin_npc_id(W1), Some(DEWEY));
    assert_eq!(engine.giver_npc_id(W4), Some(MORNINGLORY));
    assert_eq!(engine.turnin_npc_id(W4), Some(MORNINGLORY));
    for npc in [DEWEY, MORNINGLORY, STEELE] {
        let mut visibility =
            world::NpcVisibility::load(&content_dir()).expect("NpcVisibility loads");
        assert_eq!(
            visibility.content_key(npc),
            Some(npc),
            "NPC {npc} must bind its content key to a drawn actor"
        );
        assert!(
            !visibility
                .update(OPEN_WORLD, content_position("snowhill.json", npc))
                .unwrap()
                .is_empty()
        );
    }

    // ---- record table: W1, W4 and the W5 level gate are enabled; the rest stay hollow --
    let document = read_json(&content_dir().join("quests/warrior.json"));
    let records = document["records"].as_array().unwrap();
    let ids: Vec<u64> = records
        .iter()
        .map(|row| row["id"].as_u64().unwrap())
        .collect();
    assert_eq!(
        ids.iter().copied().collect::<BTreeSet<_>>().len(),
        ids.len(),
        "Warrior quest ids are unique"
    );
    let enabled: Vec<&Value> = records
        .iter()
        .filter(|row| row["enabled"].as_bool().unwrap_or(false))
        .collect();
    assert_eq!(
        enabled
            .iter()
            .map(|row| row["id"].as_u64().unwrap())
            .collect::<Vec<_>>(),
        vec![u64::from(W1), u64::from(W4), u64::from(W5)]
    );
    for record in &enabled {
        if record["id"].as_u64() == Some(u64::from(W5)) {
            continue; // the level gate has its own shape below
        }
        assert_eq!(record["job"].as_i64(), Some(i64::from(WARRIOR)));
        assert!(
            record["source"]
                .as_array()
                .is_some_and(|value| !value.is_empty())
        );
        let objectives = record["objectives"].as_array().unwrap();
        assert_eq!(objectives.len(), 1, "one talk goal per enabled record");
        assert_eq!(objectives[0]["kind"], "talk_to_npc");
        assert!(
            objectives[0]["source"]
                .as_array()
                .is_some_and(|value| !value.is_empty())
        );
        let rewards = record["rewards"].as_array().unwrap();
        assert_eq!(rewards.len(), 1);
        assert_eq!(rewards[0]["coins"].as_u64(), Some(u64::from(REWARD_COINS)));
        assert_eq!(
            rewards[0]["job_xp"]["32"].as_u64(),
            Some(u64::from(REWARD_XP))
        );
        assert_eq!(rewards[0]["items"], serde_json::json!([]));
        assert!(
            rewards[0]["source"]
                .as_array()
                .is_some_and(|value| !value.is_empty())
        );
        for source in record["source"].as_array().unwrap() {
            assert!(
                source["evidence"]
                    .as_str()
                    .is_some_and(|text| !text.trim().is_empty())
            );
            if source["kind"] == "reconstructed" {
                assert!(
                    source["evidence"].as_str().unwrap().starts_with("(R)"),
                    "reconstructed provenance starts with (R)"
                );
            }
        }
    }

    let w1 = records
        .iter()
        .find(|row| row["id"].as_u64() == Some(u64::from(W1)))
        .unwrap();
    assert_eq!(w1["title_string_id"].as_u64(), Some(u64::from(W1_TITLE)));
    assert_eq!(w1["giver_npc_id"].as_u64(), Some(DEWEY));
    assert_eq!(w1["prerequisite_quest_ids"], serde_json::json!([]));
    assert_eq!(w1["objectives"][0]["npc_id"].as_u64(), Some(DEWEY));
    assert!(w1.get("description_string_id").is_none());
    assert!(w1.get("intro_text_string_id").is_none());
    assert!(w1.get("end_text_string_id").is_none());

    let w4 = records
        .iter()
        .find(|row| row["id"].as_u64() == Some(u64::from(W4)))
        .unwrap();
    assert_eq!(w4["title_string_id"].as_u64(), Some(u64::from(W4_TITLE)));
    assert_eq!(
        w4["description_string_id"].as_u64(),
        Some(u64::from(W4_DESCRIPTION))
    );
    assert_eq!(
        w4["intro_text_string_id"].as_u64(),
        Some(u64::from(W4_INTRO))
    );
    assert_eq!(w4["giver_npc_id"].as_u64(), Some(MORNINGLORY));
    assert_eq!(
        w4["prerequisite_quest_ids"],
        serde_json::json!([W1]),
        "W4 is the explicit playtest entry after W1 (W2/W3 unbuilt)"
    );
    assert_eq!(w4["objectives"][0]["npc_id"].as_u64(), Some(STEELE));
    assert!(w4.get("end_text_string_id").is_none());

    let w5 = records
        .iter()
        .find(|row| row["id"].as_u64() == Some(u64::from(W5)))
        .unwrap();
    assert_eq!(w5["enabled"].as_bool(), Some(true));
    assert_eq!(w5["job"].as_i64(), Some(i64::from(WARRIOR)));
    assert_eq!(w5["objectives"][0]["kind"], "job_level");
    assert_eq!(
        w5["objectives"][0]["profile_id"].as_i64(),
        Some(i64::from(WARRIOR))
    );
    assert_eq!(w5["objectives"][0]["level"].as_u64(), Some(5));
    assert_eq!(w5["rewards"][0]["coins"].as_u64(), Some(15));
    assert_eq!(w5["rewards"][0]["items"], serde_json::json!([]));
    assert_eq!(w5["rewards"][0]["job_xp"], serde_json::json!({}));

    // Every other Warrior record stays disabled and hollow, with the fields the
    // generated content already carried.
    for record in records {
        let id = record["id"].as_u64().unwrap();
        if id == u64::from(W1) || id == u64::from(W4) || id == u64::from(W5) {
            continue;
        }
        assert_eq!(record["enabled"].as_bool(), Some(false), "quest {id}");
        assert_eq!(record["objectives"], serde_json::json!([]), "quest {id}");
        assert_eq!(record["rewards"], serde_json::json!([]), "quest {id}");
    }
    for (id, title, giver, prerequisites) in [
        (W2, 77_223_u64, 9_000_000_302_u64, vec![u64::from(W1)]),
        (W3, 70_893, 9_000_000_302, vec![u64::from(W2)]),
        (W5, 387_433, 9_000_000_208, vec![u64::from(W4)]),
    ] {
        let record = records
            .iter()
            .find(|record| record["id"].as_u64() == Some(u64::from(id)))
            .unwrap();
        assert_eq!(record["title_string_id"].as_u64(), Some(title));
        assert_eq!(record["giver_npc_id"].as_u64(), Some(giver));
        assert_eq!(
            record["prerequisite_quest_ids"]
                .as_array()
                .unwrap()
                .iter()
                .filter_map(Value::as_u64)
                .collect::<Vec<_>>(),
            prerequisites
        );
    }
}

#[test]
fn progress_survives_a_reload() {
    let fixture = Fixture::new();
    let character = avatar_at(DEWEY);
    let mut now = 6_000;
    let mut game = warrior_game(&fixture);

    // Accept W1, complete the battle cry, and disconnect while the end screen is held.
    now += 10;
    offer(&mut game, &character, DEWEY, W1, now);
    now += 10;
    accept(&mut game, &character, W1, now);
    now += 10;
    assert!(talk(&mut game, &character, DEWEY, now).changed);
    assert_eq!(pending_turnin(&game), Some(W1));
    assert_eq!(coins(&game), 0);
    assert_eq!(profile_xp(&game, WARRIOR), 0);

    let path = fixture.path().join(format!(
        "t257-save-{}.json",
        NEXT_SAVE.fetch_add(1, Ordering::Relaxed)
    ));
    let hold = game_store::Snapshot {
        character: character.clone(),
        gameplay: Some(game.state()),
        pets: None,
    };
    game_store::save(&path, &hold).unwrap();
    let loaded = game_store::load_or_create(&path, character.clone()).unwrap();
    assert_eq!(
        loaded.gameplay.as_ref(),
        Some(&game.state()),
        "the save envelope changed the held turn-in"
    );
    game = fixture.game(loaded.gameplay);
    assert_eq!(quest_status(&game, W1), QuestStatus::Active);
    assert_eq!(counts(&game, W1), vec![1]);
    assert_eq!(pending_turnin(&game), Some(W1));

    // The reconnect visibility batch re-sends the held 49/13 after Dewey's 35/2,
    // exactly as the client's ShowQuestEndScreen path expects; the client then
    // answers with 49/14.
    let packets = game
        .update_npc_visibility(&character)
        .expect("relog NPC visibility");
    let end = packets
        .iter()
        .find(|packet| has_family(packet, 49, 13, Some(W1)))
        .expect("the held W1 end screen is re-sent after relog");
    assert_reward_bundle(end, W1, character.guid);
    now += 10;
    finish_quest(
        &mut game,
        &character,
        W1,
        WARRIOR,
        REWARD_COINS,
        REWARD_XP,
        now,
    );
    assert_eq!(coins(&game), REWARD_COINS);
    assert_eq!(profile_xp(&game, WARRIOR), REWARD_XP);

    // Reconnect with the completed history: W1 stays completed, the XP survives.
    let done = game_store::Snapshot {
        character: character.clone(),
        gameplay: Some(game.state()),
        pets: None,
    };
    game_store::save(&path, &done).unwrap();
    let loaded = game_store::load_or_create(&path, character.clone()).unwrap();
    game = fixture.game(loaded.gameplay);
    assert_eq!(quest_status(&game, W1), QuestStatus::Completed);
    assert_eq!(pending_turnin(&game), None);
    assert_eq!(coins(&game), REWARD_COINS);
    assert_eq!(profile_xp(&game, WARRIOR), REWARD_XP);

    // W4 is still offered after the reload.
    let morninglory = avatar_at(MORNINGLORY);
    now += 10;
    offer(&mut game, &morninglory, MORNINGLORY, W4, now);
    assert_eq!(pending_offer(&game), Some(W4));
    assert!(
        game.state()
            .npc_quests
            .as_ref()
            .unwrap()
            .quests
            .get(&W4)
            .is_some_and(|progress| progress.status == QuestStatus::Offered)
    );
}

#[test]
fn the_new_spawns_are_visible_to_a_second_session() {
    let fixture = Fixture::new();
    let first_character = avatar_for(CHARACTER, MORNINGLORY);
    let second_character = avatar_for(SECOND_CHARACTER, MORNINGLORY);
    let mut first = warrior_game(&fixture);
    let mut second = warrior_game_for(&fixture, SECOND_CHARACTER);

    for npc_id in [MORNINGLORY, STEELE] {
        let at = CharacterState {
            position: content_position("snowhill.json", npc_id),
            ..first_character.clone()
        };
        for (label, game) in [("first", &mut first), ("second", &mut second)] {
            let packets = game
                .update_npc_visibility(&at)
                .unwrap_or_else(|error| panic!("{label} NPC visibility: {error}"));
            let spawn = packets
                .iter()
                .filter_map(|packet| parse_add_npc(packet).ok())
                .find(|npc| npc.actor_id == npc_id)
                .unwrap_or_else(|| panic!("{label} session must draw NPC {npc_id}"));
            let row = one_spawn_row(npc_id);
            assert_eq!(
                spawn.model_id,
                row["model_id"].as_i64().unwrap() as i32,
                "{label}: NPC {npc_id} model"
            );
            assert_eq!(
                spawn.name_string_id,
                row["name_string_id"].as_i64().unwrap() as i32,
                "{label}: NPC {npc_id} name id"
            );
        }
    }

    // The second session's view is its own: it still draws the same static actors
    // and its character id is the one bound to its quest ledger.
    assert_eq!(
        second.state().npc_quests.as_ref().unwrap().character_id,
        second_character.guid
    );
}

#[test]
fn a_refused_turn_in_for_one_player_does_not_stop_the_other() {
    let fixture = Fixture::new();
    let first_character = avatar_at(DEWEY);
    let second_character = avatar_for(SECOND_CHARACTER, DEWEY);
    let mut first = warrior_game(&fixture);
    let mut second = warrior_game_for(&fixture, SECOND_CHARACTER);
    let mut now = 7_000;

    // The first player accepts W1 but is not ready. A forged 49/14 is a per-player
    // refusal: it changes nothing and does not escape as an error.
    now += 10;
    offer(&mut first, &first_character, DEWEY, W1, now);
    now += 10;
    accept(&mut first, &first_character, W1, now);
    let before = saved(&first);
    let refused = send(&mut first, &first_character, &quest_end_reply(W1), now + 10);
    assert!(!refused.changed, "an unfinished W1 must not turn in");
    assert_eq!(
        saved(&first),
        before,
        "the refused turn-in changed the save"
    );

    // The other player completes the same chapter and is paid exactly once.
    complete_w1(&mut second, &second_character, &mut now);
    assert_eq!(coins(&second), REWARD_COINS);
    assert_eq!(profile_xp(&second, WARRIOR), REWARD_XP);

    // The refusal did not damage the first player's progress: their saved W1 is
    // still active and can be finished normally.
    assert_eq!(quest_status(&first, W1), QuestStatus::Active);
    finish_w1(&mut first, &first_character, &mut now);
    assert_eq!(coins(&first), REWARD_COINS);
    assert_eq!(profile_xp(&first, WARRIOR), REWARD_XP);

    // Neither ledger saw the other player's payout.
    assert_eq!(coins(&second), REWARD_COINS);
    assert_eq!(profile_xp(&second, WARRIOR), REWARD_XP);
    assert_eq!(quest_status(&second, W1), QuestStatus::Completed);
    assert_eq!(quest_status(&first, W1), QuestStatus::Completed);
}

#[test]
fn a_malformed_warrior_record_fails_at_load() {
    let fixture = Fixture::new();

    // Control: a well-formed synthetic record binds through the production loader.
    let control = record(W1, vec![talk_goal(1, DEWEY)], vec![reward(REWARD_COINS)]);
    let control_dir = synthetic_content(&control);
    let mut control_game = fixture.game(None);
    control_game
        .bind_npc_quest_catalogue(&control_dir, CHARACTER)
        .expect("the control record loads");
    let _ = fs::remove_dir_all(&control_dir);

    // Zero and negative ids are load errors, not runtime surprises.
    assert_load_refused(
        &fixture,
        record(0, vec![talk_goal(1, DEWEY)], vec![reward(REWARD_COINS)]),
        "zero quest id",
    );
    assert_load_refused(
        &fixture,
        record(-1, vec![talk_goal(1, DEWEY)], vec![reward(REWARD_COINS)]),
        "negative quest id",
    );
    // An objective without provenance is refused for a runtime goal.
    let mut no_source = visit_goal(1, [0.0, 0.0, 0.0], 5.0);
    no_source["source"] = serde_json::json!([]);
    assert_load_refused(
        &fixture,
        record(W1, vec![no_source], vec![reward(REWARD_COINS)]),
        "empty objective source",
    );
    // More objectives than the schema permits.
    let over_cap: Vec<Value> = (1..=65).map(|id| talk_goal(id, DEWEY)).collect();
    assert_load_refused(
        &fixture,
        record(W1, over_cap, vec![reward(REWARD_COINS)]),
        "over-cap objective count",
    );
    // A reward outside the 49/13 wire bounds.
    let mut too_much = reward(REWARD_COINS);
    too_much["coins"] = serde_json::json!(u32::MAX);
    assert_load_refused(
        &fixture,
        record(W1, vec![talk_goal(1, DEWEY)], vec![too_much]),
        "out-of-range reward",
    );
    // A job-XP row that names no known job profile.
    let mut unknown_profile = reward(REWARD_COINS);
    unknown_profile["job_xp"] = serde_json::json!({"999": REWARD_XP});
    assert_load_refused(
        &fixture,
        record(W1, vec![talk_goal(1, DEWEY)], vec![unknown_profile]),
        "unknown job profile",
    );
}

fn talk_goal(id: u32, npc_id: u64) -> Value {
    serde_json::json!({
        "kind": "talk_to_npc",
        "id": id,
        "npc_id": npc_id,
        "source": [{"kind": "reconstructed", "evidence": "(R) T257 synthetic input"}]
    })
}

fn visit_goal(id: u32, position: [f32; 3], radius: f32) -> Value {
    serde_json::json!({
        "kind": "visit_position",
        "id": id,
        "zone_id": 0,
        "position": position,
        "radius": radius,
        "source": [{"kind": "reconstructed", "evidence": "(R) T257 synthetic input"}]
    })
}

fn reward(coins: u32) -> Value {
    serde_json::json!({
        "coins": coins,
        "items": [],
        "job_xp": {"32": REWARD_XP},
        "source": [{"kind": "reconstructed", "evidence": "(R) T257 synthetic input"}]
    })
}

/// One synthetic quest record for the loader-bound test.
fn record(id: impl Into<i64>, objectives: Vec<Value>, rewards: Vec<Value>) -> Value {
    let id = id.into();
    serde_json::json!({
        "id": id,
        "job": WARRIOR,
        "title_string_id": W1_TITLE,
        "giver_npc_id": DEWEY,
        "prerequisite_quest_ids": [],
        "objectives": objectives,
        "rewards": rewards,
        "source": [{"kind": "reconstructed", "evidence": "(R) T257 synthetic record"}]
    })
}

fn synthetic_content(quest: &Value) -> PathBuf {
    let id = quest["id"].as_i64().unwrap();
    let directory = std::env::temp_dir().join(format!(
        "osfr-warrior-malformed-{}-{}-{id}",
        std::process::id(),
        NEXT_SAVE.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = fs::remove_dir_all(&directory);
    fs::create_dir_all(directory.join("quests")).unwrap();
    fs::create_dir_all(directory.join("npcs")).unwrap();
    fs::write(
        directory.join("quests/warrior.json"),
        serde_json::to_vec(&serde_json::json!({
            "schema_version": 1,
            "records": [quest]
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        directory.join("npcs/snowhill.json"),
        serde_json::to_vec(&serde_json::json!({"schema_version": 1, "records": []})).unwrap(),
    )
    .unwrap();
    fs::write(
        directory.join("jobs.json"),
        serde_json::to_vec(&serde_json::json!({
            "schema_version": 1,
            "records": [{
                "id": "warrior",
                "name": "Warrior",
                "profile_id": WARRIOR,
                "category": "combat",
                "max_level": 20,
                "members_only": true,
                "trainer_npc_ids": [],
                "quest_ids": [id],
                "abilities": [],
                "source": [{"kind": "reconstructed", "evidence": "(R) T257 synthetic job"}]
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
    let before = saved(&game);
    let result = game.bind_npc_quest_catalogue(&directory, CHARACTER);
    let _ = fs::remove_dir_all(&directory);
    assert!(result.is_err(), "{label} must be refused at content load");
    assert_eq!(saved(&game), before, "{label} changed the live catalogue");
}
