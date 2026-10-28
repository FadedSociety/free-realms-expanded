//! T300/T334/T411: the Miner's first three playable records, "So, You Want To Be A Miner?"
//! (3100002), its follow-up "All I Got" (3100003) and "Rush Order" (3100004).
//!
//! 3100003 is the Trixi gear referral: the record's own description 76088 ("Visit Trixi in
//! the Singing Crystal Mines and find out if she has any better mining gear for you.") and
//! its one Talk-to-Trixi objective are the authority; the guide's row-2 title match is
//! reconstructed (R). Its job gate (14), prerequisite (3100002) and 5-coin / 15-Miner-XP
//! reward mirror the first rung.
//!
//! 3100004 is Trixi's "Rush Order": Talk to Smitty (the video tracker's first goal "Speak
//! with Smitty"), mine six Copper Ore at a live copper vein, then return to Trixi
//! (description 76210). Quantity 6 (one shipped Mining18 goal row), the 5-coin / 20-Miner-XP
//! reward and the deferred smelting/coal step are (R). This suite's synthetic board pays one
//! ore per win, so the Rush Order tests patch only their private fixture copy to pay six.
//!
//! Therin (9000000217), the guildmaster of the Singing Crystal Mines, is a POI anchor in the
//! Snowhill content; the `npc_spawns_extra.json` row added here is the drawable actor. The
//! quest is offered by the real NPC router and counts one Copper Ore (item definition 2739)
//! gathered from a live Snowhill copper vein through the ordinary Type8 Mining18 round, then
//! the return Talk at Therin opens the 49/13 end screen and 49/14 pays 5 coins + 15 Miner XP.
//!
//! Client packet layouts used here (IDA native map, re-read this run):
//! - 26/8 CommandPacketInteractRequest constructor `008b6bc0`: two u32 target words and a
//!   u8 trailing flag.
//! - 26/10 CommandPacketInteractionSelect constructor `00a36f80`: two u32 target words and
//!   an i32 choice.
//! - 26/6 PacketDialogResponse constructor `009f5890`: one i32 row (the quest row is 301).
//! - 49/1 QuestInfo reader `00c7b7a0` (entry `00c7bb60`): quest at 6, intro at 10, title at
//!   14/18, description at 22 (the layout pinned by the Wizard/Archer/Medic chapter tests).
//! - 49/9 objective-progress reader `00c7af60` and 49/10 objective-complete reader
//!   `00c7b070`: quest then objective dwords; no new shape is used here.
//! - 49/2 QuestReply constructor `00a86400`: u32 quest id and a u8 accept flag.
//! - 49/13 QuestEnd reader `00c7b990`: u64 actor, quest i32, end-text u32, then the reward
//!   bundle read by `008e7930` (success at 26, coins at 27, displayed XP total at 31).
//! - 49/14 QuestEndReply constructor `00a864c0`: one u32 quest id.
//! - 35/2 AddNpc body reader `00921a10`: actor u64 first, then name id, then model id. The
//!   name is a `Global.Text.<id>` id (client formats `Global.Text.%d` at `0x01804c14`), so the
//!   spawn supplies `Global.Text.185` ("Therin"), not the Snowhill row's raw `ucdt` hash
//!   943830538. The shipped client `Models.txt` row 381 is `dwarf_therin_m.adr` and that model
//!   is in the pack catalogue/manifest; Legacy's row 46 `dwarf_m_therin.adr` is not shipped.
//!
//! The shared fixture, packet builders and refusal helpers come from
//! `tests/support/npc_quest_chain.rs`; the one-goal work board and the node/round helpers
//! come from `tests/support/mining_quest_chain.rs`, shared with the T241 Farnum suite. The
//! board is the established T241/W171 synthetic fixture: the shipped Mining18 board pays a
//! 12-ore batch per win, so it cannot be used to prove a one-ore objective without a chain
//! of level payloads. Node entry, the win transaction, the gather event and the quest
//! payout are the production path.

#[path = "support/mining_quest_chain.rs"]
mod mining_quest_chain;
#[path = "support/npc_quest_chain.rs"]
mod npc_quest_chain;

use mining_quest_chain::{
    COPPER_ORE, Fixture, MINING_XP_PER_WIN, active_mining, assert_offer_ids, copper_nodes,
    copper_ore, fill_bag, gather_copper, offer_packet, open_end_screen, type8_frame, type8_payload,
    word,
};
use npc_quest_chain::{
    CHARACTER, accept, active_job, assert_no_offer, assert_packet_changes_nothing,
    character_at_for, coins, content_dir, content_position, counts, end_screen_reward,
    finish_quest, has_family, npc_interact, npc_select, objective_activated,
    objective_activated_packet, objective_complete, offer, pending_offer, pending_turnin,
    profile_xp, quest_end_reply, quest_reply, quest_status, read_json, saved, select_job, send,
    talk, visible_npcs,
};
use osfr_tools::{
    character_store::CharacterState, gameplay::Gameplay, inventory::MAX_ITEMS, mining,
    profiles::ProfileState, quest_engine::QuestStatus, startup_world::NpcSpawn,
};
use serde_json::{Value, json};
use std::collections::BTreeMap;

const THERIN: u64 = 9_000_000_217;
/// Trixi (9000000220) is the Snowhill legacy_data regional row the 3100003 Talk targets and
/// the giver of 3100004.
const TRIXI: u64 = 9_000_000_220;
/// Smitty (9000000206), the Snowhill Blacksmith trainer (legacy_data row): a wrong Talk
/// target for 3100003 and the first Talk of 3100004.
const SMITTY: u64 = 9_000_000_206;
const MINER_FIRST: u32 = 3_100_002;
/// T334: the second Miner chapter, the Trixi mining-gear referral.
const ALL_I_GOT: u32 = 3_100_003;
/// T411: the third Miner chapter, Smitty's rush order of mining gear for Trixi.
const RUSH_ORDER: u32 = 3_100_004;
const RUSH_ORDER_INTRO: u32 = 76_208;
const RUSH_ORDER_TITLE: u32 = 76_209;
const RUSH_ORDER_DESCRIPTION: u32 = 76_210;
const RUSH_ORDER_END_TEXT: u32 = 76_211;
/// (R) T411: Rush Order counts six Copper Ore, one shipped Mining18 goal row.
const RUSH_ORE: u32 = 6;
const DRAINED_RECORDS: [u32; 2] = [3_100_001, 3_100_005];
const TITLE: u32 = 23_217;
const DESCRIPTION: u32 = 23_218;
const END_TEXT: u32 = 23_219;
const ALL_I_GOT_INTRO: u32 = 76_086;
const ALL_I_GOT_TITLE: u32 = 76_087;
const ALL_I_GOT_DESCRIPTION: u32 = 76_088;
const ALL_I_GOT_END_TEXT: u32 = 76_089;
const ADVENTURER: i32 = 1;
const MINER: i32 = 14;
const BRAWLER: i32 = 43;
/// 39/6 Quit: leave an offered/in-progress run without a payout.
const QUIT_FRAME_SUB: u8 = 6;

fn therin_position() -> [f32; 3] {
    content_position("snowhill.json", THERIN)
}

fn trixi_position() -> [f32; 3] {
    content_position("snowhill.json", TRIXI)
}

fn smitty_position() -> [f32; 3] {
    content_position("snowhill.json", SMITTY)
}

/// T411: the shared synthetic board pays one Copper Ore per win; Rush Order counts six, the
/// batch one shipped Mining18 goal row pays. Only this fixture's private board copy changes.
fn six_ore_fixture() -> Fixture {
    let fixture = Fixture::new();
    let path = fixture.path().join(mining::CONTENT_FILE);
    let mut board = read_json(&path);
    board["spec"]["outputs"][0]["cluster_quantity"] = json!(RUSH_ORE);
    std::fs::write(&path, board.to_string()).expect("the six-ore board writes");
    fixture
}

fn character_at(position: [f32; 3]) -> CharacterState {
    character_at_for(CHARACTER, 0, position)
}

/// Every occupied stack as a multiset of (definition, count), for "no free item" proofs.
fn item_ledger(game: &Gameplay) -> Vec<(i32, u32)> {
    let mut ledger: Vec<_> = game
        .state()
        .inventory
        .items
        .iter()
        .map(|item| (item.definition_id, item.count))
        .collect();
    ledger.sort_unstable();
    ledger
}

fn count(game: &Gameplay, objective: usize) -> u32 {
    counts(game, MINER_FIRST)[objective]
}

/// Complete 3100002 through the production offer, one live copper-vein Mining18 win, the
/// held Therin end screen and the 49/14 payout. Returns the timestamp after the turn-in.
fn complete_the_first_ore(game: &mut Gameplay, character: &CharacterState, now: u64) -> u64 {
    offer(game, character, THERIN, MINER_FIRST, now);
    accept(game, character, MINER_FIRST, now + 100);
    let (guid, position) = copper_nodes()[0];
    let mut at = now + 200;
    assert!(
        gather_copper(game, character, guid, position, &mut at).is_some(),
        "the live copper vein starts a Mining18 round"
    );
    open_end_screen(game, character, THERIN, MINER_FIRST, END_TEXT, at + 100);
    finish_quest(game, character, MINER_FIRST, MINER, 5, 15, at + 200);
    at + 300
}

/// T411: complete 3100002 and then 3100003 (Therin offer, Trixi Talk, 49/14) through the
/// production paths. Returns the timestamp after the 3100003 turn-in.
fn complete_all_i_got(game: &mut Gameplay, character: &CharacterState, now: u64) -> u64 {
    let now = complete_the_first_ore(game, character, now);
    offer(game, character, THERIN, ALL_I_GOT, now);
    accept(game, character, ALL_I_GOT, now + 100);
    let at_trixi = character_at_for(character.guid, 0, trixi_position());
    open_end_screen(
        game,
        &at_trixi,
        TRIXI,
        ALL_I_GOT,
        ALL_I_GOT_END_TEXT,
        now + 200,
    );
    finish_quest(game, &at_trixi, ALL_I_GOT, MINER, 5, 15, now + 300);
    now + 400
}

/// T411: a refused request leaves the serialized `GameplayState` and the real save bytes
/// equal, and adds or completes no quest.
fn assert_inert(
    fixture: &Fixture,
    game: &mut Gameplay,
    character: &CharacterState,
    packet: &[u8],
    now: u64,
    label: &str,
) {
    let before = fixture.saved_bytes(game, character);
    assert_packet_changes_nothing(game, character, packet, now, label);
    assert_eq!(
        fixture.saved_bytes(game, character),
        before,
        "{label} changed the save"
    );
}

/// One actor from the production per-session visibility batch for a session standing at `at`.
/// Each nearby session builds its own batch.
fn drawn_npc(fixture: &Fixture, character_id: u64, at: [f32; 3], actor_id: u64) -> NpcSpawn {
    let mut game = fixture.game_as(character_id, MINER, &[ADVENTURER, MINER]);
    let character = character_at_for(character_id, 0, at);
    visible_npcs(&mut game, &character)
        .into_iter()
        .find(|npc| npc.actor_id == actor_id)
        .unwrap_or_else(|| panic!("actor {actor_id} is drawn for every nearby session"))
}

fn row<'a>(records: &'a [Value], id: u64) -> &'a Value {
    records
        .iter()
        .find(|record| record["id"].as_u64() == Some(id) || record["npc_id"].as_u64() == Some(id))
        .unwrap_or_else(|| panic!("record {id} exists"))
}

fn has_source(value: &Value, kind: &str) -> bool {
    value["source"].as_array().is_some_and(|sources| {
        sources
            .iter()
            .any(|source| source["kind"].as_str() == Some(kind))
    })
}

#[test]
fn a_miner_is_offered_the_first_ore_quest_at_therin() {
    let fixture = Fixture::new();
    let character = character_at(therin_position());
    let mut miner = fixture.game(MINER, &[ADVENTURER, MINER]);

    let offered = offer(&mut miner, &character, THERIN, MINER_FIRST, 1_000);
    assert_offer_ids(
        &offer_packet(&offered, MINER_FIRST),
        MINER_FIRST,
        0,
        TITLE,
        DESCRIPTION,
    );
    assert_eq!(quest_status(&miner, MINER_FIRST), QuestStatus::Offered);

    // A second Talk while the offer is open neither re-offers nor consumes the open menu.
    let mut now = 2_000;
    assert_no_offer(
        &mut miner,
        &character,
        THERIN,
        &mut now,
        "a second Talk while the offer is open",
    );

    // A non-Miner is refused by the job gate: active Adventurer.
    let mut adventurer = fixture.game(ADVENTURER, &[ADVENTURER, MINER]);
    let mut now = 3_000;
    assert_no_offer(
        &mut adventurer,
        &character,
        THERIN,
        &mut now,
        "an active Adventurer",
    );

    // An unlocked, ranked-but-inactive Miner is still refused: the engine's job levels are
    // built from the saved active profile only.
    let mut state = Gameplay::load(fixture.path(), None)
        .expect("opening reference loads")
        .state();
    state.profiles = Some(ProfileState {
        version: 1,
        active: ADVENTURER,
        unlocked: [ADVENTURER, MINER].into_iter().collect(),
        experience: BTreeMap::from([(MINER, 200)]),
    });
    let mut inactive = fixture.game_with_state(state);
    let mut now = 4_000;
    assert_no_offer(
        &mut inactive,
        &character,
        THERIN,
        &mut now,
        "a Miner who is unlocked and ranked but not active",
    );
}

#[test]
fn mining_one_copper_then_talking_to_therin_pays_once() {
    let fixture = Fixture::new();
    let mut game = fixture.game(MINER, &[ADVENTURER, MINER, BRAWLER]);
    let character = character_at(therin_position());
    offer(&mut game, &character, THERIN, MINER_FIRST, 1_000);
    accept(&mut game, &character, MINER_FIRST, 1_100);
    assert_eq!(counts(&game, MINER_FIRST), [0, 0]);

    let nodes = copper_nodes();
    assert!(nodes.len() >= 2, "the copper pool has live spawns");
    let mut now = 2_000;
    let (guid, position) = nodes[0];
    let instance = gather_copper(&mut game, &character, guid, position, &mut now)
        .expect("the copper node starts a Mining18 round");
    assert_eq!(count(&game, 0), 1);
    assert_eq!(count(&game, 1), 0, "the return step opens after the ore");
    assert_eq!(copper_ore(&game), 1, "one win paid one Copper Ore");
    let miner_after_mining = profile_xp(&game, MINER);
    assert_eq!(miner_after_mining, MINING_XP_PER_WIN);

    let end = open_end_screen(
        &mut game,
        &character,
        THERIN,
        MINER_FIRST,
        END_TEXT,
        now + 100,
    );
    assert_eq!(end_screen_reward(&end), (5, 15), "49/13 reward bundle");
    assert_eq!(coins(&game), 0, "the end screen pays nothing");
    finish_quest(&mut game, &character, MINER_FIRST, MINER, 5, 15, now + 200);
    assert_eq!(profile_xp(&game, MINER), miner_after_mining + 15);

    // A replayed 49/14 and a stale win payload for the closed run change nothing.
    let finished = saved(&game);
    assert_packet_changes_nothing(
        &mut game,
        &character,
        &quest_end_reply(MINER_FIRST),
        now + 300,
        "a replayed 49/14",
    );
    let _ = game.handle_in_world(
        &type8_payload(instance, "OnNextLevelMsg"),
        &character,
        now + 400,
    );
    assert_eq!(saved(&game), finished, "a replayed win changed the save");
}

#[test]
fn the_miner_xp_goes_to_profile_14_even_after_a_job_switch() {
    let fixture = Fixture::new();
    let mut game = fixture.game(MINER, &[ADVENTURER, MINER, BRAWLER]);
    let character = character_at(therin_position());
    offer(&mut game, &character, THERIN, MINER_FIRST, 1_000);
    accept(&mut game, &character, MINER_FIRST, 1_100);

    let nodes = copper_nodes();
    let mut now = 2_000;
    let (guid, position) = nodes[0];
    assert!(gather_copper(&mut game, &character, guid, position, &mut now).is_some());
    let miner_after_mining = profile_xp(&game, MINER);
    assert_eq!(miner_after_mining, MINING_XP_PER_WIN);

    // Switch away after the gather and before the turn-in: the reward map names Miner.
    let switched = send(&mut game, &character, &select_job(BRAWLER), now + 100);
    assert!(switched.changed, "the Brawler outfit is selectable");
    assert_eq!(active_job(&game), BRAWLER);
    let brawler_before = profile_xp(&game, BRAWLER);

    let end = open_end_screen(
        &mut game,
        &character,
        THERIN,
        MINER_FIRST,
        END_TEXT,
        now + 200,
    );
    assert_eq!(end_screen_reward(&end), (5, 15));
    finish_quest(&mut game, &character, MINER_FIRST, MINER, 5, 15, now + 300);
    assert_eq!(
        profile_xp(&game, MINER),
        miner_after_mining + 15,
        "the quest pays profile 14"
    );
    assert_eq!(
        profile_xp(&game, BRAWLER),
        brawler_before,
        "the active Brawler profile receives none of the quest XP"
    );
}

#[test]
fn progress_and_payment_survive_a_reload_and_a_replay_pays_nothing() {
    let fixture = Fixture::new();
    let mut game = fixture.game(MINER, &[ADVENTURER, MINER, BRAWLER]);
    let character = character_at(therin_position());
    offer(&mut game, &character, THERIN, MINER_FIRST, 1_000);
    accept(&mut game, &character, MINER_FIRST, 1_100);

    let nodes = copper_nodes();
    let mut now = 2_000;
    let (guid, position) = nodes[0];
    assert!(gather_copper(&mut game, &character, guid, position, &mut now).is_some());
    assert_eq!(count(&game, 0), 1);

    // Reload before the completion Talk: the counted ore survives, nothing is paid, and no
    // end screen is pending.
    let coins_before = coins(&game);
    let xp_before = profile_xp(&game, MINER);
    let save_bytes = saved(&game);
    let mut game = fixture.save_and_reload(&game, &character);
    assert_eq!(quest_status(&game, MINER_FIRST), QuestStatus::Active);
    assert_eq!(counts(&game, MINER_FIRST), [1, 0]);
    assert_eq!(pending_turnin(&game), None);
    assert_eq!(coins(&game), coins_before, "the reload paid nothing");
    assert_eq!(profile_xp(&game, MINER), xp_before);
    assert_eq!(
        saved(&game),
        save_bytes,
        "the reload round-trips the gameplay payload"
    );

    // The completion Talk opens the held 49/13, and a second reload keeps it pending.
    let end = open_end_screen(
        &mut game,
        &character,
        THERIN,
        MINER_FIRST,
        END_TEXT,
        now + 100,
    );
    assert_eq!(end_screen_reward(&end), (5, 15));
    assert_eq!(pending_turnin(&game), Some(MINER_FIRST));
    let mut game = fixture.save_and_reload(&game, &character);
    assert_eq!(
        pending_turnin(&game),
        Some(MINER_FIRST),
        "the pending turn-in survives the reload"
    );
    let relog = game
        .update_npc_visibility(&character)
        .expect("the relog visibility batch builds");
    assert!(
        relog
            .iter()
            .any(|packet| has_family(packet, 49, 13, Some(MINER_FIRST))),
        "the relog batch re-sends the held end screen"
    );

    finish_quest(&mut game, &character, MINER_FIRST, MINER, 5, 15, now + 200);
    let coins_after = coins(&game);
    let xp_after = profile_xp(&game, MINER);
    assert_eq!(coins_after, coins_before + 5);
    assert_eq!(xp_after, xp_before + 15);

    // A reconnect after the turn-in keeps the completed history and pays nothing more.
    let mut game = fixture.save_and_reload(&game, &character);
    assert_eq!(quest_status(&game, MINER_FIRST), QuestStatus::Completed);
    assert_eq!(coins(&game), coins_after);
    assert_eq!(profile_xp(&game, MINER), xp_after);
    assert_packet_changes_nothing(
        &mut game,
        &character,
        &quest_end_reply(MINER_FIRST),
        now + 300,
        "a 49/14 replayed after the reconnect",
    );
}

#[test]
fn a_full_bag_refuses_the_node_before_the_run_and_changes_nothing() {
    let fixture = Fixture::new();
    let mut game = fixture.game_with_full_bag(MINER, &[ADVENTURER, MINER, BRAWLER]);
    let character = character_at(therin_position());
    offer(&mut game, &character, THERIN, MINER_FIRST, 1_000);
    accept(&mut game, &character, MINER_FIRST, 1_100);
    assert_eq!(game.state().inventory.items.len(), MAX_ITEMS);
    assert!(
        game.state()
            .inventory
            .items
            .iter()
            .all(|item| item.definition_id != COPPER_ORE as i32),
        "the full bag has no Copper Ore stack for the payout to merge into"
    );

    let nodes = copper_nodes();
    let (guid, position) = nodes[0];
    let at_node = character_at(position);
    let now = 2_000;
    game.tick(&at_node, now).expect("node tick");
    let before = saved(&game);
    let refused = send(&mut game, &at_node, &npc_interact(guid), now + 1);
    assert!(
        !refused.changed,
        "a full bag refuses the node before the run begins"
    );
    assert!(
        refused
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[15, 0, 3, 0])),
        "the refusal is a chat message, not a silent drop"
    );
    assert!(active_mining(&game).is_none(), "no mining run started");
    assert_eq!(counts(&game, MINER_FIRST), [0, 0]);
    assert_eq!(copper_ore(&game), 0);
    assert_eq!(
        game.state().inventory.items.len(),
        MAX_ITEMS,
        "the refused node changed the bag"
    );
    assert_eq!(saved(&game), before, "the refused node changed the save");
}

#[test]
fn a_turn_in_with_no_item_does_not_check_bag_space() {
    let fixture = Fixture::new();
    let mut game = fixture.game(MINER, &[ADVENTURER, MINER, BRAWLER]);
    let character = character_at(therin_position());
    offer(&mut game, &character, THERIN, MINER_FIRST, 1_000);
    accept(&mut game, &character, MINER_FIRST, 1_100);

    let nodes = copper_nodes();
    let mut now = 2_000;
    let (guid, position) = nodes[0];
    assert!(gather_copper(&mut game, &character, guid, position, &mut now).is_some());
    let end = open_end_screen(
        &mut game,
        &character,
        THERIN,
        MINER_FIRST,
        END_TEXT,
        now + 100,
    );
    assert_eq!(end_screen_reward(&end), (5, 15));

    // The pending turn-in is saved; fill the bag at the save boundary and reload it.
    let mut state = game.state();
    fill_bag(&mut state.inventory);
    let mut game = fixture.game_with_state(state);
    assert_eq!(pending_turnin(&game), Some(MINER_FIRST));
    assert_eq!(game.state().inventory.items.len(), MAX_ITEMS);
    let items_before = item_ledger(&game);
    let coins_before = coins(&game);
    let xp_before = profile_xp(&game, MINER);

    finish_quest(&mut game, &character, MINER_FIRST, MINER, 5, 15, now + 200);
    assert_eq!(coins(&game), coins_before + 5, "the coins still pay");
    assert_eq!(
        profile_xp(&game, MINER),
        xp_before + 15,
        "the XP still pays"
    );
    assert_eq!(
        game.state().inventory.items.len(),
        MAX_ITEMS,
        "the item-free reward did not check or change bag space"
    );
    assert_eq!(
        item_ledger(&game),
        items_before,
        "the reward granted no item"
    );
}

#[test]
fn refusals_and_replay_change_nothing() {
    let fixture = Fixture::new();
    let character = character_at(therin_position());

    // A forged accept with no offer, a forged end reply with no end screen, and a disabled
    // prerequisite's ids are all clean no-ops.
    let mut miner = fixture.game(MINER, &[ADVENTURER, MINER, BRAWLER]);
    assert_packet_changes_nothing(
        &mut miner,
        &character,
        &quest_reply(MINER_FIRST, true),
        100,
        "a forged 49/2 with no offer",
    );
    assert_packet_changes_nothing(
        &mut miner,
        &character,
        &quest_end_reply(MINER_FIRST),
        200,
        "a forged 49/14 with no end screen",
    );
    for disabled in DRAINED_RECORDS {
        assert_packet_changes_nothing(
            &mut miner,
            &character,
            &quest_reply(disabled, true),
            300,
            "a disabled Miner record accept",
        );
    }

    // A wrong NPC (Trixi, whose Rush Order waits on 3100003) offers nothing.
    let trixi = character_at(content_position("snowhill.json", TRIXI));
    let mut now = 400;
    assert_no_offer(&mut miner, &trixi, TRIXI, &mut now, "a wrong NPC");

    // A wrong active job cannot be offered the Miner quest.
    let mut brawler = fixture.game(BRAWLER, &[ADVENTURER, MINER, BRAWLER]);
    let mut now = 500;
    assert_no_offer(&mut brawler, &character, THERIN, &mut now, "a Brawler");

    // An unknown runtime actor and an unknown menu choice are clean no-ops.
    assert_packet_changes_nothing(
        &mut brawler,
        &character,
        &npc_interact(u64::MAX),
        now + 100,
        "an unknown NPC target",
    );
    assert_packet_changes_nothing(
        &mut brawler,
        &character,
        &npc_select(u64::MAX, 1),
        now + 101,
        "an unknown menu choice",
    );

    // Out-of-range NPC interaction is refused without touching the state.
    let far = character_at([10_000.0, 0.0, 10_000.0]);
    let before = saved(&brawler);
    assert!(
        brawler
            .handle_in_world(&npc_interact(THERIN), &far, now + 200)
            .is_ok_and(|out| out.replies.is_empty() && !out.changed),
        "an out-of-range 26/8 is refused"
    );
    assert_eq!(saved(&brawler), before);

    // A turn-in before the gather step is refused.
    offer(&mut miner, &character, THERIN, MINER_FIRST, 600);
    accept(&mut miner, &character, MINER_FIRST, 610);
    assert_packet_changes_nothing(
        &mut miner,
        &character,
        &quest_end_reply(MINER_FIRST),
        700,
        "a turn-in before the gather step",
    );
    // A replayed accept after the quest is already active is inert.
    assert_packet_changes_nothing(
        &mut miner,
        &character,
        &quest_reply(MINER_FIRST, true),
        710,
        "a replayed 49/2",
    );

    // Complete the quest, then replay both replies and try to re-offer it.
    let nodes = copper_nodes();
    let mut now = 800;
    let (guid, position) = nodes[0];
    assert!(gather_copper(&mut miner, &character, guid, position, &mut now).is_some());
    open_end_screen(
        &mut miner,
        &character,
        THERIN,
        MINER_FIRST,
        END_TEXT,
        now + 100,
    );
    finish_quest(&mut miner, &character, MINER_FIRST, MINER, 5, 15, now + 200);
    assert_packet_changes_nothing(
        &mut miner,
        &character,
        &quest_end_reply(MINER_FIRST),
        now + 300,
        "a replayed 49/14 after completion",
    );
    // The completed 3100002 is not re-offered. Since T334 the same Talk offers the next
    // Miner rung (3100003); a second Talk while that offer is open still changes nothing.
    let mut reoffer = now + 400;
    let next = offer(&mut miner, &character, THERIN, ALL_I_GOT, reoffer);
    assert_offer_ids(
        &offer_packet(&next, ALL_I_GOT),
        ALL_I_GOT,
        ALL_I_GOT_INTRO,
        ALL_I_GOT_TITLE,
        ALL_I_GOT_DESCRIPTION,
    );
    reoffer += 100;
    assert_no_offer(
        &mut miner,
        &character,
        THERIN,
        &mut reoffer,
        "a second Talk while the follow-up is offered",
    );
}

#[test]
fn two_miners_progress_independently() {
    const FIRST: u64 = 300_001;
    const SECOND: u64 = 300_002;
    let fixture = Fixture::new();
    let first = character_at_for(FIRST, 0, therin_position());
    let second = character_at_for(SECOND, 0, therin_position());
    let mut a = fixture.game_as(FIRST, MINER, &[ADVENTURER, MINER, BRAWLER]);
    let mut b = fixture.game_as(SECOND, MINER, &[ADVENTURER, MINER, BRAWLER]);

    // A's offer is its own and B has no pending offer yet. B's out-of-range click is a
    // refusal that consumes neither A's pending offer nor either session's state.
    offer(&mut a, &first, THERIN, MINER_FIRST, 1_000);
    assert_eq!(pending_offer(&a), Some(MINER_FIRST));
    assert_eq!(pending_offer(&b), None, "A's offer created B's offer");
    let far = character_at_for(SECOND, 0, [10_000.0, 0.0, 10_000.0]);
    let a_range_before = saved(&a);
    let b_range_before = saved(&b);
    assert!(
        b.handle_in_world(&npc_interact(THERIN), &far, 1_005)
            .is_ok_and(|out| out.replies.is_empty() && !out.changed),
        "an out-of-range 26/8 is refused"
    );
    assert_eq!(
        pending_offer(&a),
        Some(MINER_FIRST),
        "B's refusal consumed A's pending offer"
    );
    assert_eq!(saved(&a), a_range_before, "B's refusal changed A's state");
    assert_eq!(saved(&b), b_range_before, "B's refusal changed B's state");
    offer(&mut b, &second, THERIN, MINER_FIRST, 1_010);
    accept(&mut a, &first, MINER_FIRST, 1_100);
    accept(&mut b, &second, MINER_FIRST, 1_100);

    // A gathers; B's counter stays untouched.
    let nodes = copper_nodes();
    let mut now = 2_000;
    let (guid, position) = nodes[0];
    assert!(gather_copper(&mut a, &first, guid, position, &mut now).is_some());
    assert_eq!(counts(&a, MINER_FIRST), [1, 0]);
    assert_eq!(counts(&b, MINER_FIRST), [0, 0]);

    // B's gather and end screen do not open A's pending turn-in.
    assert!(gather_copper(&mut b, &second, guid, position, &mut now).is_some());
    let end_b = open_end_screen(&mut b, &second, THERIN, MINER_FIRST, END_TEXT, now + 100);
    assert_eq!(end_screen_reward(&end_b), (5, 15));
    assert_eq!(pending_turnin(&b), Some(MINER_FIRST));
    assert_eq!(pending_turnin(&a), None, "B's Talk touched A's quest");

    // A's forged 49/14 has no server-opened end screen: it is refused and changes neither
    // session, while B's held end screen still pays below.
    let a_before = saved(&a);
    let b_pending_before = saved(&b);
    assert_packet_changes_nothing(
        &mut a,
        &first,
        &quest_end_reply(MINER_FIRST),
        now + 150,
        "A's forged 49/14 while B holds its end screen",
    );
    assert_eq!(
        pending_turnin(&b),
        Some(MINER_FIRST),
        "A's refusal consumed B's held end screen"
    );
    assert_eq!(saved(&a), a_before);
    assert_eq!(saved(&b), b_pending_before);

    // B's held turn-in survives a reconnect saved under B's own character id; the fixture
    // default is a different character, so a reconnect bound to the wrong id would fail.
    b = fixture.save_and_reload(&b, &second);
    assert_eq!(
        pending_turnin(&b),
        Some(MINER_FIRST),
        "B's held end screen survives the reconnect"
    );
    assert_eq!(
        saved(&b),
        b_pending_before,
        "the reconnect changed B's save"
    );

    finish_quest(&mut b, &second, MINER_FIRST, MINER, 5, 15, now + 200);
    let end_a = open_end_screen(&mut a, &first, THERIN, MINER_FIRST, END_TEXT, now + 300);
    assert_eq!(end_screen_reward(&end_a), (5, 15));
    finish_quest(&mut a, &first, MINER_FIRST, MINER, 5, 15, now + 400);

    assert_eq!(coins(&a), 5, "each Miner is paid its own quest");
    assert_eq!(coins(&b), 5);
    assert_eq!(
        profile_xp(&a, MINER),
        MINING_XP_PER_WIN + 15,
        "A's ledger is its own"
    );
    assert_eq!(
        profile_xp(&b, MINER),
        MINING_XP_PER_WIN + 15,
        "B's ledger is its own"
    );
    assert_eq!(quest_status(&a, MINER_FIRST), QuestStatus::Completed);
    assert_eq!(quest_status(&b, MINER_FIRST), QuestStatus::Completed);
}

#[test]
fn an_active_encounter_or_movie_refuses_the_node_before_begin() {
    let fixture = Fixture::new();
    let mut game = fixture.game(MINER, &[ADVENTURER, MINER, BRAWLER]);
    let character = character_at(therin_position());
    offer(&mut game, &character, THERIN, MINER_FIRST, 1_000);
    accept(&mut game, &character, MINER_FIRST, 1_100);

    let nodes = copper_nodes();
    assert!(nodes.len() >= 2);
    let mut now = 2_000;
    game.tick(&character, now).expect("node tick");
    send(&mut game, &character, &npc_interact(nodes[0].0), now + 1);
    let instance = active_mining(&game).expect("the first node starts the Mining18 movie");

    // The running movie is an active occupation: a second node in range is refused before
    // any run, count or save change.
    let before = saved(&game);
    let refused = send(&mut game, &character, &npc_interact(nodes[1].0), now + 2);
    assert!(!refused.changed, "an active movie refuses a second node");
    assert!(
        refused
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[15, 0, 3, 0])),
        "the busy refusal is a chat message"
    );
    assert_eq!(
        active_mining(&game),
        Some(instance),
        "no second mining run started"
    );
    assert_eq!(counts(&game, MINER_FIRST), [0, 0]);
    assert_eq!(saved(&game), before, "the refused node changed the save");

    // The NPC router's shared busy gate closes Therin's menu too.
    let npc = send(&mut game, &character, &npc_interact(THERIN), now + 3);
    assert!(!npc.changed);
    assert!(
        !npc.replies
            .iter()
            .any(|packet| packet.starts_with(&[26, 0, 9, 0])),
        "no NPC menu opens during the movie"
    );
    assert_eq!(saved(&game), before);

    // Quit is an exit path that frees the player and leaves the node in place.
    send(
        &mut game,
        &character,
        &type8_frame(QUIT_FRAME_SUB, instance),
        now + 4,
    );
    now += 10;
    game.tick(&character, now).expect("quit tick");
    assert!(active_mining(&game).is_none(), "quit cleared the run");
    send(&mut game, &character, &npc_interact(nodes[0].0), now + 1);
    assert!(
        active_mining(&game).is_some(),
        "the node starts a fresh run after the quit"
    );
}

#[test]
fn therin_is_drawable_and_the_quest_content_resolves() {
    let content = content_dir();
    let spawns = read_json(&content.join("npc_spawns_extra.json"));
    let rows = spawns["records"].as_array().unwrap();
    let therin: Vec<&Value> = rows
        .iter()
        .filter(|row| row["spawn_guid"].as_u64() == Some(THERIN))
        .collect();
    assert_eq!(therin.len(), 1, "exactly one Therin spawn");
    assert_eq!(
        rows.iter()
            .filter(|row| row["content_key"].as_u64() == Some(THERIN))
            .count(),
        1,
        "no second row aliases the Therin content key"
    );
    let mut guids = std::collections::HashSet::new();
    for row in rows {
        assert!(
            guids.insert(row["spawn_guid"].as_u64().unwrap()),
            "duplicate spawn_guid"
        );
    }
    let therin_row = therin[0];
    assert_eq!(therin_row["name"], json!("Therin"));
    assert_eq!(therin_row["name_string_id"], json!(185), "Global.Text.185");
    assert_eq!(therin_row["model_id"], json!(381));
    assert_eq!(therin_row["model_file_name"], json!("dwarf_therin_m.adr"));
    assert_eq!(therin_row["legacy_npc_id"], json!(1201));
    assert_eq!(therin_row["heading"], json!(0.0));
    let position: [f64; 3] = serde_json::from_value(therin_row["position"].clone()).unwrap();
    assert!(position.iter().all(|value| value.is_finite()));
    for source in ["legacy-data", "client", "reconstructed"] {
        assert!(
            has_source(therin_row, source),
            "spawn lacks {source} provenance"
        );
    }

    // The Snowhill row is unchanged (a POI anchor, not a duplicate drawable) and the spawn
    // reuses its position exactly.
    let snowhill = read_json(&content.join("npcs/snowhill.json"));
    let therin_npc = row(&snowhill["records"].as_array().unwrap(), THERIN);
    assert_eq!(therin_npc["placement"], json!("poi_anchor"));
    assert_eq!(therin_npc["name"], json!("Therin"));
    assert_eq!(therin_npc["name_string_id"], json!(943_830_538));
    assert_eq!(therin_npc["position"], therin_row["position"]);

    // The production per-session visibility batch draws the one shared actor for two
    // independent nearby sessions. Static NPCs are not server-broadcast: each session's
    // gateway calls this same method for its own character (`gateway.rs` Event::Loaded and
    // movement), so two Gameplay sessions are the production input. The osfr-gateway
    // session loop and the shared-field NPC fanout are not exercised by this content test.
    const FIRST_VIEWER: u64 = 300_011;
    const SECOND_VIEWER: u64 = 300_012;
    let fixture = Fixture::new();
    let drawn: Vec<NpcSpawn> = [FIRST_VIEWER, SECOND_VIEWER]
        .into_iter()
        .map(|viewer| drawn_npc(&fixture, viewer, therin_position(), THERIN))
        .collect();
    for npc in &drawn {
        assert_eq!(npc.actor_id, THERIN);
        assert_eq!(npc.name_string_id, 185);
        assert_eq!(npc.model_id, 381);
        for axis in 0..3 {
            assert!((npc.position[axis] - position[axis] as f32).abs() < 0.01);
        }
    }

    // The quest record carries the recovered text, job gate, ordered objectives and reward.
    let quests = read_json(&content.join("quests/miner.json"));
    let quest = row(
        &quests["records"].as_array().unwrap(),
        u64::from(MINER_FIRST),
    );
    assert_eq!(quest["enabled"], json!(true));
    assert_eq!(quest["job"], json!(MINER));
    assert_eq!(quest["giver_npc_id"], json!(THERIN));
    assert_eq!(quest["title_string_id"], json!(TITLE));
    assert_eq!(quest["description_string_id"], json!(DESCRIPTION));
    assert_eq!(quest["end_text_string_id"], json!(END_TEXT));
    assert!(
        quest.get("intro_text_string_id").is_none(),
        "23216 is empty; no intro id is invented"
    );
    assert_eq!(quest["prerequisite_quest_ids"], json!([]));
    let objectives = quest["objectives"].as_array().unwrap();
    assert_eq!(objectives.len(), 2, "gather then return to Therin");
    assert_eq!(objectives[0]["kind"], json!("gather_resource"));
    assert_eq!(objectives[0]["resource_id"], json!(COPPER_ORE));
    assert_eq!(objectives[0]["quantity"], json!(1));
    assert_eq!(objectives[1]["kind"], json!("talk_to_npc"));
    assert_eq!(objectives[1]["npc_id"], json!(THERIN));
    for objective in objectives {
        assert!(has_source(objective, "client"), "objective provenance");
        assert!(has_source(objective, "reconstructed"));
    }
    let rewards = quest["rewards"].as_array().unwrap();
    assert_eq!(rewards.len(), 1);
    assert_eq!(rewards[0]["coins"], json!(5));
    assert_eq!(rewards[0]["items"], json!([]));
    assert_eq!(rewards[0]["job_xp"], json!({"14": 15}));
    assert!(has_source(&rewards[0], "reconstructed"));
    assert!(has_source(quest, "reconstructed"));
    assert!(
        quest["source"]
            .as_array()
            .unwrap()
            .iter()
            .any(|tag| tag["evidence"]
                .as_str()
                .is_some_and(|text| text.starts_with("(R) T300:"))),
        "the record carries its T300 decisions"
    );

    // The neighbouring records keep their recovered titles. T334 enabled 3100003 and T411
    // enables 3100004 ("Rush Order"); T542 enables 3100001 (Become a Miner) and 3100005
    // ("Gem Appraisal") stays disabled.
    for (id, title, enabled) in [
        (3_100_001_u32, 139_748_u32, true),
        (RUSH_ORDER, RUSH_ORDER_TITLE, true),
        (3_100_005, 76_770, false),
    ] {
        let record = row(&quests["records"].as_array().unwrap(), u64::from(id));
        assert_eq!(
            record["enabled"],
            json!(enabled),
            "quest {id} enabled state"
        );
        assert_eq!(record["title_string_id"], json!(title));
    }

    // The gather target is a live Snowhill copper pool beside Therin, and the shipped
    // Mining18 board pays the same Copper Ore definition the objective names.
    let mining = read_json(&content.join("mining_nodes.json"));
    let pool = mining["pools"]
        .as_array()
        .unwrap()
        .iter()
        .find(|pool| pool["key"] == "snowhill-copper")
        .expect("the snowhill-copper pool");
    assert_eq!(pool["zone_definition_id"], json!(13));
    assert_eq!(pool["node_type"], json!("copper-vein"));
    let node_type = mining["node_types"]
        .as_array()
        .unwrap()
        .iter()
        .find(|kind| kind["key"] == "copper-vein")
        .expect("the copper-vein node type");
    assert_eq!(node_type["interact_range"], json!(12));
    let copper_spawns: Vec<&Value> = mining["spawns"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|spawn| spawn["pool_key"] == "snowhill-copper")
        .collect();
    assert_eq!(copper_spawns.len(), 5, "five reconstructed copper veins");
    for spawn in &copper_spawns {
        let node: [f64; 3] = serde_json::from_value(spawn["position"].clone()).unwrap();
        let distance = node
            .iter()
            .zip(position)
            .map(|(a, b)| (a - b).powi(2))
            .sum::<f64>()
            .sqrt();
        assert!(
            distance < 20.0,
            "a copper vein is not beside Therin: {distance} m"
        );
    }
    let board = read_json(&content.join(mining::CONTENT_FILE));
    assert_eq!(board["game_id"], json!(mining::MINING_GAME_ID));
    assert_eq!(board["profile_id"], json!(MINER));
    assert!(
        board["spec"]["outputs"]
            .as_array()
            .unwrap()
            .iter()
            .any(|output| output["definition_id"].as_u64() == Some(u64::from(COPPER_ORE))),
        "the shipped Mining18 board emits Copper Ore 2739"
    );
}

#[test]
fn the_reward_is_paid_once_and_grants_no_free_item() {
    let fixture = Fixture::new();
    let mut game = fixture.game(MINER, &[ADVENTURER, MINER, BRAWLER]);
    let character = character_at(therin_position());
    offer(&mut game, &character, THERIN, MINER_FIRST, 1_000);
    accept(&mut game, &character, MINER_FIRST, 1_100);
    let before_mining = item_ledger(&game);
    assert_eq!(coins(&game), 0);

    let nodes = copper_nodes();
    let mut now = 2_000;
    let (guid, position) = nodes[0];
    assert!(gather_copper(&mut game, &character, guid, position, &mut now).is_some());
    let after_mining = item_ledger(&game);
    let added: Vec<_> = after_mining
        .iter()
        .copied()
        .filter(|entry| !before_mining.contains(entry))
        .collect();
    assert_eq!(
        added,
        vec![(COPPER_ORE as i32, 1)],
        "the only new item is the player's own mined Copper Ore"
    );
    assert_eq!(coins(&game), 0, "the node paid no coins");

    open_end_screen(
        &mut game,
        &character,
        THERIN,
        MINER_FIRST,
        END_TEXT,
        now + 100,
    );
    finish_quest(&mut game, &character, MINER_FIRST, MINER, 5, 15, now + 200);
    assert_eq!(
        item_ledger(&game),
        after_mining,
        "the quest granted no item and did not consume the ore"
    );
    assert_eq!(coins(&game), 5);
    assert_eq!(profile_xp(&game, MINER), MINING_XP_PER_WIN + 15);

    // A replayed turn-in and a later Talk cannot pay or re-grant anything. The later Talk
    // offers the T334 follow-up (3100003), which proves the completed 3100002 is not
    // re-offered; the offer itself pays nothing.
    let finished_ledger = item_ledger(&game);
    assert_packet_changes_nothing(
        &mut game,
        &character,
        &quest_end_reply(MINER_FIRST),
        now + 300,
        "a replayed 49/14",
    );
    let mut reoffer = now + 400;
    let next = offer(&mut game, &character, THERIN, ALL_I_GOT, reoffer);
    assert_offer_ids(
        &offer_packet(&next, ALL_I_GOT),
        ALL_I_GOT,
        ALL_I_GOT_INTRO,
        ALL_I_GOT_TITLE,
        ALL_I_GOT_DESCRIPTION,
    );
    reoffer += 100;
    assert_no_offer(
        &mut game,
        &character,
        THERIN,
        &mut reoffer,
        "the completed quest",
    );
    assert_eq!(quest_status(&game, MINER_FIRST), QuestStatus::Completed);
    assert_eq!(coins(&game), 5);
    assert_eq!(profile_xp(&game, MINER), MINING_XP_PER_WIN + 15);
    assert_eq!(item_ledger(&game), finished_ledger);
}

// ---------------------------------------------------------------------------------------------
// T334: "All I Got", the Trixi mining-gear referral (3100003).
// ---------------------------------------------------------------------------------------------

/// T334: after T300's practice test, the same Therin offers the record whose own client
/// description 76088 sends the Miner to Trixi. The job and prerequisite gates are unchanged.
#[test]
fn all_i_got_is_offered_after_the_first_ore() {
    let fixture = Fixture::new();
    let character = character_at(therin_position());

    // Complete 3100002 once in a donor session: it is 3100003's prerequisite.
    let mut donor = fixture.game(MINER, &[ADVENTURER, MINER]);
    let now = complete_the_first_ore(&mut donor, &character, 1_000);
    let completed = donor.state();

    // An active Miner who completed the practice test is offered the Trixi referral.
    let mut miner = fixture.game_with_state(completed.clone());
    let offered = offer(&mut miner, &character, THERIN, ALL_I_GOT, now);
    assert_offer_ids(
        &offer_packet(&offered, ALL_I_GOT),
        ALL_I_GOT,
        ALL_I_GOT_INTRO,
        ALL_I_GOT_TITLE,
        ALL_I_GOT_DESCRIPTION,
    );
    assert_eq!(quest_status(&miner, ALL_I_GOT), QuestStatus::Offered);

    // A second Talk while the offer is open neither re-offers nor consumes the open menu.
    let mut later = now + 100;
    assert_no_offer(
        &mut miner,
        &character,
        THERIN,
        &mut later,
        "a second Talk while 3100003 is offered",
    );

    // A fresh Miner without 3100002 is offered only the practice test, never 3100003.
    let mut fresh = fixture.game(MINER, &[ADVENTURER, MINER]);
    offer(&mut fresh, &character, THERIN, MINER_FIRST, now + 200);

    // A completed practice test on a character without the Miner profile is refused.
    let mut locked_state = completed.clone();
    {
        let profiles = locked_state
            .profiles
            .as_mut()
            .expect("the fixture saves profiles");
        profiles.active = ADVENTURER;
        profiles.unlocked = [ADVENTURER].into_iter().collect();
        profiles.experience.remove(&MINER);
    }
    let mut locked = fixture.game_with_state(locked_state);
    let mut later = now + 300;
    assert_no_offer(&mut locked, &character, THERIN, &mut later, "a non-Miner");

    // An unlocked, ranked Miner who is not the active job is refused too: the engine's job
    // levels are built from the saved active profile only.
    let mut inactive_state = completed.clone();
    {
        let profiles = inactive_state
            .profiles
            .as_mut()
            .expect("the fixture saves profiles");
        profiles.active = ADVENTURER;
        profiles.unlocked = [ADVENTURER, MINER].into_iter().collect();
        profiles.experience.insert(MINER, 200);
    }
    let mut inactive = fixture.game_with_state(inactive_state);
    let mut later = now + 400;
    assert_no_offer(
        &mut inactive,
        &character,
        THERIN,
        &mut later,
        "an unlocked-but-inactive Miner",
    );
}

/// T334: the Talk to Trixi completes the one objective, opens the held 49/13 and the 49/14
/// pays exactly 5 coins + 15 Miner XP with no item. Both replayed replies are inert.
#[test]
fn talking_to_trixi_completes_and_pays_once() {
    let fixture = Fixture::new();
    let mut game = fixture.game(MINER, &[ADVENTURER, MINER]);
    let character = character_at(therin_position());
    let now = complete_the_first_ore(&mut game, &character, 1_000);
    offer(&mut game, &character, THERIN, ALL_I_GOT, now);
    accept(&mut game, &character, ALL_I_GOT, now + 100);
    assert_eq!(counts(&game, ALL_I_GOT), [0]);

    let at_trixi = character_at(trixi_position());
    let items_before = item_ledger(&game);
    let coins_before = coins(&game);
    let xp_before = profile_xp(&game, MINER);
    let end = open_end_screen(
        &mut game,
        &at_trixi,
        TRIXI,
        ALL_I_GOT,
        ALL_I_GOT_END_TEXT,
        now + 200,
    );
    assert_eq!(end_screen_reward(&end), (5, 15), "49/13 reward bundle");
    assert_eq!(counts(&game, ALL_I_GOT), [1], "the Talk completed the step");
    assert_eq!(pending_turnin(&game), Some(ALL_I_GOT));
    assert_eq!(coins(&game), coins_before, "the end screen pays nothing");

    finish_quest(&mut game, &at_trixi, ALL_I_GOT, MINER, 5, 15, now + 300);
    assert_eq!(quest_status(&game, ALL_I_GOT), QuestStatus::Completed);
    assert_eq!(coins(&game), coins_before + 5, "exactly 5 coins");
    assert_eq!(
        profile_xp(&game, MINER),
        xp_before + 15,
        "exactly 15 Miner XP"
    );
    assert_eq!(
        item_ledger(&game),
        items_before,
        "the referral grants no item and consumes none"
    );

    // A replayed end reply and a replayed accept pay nothing.
    assert_packet_changes_nothing(
        &mut game,
        &at_trixi,
        &quest_end_reply(ALL_I_GOT),
        now + 400,
        "a replayed 49/14",
    );
    assert_packet_changes_nothing(
        &mut game,
        &at_trixi,
        &quest_reply(ALL_I_GOT, true),
        now + 500,
        "a replayed 49/2",
    );
    assert_eq!(coins(&game), coins_before + 5);
    assert_eq!(profile_xp(&game, MINER), xp_before + 15);
}

/// T334: the explicit `{"14": 15}` map pays profile 14 even when the active job changed
/// between accept and turn-in.
#[test]
fn the_reward_goes_to_profile_14_after_a_job_switch() {
    let fixture = Fixture::new();
    let mut game = fixture.game(MINER, &[ADVENTURER, MINER, BRAWLER]);
    let character = character_at(therin_position());
    let now = complete_the_first_ore(&mut game, &character, 1_000);
    offer(&mut game, &character, THERIN, ALL_I_GOT, now);
    accept(&mut game, &character, ALL_I_GOT, now + 100);

    // Switch away before the Trixi Talk: the reward map names Miner.
    let switched = send(&mut game, &character, &select_job(BRAWLER), now + 200);
    assert!(switched.changed, "the Brawler outfit is selectable");
    assert_eq!(active_job(&game), BRAWLER);
    let brawler_before = profile_xp(&game, BRAWLER);
    let miner_before = profile_xp(&game, MINER);

    let at_trixi = character_at(trixi_position());
    let end = open_end_screen(
        &mut game,
        &at_trixi,
        TRIXI,
        ALL_I_GOT,
        ALL_I_GOT_END_TEXT,
        now + 300,
    );
    assert_eq!(end_screen_reward(&end), (5, 15));
    finish_quest(&mut game, &at_trixi, ALL_I_GOT, MINER, 5, 15, now + 400);
    assert_eq!(
        profile_xp(&game, MINER),
        miner_before + 15,
        "the quest pays profile 14"
    );
    assert_eq!(
        profile_xp(&game, BRAWLER),
        brawler_before,
        "the active Brawler profile receives none of the quest XP"
    );
}

/// T334: a wrong Talk target, an out-of-range click, forged replies, a wrong quest id and
/// replays all leave the whole `GameplayState` and the real save bytes equal.
#[test]
fn refusals_wrong_target_and_replay_change_nothing() {
    let fixture = Fixture::new();
    let character = character_at(therin_position());
    let mut game = fixture.game(MINER, &[ADVENTURER, MINER]);
    let mut now = 1_000;

    // Forged replies before 3100003 is offered change nothing.
    assert_packet_changes_nothing(
        &mut game,
        &character,
        &quest_reply(ALL_I_GOT, true),
        now,
        "a forged 49/2 before the offer",
    );
    assert_packet_changes_nothing(
        &mut game,
        &character,
        &quest_end_reply(ALL_I_GOT),
        now + 10,
        "a forged 49/14 with no end screen",
    );

    now = complete_the_first_ore(&mut game, &character, now + 100);
    offer(&mut game, &character, THERIN, ALL_I_GOT, now);
    accept(&mut game, &character, ALL_I_GOT, now + 10);

    // A wrong NPC (Smitty) neither advances the Trixi step nor offers anything to a Miner.
    let at_smitty = character_at(content_position("snowhill.json", SMITTY));
    let mut later = now + 100;
    assert_no_offer(
        &mut game,
        &at_smitty,
        SMITTY,
        &mut later,
        "a wrong Talk target (Smitty)",
    );
    assert_eq!(
        counts(&game, ALL_I_GOT),
        [0],
        "Smitty advanced the Trixi step"
    );

    // Out-of-range 26/8 is an `Err` at the session boundary and changes nothing.
    let far = character_at([10_000.0, 0.0, 10_000.0]);
    let before = saved(&game);
    let before_save = fixture.saved_bytes(&game, &character);
    assert!(
        game.handle_in_world(&npc_interact(TRIXI), &far, now + 200)
            .is_ok_and(|out| out.replies.is_empty() && !out.changed),
        "an out-of-range 26/8 is refused"
    );
    assert_eq!(saved(&game), before);
    assert_eq!(fixture.saved_bytes(&game, &character), before_save);

    // A replayed accept while the quest is already active is inert.
    assert_packet_changes_nothing(
        &mut game,
        &character,
        &quest_reply(ALL_I_GOT, true),
        now + 300,
        "a replayed 49/2",
    );

    // The Trixi Talk holds the end screen; a 49/14 for another quest cannot settle it.
    let at_trixi = character_at(trixi_position());
    open_end_screen(
        &mut game,
        &at_trixi,
        TRIXI,
        ALL_I_GOT,
        ALL_I_GOT_END_TEXT,
        now + 400,
    );
    let before = saved(&game);
    let before_save = fixture.saved_bytes(&game, &character);
    assert_packet_changes_nothing(
        &mut game,
        &at_trixi,
        &quest_end_reply(MINER_FIRST),
        now + 500,
        "a 49/14 for another quest",
    );
    assert_eq!(
        pending_turnin(&game),
        Some(ALL_I_GOT),
        "the wrong quest consumed the held end screen"
    );
    assert_eq!(saved(&game), before);
    assert_eq!(fixture.saved_bytes(&game, &character), before_save);

    finish_quest(&mut game, &at_trixi, ALL_I_GOT, MINER, 5, 15, now + 600);
    let finished = saved(&game);
    let finished_save = fixture.saved_bytes(&game, &character);
    assert_packet_changes_nothing(
        &mut game,
        &at_trixi,
        &quest_end_reply(ALL_I_GOT),
        now + 700,
        "a replayed 49/14 after completion",
    );
    assert_packet_changes_nothing(
        &mut game,
        &at_trixi,
        &quest_reply(ALL_I_GOT, true),
        now + 800,
        "a replayed 49/2 after completion",
    );
    assert_eq!(saved(&game), finished);
    assert_eq!(fixture.saved_bytes(&game, &character), finished_save);
}

/// T334: the accepted state, the Trixi step counter and the held end screen live in the
/// character save and survive the production `game_store` round trip.
#[test]
fn progress_survives_a_reload() {
    let fixture = Fixture::new();
    let mut game = fixture.game(MINER, &[ADVENTURER, MINER]);
    let character = character_at(therin_position());
    let now = complete_the_first_ore(&mut game, &character, 1_000);
    offer(&mut game, &character, THERIN, ALL_I_GOT, now);
    accept(&mut game, &character, ALL_I_GOT, now + 100);

    // Reload after the accept: the active state survives and nothing is paid.
    let coins_before = coins(&game);
    let xp_before = profile_xp(&game, MINER);
    let mut game = fixture.save_and_reload(&game, &character);
    assert_eq!(quest_status(&game, ALL_I_GOT), QuestStatus::Active);
    assert_eq!(counts(&game, ALL_I_GOT), [0]);
    assert_eq!(pending_turnin(&game), None);
    assert_eq!(coins(&game), coins_before, "the reload paid nothing");
    assert_eq!(profile_xp(&game, MINER), xp_before);

    // The Trixi Talk, then a reload: the counter and the held screen survive, and the relog
    // batch re-sends the 49/13 beside Trixi's AddNpc.
    let at_trixi = character_at(trixi_position());
    let end = open_end_screen(
        &mut game,
        &at_trixi,
        TRIXI,
        ALL_I_GOT,
        ALL_I_GOT_END_TEXT,
        now + 200,
    );
    assert_eq!(end_screen_reward(&end), (5, 15));
    assert_eq!(counts(&game, ALL_I_GOT), [1]);
    let mut game = fixture.save_and_reload(&game, &at_trixi);
    assert_eq!(counts(&game, ALL_I_GOT), [1], "the Talk counter survives");
    assert_eq!(
        pending_turnin(&game),
        Some(ALL_I_GOT),
        "the held end screen survives"
    );
    let relog = game
        .update_npc_visibility(&at_trixi)
        .expect("the relog visibility batch builds");
    assert!(
        relog
            .iter()
            .any(|packet| has_family(packet, 49, 13, Some(ALL_I_GOT))),
        "the relog batch re-sends the held end screen"
    );

    // The saved turn-in pays once; a later reload keeps the completed history.
    finish_quest(&mut game, &at_trixi, ALL_I_GOT, MINER, 5, 15, now + 300);
    let mut game = fixture.save_and_reload(&game, &at_trixi);
    assert_eq!(quest_status(&game, ALL_I_GOT), QuestStatus::Completed);
    assert_eq!(coins(&game), coins_before + 5);
    assert_eq!(profile_xp(&game, MINER), xp_before + 15);
    assert_packet_changes_nothing(
        &mut game,
        &at_trixi,
        &quest_end_reply(ALL_I_GOT),
        now + 400,
        "a 49/14 replayed after the reconnect",
    );
}

/// T334: two Miners each own their offer, step counter and reward; one player's refusal
/// changes neither the other session nor the server loop.
#[test]
fn two_miners_progress_independently_on_all_i_got() {
    const FIRST: u64 = 300_031;
    const SECOND: u64 = 300_032;
    let fixture = Fixture::new();
    let first = character_at_for(FIRST, 0, therin_position());
    let second = character_at_for(SECOND, 0, therin_position());
    let mut a = fixture.game_as(FIRST, MINER, &[ADVENTURER, MINER]);
    let mut b = fixture.game_as(SECOND, MINER, &[ADVENTURER, MINER]);
    let mut now = 1_000;
    now = complete_the_first_ore(&mut a, &first, now);
    now = complete_the_first_ore(&mut b, &second, now);

    // A's offer is its own; B keeps a free menu. B's out-of-range click is a refusal that
    // consumes neither A's pending offer nor B's state.
    offer(&mut a, &first, THERIN, ALL_I_GOT, now);
    assert_eq!(pending_offer(&a), Some(ALL_I_GOT));
    assert_eq!(pending_offer(&b), None, "A's offer consumed B's menu");
    let far = character_at_for(SECOND, 0, [10_000.0, 0.0, 10_000.0]);
    let a_before = saved(&a);
    let b_before = saved(&b);
    assert!(
        b.handle_in_world(&npc_interact(THERIN), &far, now + 100)
            .is_ok_and(|out| out.replies.is_empty() && !out.changed),
        "an out-of-range 26/8 is refused"
    );
    assert_eq!(
        pending_offer(&a),
        Some(ALL_I_GOT),
        "B's refusal consumed A's pending offer"
    );
    assert_eq!(saved(&a), a_before);
    assert_eq!(saved(&b), b_before, "B's refusal changed B's state");

    offer(&mut b, &second, THERIN, ALL_I_GOT, now + 200);
    accept(&mut a, &first, ALL_I_GOT, now + 300);
    accept(&mut b, &second, ALL_I_GOT, now + 300);
    assert_eq!(counts(&a, ALL_I_GOT), [0]);
    assert_eq!(counts(&b, ALL_I_GOT), [0]);

    // A's Trixi Talk and held end screen are its own; B's step counter stays at zero.
    let at_trixi_a = character_at_for(FIRST, 0, trixi_position());
    let at_trixi_b = character_at_for(SECOND, 0, trixi_position());
    let end_a = open_end_screen(
        &mut a,
        &at_trixi_a,
        TRIXI,
        ALL_I_GOT,
        ALL_I_GOT_END_TEXT,
        now + 400,
    );
    assert_eq!(end_screen_reward(&end_a), (5, 15));
    assert_eq!(counts(&a, ALL_I_GOT), [1]);
    assert_eq!(counts(&b, ALL_I_GOT), [0], "A's Talk touched B's quest");
    assert_eq!(pending_turnin(&a), Some(ALL_I_GOT));
    assert_eq!(pending_turnin(&b), None);

    // A's forged replay cannot touch B's progress while B is still at the Talk step.
    finish_quest(&mut a, &at_trixi_a, ALL_I_GOT, MINER, 5, 15, now + 500);
    let b_progress = saved(&b);
    assert_packet_changes_nothing(
        &mut a,
        &at_trixi_a,
        &quest_end_reply(ALL_I_GOT),
        now + 600,
        "A's replayed 49/14 while B is active",
    );
    assert_eq!(saved(&b), b_progress);

    // B's own Trixi Talk pays its own reward.
    let end_b = open_end_screen(
        &mut b,
        &at_trixi_b,
        TRIXI,
        ALL_I_GOT,
        ALL_I_GOT_END_TEXT,
        now + 700,
    );
    assert_eq!(end_screen_reward(&end_b), (5, 15));
    finish_quest(&mut b, &at_trixi_b, ALL_I_GOT, MINER, 5, 15, now + 800);
    assert_eq!(quest_status(&a, ALL_I_GOT), QuestStatus::Completed);
    assert_eq!(quest_status(&b, ALL_I_GOT), QuestStatus::Completed);
    // Each also finished T300's first Miner quest, so each wallet holds both 5-coin rungs.
    assert_eq!(coins(&a), 10);
    assert_eq!(coins(&b), 10);
    assert_eq!(profile_xp(&a, MINER), MINING_XP_PER_WIN + 30);
    assert_eq!(profile_xp(&b, MINER), MINING_XP_PER_WIN + 30);
}

/// T334: the record and both NPCs resolve through the production content parser and the
/// client 35/2 AddNpc reader: Therin (extra spawn) offers, Trixi (legacy_data) is the target.
#[test]
fn the_record_and_both_npcs_resolve() {
    let content = content_dir();
    let quests = read_json(&content.join("quests/miner.json"));
    let quest = row(&quests["records"].as_array().unwrap(), u64::from(ALL_I_GOT));
    assert_eq!(quest["enabled"], json!(true));
    assert_eq!(quest["job"], json!(MINER));
    assert_eq!(quest["giver_npc_id"], json!(THERIN));
    assert_eq!(quest["prerequisite_quest_ids"], json!([MINER_FIRST]));
    assert_eq!(quest["title_string_id"], json!(ALL_I_GOT_TITLE));
    assert_eq!(quest["description_string_id"], json!(ALL_I_GOT_DESCRIPTION));
    assert_eq!(quest["intro_text_string_id"], json!(ALL_I_GOT_INTRO));
    assert_eq!(quest["end_text_string_id"], json!(ALL_I_GOT_END_TEXT));
    let objectives = quest["objectives"].as_array().unwrap();
    assert_eq!(objectives.len(), 1, "one ordered Talk to Trixi");
    assert_eq!(objectives[0]["kind"], json!("talk_to_npc"));
    assert_eq!(objectives[0]["npc_id"], json!(TRIXI));
    assert!(has_source(&objectives[0], "client"));
    let rewards = quest["rewards"].as_array().unwrap();
    assert_eq!(rewards.len(), 1);
    assert_eq!(rewards[0]["coins"], json!(5));
    assert_eq!(rewards[0]["items"], json!([]));
    assert_eq!(rewards[0]["job_xp"], json!({"14": 15}));
    assert!(has_source(&rewards[0], "reconstructed"));
    assert!(has_source(quest, "reconstructed"));
    assert!(
        quest["source"]
            .as_array()
            .unwrap()
            .iter()
            .any(|tag| tag["evidence"]
                .as_str()
                .is_some_and(|text| text.starts_with("(R) T334:"))),
        "the record carries its T334 decisions"
    );
    assert!(
        !quest["source"]
            .as_array()
            .unwrap()
            .iter()
            .any(|tag| tag["evidence"]
                .as_str()
                .is_some_and(|text| text.starts_with("(R) W110:"))),
        "the stale W110 unspawned-giver tag is gone"
    );
    assert!(
        quest["source"].as_array().unwrap().iter().any(|tag| {
            tag["kind"].as_str() == Some("client")
                && tag["evidence"]
                    .as_str()
                    .is_some_and(|text| text.contains("Visit Trixi in the Singing Crystal Mines"))
        }),
        "the client description 76088 names the Trixi visit"
    );

    let snowhill = read_json(&content.join("npcs/snowhill.json"));
    let records = snowhill["records"].as_array().unwrap();
    let trixi = row(records, TRIXI);
    assert_eq!(trixi["name"], json!("Trixi"));
    assert_eq!(trixi["placement"], json!("legacy_data"));
    assert_eq!(trixi["model_id"], json!(35));
    let position: [f64; 3] = serde_json::from_value(trixi["position"].clone()).unwrap();
    assert!((position[0] - 478.2027).abs() < 0.001);
    assert!((position[1] - 47.11174).abs() < 0.001);
    assert!((position[2] - 237.3823).abs() < 0.001);

    // Trixi's only live quest is T411's Rush Order, which waits on 3100003 itself, so the
    // Talk that completes 3100003 offers no second quest. The others stay disabled.
    for quest_id in trixi["quest_ids_given"].as_array().unwrap() {
        let id = quest_id.as_u64().unwrap();
        let file = if id == 3_020_006 {
            "quests/blacksmith.json"
        } else {
            "quests/miner.json"
        };
        let document = read_json(&content.join(file));
        let record = row(&document["records"].as_array().unwrap(), id);
        assert_eq!(
            record["enabled"],
            json!(id == u64::from(RUSH_ORDER)),
            "quest {id} enabled state"
        );
    }

    // Both NPCs draw through the production 35/2 AddNpc parser for a nearby session. Therin
    // is T300's extra spawn (Global.Text.185, model 381); Trixi is a legacy_data regional row.
    const THERIN_VIEWER: u64 = 300_041;
    const TRIXI_VIEWER: u64 = 300_042;
    let fixture = Fixture::new();
    let therin = drawn_npc(&fixture, THERIN_VIEWER, therin_position(), THERIN);
    assert_eq!(therin.name_string_id, 185);
    assert_eq!(therin.model_id, 381);
    let drawn_trixi = drawn_npc(&fixture, TRIXI_VIEWER, trixi_position(), TRIXI);
    assert_eq!(drawn_trixi.name_string_id, 3230);
    assert_eq!(drawn_trixi.model_id, 35);
    for axis in 0..3 {
        assert!((drawn_trixi.position[axis] - position[axis] as f32).abs() < 0.01);
    }
}

/// T334: a near-cap coin balance (the wire field is i32) refuses the 49/14 as a per-player
/// application rejection: no coins, no XP, no item, no save change and no panic. Spending the
/// overflow frees the same held end screen for a normal retry.
#[test]
fn a_near_cap_coin_balance_refuses_the_turn_in_without_paying_xp_or_item() {
    let fixture = Fixture::new();
    let mut game = fixture.game(MINER, &[ADVENTURER, MINER]);
    let character = character_at(therin_position());
    let now = complete_the_first_ore(&mut game, &character, 1_000);
    offer(&mut game, &character, THERIN, ALL_I_GOT, now);
    accept(&mut game, &character, ALL_I_GOT, now + 100);

    let at_trixi = character_at(trixi_position());
    open_end_screen(
        &mut game,
        &at_trixi,
        TRIXI,
        ALL_I_GOT,
        ALL_I_GOT_END_TEXT,
        now + 200,
    );

    // Seed the saved balance at the i32 wire cap with the end screen held.
    let mut state = game.state();
    state.inventory.coins = i32::MAX as u32;
    let mut game = fixture.game_with_state(state);
    assert_eq!(pending_turnin(&game), Some(ALL_I_GOT));
    let xp_before = profile_xp(&game, MINER);
    let items_before = item_ledger(&game);
    let before = saved(&game);
    let before_save = fixture.saved_bytes(&game, &character);

    // T382: the refusal is a per-player denial, not a session-ending error or a panic.
    let refused = game
        .handle_in_world(&quest_end_reply(ALL_I_GOT), &at_trixi, now + 300)
        .expect("a near-cap coin balance is a refusal, not an error");
    assert!(!refused.changed, "the near-cap refusal is not a change");
    assert_eq!(coins(&game), i32::MAX as u32, "the refused turn-in paid");
    assert_eq!(
        profile_xp(&game, MINER),
        xp_before,
        "the refused turn-in paid XP"
    );
    assert_eq!(
        item_ledger(&game),
        items_before,
        "the refused turn-in granted an item"
    );
    assert_eq!(
        pending_turnin(&game),
        Some(ALL_I_GOT),
        "the held end screen survives for a retry"
    );
    assert_eq!(quest_status(&game, ALL_I_GOT), QuestStatus::Active);
    assert_eq!(saved(&game), before);
    assert_eq!(fixture.saved_bytes(&game, &character), before_save);

    // Spending the overflow lets the same held end screen pay once.
    let mut state = game.state();
    state.inventory.coins = 0;
    let mut game = fixture.game_with_state(state);
    finish_quest(&mut game, &at_trixi, ALL_I_GOT, MINER, 5, 15, now + 400);
    assert_eq!(coins(&game), 5);
}

// ---------------------------------------------------------------------------------------------
// T411: "Rush Order" (3100004): Smitty, six Copper Ore, back to Trixi.
// ---------------------------------------------------------------------------------------------

/// T411: Trixi offers Rush Order only after 3100003. The 49/3 lists its three ordered
/// objectives (count then one 107-byte record each: `0092b050` -> `008fd770`) and the 49/8
/// (`00c7aeb0`: quest, objective, required) activates only the Smitty Talk.
#[test]
fn rush_order_is_offered_after_all_i_got() {
    let fixture = six_ore_fixture();
    let character = character_at(therin_position());
    let at_trixi = character_at(trixi_position());
    let mut game = fixture.game(MINER, &[ADVENTURER, MINER]);

    let mut now = 500;
    assert_no_offer(
        &mut game,
        &at_trixi,
        TRIXI,
        &mut now,
        "Trixi before All I Got",
    );

    let now = complete_all_i_got(&mut game, &character, 1_000);
    let offered = offer(&mut game, &at_trixi, TRIXI, RUSH_ORDER, now);
    assert_offer_ids(
        &offer_packet(&offered, RUSH_ORDER),
        RUSH_ORDER,
        RUSH_ORDER_INTRO,
        RUSH_ORDER_TITLE,
        RUSH_ORDER_DESCRIPTION,
    );

    let accepted = accept(&mut game, &at_trixi, RUSH_ORDER, now + 100);
    let add = accepted
        .replies
        .iter()
        .find(|packet| has_family(packet, 49, 3, Some(RUSH_ORDER)))
        .expect("accept sends the 49/3 quest record");
    assert_eq!(word(add, 109), 3, "the 49/3 carries three objectives");
    let ids: Vec<u32> = (0..3).map(|index| word(add, 113 + index * 107)).collect();
    assert_eq!(ids, [1, 2, 3], "Smitty, copper ore, Trixi, in order");
    assert!(
        accepted
            .replies
            .contains(&objective_activated_packet(RUSH_ORDER, 1, 1)),
        "accept activates the Smitty Talk"
    );
    assert!(
        !objective_activated(&accepted, RUSH_ORDER, 2)
            && !objective_activated(&accepted, RUSH_ORDER, 3),
        "the later steps wait for the Smitty Talk"
    );
    assert_eq!(counts(&game, RUSH_ORDER), [0, 0, 0]);
}

/// T411: Smitty Talk, one six-ore copper-vein win, Trixi Talk; the 49/14 pays exactly 5 coins
/// + 20 profile-14 XP once from the live state (after a job switch), grants no item, and the
/// counters, coins and XP survive `game_store` reloads.
#[test]
fn smitty_then_copper_then_trixi_pays_once() {
    let fixture = six_ore_fixture();
    let mut game = fixture.game(MINER, &[ADVENTURER, MINER, BRAWLER]);
    let character = character_at(therin_position());
    let at_trixi = character_at(trixi_position());
    let at_smitty = character_at(smitty_position());
    let now = complete_all_i_got(&mut game, &character, 1_000);
    offer(&mut game, &at_trixi, TRIXI, RUSH_ORDER, now);
    accept(&mut game, &at_trixi, RUSH_ORDER, now + 100);
    let coins_before = coins(&game);
    let xp_at_accept = profile_xp(&game, MINER);

    // Goal 1: the Smitty Talk completes and activates the six-ore gather.
    let smitty = talk(&mut game, &at_smitty, SMITTY, now + 200);
    assert!(smitty.changed, "the Smitty Talk advanced the quest");
    assert!(objective_complete(&smitty, RUSH_ORDER, 1));
    assert!(
        smitty
            .replies
            .contains(&objective_activated_packet(RUSH_ORDER, 2, RUSH_ORE)),
        "the gather step opens with its count of six"
    );
    assert!(
        !smitty
            .replies
            .iter()
            .any(|packet| has_family(packet, 49, 13, None)),
        "Smitty is not the turn-in"
    );
    let mut game = fixture.save_and_reload(&game, &at_smitty);
    assert_eq!(
        counts(&game, RUSH_ORDER),
        [1, 0, 0],
        "the Talk counter survives"
    );

    // Goal 2: one ordinary copper-vein win pays the six ore the objective counts.
    let ore_before = copper_ore(&game);
    let (guid, position) = copper_nodes()[1];
    let mut at = now + 300;
    assert!(gather_copper(&mut game, &character, guid, position, &mut at).is_some());
    assert_eq!(copper_ore(&game), ore_before + RUSH_ORE);
    assert_eq!(counts(&game, RUSH_ORDER), [1, RUSH_ORE, 0]);
    let xp_after_mining = profile_xp(&game, MINER);
    assert_eq!(xp_after_mining, xp_at_accept + MINING_XP_PER_WIN);
    let mut game = fixture.save_and_reload(&game, &character);
    assert_eq!(
        counts(&game, RUSH_ORDER),
        [1, RUSH_ORE, 0],
        "the gather survives"
    );
    assert_eq!(pending_turnin(&game), None);
    assert_eq!(
        coins(&game),
        coins_before,
        "nothing is paid before the turn-in"
    );

    // Switch away before the turn-in: the reward map names profile 14.
    let switched = send(&mut game, &character, &select_job(BRAWLER), at + 100);
    assert!(switched.changed, "the Brawler outfit is selectable");
    let brawler_before = profile_xp(&game, BRAWLER);
    let items_before = item_ledger(&game);

    // Goal 3: the Trixi Talk holds the 49/13; the 49/14 pays once.
    let end = open_end_screen(
        &mut game,
        &at_trixi,
        TRIXI,
        RUSH_ORDER,
        RUSH_ORDER_END_TEXT,
        at + 200,
    );
    assert_eq!(end_screen_reward(&end), (5, 20), "49/13 reward bundle");
    assert_eq!(counts(&game, RUSH_ORDER), [1, RUSH_ORE, 1]);
    assert_eq!(coins(&game), coins_before, "the end screen pays nothing");
    finish_quest(&mut game, &at_trixi, RUSH_ORDER, MINER, 5, 20, at + 300);
    assert_eq!(profile_xp(&game, MINER), xp_after_mining + 20);
    assert_eq!(profile_xp(&game, BRAWLER), brawler_before);
    assert_eq!(
        item_ledger(&game),
        items_before,
        "the quest grants no item and consumes none"
    );

    // A reload keeps the paid result; replays pay nothing and Trixi offers nothing more.
    let mut game = fixture.save_and_reload(&game, &at_trixi);
    assert_eq!(quest_status(&game, RUSH_ORDER), QuestStatus::Completed);
    assert_eq!(coins(&game), coins_before + 5);
    assert_eq!(profile_xp(&game, MINER), xp_after_mining + 20);
    assert_inert(
        &fixture,
        &mut game,
        &at_trixi,
        &quest_end_reply(RUSH_ORDER),
        at + 400,
        "a replayed 49/14",
    );
    assert_inert(
        &fixture,
        &mut game,
        &at_trixi,
        &quest_reply(RUSH_ORDER, true),
        at + 500,
        "a replayed 49/2",
    );
    let mut later = at + 600;
    assert_no_offer(
        &mut game,
        &at_trixi,
        TRIXI,
        &mut later,
        "Trixi after Rush Order",
    );
}

/// T411: an inactive Miner, an out-of-order or wrong NPC, a click 30 m outside the (R) 12 m
/// interaction radius, forged 26/8 / 49/2 / 49/14, a turn-in before the gather and a full bag
/// at the node leave the whole `GameplayState` (and, under the save cap, the save bytes) equal.
#[test]
fn a_non_miner_wrong_npc_or_forged_request_changes_nothing() {
    let fixture = six_ore_fixture();
    let character = character_at(therin_position());
    let at_trixi = character_at(trixi_position());
    let at_smitty = character_at(smitty_position());
    let mut donor = fixture.game(MINER, &[ADVENTURER, MINER]);
    let now = complete_all_i_got(&mut donor, &character, 1_000);
    let completed = donor.state();

    let mut state = completed.clone();
    state
        .profiles
        .as_mut()
        .expect("the fixture saves profiles")
        .active = ADVENTURER;
    let mut adventurer = fixture.game_with_state(state);
    let mut later = now;
    assert_no_offer(
        &mut adventurer,
        &at_trixi,
        TRIXI,
        &mut later,
        "an active Adventurer with Miner unlocked",
    );

    let mut game = fixture.game_with_state(completed);
    for (packet, label) in [
        (
            quest_reply(RUSH_ORDER, true),
            "a forged 49/2 before the offer",
        ),
        (
            quest_end_reply(RUSH_ORDER),
            "a forged 49/14 with no end screen",
        ),
        (npc_interact(u64::MAX), "a forged 26/8 at an unknown actor"),
    ] {
        assert_inert(&fixture, &mut game, &at_trixi, &packet, now + 10, label);
    }
    offer(&mut game, &at_trixi, TRIXI, RUSH_ORDER, now + 100);
    accept(&mut game, &at_trixi, RUSH_ORDER, now + 200);

    let mut later = now + 300;
    assert_no_offer(
        &mut game,
        &at_trixi,
        TRIXI,
        &mut later,
        "Trixi before the Smitty step",
    );
    assert_no_offer(&mut game, &character, THERIN, &mut later, "Therin");
    let [x, y, z] = smitty_position();
    let outside = character_at([x + 42.0, y, z]);
    let before = fixture.saved_bytes(&game, &character);
    assert!(
        game.handle_in_world(&npc_interact(SMITTY), &outside, later + 10)
            .is_ok_and(|out| out.replies.is_empty() && !out.changed),
        "a 26/8 30 m outside the interaction radius is refused"
    );
    assert_eq!(fixture.saved_bytes(&game, &character), before);
    assert_eq!(counts(&game, RUSH_ORDER), [0, 0, 0]);

    // After the Smitty step, a turn-in before the copper ore holds no end screen.
    talk(&mut game, &at_smitty, SMITTY, later + 100);
    assert_eq!(counts(&game, RUSH_ORDER), [1, 0, 0]);
    let mut later = later + 200;
    assert_no_offer(
        &mut game,
        &at_trixi,
        TRIXI,
        &mut later,
        "Trixi before the copper ore",
    );
    assert_inert(
        &fixture,
        &mut game,
        &at_trixi,
        &quest_end_reply(RUSH_ORDER),
        later + 10,
        "a 49/14 before the copper ore",
    );

    // A full bag with no Copper Ore stack to merge into refuses the node before the run.
    let mut state = game.state();
    state
        .inventory
        .items
        .retain(|item| item.definition_id != COPPER_ORE as i32);
    fill_bag(&mut state.inventory);
    let mut full = fixture.game_with_state(state);
    let (guid, position) = copper_nodes()[1];
    let at_node = character_at(position);
    full.tick(&at_node, later + 100).expect("node tick");
    // A 4,096-stack bag exceeds the save envelope's 1 MiB cap, so compare the state (T300).
    let before = saved(&full);
    let refused = send(&mut full, &at_node, &npc_interact(guid), later + 101);
    assert!(
        !refused.changed,
        "a full bag refuses the node before the run"
    );
    assert!(
        !refused
            .replies
            .iter()
            .any(|packet| has_family(packet, 49, 9, None)),
        "the refused node sends no objective progress"
    );
    assert!(active_mining(&full).is_none(), "no mining run started");
    assert_eq!(counts(&full, RUSH_ORDER), [1, 0, 0]);
    assert_eq!(saved(&full), before, "the refused node changed the state");
}

/// T411: the record's content; the shipped Mining18 board produces its gather target (T358's
/// rule) and one shipped win pays at least the six it counts; Smitty and Trixi draw through
/// the production 35/2 reader with resolved names and shipped models.
#[test]
fn the_objective_is_produced_by_the_shipped_mining_board() {
    let content = content_dir();
    let quests = read_json(&content.join("quests/miner.json"));
    let records = quests["records"].as_array().unwrap();
    let quest = row(records, u64::from(RUSH_ORDER));
    assert_eq!(quest["enabled"], json!(true));
    assert_eq!(quest["job"], json!(MINER));
    assert_eq!(quest["giver_npc_id"], json!(TRIXI));
    assert_eq!(quest["prerequisite_quest_ids"], json!([ALL_I_GOT]));
    assert_eq!(quest["intro_text_string_id"], json!(RUSH_ORDER_INTRO));
    assert_eq!(quest["title_string_id"], json!(RUSH_ORDER_TITLE));
    assert_eq!(
        quest["description_string_id"],
        json!(RUSH_ORDER_DESCRIPTION)
    );
    assert_eq!(quest["end_text_string_id"], json!(RUSH_ORDER_END_TEXT));
    let objectives = quest["objectives"].as_array().unwrap();
    let bare: Vec<Value> = objectives
        .iter()
        .map(|objective| {
            let mut bare = objective.clone();
            let fields = bare.as_object_mut().unwrap();
            fields.remove("source");
            // Goal text (T403) is display content, checked by tests/quest_engine.rs.
            fields.remove("text_string_id");
            bare
        })
        .collect();
    assert_eq!(
        bare,
        [
            json!({"kind": "talk_to_npc", "id": 1, "npc_id": SMITTY}),
            json!({"kind": "gather_resource", "id": 2, "resource_id": COPPER_ORE, "quantity": RUSH_ORE}),
            json!({"kind": "talk_to_npc", "id": 3, "npc_id": TRIXI}),
        ]
    );
    for objective in objectives {
        assert!(has_source(objective, "client") || has_source(objective, "video"));
        assert!(has_source(objective, "reconstructed"));
    }
    let rewards = quest["rewards"].as_array().unwrap();
    assert_eq!(rewards.len(), 1);
    assert_eq!(rewards[0]["coins"], json!(5));
    assert_eq!(rewards[0]["items"], json!([]));
    assert_eq!(rewards[0]["job_xp"], json!({"14": 20}));
    assert!(has_source(&rewards[0], "reconstructed"));
    let evidence: Vec<&str> = quest["source"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|tag| tag["evidence"].as_str())
        .collect();
    assert!(evidence.iter().any(|text| text.starts_with("(R) T411:")));
    assert!(
        !evidence
            .iter()
            .any(|text| text.contains("objectives is empty") || text.contains("rewards left empty")),
        "the stale empty-record tags are gone"
    );

    // Gem Appraisal stays disabled. T729 enabled the Southvale gate 3100019, so 3100020 follows it
    // and now pays a reward (the reachability gate refuses an empty one).
    assert_eq!(row(records, 3_100_005)["enabled"], json!(false));
    assert_eq!(
        row(records, 3_100_020)["prerequisite_quest_ids"],
        json!([3_100_019])
    );
    assert_ne!(row(records, 3_100_019)["enabled"], json!(false));

    // The production loader reads the shipped board; `reward_transaction` (src/mining.rs)
    // pays collected / per_reward * cluster_quantity per output row, so a won board pays
    // each Copper Ore row's full quota.
    let board = mining::Type8Engine::load(
        &content,
        mining::CONTENT_FILE,
        mining::MINING_GAME_ID,
        mining::FIRST_MINING_INSTANCE,
        None,
    )
    .expect("the shipped Mining18 board loads");
    let spec = board.spec();
    let per_win: u32 = spec
        .outputs
        .iter()
        .filter(|output| output.definition_id == COPPER_ORE as i32)
        .map(|output| {
            let goal = spec
                .goals
                .iter()
                .find(|goal| goal.value == output.value)
                .expect("every output has its goal");
            goal.quota / goal.per_reward * output.cluster_quantity
        })
        .sum();
    assert!(
        per_win >= RUSH_ORE,
        "one shipped win pays {per_win} Copper Ore, fewer than Rush Order counts"
    );

    let fixture = Fixture::new();
    let smitty = drawn_npc(&fixture, 300_061, smitty_position(), SMITTY);
    assert_eq!((smitty.name_string_id, smitty.model_id), (3061, 48));
    let trixi = drawn_npc(&fixture, 300_062, trixi_position(), TRIXI);
    assert_eq!((trixi.name_string_id, trixi.model_id), (3230, 35));
}

/// T411: two Miners own their Rush Order counters, end screen and reward; one player's Talk,
/// gather, refusal or forged reply never moves the other's.
#[test]
fn two_miners_progress_independently_on_rush_order() {
    const FIRST: u64 = 300_071;
    const SECOND: u64 = 300_072;
    let fixture = six_ore_fixture();
    let at = |guid, position| character_at_for(guid, 0, position);
    let mut a = fixture.game_as(FIRST, MINER, &[ADVENTURER, MINER]);
    let mut b = fixture.game_as(SECOND, MINER, &[ADVENTURER, MINER]);
    let mut now = complete_all_i_got(&mut a, &at(FIRST, therin_position()), 1_000);
    now = complete_all_i_got(&mut b, &at(SECOND, therin_position()), now);
    for (game, guid) in [(&mut a, FIRST), (&mut b, SECOND)] {
        offer(game, &at(guid, trixi_position()), TRIXI, RUSH_ORDER, now);
        accept(game, &at(guid, trixi_position()), RUSH_ORDER, now + 100);
    }

    let (guid, position) = copper_nodes()[1];
    let mut ms = now + 200;
    talk(&mut a, &at(FIRST, smitty_position()), SMITTY, ms);
    assert!(gather_copper(&mut a, &at(FIRST, position), guid, position, &mut ms).is_some());
    assert_eq!(counts(&a, RUSH_ORDER), [1, RUSH_ORE, 0]);
    assert_eq!(counts(&b, RUSH_ORDER), [0, 0, 0], "A's progress moved B");

    let a_before = saved(&a);
    let b_before = saved(&b);
    let [x, y, z] = smitty_position();
    assert!(
        b.handle_in_world(&npc_interact(SMITTY), &at(SECOND, [x + 42.0, y, z]), ms)
            .is_ok_and(|out| out.replies.is_empty() && !out.changed),
        "B's out-of-range click is refused"
    );
    assert_packet_changes_nothing(
        &mut b,
        &at(SECOND, trixi_position()),
        &quest_end_reply(RUSH_ORDER),
        ms + 10,
        "B's forged 49/14",
    );
    assert_eq!(saved(&a), a_before);
    assert_eq!(saved(&b), b_before);

    let end = open_end_screen(
        &mut a,
        &at(FIRST, trixi_position()),
        TRIXI,
        RUSH_ORDER,
        RUSH_ORDER_END_TEXT,
        ms + 100,
    );
    assert_eq!(end_screen_reward(&end), (5, 20));
    assert_eq!(pending_turnin(&b), None, "A's end screen opened B's");
    finish_quest(
        &mut a,
        &at(FIRST, trixi_position()),
        RUSH_ORDER,
        MINER,
        5,
        20,
        ms + 200,
    );
    assert_eq!(counts(&b, RUSH_ORDER), [0, 0, 0]);

    talk(&mut b, &at(SECOND, smitty_position()), SMITTY, ms + 300);
    ms += 400;
    assert!(gather_copper(&mut b, &at(SECOND, position), guid, position, &mut ms).is_some());
    open_end_screen(
        &mut b,
        &at(SECOND, trixi_position()),
        TRIXI,
        RUSH_ORDER,
        RUSH_ORDER_END_TEXT,
        ms + 100,
    );
    finish_quest(
        &mut b,
        &at(SECOND, trixi_position()),
        RUSH_ORDER,
        MINER,
        5,
        20,
        ms + 200,
    );

    // Each wallet holds its three 5-coin rungs; each Miner profile two wins plus 15/15/20.
    for game in [&a, &b] {
        assert_eq!(quest_status(game, RUSH_ORDER), QuestStatus::Completed);
        assert_eq!(coins(game), 15);
        assert_eq!(profile_xp(game, MINER), 2 * MINING_XP_PER_WIN + 50);
    }
}

/// T411: the chapter runs on the production quest, menu and node packets the shared helpers
/// build (26/8, 26/10, 26/6, 49/2, 49/14, 39/x, 26/42): no realm chat command is sent, no
/// reply teleports the player (38/12), and the character ends free with nothing open.
#[test]
fn the_chapter_uses_no_dev_commands() {
    let fixture = six_ore_fixture();
    let character = character_at(therin_position());
    let at_trixi = character_at(trixi_position());
    let mut game = fixture.game(MINER, &[ADVENTURER, MINER]);
    let now = complete_all_i_got(&mut game, &character, 1_000);
    let mut replies = offer(&mut game, &at_trixi, TRIXI, RUSH_ORDER, now).replies;
    replies.extend(accept(&mut game, &at_trixi, RUSH_ORDER, now + 100).replies);
    let at_smitty = character_at(smitty_position());
    replies.extend(talk(&mut game, &at_smitty, SMITTY, now + 200).replies);
    let (guid, position) = copper_nodes()[1];
    let mut at = now + 300;
    assert!(gather_copper(&mut game, &character, guid, position, &mut at).is_some());
    replies.push(open_end_screen(
        &mut game,
        &at_trixi,
        TRIXI,
        RUSH_ORDER,
        RUSH_ORDER_END_TEXT,
        at + 100,
    ));
    replies.extend(finish_quest(&mut game, &at_trixi, RUSH_ORDER, MINER, 5, 20, at + 200).replies);
    assert!(
        !replies
            .iter()
            .any(|packet| packet.starts_with(&[38, 0, 12, 0])),
        "the chapter must not teleport the player"
    );
    assert_eq!(quest_status(&game, RUSH_ORDER), QuestStatus::Completed);
    assert_eq!(pending_offer(&game), None);
    assert_eq!(pending_turnin(&game), None);
    assert!(active_mining(&game).is_none(), "no mining run is left open");
    // The NPC router's shared busy gate closes menus while an activity runs; Therin's opens.
    let menu = send(&mut game, &character, &npc_interact(THERIN), at + 300);
    assert!(
        menu.replies
            .iter()
            .any(|packet| packet.starts_with(&[26, 0, 9, 0])),
        "the character ends free: Therin's menu opens"
    );
}
