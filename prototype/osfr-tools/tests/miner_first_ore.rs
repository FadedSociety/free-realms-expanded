//! T300: the Miner's first playable quest, "So, You Want To Be A Miner?" (3100002).
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
};
use npc_quest_chain::{
    CHARACTER, accept, active_job, assert_no_offer, assert_packet_changes_nothing,
    character_at_for, coins, content_dir, content_position, counts, end_screen_reward,
    finish_quest, has_family, npc_interact, npc_select, offer, pending_offer, pending_turnin,
    profile_xp, quest_end_reply, quest_reply, quest_status, read_json, saved, select_job, send,
};
use osfr_tools::{
    character_store::CharacterState,
    gameplay::Gameplay,
    inventory::MAX_ITEMS,
    mining,
    profiles::ProfileState,
    quest_engine::QuestStatus,
    startup_world::{NpcSpawn, parse_add_npc},
};
use serde_json::{Value, json};
use std::collections::BTreeMap;

const THERIN: u64 = 9_000_000_217;
/// Trixi (9000000220) is a spawned Snowhill NPC who gives no live Miner quest: wrong NPC.
const TRIXI: u64 = 9_000_000_220;
const MINER_FIRST: u32 = 3_100_002;
const DRAINED_RECORDS: [u32; 4] = [3_100_001, 3_100_003, 3_100_004, 3_100_005];
const TITLE: u32 = 23_217;
const DESCRIPTION: u32 = 23_218;
const END_TEXT: u32 = 23_219;
const ADVENTURER: i32 = 1;
const MINER: i32 = 14;
const BRAWLER: i32 = 43;
/// 39/6 Quit: leave an offered/in-progress run without a payout.
const QUIT_FRAME_SUB: u8 = 6;

fn therin_position() -> [f32; 3] {
    content_position("snowhill.json", THERIN)
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

/// The production per-session visibility batch, for one session at Therin's spawn. Each
/// nearby session builds its own batch through `Gameplay::update_npc_visibility`.
fn drawn_therin(fixture: &Fixture, character_id: u64) -> NpcSpawn {
    let mut game = fixture.game_as(character_id, MINER, &[ADVENTURER, MINER]);
    let at = character_at_for(character_id, 0, therin_position());
    game.update_npc_visibility(&at)
        .expect("visibility batch builds")
        .iter()
        .filter_map(|packet| parse_add_npc(packet).ok())
        .find(|npc| npc.actor_id == THERIN)
        .expect("Therin is drawn for every nearby session")
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

    // A wrong NPC (Trixi, a spawned Snowhill NPC with no live Miner quest) offers nothing.
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
            .is_err(),
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
    let mut reoffer = now + 400;
    assert_no_offer(
        &mut miner,
        &character,
        THERIN,
        &mut reoffer,
        "a completed Miner quest",
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

    // A's offer is its own; B keeps a free menu and its own offer. B's out-of-range click
    // is a refusal that consumes neither A's pending offer nor B's own menu.
    offer(&mut a, &first, THERIN, MINER_FIRST, 1_000);
    assert_eq!(pending_offer(&a), Some(MINER_FIRST));
    assert_eq!(pending_offer(&b), None, "A's offer consumed B's menu");
    let far = character_at_for(SECOND, 0, [10_000.0, 0.0, 10_000.0]);
    let b_before = saved(&b);
    assert!(
        b.handle_in_world(&npc_interact(THERIN), &far, 1_005)
            .is_err(),
        "an out-of-range 26/8 is refused"
    );
    assert_eq!(
        pending_offer(&a),
        Some(MINER_FIRST),
        "B's refusal consumed A's pending offer"
    );
    assert_eq!(saved(&b), b_before, "B's refusal changed B's state");
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
    // independent nearby sessions.
    const FIRST_VIEWER: u64 = 300_011;
    const SECOND_VIEWER: u64 = 300_012;
    let fixture = Fixture::new();
    let drawn: Vec<NpcSpawn> = [FIRST_VIEWER, SECOND_VIEWER]
        .into_iter()
        .map(|viewer| drawn_therin(&fixture, viewer))
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

    // The records this slice must not open stay hollow with their recovered titles.
    for (id, title) in [
        (DRAINED_RECORDS[0], 139_748),
        (DRAINED_RECORDS[1], 76_087),
        (DRAINED_RECORDS[2], 76_209),
        (DRAINED_RECORDS[3], 76_770),
    ] {
        let record = row(&quests["records"].as_array().unwrap(), u64::from(id));
        assert_eq!(record["enabled"], json!(false), "quest {id} stays disabled");
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

    // A replayed turn-in and a later Talk cannot pay or re-grant anything.
    let finished = saved(&game);
    let finished_ledger = item_ledger(&game);
    assert_packet_changes_nothing(
        &mut game,
        &character,
        &quest_end_reply(MINER_FIRST),
        now + 300,
        "a replayed 49/14",
    );
    let mut reoffer = now + 400;
    assert_no_offer(
        &mut game,
        &character,
        THERIN,
        &mut reoffer,
        "the completed quest",
    );
    assert_eq!(coins(&game), 5);
    assert_eq!(profile_xp(&game, MINER), MINING_XP_PER_WIN + 15);
    assert_eq!(item_ledger(&game), finished_ledger);
    assert_eq!(saved(&game), finished);
}
