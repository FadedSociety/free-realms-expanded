//! T360: Anne at Farnum Farmstead gives "Investigate the Wolves" (quest
//! 3000061), one `complete_dungeon` step on the T319 Sheep Watch battle 12350,
//! turned back in to Anne for 20 coins.
//!
//! This is content plus a test: the quest record lives in
//! `data/content/quests/adventurer.json`, the registry ids in `jobs.json` (the
//! split-loader union) and `npcs/farnum_farmstead.json` (Anne). No engine,
//! wire field, selector or save field is added. The tests drive the production
//! in-world router: 26/8 -> 26/9 -> 26/10 -> 26/6 -> 49/1 -> 49/2 and the field
//! battle's marker 26/8 -> 41/114 start screen -> 41/108 Go -> arena zone 9600
//! -> `realm win` -> 26/42 Close -> Talk -> 49/13 -> 49/14. The shared fixture,
//! NPC chain, save/reload and refusal helpers come from
//! `tests/support/npc_quest_chain.rs`; the canonical 41/108 Go builder and lab
//! path come from `tests/common/field_battle_fixture.rs`; only the win and
//! cancel packets are local to this file.
//!
//! Client layouts used here (IDA). 26/8, 26/10, 26/6, 49/1, 49/2, 49/13,
//! 49/14, 41/108 and 35/2 were re-decompiled for T360; 49/3, 49/8, 49/9,
//! 49/10, 41/114, 41/107, 41/109 and 26/42 are the addresses pinned by the
//! existing T282/T319 suites and are cited here without a new claim:
//! * 26/8 interact constructor `0x008b6bc0`: two u32 target words and a u8 flag.
//! * 26/10 select constructor `0x00a36f80`: two u32 target words and an i32 choice.
//! * 26/6 dialog response constructor `0x009f5890`: one i32 row (the quest row is 301).
//! * 49/1 offer reader `0x00c7bb60` -> `0x00c7b7a0`: quest at 6, intro at 10,
//!   title at 14 and 18, description at 22, members-only at 30, then the reward
//!   bundle at 31 (`0x008e7930`: success at 31, coins at 32..36, XP stars at 36..40).
//! * 49/2 reply constructor `0x00a86400`: quest u32 and an accept byte.
//! * 49/3 QuestAdd reader `0x00c7bd10`; 49/8 ObjectiveActivated `0x00c7aeb0`;
//!   49/9 ObjectiveUpdate `0x00c7af60` (quest 6..10, objective 10..14, current
//!   14..18); 49/10 ObjectiveComplete `0x00c7b070`.
//! * 49/13 end-screen reader `0x00c7b990` / writer `0x00c7bbd0`: actor 6..14,
//!   quest 14..18, end text 18..22, animation dword 22..26, reward bundle at 26
//!   (coins 27..31, displayed XP stars 31..35), trailing float.
//! * 49/14 reply constructor `0x00a864c0`: one quest u32.
//! * 41/108 Go constructor `0x008b6e70` (sender `0x00938f00`): the encounter and
//!   instance pair copied from 41/114, then eight zero bytes.
//! * 41/114 start-screen reader chain `0x00aa3430` -> `0x00aa32d0` ->
//!   `0x00a29120` -> `0x009bdd70`; the ready header is 41/107.
//! * 41/109 leave constructor `0x008a31a0`; 26/42 results Close `0x008a2eb0`,
//!   sender `0x0092f300`.
//! * 35/2 AddNpc reader `0x00921a10` (Anne is already a drawable legacy_data
//!   quest giver; this slice adds no spawn).
//!
//! Recovered Global.Text ids (`lab/reports/locale-id-to-text.json`): title
//! 101015 = "Investigate the Wolves", intro 101014, description 101016, end
//! 101007 = "Thank you! Hopefully, the sheep can get some rest now." and the
//! battle goal 388319 = "Defeat the Ravenous Wolves at Sheep Watch".
//!
//! Giver id note: the brief's `9000000266` is a job reference on another Wilds
//! NPC row (Caleb Sleepyswing). Anne's own content key is `9500000266`, and the
//! production split loader binds a quest giver to the NPC record's `npc_id`
//! (`quest_engine::from_split_content`), so this record names `9500000266`.
//! That is the id the client's 26/8 click and the 35/2 AddNpc stream use.
//!
//! Job gate note: the record omits the `job` key instead of writing an explicit
//! `null`. The split loader reads both as "no job gate", but
//! `tests/quest_job_xp.rs::configured_job_xp_and_job_gates_match_the_task_table`
//! (protected) requires every unlisted record to carry no job key at all.

#[path = "common/field_battle_fixture.rs"]
mod battle;
#[path = "support/npc_quest_chain.rs"]
mod chain;
#[allow(dead_code)] // The shared vendor kit carries helpers this suite does not send.
#[path = "common/gateway_vendor.rs"]
mod gateway_vendor;

use battle::{go, lab};
use chain::{
    CHARACTER, FILLER_ITEM, Fixture, accept, assert_no_offer, assert_packet_changes_nothing,
    character_at_for, coins, content_dir, content_position, counts, end_screen_reward,
    finish_quest, has_family, interact, menu_option, npc_interact, npc_select, objective_complete,
    objective_update, offer, pending_turnin, quest_end_reply, quest_reply, quest_status, read_json,
    release_end_screen, saved, select_job, send, talk,
};
use gateway_vendor::{merchant_status, vendor_buy_packet};
use osfr_tools::{
    account_store::AccountRole,
    character_store::CharacterState,
    chat,
    encounter::EncounterPhase,
    gameplay::{Gameplay, Outcome},
    profiles::ProfileState,
    quest_engine::{CharacterEligibility, QuestEngine, QuestStatus},
};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

const QUEST: u32 = 3_000_061;
const ANNE: u64 = 9_500_000_266;
const TITLE: u32 = 101_015;
const INTRO: u32 = 101_014;
const DESCRIPTION: u32 = 101_016;
const END_TEXT: u32 = 101_007;
const GOAL_TEXT: u32 = 388_319;
const COINS: u32 = 20;
const ADVENTURER: i32 = 1;
const MINER: i32 = 14;
const BRAWLER: i32 = 43;
const BATTLE: u32 = 12_350;
const MARKER_GUID: u64 = 100_001_235_000;
const ARENA_ZONE: u32 = 9_600;
const ARENA_ARRIVAL: [f32; 3] = [136.0, -0.5, 155.0];
const VELA: u64 = 9_500_000_281;
/// `npc_interaction::SHOP_STRING_ID`: the 26/9 Shop option label.
const SHOP_LABEL: u32 = 432_156;

// ---------------------------------------------------------------------------
// Fixture helpers.
// ---------------------------------------------------------------------------

fn character(guid: u64, position: [f32; 3]) -> CharacterState {
    character_at_for(guid, 0, position)
}

fn anne_position() -> [f32; 3] {
    content_position("farnum_farmstead.json", ANNE)
}

fn battle_row() -> Value {
    read_json(&content_dir().join("field_battles.json"))["battles"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["id"].as_u64() == Some(u64::from(BATTLE)))
        .cloned()
        .expect("field battle 12350 is authored")
}

fn position(value: &Value) -> [f32; 3] {
    let axis = value.as_array().expect("a finite xyz position");
    assert_eq!(axis.len(), 3);
    [
        axis[0].as_f64().unwrap() as f32,
        axis[1].as_f64().unwrap() as f32,
        axis[2].as_f64().unwrap() as f32,
    ]
}

fn marker(row: &Value) -> [f32; 3] {
    position(&row["spawner"]["position"])
}

fn marker_guid(row: &Value) -> u64 {
    row["spawner"]["guid"].as_u64().expect("the marker GUID")
}

fn battle_coins(row: &Value) -> u32 {
    row["reward_coins"].as_u64().expect("the battle payout") as u32
}

/// The cheapest item the vendor stocks and the catalogue price the 66/43 route
/// charges: `vendors.json` authorizes the stock, `stores.json` prices it.
fn vendor_item(npc_id: u64) -> (i32, u32) {
    let vendors = read_json(&content_dir().join("vendors.json"));
    let stock = vendors["vendors"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["npc_id"].as_u64() == Some(npc_id))
        .expect("the vendor's stock row")["item_ids"]
        .as_array()
        .unwrap();
    let stores = read_json(&content_dir().join("stores.json"));
    let prices: BTreeMap<i64, u64> = stores["items"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|row| {
            Some((
                row["item_id"].as_i64()?,
                row["purchase_price_coins"].as_u64()?,
            ))
        })
        .collect();
    stock
        .iter()
        .filter_map(|id| id.as_i64())
        .filter_map(|id| prices.get(&id).map(|price| (id, *price)))
        .min_by_key(|(_, price)| *price)
        .map(|(id, price)| (id as i32, price as u32))
        .expect("the vendor stocks a priced item")
}

fn profiles(active: i32, unlocked: &[i32]) -> ProfileState {
    ProfileState {
        version: 1,
        active,
        unlocked: unlocked.iter().copied().collect::<BTreeSet<_>>(),
        experience: BTreeMap::new(),
    }
}

/// A session over the opening fixture with the split catalogue bound, the
/// developer seam on (the rig is an admin gateway, the T319/T330 pattern) and
/// the production world-entry pass run at `at`.
fn player_as(
    fixture: &Fixture,
    character_id: u64,
    at: [f32; 3],
    profiles: ProfileState,
) -> (Gameplay, CharacterState) {
    let mut state = Gameplay::load(fixture.path(), None)
        .expect("the opening fixture loads")
        .state();
    state.profiles = Some(profiles);
    let mut game = fixture.game_as(character_id, Some(state));
    game.set_dev_tools(true);
    game.set_role(AccountRole::Admin);
    let character = character(character_id, at);
    game.initialize_world(&character, 0)
        .expect("the opening world initializes");
    (game, character)
}

fn player(fixture: &Fixture, profiles: ProfileState) -> (Gameplay, CharacterState) {
    player_as(fixture, CHARACTER, anne_position(), profiles)
}

fn reconnect(fixture: &Fixture, game: &Gameplay) -> Gameplay {
    let mut restored = fixture.reload_serialized(game);
    // The rig keeps its developer seam across a reconnect, exactly as the
    // gateway keeps --dev-tools; the quest and reward code never reads it.
    restored.set_dev_tools(true);
    restored.set_role(AccountRole::Admin);
    restored
}

fn u32_at(packet: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(packet[offset..offset + 4].try_into().unwrap())
}

fn chat_text(outcome: &Outcome) -> String {
    outcome
        .replies
        .iter()
        .filter(|reply| reply.starts_with(&[15, 0, 3, 0]))
        .map(|reply| {
            let length = u32::from_le_bytes(reply[4..8].try_into().unwrap()) as usize;
            String::from_utf8_lossy(&reply[8..8 + length]).into_owned()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn item_count(game: &Gameplay, definition_id: i32) -> u32 {
    game.state()
        .inventory
        .items
        .iter()
        .filter(|item| item.definition_id == definition_id && item.rental.is_none())
        .map(|item| item.count)
        .sum()
}

// ---------------------------------------------------------------------------
// Quest offer/accept and the field-battle start/win packets.
// ---------------------------------------------------------------------------

/// 26/8 -> 26/10 Talk offers exactly quest 3000061; 49/2 accepts it. The 49/1
/// card is checked against the client reader offsets.
fn accept_at_anne(game: &mut Gameplay, at: &CharacterState, now: &mut u64) {
    let offered = offer(game, at, ANNE, QUEST, *now);
    let packet = offered
        .replies
        .iter()
        .find(|packet| has_family(packet, 49, 1, Some(QUEST)))
        .expect("the 49/1 offer");
    assert_eq!(u32_at(packet, 10), INTRO, "49/1 intro id");
    assert_eq!(u32_at(packet, 14), TITLE, "49/1 title id");
    assert_eq!(u32_at(packet, 18), TITLE, "49/1 title id (card)");
    assert_eq!(u32_at(packet, 22), DESCRIPTION, "49/1 description id");
    assert_eq!(packet[31], 1, "49/1 reward bundle success flag");
    assert_eq!(u32_at(packet, 32), COINS, "49/1 displayed coins");
    assert_eq!(u32_at(packet, 36), 0, "49/1 displayed XP stars");
    *now += 10;
    accept(game, at, QUEST, *now);
    *now += 10;
    assert_eq!(quest_status(game, QUEST), QuestStatus::Active);
    assert_eq!(counts(game, QUEST), [0]);
}

fn arm_brawler(game: &mut Gameplay, at: &CharacterState, now: &mut u64) {
    assert!(
        send(game, at, &select_job(BRAWLER), *now).changed,
        "the Brawler outfit is selectable"
    );
    *now += 10;
}

/// 26/8 on the Sheep Watch marker: the 41/114 start screen and its 41/107
/// ready header. Returns the 41/114 body for the Go packet.
fn open_marker(game: &mut Gameplay, at: &CharacterState, now: u64) -> Vec<u8> {
    let screen = interact(game, at, MARKER_GUID, now);
    let details = screen
        .replies
        .iter()
        .find(|packet| packet.starts_with(&[41, 0, 114, 0]))
        .cloned()
        .expect("the marker opens the 41/114 start screen");
    assert!(
        screen
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[41, 0, 107, 0])),
        "the start screen is ready (41/107)"
    );
    assert_eq!(u32_at(&details, 4), BATTLE, "41/114 names battle 12350");
    assert!(u32_at(&details, 8) > 0, "the start screen has an instance");
    details
}

/// Go into the arena zone 9600 and run the production zone load that spawns the
/// arena roster. Returns the launch instance id for the leave packet.
fn enter_arena(game: &mut Gameplay, at: &mut CharacterState, details: &[u8], now: &mut u64) -> i32 {
    let packet = go(details);
    // 41/108 is the 4-byte header, the encounter/instance pair copied from
    // 41/114, and eight zero session bytes. Client constructor `0x008b6e70`,
    // sender `0x00938f00`; native frames pin the zero tail
    // (`tests/common/field_battle_fixture.rs`).
    assert_eq!(packet.len(), 20, "41/108 is a 20-byte Go packet");
    assert_eq!(
        &packet[4..12],
        &details[4..12],
        "41/108 copies the 41/114 encounter/instance pair"
    );
    assert_eq!(
        &packet[12..],
        &[0_u8; 8][..],
        "41/108 trailing session is eight zero bytes"
    );
    let zoned = send(game, at, &packet, *now);
    let zone = zoned.zoning.expect("Go zones into the arena");
    assert_eq!(zone.zone_id, ARENA_ZONE);
    assert_eq!(zone.name, "sg_random_encounter_clearing");
    let instance = i32::from_le_bytes(details[8..12].try_into().unwrap());
    *now += 10;
    at.zone_id = ARENA_ZONE;
    at.position = ARENA_ARRIVAL;
    game.initialize_world(at, *now)
        .expect("the arena zone loads");
    *now += 10;
    instance
}

fn win(game: &mut Gameplay, at: &CharacterState, now: u64) -> Outcome {
    let won = send(game, at, &chat::local_request("realm win"), now);
    assert!(
        won.changed,
        "the admin-gateway win commits through the real encounter path"
    );
    won
}

/// 26/42 results Close: back to the open world at the marker's return pose.
fn close_results(game: &mut Gameplay, at: &mut CharacterState, now: &mut u64) {
    let closed = send(game, at, &[26, 0, 42, 0], *now);
    let zone = closed
        .zoning
        .expect("the results Close returns to the open world");
    assert_eq!(zone.zone_id, 0);
    *now += 10;
    at.zone_id = zone.zone_id;
    at.position = zone.position;
    game.initialize_world(at, *now)
        .expect("the open world reloads");
    *now += 10;
}

/// Win the battle through the real marker/Go/win/Close path for an already
/// accepted quest, leaving it Active with objective 1 complete.
fn win_sheep_watch(game: &mut Gameplay, at: &mut CharacterState, row: &Value, now: &mut u64) {
    arm_brawler(game, at, now);
    let before = coins(game);
    at.position = marker(row);
    let details = open_marker(game, at, *now);
    *now += 10;
    enter_arena(game, at, &details, now);
    win(game, at, *now);
    *now += 10;
    assert_eq!(counts(game, QUEST), [1]);
    assert_eq!(
        coins(game),
        before + battle_coins(row),
        "the battle pays its own row; the quest pays nothing at the win"
    );
    close_results(game, at, now);
    assert_eq!(counts(game, QUEST), [1], "the Close does not credit again");
    assert_eq!(quest_status(game, QUEST), QuestStatus::Active);
}

/// Talk to Anne and release the held 49/13. The win already completed the only
/// objective, so an ordinary Talk opens the end screen (no Talk objective).
fn open_end_screen(game: &mut Gameplay, at: &CharacterState, now: &mut u64) -> Vec<u8> {
    let dialogue = talk(game, at, ANNE, *now);
    assert!(
        dialogue
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[26, 0, 3, 0])),
        "Anne opens the 26/3 dialogue"
    );
    *now += 10;
    let end = release_end_screen(game, at, QUEST, ANNE, END_TEXT, *now);
    *now += 10;
    assert_eq!(
        end.len(),
        99,
        "the coins-only end-screen bundle is 99 bytes"
    );
    assert_eq!(u32_at(&end, 14), QUEST, "49/13 quest id");
    assert_eq!(u32_at(&end, 18), END_TEXT, "49/13 end text");
    assert_eq!(
        end_screen_reward(&end),
        (COINS, 0),
        "the shared 49/13 reader: 20 coins and no job-XP stars"
    );
    end
}

/// Open a vendor's real 165/10 coin window through 26/8 -> 26/10 and return the list.
fn open_shop(game: &mut Gameplay, at: &CharacterState, vendor: u64, now: u64) -> Vec<u8> {
    let menu = interact(game, at, vendor, now);
    let choice = menu_option(&menu, SHOP_LABEL);
    let opened = send(game, at, &npc_select(vendor, choice), now + 1);
    opened
        .replies
        .iter()
        .find(|packet| packet.starts_with(&[165, 0, 10, 0]))
        .cloned()
        .expect("the Shop choice sends the 165/10 coin merchant list")
}

// ---------------------------------------------------------------------------
// Tests.
// ---------------------------------------------------------------------------

/// Anne offers "Investigate the Wolves" with the four recovered text ids and
/// accepts it: Active, one objective at 0, no prerequisite and no job gate.
#[test]
fn anne_offers_investigate_the_wolves_and_accepts_it() {
    let fixture = Fixture::new();
    let (mut game, at) = player(&fixture, profiles(ADVENTURER, &[ADVENTURER, BRAWLER]));
    let mut now = 1_000;

    // The production 49/1 card assertions live in `accept_at_anne`.
    accept_at_anne(&mut game, &at, &mut now);
    assert_eq!(quest_status(&game, QUEST), QuestStatus::Active);
    assert_eq!(counts(&game, QUEST), [0], "objective 1 starts at 0/1");

    // A second Talk offers nothing while the quest is Active, and it does not
    // consume the open NPC menu.
    assert_no_offer(&mut game, &at, ANNE, &mut now, "an active quest");
    assert_eq!(counts(&game, QUEST), [0]);
}

/// The real marker -> Go -> win path credits objective 1 once; a duplicate win
/// changes nothing; the results Close returns to the world; the return Talk
/// carries end text 101007 and the 20-coin reward, paid once by 49/14.
#[test]
fn winning_sheep_watch_completes_the_objective_and_returns_to_anne() {
    let fixture = Fixture::new();
    let row = battle_row();
    assert_eq!(marker_guid(&row), MARKER_GUID);
    let (mut game, mut at) = player(&fixture, profiles(ADVENTURER, &[ADVENTURER, BRAWLER]));
    let mut now = 1_000;

    accept_at_anne(&mut game, &at, &mut now);
    arm_brawler(&mut game, &at, &mut now);
    at.position = marker(&row);
    let details = open_marker(&mut game, &at, now);
    now += 10;
    enter_arena(&mut game, &mut at, &details, &mut now);
    assert_eq!(
        counts(&game, QUEST),
        [0],
        "entering the arena credits nothing"
    );
    let before_win = coins(&game);
    let won = win(&mut game, &at, now);
    now += 10;
    assert!(
        objective_update(&won, QUEST, 1, 1),
        "49/9 reports 1/1 through the client reader offsets"
    );
    assert!(
        objective_complete(&won, QUEST, 1),
        "49/10 completes objective 1"
    );
    assert_eq!(counts(&game, QUEST), [1]);
    assert_eq!(quest_status(&game, QUEST), QuestStatus::Active);
    assert_eq!(
        coins(&game),
        before_win + battle_coins(&row),
        "the battle row pays its own coins; the quest reward is still pending"
    );

    // A duplicate win in the same run pays and credits nothing.
    let won_state = saved(&game);
    let duplicate = send(&mut game, &at, &chat::local_request("realm win"), now);
    assert!(!duplicate.changed, "a won run cannot win twice");
    assert_eq!(saved(&game), won_state, "a duplicate win changed the save");
    now += 10;

    close_results(&mut game, &mut at, &mut now);
    assert_eq!(counts(&game, QUEST), [1], "the Close does not credit again");

    at.position = anne_position();
    let end = open_end_screen(&mut game, &at, &mut now);
    assert_eq!(u32_at(&end, 18), END_TEXT);
    assert_eq!(pending_turnin(&game), Some(QUEST));
    let coins_before = coins(&game);
    finish_quest(&mut game, &at, QUEST, ADVENTURER, COINS, 0, now);
    now += 10;
    assert_eq!(coins(&game), coins_before + COINS, "49/14 pays 20 coins");
    assert_eq!(quest_status(&game, QUEST), QuestStatus::Completed);

    // Replayed turn-in and accept pay nothing and change no byte.
    let finished = saved(&game);
    assert_packet_changes_nothing(
        &mut game,
        &at,
        &quest_end_reply(QUEST),
        now,
        "a replayed 49/14",
    );
    assert_packet_changes_nothing(
        &mut game,
        &at,
        &quest_reply(QUEST, true),
        now,
        "a replayed 49/2",
    );
    assert_eq!(saved(&game), finished);
}

/// Cancelling the run through the existing 41/109 leave path returns the player
/// to the marker, clears the encounter and leaves the quest Active at 0/1; the
/// return Talk neither offers nor turns in anything.
#[test]
fn a_cancel_credits_nothing() {
    let fixture = Fixture::new();
    let row = battle_row();
    let (mut game, mut at) = player(&fixture, profiles(ADVENTURER, &[ADVENTURER, BRAWLER]));
    let mut now = 1_000;

    accept_at_anne(&mut game, &at, &mut now);
    arm_brawler(&mut game, &at, &mut now);
    at.position = marker(&row);
    let details = open_marker(&mut game, &at, now);
    now += 10;
    let instance = enter_arena(&mut game, &mut at, &details, &mut now);
    assert_eq!(counts(&game, QUEST), [0]);

    let mut cancel = vec![41, 0, 109, 0];
    cancel.extend(0_i32.to_le_bytes());
    cancel.extend(instance.to_le_bytes());
    let left = send(&mut game, &at, &cancel, now);
    now += 10;
    let zone = left.zoning.expect("41/109 returns to the open world");
    assert_eq!(zone.zone_id, 0);
    assert_eq!(zone.position, marker(&row), "the marker's return pose");
    assert_eq!(counts(&game, QUEST), [0], "a cancel credits nothing");
    assert_eq!(quest_status(&game, QUEST), QuestStatus::Active);
    assert!(
        game.state()
            .encounter
            .is_none_or(|state| state.active.is_none()),
        "the cancelled run is cleared"
    );
    at.zone_id = zone.zone_id;
    at.position = zone.position;
    game.initialize_world(&at, now)
        .expect("the open world reloads");
    now += 10;

    // The return Talk offers nothing, turns in nothing and keeps the menu open.
    at.position = anne_position();
    assert_no_offer(&mut game, &at, ANNE, &mut now, "a cancelled run");
    assert_packet_changes_nothing(
        &mut game,
        &at,
        &quest_end_reply(QUEST),
        now,
        "a 49/14 after a cancel",
    );
    assert_eq!(counts(&game, QUEST), [0]);
    assert_eq!(quest_status(&game, QUEST), QuestStatus::Active);
}

/// A lost run (the arena roster knocks the player out) leaves the quest Active
/// at 0/1 and pays nothing. The client's free 41/122 revive (constructor
/// `0x0093bd80`; header words plus the trailing bool) then restores full health
/// (the 38/1 reply's current equals its maximum and the arena-entry value),
/// clears the lost instance and returns the marker's return pose; the return
/// Talk neither offers nor turns in anything.
#[test]
fn a_loss_credits_nothing_and_the_free_revive_restores_the_player() {
    let fixture = Fixture::new();
    let row = battle_row();
    let (mut game, mut at) = player(&fixture, profiles(ADVENTURER, &[ADVENTURER, BRAWLER]));
    let mut now = 1_000;

    accept_at_anne(&mut game, &at, &mut now);
    arm_brawler(&mut game, &at, &mut now);
    at.position = marker(&row);
    let details = open_marker(&mut game, &at, now);
    now += 10;
    let instance = enter_arena(&mut game, &mut at, &details, &mut now);
    let coins_before = coins(&game);
    let health_before = game
        .state()
        .encounter
        .as_ref()
        .map(|state| state.player_health)
        .expect("the arena encounter state exists");
    assert!(health_before > 0, "the player enters the arena standing");

    // Stand on the first roster enemy's authored spawn and run the production
    // 20 Hz tick until its melee knocks the player out. The arena damage and the
    // loss are server-driven; no client packet reports either.
    at.position = position(&row["enemies"][0]["position"]);
    let mut knock_out = None;
    for _ in 0..60 {
        now += 1_000;
        let ticked = game.tick(&at, now).expect("the arena tick runs");
        let lost = game
            .state()
            .encounter
            .as_ref()
            .and_then(|state| state.active.as_ref())
            .is_some_and(|active| active.phase == EncounterPhase::Lost);
        if lost {
            knock_out = Some(ticked);
            break;
        }
    }
    let knock_out = knock_out.expect("the wolves knock the player out");
    assert_eq!(counts(&game, QUEST), [0], "a loss credits nothing");
    assert_eq!(quest_status(&game, QUEST), QuestStatus::Active);
    assert_eq!(coins(&game), coins_before, "a loss pays nothing");
    let over = knock_out
        .replies
        .iter()
        .find(|packet| packet.starts_with(&[39, 0, 18]))
        .expect("the loss sends 39/18 (reader 0x009b7ec0)");
    assert_eq!(over.len(), 28, "39/18 is the header, won, and three dwords");
    assert_eq!(over[15], 0, "39/18 reports a lost run (won = false)");
    assert!(
        game.state()
            .encounter
            .as_ref()
            .is_some_and(|state| state.player_health == 0),
        "the knockout holds the player at zero health"
    );

    // 41/122 free revive: header, encounter, instance, bool 0 (paid is 1).
    let mut revive = vec![41, 0, 122, 0];
    revive.extend(BATTLE.to_le_bytes());
    revive.extend(instance.to_le_bytes());
    revive.push(0);
    let restored = send(&mut game, &at, &revive, now);
    now += 10;
    // 38/1 Hitpoints: two u16 header words, current health, maximum health
    // (constructor `0x0089d2b0` -> `sub_89D180(1)`; the 38/x dispatch arm
    // `0x0093938f` reads the body through `0x008f8890` -> `0x008d8210`, which
    // refuses trailing bytes, and stores the first dword as the current health
    // at client+0x484). A lost instance exits through the shared 41/109 exit,
    // which restores full health.
    let healed = restored
        .replies
        .iter()
        .find(|packet| packet.starts_with(&[38, 0, 1, 0]))
        .expect("the free revive sends 38/1 player health");
    assert_eq!(healed.len(), 12, "38/1 is the header and two dwords");
    assert_eq!(
        u32_at(healed, 4),
        health_before,
        "the revive restores the arena-entry health"
    );
    assert_eq!(
        u32_at(healed, 8),
        health_before,
        "the revived health is the maximum"
    );
    assert_eq!(
        game.state()
            .encounter
            .as_ref()
            .map(|state| state.player_health),
        Some(health_before),
        "the saved encounter health is full again"
    );
    let zone = restored
        .zoning
        .expect("the free revive leaves the lost instance");
    assert_eq!(zone.zone_id, 0);
    assert_eq!(zone.position, marker(&row), "the marker's return pose");
    assert!(
        game.state()
            .encounter
            .as_ref()
            .is_some_and(|state| state.active.is_none()),
        "the lost run is cleared"
    );
    assert_eq!(counts(&game, QUEST), [0], "the revive credits nothing");
    assert_eq!(quest_status(&game, QUEST), QuestStatus::Active);
    at.zone_id = zone.zone_id;
    at.position = zone.position;
    game.initialize_world(&at, now)
        .expect("the open world reloads");
    now += 10;

    at.position = anne_position();
    assert_no_offer(&mut game, &at, ANNE, &mut now, "a lost run");
    assert_packet_changes_nothing(
        &mut game,
        &at,
        &quest_end_reply(QUEST),
        now,
        "a 49/14 after a loss",
    );
    assert_eq!(counts(&game, QUEST), [0]);
    assert_eq!(quest_status(&game, QUEST), QuestStatus::Active);
}

/// Premature, forged, out-of-range and replayed requests change nothing: a
/// 49/14 before the win, a forged quest id both before and while the real end
/// screen is pending, an accept with no open offer, a Talk from outside Anne's
/// range and a replayed results Close all leave the saved state byte-equal.
/// The canaries are the same packets succeeding on their qualifying path.
#[test]
fn premature_and_forged_turnins_change_nothing() {
    let fixture = Fixture::new();
    let row = battle_row();
    let (mut game, mut at) = player(&fixture, profiles(ADVENTURER, &[ADVENTURER, BRAWLER]));
    let mut now = 1_000;

    // No quest exists yet: none of these requests may create or pay one.
    for (packet, label) in [
        (quest_end_reply(QUEST), "a 49/14 before the quest"),
        (quest_end_reply(QUEST + 1), "a forged 49/14 id"),
        (quest_reply(QUEST, true), "a 49/2 with no open offer"),
    ] {
        assert_packet_changes_nothing(&mut game, &at, &packet, now, label);
        now += 10;
    }

    // An out-of-range Talk click is refused with no state change; the canary is
    // that the same click at Anne opens her menu.
    let far = character(CHARACTER, [10_000.0, 0.0, 10_000.0]);
    let before = saved(&game);
    assert!(
        game.handle_in_world(&npc_interact(ANNE), &far, now)
            .is_err(),
        "an out-of-range Anne click is refused"
    );
    assert_eq!(
        saved(&game),
        before,
        "an out-of-range click changed the save"
    );
    now += 10;
    let menu = interact(&mut game, &at, ANNE, now);
    assert!(
        menu.replies
            .iter()
            .any(|packet| packet.starts_with(&[26, 0, 9, 0])),
        "the in-range canary opens the 26/9 menu"
    );
    now += 10;

    accept_at_anne(&mut game, &at, &mut now);
    // The objective is still open: 49/14 is a clean refusal.
    assert_packet_changes_nothing(
        &mut game,
        &at,
        &quest_end_reply(QUEST),
        now,
        "a 49/14 before the win",
    );
    now += 10;

    win_sheep_watch(&mut game, &mut at, &row, &mut now);
    at.position = anne_position();
    open_end_screen(&mut game, &at, &mut now);
    assert_eq!(pending_turnin(&game), Some(QUEST));

    // A forged quest id while the real end screen is pending neither pays nor
    // consumes the pending turn-in.
    let pending = saved(&game);
    let coins_before = coins(&game);
    let forged = send(&mut game, &at, &quest_end_reply(QUEST + 1), now);
    now += 10;
    assert!(!forged.changed, "a forged 49/14 id changed gameplay");
    assert_eq!(saved(&game), pending, "a forged 49/14 changed the save");
    assert_eq!(
        pending_turnin(&game),
        Some(QUEST),
        "a forged 49/14 consumed the pending end screen"
    );
    assert_eq!(coins(&game), coins_before, "a forged 49/14 paid coins");

    // A replayed results Close outside a run changes nothing.
    let outside = saved(&game);
    let replay_close = send(&mut game, &at, &[26, 0, 42, 0], now);
    now += 10;
    assert!(!replay_close.changed && replay_close.zoning.is_none());
    assert_eq!(saved(&game), outside, "a replayed Close changed the save");

    // Canary: the real 49/14 pays the real quest once, and its replay is inert.
    finish_quest(&mut game, &at, QUEST, ADVENTURER, COINS, 0, now);
    now += 10;
    assert_eq!(coins(&game), coins_before + COINS);
    let paid = saved(&game);
    assert_packet_changes_nothing(
        &mut game,
        &at,
        &quest_end_reply(QUEST),
        now,
        "a replayed 49/14 after payment",
    );
    assert_eq!(saved(&game), paid);
}

/// A reconnect before the win restores Active/0; a reconnect after the win but
/// before 49/14 restores Active/1 and a fresh Talk re-sends the saved pending
/// end screen; the completed save keeps its coins and never pays again.
#[test]
fn the_quest_and_reward_survive_a_reload() {
    let fixture = Fixture::new();
    let row = battle_row();
    let (mut game, mut at) = player(&fixture, profiles(ADVENTURER, &[ADVENTURER, BRAWLER]));
    let mut now = 1_000;

    accept_at_anne(&mut game, &at, &mut now);
    let mut game = reconnect(&fixture, &game);
    assert_eq!(quest_status(&game, QUEST), QuestStatus::Active);
    assert_eq!(counts(&game, QUEST), [0], "mid-quest reconnect keeps 0/1");
    assert_eq!(pending_turnin(&game), None);

    win_sheep_watch(&mut game, &mut at, &row, &mut now);
    assert_eq!(counts(&game, QUEST), [1]);

    let mut game = reconnect(&fixture, &game);
    assert_eq!(quest_status(&game, QUEST), QuestStatus::Active);
    assert_eq!(counts(&game, QUEST), [1], "the win survives the reconnect");
    assert_eq!(pending_turnin(&game), None);

    at.position = anne_position();
    open_end_screen(&mut game, &at, &mut now);
    let coins_before = coins(&game);

    // The pending end screen is saved state. A fresh Talk re-sends it, so a
    // reconnect can still settle the reward.
    let mut game = reconnect(&fixture, &game);
    assert_eq!(pending_turnin(&game), Some(QUEST));
    assert_eq!(counts(&game, QUEST), [1]);
    assert_eq!(coins(&game), coins_before);
    let resent = open_end_screen(&mut game, &at, &mut now);
    assert_eq!(u32_at(&resent, 18), END_TEXT);
    assert_eq!(pending_turnin(&game), Some(QUEST));
    finish_quest(&mut game, &at, QUEST, ADVENTURER, COINS, 0, now);
    now += 10;
    assert_eq!(coins(&game), coins_before + COINS);
    assert_eq!(quest_status(&game, QUEST), QuestStatus::Completed);

    let paid = coins(&game);
    let mut game = reconnect(&fixture, &game);
    assert_eq!(quest_status(&game, QUEST), QuestStatus::Completed);
    assert_eq!(counts(&game, QUEST), [1]);
    assert_eq!(coins(&game), paid, "the completed save keeps its coins");
    assert_packet_changes_nothing(
        &mut game,
        &at,
        &quest_end_reply(QUEST),
        now,
        "a post-relog 49/14",
    );
    assert_eq!(coins(&game), paid);
}

/// Two independent characters: the first player's battle win and turn-in never
/// credit, pay or save the second, and the second's own win pays only the
/// second.
#[test]
fn a_second_player_cannot_use_the_first_players_win() {
    let first_fixture = Fixture::for_character(CHARACTER);
    let second_fixture = Fixture::for_character(CHARACTER + 1);
    let row = battle_row();
    let (mut first, mut first_at) =
        player(&first_fixture, profiles(ADVENTURER, &[ADVENTURER, BRAWLER]));
    let (mut second, mut second_at) = player_as(
        &second_fixture,
        CHARACTER + 1,
        anne_position(),
        profiles(ADVENTURER, &[ADVENTURER, BRAWLER]),
    );
    let mut now = 1_000;
    let mut second_now = 5_000;
    accept_at_anne(&mut first, &first_at, &mut now);
    accept_at_anne(&mut second, &second_at, &mut second_now);
    let second_before = saved(&second);
    let second_coins = coins(&second);

    win_sheep_watch(&mut first, &mut first_at, &row, &mut now);
    assert_eq!(counts(&first, QUEST), [1]);
    assert_eq!(
        saved(&second),
        second_before,
        "the first player's win changed the second player's save"
    );
    assert_eq!(counts(&second, QUEST), [0]);
    assert_eq!(coins(&second), second_coins);

    first_at.position = anne_position();
    open_end_screen(&mut first, &first_at, &mut now);
    finish_quest(&mut first, &first_at, QUEST, ADVENTURER, COINS, 0, now);
    assert_eq!(quest_status(&first, QUEST), QuestStatus::Completed);
    assert_eq!(
        saved(&second),
        second_before,
        "the first player's turn-in changed the second player's save"
    );
    assert_eq!(counts(&second, QUEST), [0]);

    let first_paid = saved(&first);
    win_sheep_watch(&mut second, &mut second_at, &row, &mut second_now);
    assert_eq!(counts(&second, QUEST), [1]);
    assert_eq!(
        saved(&first),
        first_paid,
        "the second player's win changed the first player's paid save"
    );
    second_at.position = anne_position();
    open_end_screen(&mut second, &second_at, &mut second_now);
    finish_quest(
        &mut second,
        &second_at,
        QUEST,
        ADVENTURER,
        COINS,
        0,
        second_now,
    );
    assert_eq!(quest_status(&second, QUEST), QuestStatus::Completed);
    assert_eq!(
        saved(&first),
        first_paid,
        "the second player's turn-in changed the first player's paid save"
    );
}

/// 49/14 pays on the live inventory: a coin and an item grant made after the
/// end screen opened (the same reward seam a vendor buy, trade or claim uses)
/// survive the payout instead of being overwritten by the accept-time snapshot.
#[test]
fn a_live_change_before_the_turn_in_survives() {
    let fixture = Fixture::new();
    let row = battle_row();
    let (mut game, mut at) = player(&fixture, profiles(ADVENTURER, &[ADVENTURER, BRAWLER]));
    let mut now = 1_000;

    accept_at_anne(&mut game, &at, &mut now);
    win_sheep_watch(&mut game, &mut at, &row, &mut now);
    at.position = anne_position();
    let end = open_end_screen(&mut game, &at, &mut now);
    assert_eq!(u32_at(&end, 27), COINS);

    let coins_before = coins(&game);
    let items_before = item_count(&game, FILLER_ITEM);
    let granted = send(&mut game, &at, &chat::local_request("realm coins 100"), now);
    assert!(granted.changed, "the live coin grant commits");
    now += 10;
    let given = send(
        &mut game,
        &at,
        &chat::local_request(&format!("realm give {FILLER_ITEM} 1")),
        now,
    );
    assert!(given.changed, "the live item grant commits");
    now += 10;
    assert_eq!(coins(&game), coins_before + 100);
    assert_eq!(item_count(&game, FILLER_ITEM), items_before + 1);

    finish_quest(&mut game, &at, QUEST, ADVENTURER, COINS, 0, now);
    assert_eq!(
        coins(&game),
        coins_before + 100 + COINS,
        "49/14 paid on the accept-time wallet instead of the live one"
    );
    assert_eq!(
        item_count(&game, FILLER_ITEM),
        items_before + 1,
        "49/14 restored the accept-time inventory over the live one"
    );
    assert_eq!(quest_status(&game, QUEST), QuestStatus::Completed);
}

/// A live vendor purchase commits between the win and the turn-in and survives
/// the 49/14 payout: Vela's real 26/8 -> 26/10 Shop -> 66/43 buy debits the
/// catalogue price and adds a new stack of the item (the opening bag has none),
/// then 49/14 adds only its own 20 coins on the live wallet and bag, never
/// restoring the accept-time snapshot. The item and price come from
/// `vendors.json`/`stores.json`, the same content the 165/10/66/43 route reads.
/// This is the real merchant transaction; `a_live_change_before_the_turn_in_survives`
/// keeps the developer-seam grant as the complementary case.
#[test]
fn a_vendor_purchase_before_the_turn_in_survives() {
    let fixture = Fixture::new();
    let row = battle_row();
    let (mut game, mut at) = player(&fixture, profiles(ADVENTURER, &[ADVENTURER, BRAWLER]));
    let mut now = 1_000;
    let (item, price) = vendor_item(VELA);

    accept_at_anne(&mut game, &at, &mut now);
    win_sheep_watch(&mut game, &mut at, &row, &mut now);
    at.position = anne_position();
    open_end_screen(&mut game, &at, &mut now);

    // Fund the purchase through the developer seam (the fixture opens with an
    // empty wallet); the buy itself is the real merchant route.
    let funded = send(
        &mut game,
        &at,
        &chat::local_request(&format!("realm coins {}", price + 100)),
        now,
    );
    assert!(funded.changed, "the funding grant commits");
    now += 10;

    at.position = content_position("farnum_farmstead.json", VELA);
    let list = open_shop(&mut game, &at, VELA, now);
    assert!(!list.is_empty(), "Vela sends a 165/10 coin list");
    now += 10;
    let coins_before = coins(&game);
    let items_before = item_count(&game, item);
    assert_eq!(
        items_before, 0,
        "the opening bag has no stack of the bought item"
    );
    // The shared vendor kit carries the 66/43 serializer `0x00b3ab20` and the
    // 66/44 status reader `0x00b32430` (status at wire offset 48).
    let bought = send(&mut game, &at, &vendor_buy_packet(7_001, item, 1), now);
    now += 10;
    assert!(bought.changed, "the vendor buy commits");
    assert_eq!(
        merchant_status(
            bought
                .replies
                .last()
                .expect("a 66/43 request always answers with 66/44")
        ),
        1,
        "66/44 success"
    );
    assert_eq!(
        coins(&game),
        coins_before - price,
        "the buy debits the catalogue price"
    );
    assert_eq!(
        item_count(&game, item),
        items_before + 1,
        "the buy adds the item"
    );

    // The end screen is still pending at Anne; settle it on the live state.
    at.position = anne_position();
    assert_eq!(pending_turnin(&game), Some(QUEST));
    finish_quest(&mut game, &at, QUEST, ADVENTURER, COINS, 0, now);
    assert_eq!(
        coins(&game),
        coins_before - price + COINS,
        "49/14 paid on the accept-time wallet instead of the live one"
    );
    assert_eq!(
        item_count(&game, item),
        items_before + 1,
        "49/14 restored the accept-time inventory over the live purchase"
    );
    assert_eq!(quest_status(&game, QUEST), QuestStatus::Completed);
}

/// A wallet that cannot hold the 20-coin grant refuses the turn-in atomically:
/// no coins, no save bytes and the pending end screen stays retryable. The same
/// pending save with a normal wallet pays exactly once.
#[test]
fn an_over_cap_turn_in_is_refused_and_stays_pending() {
    let fixture = Fixture::new();
    let row = battle_row();
    let (mut game, mut at) = player(&fixture, profiles(ADVENTURER, &[ADVENTURER, BRAWLER]));
    let mut now = 1_000;

    accept_at_anne(&mut game, &at, &mut now);
    win_sheep_watch(&mut game, &mut at, &row, &mut now);
    at.position = anne_position();
    open_end_screen(&mut game, &at, &mut now);

    let mut state = game.state();
    state.inventory.coins = i32::MAX as u32 - (COINS - 1);
    let mut capped = fixture.game_as(CHARACTER, Some(state));
    let before_state = capped.state();
    let before_bytes = saved(&capped);
    assert!(
        !capped
            .handle_in_world(&quest_end_reply(QUEST), &at, now)
            .expect("an over-cap coin grant is a refusal, not an error (T382)")
            .changed,
        "an over-cap coin grant is refused for this player"
    );
    assert_eq!(
        capped.state(),
        before_state,
        "the over-cap refusal changed state"
    );
    assert_eq!(
        saved(&capped),
        before_bytes,
        "the over-cap refusal changed save bytes"
    );
    assert_eq!(coins(&capped), i32::MAX as u32 - (COINS - 1));
    assert_eq!(
        pending_turnin(&capped),
        Some(QUEST),
        "the refused reward stays pending, not half-paid"
    );
    assert_eq!(quest_status(&capped, QUEST), QuestStatus::Active);

    // Control: the same pending save with a normal wallet pays once.
    let mut paying = fixture.game_as(CHARACTER, Some(game.state()));
    let coins_before = coins(&paying);
    finish_quest(&mut paying, &at, QUEST, ADVENTURER, COINS, 0, now + 10);
    assert_eq!(coins(&paying), coins_before + COINS);
    assert_eq!(quest_status(&paying, QUEST), QuestStatus::Completed);
}

/// Busy rules are mutual: while the mining movie owns the character, 26/8 on
/// Anne is refused with no state change. The canary is the same Talk working
/// once the movie is cancelled.
#[test]
fn a_mining_movie_refuses_the_talk_until_it_closes() {
    let fixture = Fixture::new();
    fixture.overlay_content(&content_dir());
    // The opening inventory carries no harvesting reward definition; this test
    // only needs the mining movie (the same overlay rule as playtest_first_hour).
    std::fs::remove_file(fixture.path().join("harvesting.json")).unwrap();
    let (mut game, at) = player(&fixture, profiles(MINER, &[ADVENTURER, MINER]));
    let mut now = 2_000;

    accept_at_anne(&mut game, &at, &mut now);
    let started = send(&mut game, &at, &chat::local_request("realm mine"), now);
    now += 10;
    assert!(started.changed, "the mining movie starts");
    assert!(
        game.state()
            .mining
            .as_ref()
            .is_some_and(|state| state.active.is_some()),
        "the mining run is active"
    );

    let before = saved(&game);
    let refused = send(&mut game, &at, &npc_interact(ANNE), now);
    now += 10;
    assert!(!refused.changed, "the busy Talk changed gameplay");
    assert!(
        chat_text(&refused).contains("Finish the active activity before interacting."),
        "{}",
        chat_text(&refused)
    );
    assert_eq!(
        saved(&game),
        before,
        "the busy refusal changed the quest or the save"
    );
    assert_eq!(quest_status(&game, QUEST), QuestStatus::Active);
    assert_eq!(counts(&game, QUEST), [0]);

    let exited = send(&mut game, &at, &chat::local_request("realm exit"), now);
    now += 10;
    assert!(exited.changed);
    assert!(
        game.state()
            .mining
            .as_ref()
            .is_none_or(|state| state.active.is_none()),
        "the mining movie is cancelled"
    );
    let dialogue = talk(&mut game, &at, ANNE, now);
    assert!(
        dialogue
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[26, 0, 3, 0])),
        "the canary Talk opens Anne's dialogue once the movie closes"
    );
    assert_eq!(quest_status(&game, QUEST), QuestStatus::Active);
}

/// Loader/encoder test (not a request-path test): the authored record, its text
/// ids, Anne's binding, the Adventurer registry union and the battle it credits
/// resolve through the split loader and its 49/13/49/1 encoders. The behaviour
/// tests above, not this test, prove the live packet path.
#[test]
fn the_quest_record_text_giver_and_battle_resolve() {
    let content = content_dir();
    let document = read_json(&content.join("quests/adventurer.json"));
    let record = document["records"]
        .as_array()
        .unwrap()
        .iter()
        .find(|record| record["id"].as_u64() == Some(u64::from(QUEST)))
        .expect("the 3000061 record exists");
    assert_eq!(record["enabled"], true);
    assert!(
        record.get("job").is_none(),
        "a regional quest carries no job key (the loader treats absence as no gate)"
    );
    assert_eq!(record["giver_npc_id"], ANNE);
    assert_eq!(record["prerequisite_quest_ids"], serde_json::json!([]));
    assert_eq!(record["title_string_id"], TITLE);
    assert_eq!(record["intro_text_string_id"], INTRO);
    assert_eq!(record["description_string_id"], DESCRIPTION);
    assert_eq!(record["end_text_string_id"], END_TEXT);
    let objectives = record["objectives"].as_array().unwrap();
    assert_eq!(objectives.len(), 1, "the win completes the only objective");
    assert_eq!(objectives[0]["kind"], "complete_dungeon");
    assert_eq!(objectives[0]["id"], 1);
    assert_eq!(objectives[0]["encounter_id"], BATTLE);
    assert!(
        objectives[0]["source"]
            .as_array()
            .is_some_and(|source| !source.is_empty()),
        "the objective carries provenance"
    );
    let rewards = record["rewards"].as_array().unwrap();
    assert_eq!(rewards.len(), 1);
    assert_eq!(rewards[0]["coins"], COINS);
    assert_eq!(rewards[0]["items"], serde_json::json!([]));
    assert_eq!(rewards[0]["job_xp"], serde_json::json!({}));
    // The reward row itself cites the checked guide and separates its dungeon
    // rewards from a quest payout, so an auditor does not have to infer it from
    // the outer source.
    let reward_evidence = rewards[0]["source"][0]["evidence"].as_str().unwrap();
    assert!(
        reward_evidence.contains("Dungeons/Sheep Watch.md"),
        "the reward source cites the Sheep Watch guide"
    );
    assert!(
        reward_evidence.contains("no quest payout"),
        "the reward source records that the guide has no quest payout"
    );
    assert!(
        reward_evidence.contains("prize-wheel"),
        "the reward source records the guide's dungeon rewards"
    );
    for (index, tag) in record["source"].as_array().unwrap().iter().enumerate() {
        assert!(
            tag["evidence"]
                .as_str()
                .is_some_and(|text| !text.is_empty()),
            "source tag {index} has no evidence"
        );
    }
    let kinds: BTreeSet<&str> = record["source"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|tag| tag["kind"].as_str())
        .collect();
    assert!(kinds.contains("client"));
    assert!(kinds.contains("legacy-data"));
    assert!(kinds.contains("reconstructed"));

    // The Adventurer job's split-loader union lists it in sorted order.
    let jobs = read_json(&content.join("jobs.json"));
    let adventurer = jobs["records"]
        .as_array()
        .unwrap()
        .iter()
        .find(|job| job["profile_id"].as_i64() == Some(i64::from(ADVENTURER)))
        .expect("the Adventurer job");
    let ids: Vec<u64> = adventurer["quest_ids"]
        .as_array()
        .unwrap()
        .iter()
        .map(|id| id.as_u64().unwrap())
        .collect();
    assert!(
        ids.contains(&u64::from(QUEST)),
        "the Adventurer union lists 3000061"
    );
    assert!(ids.windows(2).all(|pair| pair[0] <= pair[1]));

    // Anne's NPC row names the quest without changing its existing greeting.
    let npcs = read_json(&content.join("npcs/farnum_farmstead.json"));
    let anne = npcs["records"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["npc_id"].as_u64() == Some(ANNE))
        .expect("Anne's NPC row");
    assert_eq!(anne["quest_ids_given"], serde_json::json!([QUEST]));
    assert_eq!(anne["quests_given"], serde_json::json!([]));
    assert_eq!(anne["greeting_string_id"], INTRO);
    assert!(
        anne["roles"]
            .as_array()
            .unwrap()
            .iter()
            .any(|role| role == "quest_giver")
    );

    // The battle row exists and carries the recovered goal text.
    let row = battle_row();
    assert_eq!(row["objective_id"].as_u64(), Some(u64::from(GOAL_TEXT)));
    assert_eq!(marker_guid(&row), MARKER_GUID);
    assert_eq!(row["zone"]["zone_id"], ARENA_ZONE);

    // The split loader resolves every id and both screens encode the reward.
    let engine =
        QuestEngine::from_split_content(&content, 1, CharacterEligibility::default(), None)
            .expect("the split catalogue loads");
    assert!(engine.is_enabled(QUEST));
    assert_eq!(engine.giver_npc_id(QUEST), Some(ANNE));
    assert_eq!(engine.title_string_id(QUEST), Some(TITLE));
    assert_eq!(engine.intro_text_string_id(QUEST), Some(INTRO));
    assert_eq!(engine.description_string_id(QUEST), Some(DESCRIPTION));
    assert_eq!(engine.end_text_string_id(QUEST), Some(END_TEXT));
    let end = engine.quest_end_packet(QUEST, ANNE).unwrap();
    assert_eq!(end.len(), 99);
    assert_eq!(u32_at(&end, 14), QUEST);
    assert_eq!(u32_at(&end, 18), END_TEXT);
    assert_eq!(u32_at(&end, 27), COINS);
    assert_eq!(u32_at(&end, 31), 0);

    // The four locale ids are the recovered non-zero Global.Text rows.
    let locale = read_json(&lab("reports/locale-id-to-text.json"));
    let texts: BTreeMap<u32, String> = locale
        .as_array()
        .unwrap()
        .iter()
        .map(|entry| {
            (
                entry["id"].as_u64().unwrap() as u32,
                entry["text"].as_str().unwrap().to_owned(),
            )
        })
        .collect();
    assert_eq!(
        texts.get(&TITLE).map(String::as_str),
        Some("Investigate the Wolves")
    );
    assert_eq!(
        texts.get(&END_TEXT).map(String::as_str),
        Some("Thank you! Hopefully, the sheep can get some rest now.")
    );
    for id in [INTRO, DESCRIPTION] {
        assert!(
            texts.get(&id).is_some_and(|text| !text.is_empty()),
            "text id {id} does not resolve"
        );
    }
    assert_eq!(
        texts.get(&GOAL_TEXT).map(String::as_str),
        Some("Defeat the Ravenous Wolves at Sheep Watch")
    );
    for id in [TITLE, INTRO, DESCRIPTION, END_TEXT] {
        assert!(id != 0);
    }
    // The coins-only reward shows up on the 49/1 card too (bundle at 31).
    let bundle = engine.quest_reward_bundle(QUEST).unwrap();
    assert_eq!(bundle[0], 1, "bundle success flag");
    assert_eq!(
        i32::from_le_bytes(bundle[1..5].try_into().unwrap()),
        COINS as i32,
        "49/1 and 49/13 share the coin dword"
    );
}
