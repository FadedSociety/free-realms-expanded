//! T241: the two live Farnum starting quests, driven through the production paths.
//!
//! Fighting the Pack (3000009) counts five of the seven live Hooligan Wolves (definition 3156)
//! killed through the field Attack menu; Cleetus' Copper Collection (3000016) counts five
//! Copper Ore (item definition 2739) gathered from the Snowhill copper-vein nodes through
//! the ordinary Type8 mining round. Both are offered by Cleetus Farnum (9000000050) and
//! turned in through the real 26/8 -> 26/10 -> 26/6 -> 49/13 -> 49/14 sequence. The
//! shared fixture, packet builders and refusal/offer helpers come from
//! `tests/support/npc_quest_chain.rs`; the synthetic one-goal board and the node/win
//! helpers come from `tests/support/mining_quest_chain.rs`, shared with T300.
//!
//! Client packet layouts used here (IDA native map):
//! - 26/8 CommandPacketInteractRequest constructor `008b6bc0`: two u32 target words and a
//!   u8 trailing flag.
//! - 26/10 CommandPacketInteractionSelect constructor `00a36f80`: two u32 target words and
//!   an i32 choice.
//! - 26/6 PacketDialogResponse constructor `009f5890`: one i32 row (the quest row is 301).
//! - 49/2 QuestReply constructor `00a86400`: u32 quest id and a u8 accept flag.
//! - 49/1 QuestInfo reader `00c7b7a0`: six dwords end at offset 30, members-only at 30,
//!   then the 69-byte reward bundle at 31 read by `008e7930`; quest at 6, intro at 10,
//!   title at 14/18, description at 22. T318: the bundle carries the quest's authored
//!   reward through the same encoder the 49/13 screen uses (see `assert_offer_reward`).
//! - 49/13 QuestEnd reader `00c7b990`: u64 actor, quest i32, end-text u32, a reserved u32,
//!   then the 69-byte reward bundle (success, coins, XP stars) and a trailing float.
//! - 49/14 QuestEndReply constructor `00a864c0`: one u32 quest id.
//! - 35/3 removal body reader `008d69f0` (base reader `008d6830`, wrapper `008f7700`,
//!   packet constructor `008b3840`): u16 flag at 4..6, then the actor u64 at 6..14. The
//!   hostile test reads it to prove the wolf died.

#[path = "support/mining_quest_chain.rs"]
mod mining_quest_chain;
#[path = "support/npc_quest_chain.rs"]
mod npc_quest_chain;

use mining_quest_chain::{
    COPPER_ORE, Fixture, MINING_XP_PER_WIN, active_mining, assert_offer_ids, copper_nodes,
    copper_ore, gather_copper, offer_packet, open_end_screen, signed_word, type8_frame,
    type8_payload, word,
};
use npc_quest_chain::{
    CHARACTER, accept, active_job, assert_no_offer, assert_packet_changes_nothing,
    character_at_for, coins, content_dir, content_position, counts, finish_quest, has_family,
    npc_interact, npc_select, offer, profile_xp, quest_end_reply as npc_quest_end_reply,
    quest_reply as npc_quest_reply, quest_status, read_json, saved, select_job, send,
};
use osfr_tools::{
    character_store::CharacterState, gameplay::Gameplay, mining, quest_engine::QuestStatus,
};
use serde_json::{Value, json};

const CLEETUS: u64 = 9_000_000_050;
/// A placed quest giver who gives neither live Farnum quest (wrong-NPC refusal).
/// Anne 9500000266 is no longer usable here: T360 gives her quest 3000061.
const BO: u64 = 9_500_000_268;
const ADVENTURER: i32 = 1;
const MINER: i32 = 14;
const BRAWLER: i32 = 43;
const FIGHTING_THE_PACK: u32 = 3_000_009;
const COPPER_COLLECTION: u32 = 3_000_016;
/// The authored predecessor; it stays disabled, so 3000009 is a playtest entry (R).
const YOUR_FIRST_WEAPON: u32 = 3_000_008;
/// A 3000020+ row this slice must not touch; pinned by id/title/enabled.
const UNTOUCHED_RECORD: u32 = 3_000_020;
const UNTOUCHED_TITLE: u32 = 94_247;
const WOLF_DEFINITION: u32 = 3_156;
const FIGHTING_TITLE: u32 = 4_420_074;
const FIGHTING_INTRO: u32 = 4_420_073;
const FIGHTING_DESCRIPTION: u32 = 4_420_075;
const FIGHTING_END_TEXT: u32 = 4_420_076;
const COPPER_TITLE: u32 = 5_100_483;
const COPPER_INTRO: u32 = 5_100_482;
const COPPER_DESCRIPTION: u32 = 5_100_484;
const COPPER_END_TEXT: u32 = 5_100_485;
const ATTACK_CHOICE: i32 = 103;
/// 39/6 Quit: leave the offered run without a payout.
const QUIT_FRAME_SUB: u8 = 6;
/// The authored `starting_farnum_wolves` group respawn timer.
const WOLF_RESPAWN_MS: u64 = 46_000;

fn cleetus_position() -> [f32; 3] {
    content_position("farnum_farmstead.json", CLEETUS)
}

fn character_at(position: [f32; 3]) -> CharacterState {
    character_at_for(CHARACTER, 0, position)
}

/// The always-on Hooligan Wolves (definition 3156). Empty `quest_ids` is
/// production's "present without a quest" rule. Production credits every defeat
/// by definition id, so T245's level-5 `MID-F` camp and T296's level-11/12
/// `HIGH-F` camp are part of this population: seven live wolves, not three.
fn wolves() -> Vec<(u64, [f32; 3])> {
    read_json(&content_dir().join("field_enemies.json"))["enemies"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|enemy| {
            enemy["definition_id"].as_u64() == Some(u64::from(WOLF_DEFINITION))
                && enemy["quest_ids"]
                    .as_array()
                    .is_some_and(|ids| ids.is_empty())
        })
        .map(|enemy| {
            (
                enemy["guid"].as_u64().expect("field GUID"),
                serde_json::from_value(enemy["position"].clone()).expect("field position"),
            )
        })
        .collect()
}

fn has_quest(game: &Gameplay, quest_id: u32) -> bool {
    game.state()
        .npc_quests
        .as_ref()
        .is_some_and(|state| state.quests.contains_key(&quest_id))
}

fn count(game: &Gameplay, quest_id: u32, objective: usize) -> u32 {
    counts(game, quest_id)[objective]
}

/// T318: the 49/1 card carries the same authored `ClientRewardBundle` body the 49/13
/// screen builds (`00c7b7a0` -> `008e7930`): success at packet 31, coins at 32, XP
/// stars at 36; the trailing 14 bytes keep the clicked actor, float bits and bools.
fn assert_offer_reward(packet: &[u8], expected_coins: u32, expected_xp: u32) {
    assert_eq!(
        packet.len(),
        114,
        "49/1 is 114 bytes with the 69-byte empty bundle"
    );
    assert_eq!(packet[31], 1, "reward bundle success flag");
    assert_eq!(
        signed_word(packet, 32),
        expected_coins as i32,
        "49/1 displayed coins"
    );
    assert_eq!(
        signed_word(packet, 36),
        expected_xp as i32,
        "49/1 displayed XP stars"
    );
    assert_eq!(
        u64::from_le_bytes(packet[100..108].try_into().unwrap()),
        CLEETUS,
        "the offer names the clicked giver after the bundle"
    );
    assert_eq!(
        &packet[108..],
        &[0; 6],
        "float bits and both bools are zero"
    );
}

/// 49/13: actor at 6, quest at 14, end text at 18, bundle success at 26, coins at 27,
/// XP stars at 31 (client reader `00c7b990`, `008e7930`).
fn assert_end_reward(packet: &[u8], quest_id: u32, expected_coins: u32, expected_xp: u32) {
    assert_eq!(
        u64::from_le_bytes(packet[6..14].try_into().unwrap()),
        CLEETUS,
        "the end screen names the runtime turn-in actor"
    );
    assert_eq!(word(packet, 14), quest_id);
    assert_eq!(packet[26], 1, "reward bundle success flag");
    assert_eq!(
        signed_word(packet, 27),
        expected_coins as i32,
        "49/13 displayed coins"
    );
    assert_eq!(
        signed_word(packet, 31),
        expected_xp as i32,
        "49/13 displayed XP stars"
    );
}

/// One field kill through the real 26/8 -> Attack menu -> 26/10 path. The packet loop is
/// bounded and stops on the removal packet (`008d69f0` reads the actor at 6..14) or on the
/// counted objective.
fn kill_wolf(game: &mut Gameplay, position: [f32; 3], guid: u64, now: &mut u64) -> bool {
    let character = character_at(position);
    *now += 100;
    let menu = send(game, &character, &npc_interact(guid), *now);
    assert!(
        menu.replies
            .iter()
            .any(|packet| has_family(packet, 26, 9, None)),
        "wolf {guid} exposes the field interaction menu"
    );
    let before = count(game, FIGHTING_THE_PACK, 0);
    for _ in 0..8 {
        *now += 700;
        let attack = send(game, &character, &npc_select(guid, ATTACK_CHOICE), *now);
        if attack.replies.iter().any(|packet| {
            has_family(packet, 35, 3, None)
                && packet.get(6..14) == Some(guid.to_le_bytes().as_slice())
        }) {
            return true;
        }
        if count(game, FIGHTING_THE_PACK, 0) > before {
            return true;
        }
    }
    false
}

fn respawn_wolves(game: &mut Gameplay, now: &mut u64) {
    let observer = character_at([10_000.0, 0.0, 10_000.0]);
    *now += WOLF_RESPAWN_MS;
    game.tick(&observer, *now).expect("field respawn tick");
}

#[test]
fn cleetus_offers_both_quests_to_the_right_job() {
    let fixture = Fixture::new();
    let character = character_at(cleetus_position());

    // Two clients at the same giver: each keeps its own offer state.
    let mut brawler = fixture.game(BRAWLER, &[ADVENTURER, MINER, BRAWLER]);
    let mut miner = fixture.game(MINER, &[ADVENTURER, MINER, BRAWLER]);

    let offered = offer(&mut brawler, &character, CLEETUS, FIGHTING_THE_PACK, 1_000);
    let packet = offer_packet(&offered, FIGHTING_THE_PACK);
    assert_offer_ids(
        &packet,
        FIGHTING_THE_PACK,
        FIGHTING_INTRO,
        FIGHTING_TITLE,
        FIGHTING_DESCRIPTION,
    );
    assert_offer_reward(&packet, 18, 25);
    assert_eq!(
        quest_status(&brawler, FIGHTING_THE_PACK),
        QuestStatus::Offered
    );

    let offered = offer(&mut miner, &character, CLEETUS, COPPER_COLLECTION, 1_000);
    let packet = offer_packet(&offered, COPPER_COLLECTION);
    assert_offer_ids(
        &packet,
        COPPER_COLLECTION,
        COPPER_INTRO,
        COPPER_TITLE,
        COPPER_DESCRIPTION,
    );
    assert_offer_reward(&packet, 5, 15);
    assert_eq!(
        quest_status(&miner, COPPER_COLLECTION),
        QuestStatus::Offered
    );

    // The offers are independent: neither client sees the other quest, and a third
    // client on another active job is offered neither. The refusal leaves its open
    // menu usable and the save unchanged.
    assert!(!has_quest(&brawler, COPPER_COLLECTION));
    assert!(!has_quest(&miner, FIGHTING_THE_PACK));
    let mut adventurer = fixture.game(ADVENTURER, &[ADVENTURER, MINER, BRAWLER]);
    let mut now = 2_000;
    assert_no_offer(
        &mut adventurer,
        &character,
        CLEETUS,
        &mut now,
        "a non-Brawler, non-Miner",
    );

    accept(&mut brawler, &character, FIGHTING_THE_PACK, 3_000);
    accept(&mut miner, &character, COPPER_COLLECTION, 3_000);
    assert!(!has_quest(&brawler, COPPER_COLLECTION));
    assert!(!has_quest(&miner, FIGHTING_THE_PACK));
}

#[test]
fn fighting_the_pack_counts_five_live_wolves_and_pays_once() {
    let fixture = Fixture::new();
    let mut game = fixture.game(BRAWLER, &[ADVENTURER, MINER, BRAWLER]);
    let character = character_at(cleetus_position());
    let wolves = wolves();
    assert_eq!(
        wolves.len(),
        7,
        "seven always-on wolves are live: three starter, two level-5 MID-F and two level-11/12 HIGH-F"
    );

    let offered = offer(&mut game, &character, CLEETUS, FIGHTING_THE_PACK, 1_000);
    assert_eq!(
        word(&offer_packet(&offered, FIGHTING_THE_PACK), 14),
        FIGHTING_TITLE
    );
    accept(&mut game, &character, FIGHTING_THE_PACK, 1_100);
    assert_eq!(counts(&game, FIGHTING_THE_PACK), [0, 0]);

    let mut now = 2_000;
    // The first five are the three starter wolves and T245's two level-5 MID-F
    // camp wolves. Production credits every 3156 defeat, so the quest completes
    // on those five without waiting for the authored 45 s respawn: the
    // production rule the previous W102-area filter hid.
    for (index, (guid, position)) in wolves.iter().copied().take(5).enumerate() {
        assert!(
            kill_wolf(&mut game, position, guid, &mut now),
            "wolf {index} died through the production attack path"
        );
        assert_eq!(count(&game, FIGHTING_THE_PACK, 0), index as u32 + 1);
    }
    assert_eq!(count(&game, FIGHTING_THE_PACK, 0), 5);
    // T296's two level-11/12 HIGH-F wolves share definition 3156. They are
    // part of the same live population; killing them after the five goal
    // cannot advance the counter past 5.
    for (guid, position) in wolves.iter().copied().skip(5) {
        assert!(kill_wolf(&mut game, position, guid, &mut now));
        assert_eq!(count(&game, FIGHTING_THE_PACK, 0), 5);
    }

    // The reward map names Brawler. Switching the active job must not switch the payer:
    // the support rule is explicit-profile, pinned by
    // `quest_job_xp::xp_follows_the_quest_job_not_the_active_job`. Miner's starter
    // outfit exists in this opening fixture, so the production swap gate can wear it.
    now += 100;
    let switched = send(&mut game, &character, &select_job(MINER), now);
    assert!(switched.changed, "the job switch commits");
    assert_eq!(active_job(&game), MINER);
    let active_before = profile_xp(&game, MINER);

    let coins_before = coins(&game);
    let xp_before = profile_xp(&game, BRAWLER);
    let end = open_end_screen(
        &mut game,
        &character,
        CLEETUS,
        FIGHTING_THE_PACK,
        FIGHTING_END_TEXT,
        now + 100,
    );
    assert_end_reward(&end, FIGHTING_THE_PACK, 18, 25);
    assert_eq!(coins(&game), coins_before, "the end screen pays nothing");

    // (R) Accepted gap, lead-deferred: 49/14 is keyed only by `pending_turnin` and does
    // not recheck the speaker or the player's range. `scavenger_hunt_quest`'s
    // `a_completed_final_talk_is_turned_in_away_from_the_giver_today` (F272 review
    // finding 7, commit `8c55a69`) pins that current behavior; enforcing a turn-in range
    // is a separate lead-approved change, so this suite does not claim the gate.
    finish_quest(
        &mut game,
        &character,
        FIGHTING_THE_PACK,
        BRAWLER,
        18,
        25,
        now + 200,
    );
    assert_eq!(
        profile_xp(&game, BRAWLER),
        xp_before + 25,
        "the quest pays the Brawler profile once"
    );
    assert_eq!(
        profile_xp(&game, MINER),
        active_before,
        "the active job receives no quest XP"
    );

    // Replayed 49/14 pays nothing, and the completed quest is not offered again.
    let finished = saved(&game);
    let finished_coins = coins(&game);
    assert_packet_changes_nothing(
        &mut game,
        &character,
        &npc_quest_end_reply(FIGHTING_THE_PACK),
        now + 300,
        "a replayed 49/14",
    );
    assert_eq!(saved(&game), finished);
    assert_eq!(coins(&game), finished_coins);
    // Back on Brawler, the only other live quest is Miner-gated, so a Talk at the giver
    // has nothing eligible: the completed quest is not offered again.
    let switched = send(&mut game, &character, &select_job(BRAWLER), now + 350);
    assert!(switched.changed, "the job switches back");
    assert_eq!(active_job(&game), BRAWLER);
    let mut reoffer = now + 400;
    assert_no_offer(
        &mut game,
        &character,
        CLEETUS,
        &mut reoffer,
        "a completed Fighting the Pack",
    );

    // A sixth live kill after turn-in changes nothing.
    respawn_wolves(&mut game, &mut now);
    let after_turnin = saved(&game);
    let after_coins = coins(&game);
    let after_xp = profile_xp(&game, BRAWLER);
    let (guid, position) = wolves[0];
    assert!(
        kill_wolf(&mut game, position, guid, &mut now),
        "the respawned sixth wolf still dies"
    );
    assert_eq!(count(&game, FIGHTING_THE_PACK, 0), 5);
    assert_eq!(saved(&game), after_turnin, "a sixth kill changed the save");
    assert_eq!(coins(&game), after_coins);
    assert_eq!(profile_xp(&game, BRAWLER), after_xp);
}

#[test]
fn copper_collection_counts_five_gathers_from_the_node_path() {
    let fixture = Fixture::new();
    let mut game = fixture.game(MINER, &[ADVENTURER, MINER, BRAWLER]);
    let character = character_at(cleetus_position());
    let nodes = copper_nodes();
    assert!(nodes.len() >= 2, "the copper pool has live spawns");

    offer(&mut game, &character, CLEETUS, COPPER_COLLECTION, 1_000);
    accept(&mut game, &character, COPPER_COLLECTION, 1_100);
    assert_eq!(counts(&game, COPPER_COLLECTION), [0, 0]);

    // An abandoned round pays and counts nothing.
    let (first_guid, first_position) = nodes[0];
    let mut now = 2_000;
    let miner = character_at(first_position);
    game.tick(&miner, now).expect("node tick");
    send(&mut game, &miner, &npc_interact(first_guid), now);
    let instance = active_mining(&game).expect("the node starts a run");
    now += 100;
    send(
        &mut game,
        &miner,
        &type8_frame(QUIT_FRAME_SUB, instance),
        now,
    );
    now += 100;
    game.tick(&miner, now).expect("quit tick");
    assert_eq!(count(&game, COPPER_COLLECTION, 0), 0);
    assert_eq!(copper_ore(&game), 0, "an abandoned round pays no ore");
    assert_eq!(coins(&game), 0);

    for (index, (guid, position)) in nodes.iter().copied().take(4).enumerate() {
        assert!(
            gather_copper(&mut game, &character_at(position), guid, position, &mut now).is_some(),
            "node {index} started a round"
        );
        assert_eq!(count(&game, COPPER_COLLECTION, 0), index as u32 + 1);
        assert_eq!(copper_ore(&game), index as u32 + 1);
    }
    assert_eq!(count(&game, COPPER_COLLECTION, 0), 4);

    // Four of the five rows are placed at once; the first node grows back after 60 s.
    let (first_guid, first_position) = nodes[0];
    now += 61_000;
    game.tick(&character, now).expect("node regrow tick");
    assert!(
        gather_copper(
            &mut game,
            &character_at(first_position),
            first_guid,
            first_position,
            &mut now
        )
        .is_some()
    );
    assert_eq!(count(&game, COPPER_COLLECTION, 0), 5);
    assert_eq!(copper_ore(&game), 5);

    let xp_before = profile_xp(&game, MINER);
    assert_eq!(xp_before, 5 * MINING_XP_PER_WIN);
    let end = open_end_screen(
        &mut game,
        &character,
        CLEETUS,
        COPPER_COLLECTION,
        COPPER_END_TEXT,
        now + 100,
    );
    assert_end_reward(&end, COPPER_COLLECTION, 5, 15);
    finish_quest(
        &mut game,
        &character,
        COPPER_COLLECTION,
        MINER,
        5,
        15,
        now + 200,
    );
    assert_eq!(
        profile_xp(&game, MINER),
        xp_before + 15,
        "the quest pays the Miner profile once"
    );

    // A stale win payload for the last round cannot pay or advance anything.
    let finished = saved(&game);
    let _ = game.handle_in_world(
        &type8_payload(instance, "OnNextLevelMsg"),
        &character,
        now + 300,
    );
    assert_eq!(saved(&game), finished, "a replayed win changed the save");
    assert_packet_changes_nothing(
        &mut game,
        &character,
        &npc_quest_end_reply(COPPER_COLLECTION),
        now + 400,
        "a replayed 49/14",
    );
}

#[test]
fn wrong_job_wrong_target_and_replay_change_nothing() {
    let fixture = Fixture::new();
    let character = character_at(cleetus_position());

    // A non-Brawler, non-Miner sees no offer; the open menu survives and the save stays.
    let mut adventurer = fixture.game(ADVENTURER, &[ADVENTURER, MINER, BRAWLER]);
    assert_packet_changes_nothing(
        &mut adventurer,
        &character,
        &npc_quest_reply(FIGHTING_THE_PACK, true),
        100,
        "a forged accept on the wrong job",
    );
    let mut now = 200;
    assert_no_offer(
        &mut adventurer,
        &character,
        CLEETUS,
        &mut now,
        "a non-Brawler, non-Miner at Cleetus",
    );

    // The disabled predecessor and an unopened end screen are both refused.
    assert_packet_changes_nothing(
        &mut adventurer,
        &character,
        &npc_quest_reply(YOUR_FIRST_WEAPON, true),
        300,
        "a forged 3000008 accept",
    );
    assert_packet_changes_nothing(
        &mut adventurer,
        &character,
        &npc_quest_end_reply(YOUR_FIRST_WEAPON),
        400,
        "a forged 3000008 turn-in",
    );
    assert_packet_changes_nothing(
        &mut adventurer,
        &character,
        &npc_quest_end_reply(FIGHTING_THE_PACK),
        500,
        "a forged 49/14 with no end screen",
    );

    // The real active quest is still not payable while its kill step is open.
    let mut brawler = fixture.game(BRAWLER, &[ADVENTURER, MINER, BRAWLER]);
    let offered = offer(&mut brawler, &character, CLEETUS, FIGHTING_THE_PACK, 600);
    assert_eq!(
        word(&offer_packet(&offered, FIGHTING_THE_PACK), 14),
        FIGHTING_TITLE
    );
    accept(&mut brawler, &character, FIGHTING_THE_PACK, 602);
    assert_packet_changes_nothing(
        &mut brawler,
        &character,
        &npc_quest_end_reply(FIGHTING_THE_PACK),
        700,
        "a turn-in before the kill step is complete",
    );

    // A wrong quest giver offers neither live quest and changes nothing.
    let bo = character_at(content_position("farnum_farmstead.json", BO));
    let mut now = 800;
    assert_no_offer(&mut brawler, &bo, BO, &mut now, "a wrong quest giver");

    // An unknown target is a clean no-op on both request types.
    assert_packet_changes_nothing(
        &mut brawler,
        &character,
        &npc_interact(u64::MAX),
        now + 100,
        "an unknown NPC interact target",
    );
    assert_packet_changes_nothing(
        &mut brawler,
        &character,
        &npc_select(u64::MAX, 1),
        now + 101,
        "an unknown NPC menu choice",
    );

    // Out-of-range NPC interaction returns the router's error and changes nothing; a
    // 49/14 without a server-opened end screen is a clean refusal from anywhere.
    let far = character_at([10_000.0, 0.0, 10_000.0]);
    let before = saved(&brawler);
    assert!(
        brawler
            .handle_in_world(&npc_interact(CLEETUS), &far, now + 200)
            .is_err(),
        "an out-of-range NPC interaction is refused"
    );
    assert_eq!(saved(&brawler), before);
    assert!(
        brawler
            .handle_in_world(&npc_quest_end_reply(FIGHTING_THE_PACK), &far, now + 201)
            .is_ok(),
        "a 49/14 without a server-opened end screen is a clean refusal"
    );
    assert_eq!(saved(&brawler), before);

    // A disabled-record replay does not unlock the authored chain either.
    assert_packet_changes_nothing(
        &mut brawler,
        &character,
        &npc_quest_reply(YOUR_FIRST_WEAPON, true),
        now + 300,
        "a 3000008 accept on a Brawler",
    );
}

#[test]
fn the_two_new_records_load_and_their_targets_exist() {
    let document = read_json(&content_dir().join("quests/adventurer.json"));
    let records = document["records"].as_array().unwrap();
    let record = |id: u32| {
        records
            .iter()
            .find(|record| record["id"].as_u64() == Some(u64::from(id)))
            .unwrap_or_else(|| panic!("quest {id} exists"))
    };
    let sources = |value: &Value| {
        value["source"]
            .as_array()
            .is_some_and(|source| !source.is_empty())
    };
    for (id, title, coins, profile, xp) in [
        (FIGHTING_THE_PACK, FIGHTING_TITLE, 18, BRAWLER, 25),
        (COPPER_COLLECTION, COPPER_TITLE, 5, MINER, 15),
    ] {
        let quest = record(id);
        assert_eq!(quest["enabled"].as_bool(), Some(true), "quest {id} enabled");
        assert_eq!(quest["giver_npc_id"].as_u64(), Some(CLEETUS));
        assert_eq!(quest["title_string_id"].as_u64(), Some(u64::from(title)));
        assert_eq!(quest["job"].as_i64(), Some(i64::from(profile)));
        assert_eq!(quest["prerequisite_quest_ids"], json!([]));
        assert!(
            quest["objectives"].as_array().is_some_and(|goals| {
                !goals.is_empty() && goals.iter().all(|goal| sources(goal))
            })
        );
        let rewards = quest["rewards"].as_array().unwrap();
        assert!(!rewards.is_empty());
        assert_eq!(rewards[0]["coins"].as_u64(), Some(coins));
        assert_eq!(rewards[0]["items"], json!([]));
        assert_eq!(rewards[0]["job_xp"], json!({ (profile.to_string()): xp }));
        assert!(sources(&rewards[0]));
        assert!(
            quest["source"]
                .as_array()
                .is_some_and(|source| source.iter().any(|tag| {
                    tag["kind"] == "reconstructed"
                        && tag["evidence"]
                            .as_str()
                            .is_some_and(|text| text.starts_with("(R) T241:"))
                })),
            "quest {id} carries its T241 (R) decision"
        );
    }
    // The crafting records stay out of this slice, and one known 3000020+ row is pinned
    // here so a concurrent rewrite of that range fails this suite.
    for id in [3_000_006, 3_000_007, YOUR_FIRST_WEAPON] {
        assert_eq!(
            record(id)["enabled"],
            json!(false),
            "quest {id} stays hollow"
        );
    }
    assert_eq!(
        record(UNTOUCHED_RECORD)["title_string_id"].as_u64(),
        Some(u64::from(UNTOUCHED_TITLE))
    );
    assert_eq!(
        record(UNTOUCHED_RECORD)["enabled"].as_bool(),
        Some(true),
        "the pinned 3000020+ record is unchanged"
    );

    // The kill target is a real always-on field population. T245's two level-5
    // MID-F camp wolves and T296's two level-11/12 HIGH-F wolves share the
    // definition, so production counts seven.
    assert_eq!(
        wolves().len(),
        7,
        "seven always-on 3156 wolves: three starter, the MID-F camp and the HIGH-F camp"
    );

    // The gather target exists in the Snowhill copper pool, and the shipped Mining18
    // board's outputs emit the same item definition the objective listens to.
    let mining = read_json(&content_dir().join("mining_nodes.json"));
    let pool = mining["pools"]
        .as_array()
        .unwrap()
        .iter()
        .find(|pool| pool["key"] == "snowhill-copper")
        .expect("the snowhill-copper pool");
    assert_eq!(pool["zone_definition_id"].as_u64(), Some(13));
    assert!(
        mining["spawns"]
            .as_array()
            .unwrap()
            .iter()
            .any(|spawn| spawn["pool_key"] == "snowhill-copper")
    );
    let board = read_json(&content_dir().join(mining::CONTENT_FILE));
    assert!(
        board["spec"]["outputs"]
            .as_array()
            .is_some_and(|outputs| outputs
                .iter()
                .any(|output| output["definition_id"].as_u64() == Some(u64::from(COPPER_ORE)))),
        "the shipped Mining18 board emits Copper Ore 2739"
    );

    // The giver is a data placement drawn by the production visibility loader.
    let cleetus = read_json(&content_dir().join("npcs/farnum_farmstead.json"))["records"]
        .as_array()
        .unwrap()
        .iter()
        .find(|npc| npc["npc_id"].as_u64() == Some(CLEETUS))
        .expect("Cleetus is a split NPC record")
        .clone();
    assert_eq!(cleetus["name"], "Cleetus Farnum");
    assert_eq!(cleetus["placement"], "legacy_data");
    let position = cleetus_position();
    let mut visibility = osfr_tools::world::NpcVisibility::load(&content_dir())
        .expect("NPC visibility content loads");
    let drawn = visibility.update(0, position).expect("visibility update");
    assert!(
        drawn.iter().any(|packet| {
            packet
                .windows(8)
                .any(|window| u64::from_le_bytes(window.try_into().unwrap()) == CLEETUS)
        }),
        "Cleetus Farnum is drawn at his spawn"
    );
}

#[test]
fn a_mid_quest_reconnect_keeps_progress_and_pays_no_extra() {
    let fixture = Fixture::new();
    let mut game = fixture.game(BRAWLER, &[ADVENTURER, MINER, BRAWLER]);
    let character = character_at(cleetus_position());
    let wolves = wolves();
    assert_eq!(
        wolves.len(),
        7,
        "three starter, two MID-F and two HIGH-F live wolves"
    );

    offer(&mut game, &character, CLEETUS, FIGHTING_THE_PACK, 1_000);
    accept(&mut game, &character, FIGHTING_THE_PACK, 1_100);
    let mut now = 2_000;
    for (guid, position) in wolves.iter().copied().take(2) {
        assert!(kill_wolf(&mut game, position, guid, &mut now));
    }
    assert_eq!(counts(&game, FIGHTING_THE_PACK), [2, 0]);
    let coins_before = coins(&game);
    let xp_before = profile_xp(&game, BRAWLER);
    let save_bytes = saved(&game);

    // Disconnect and reload through the production save envelope, not a state proxy.
    let mut game = fixture.save_and_reload(&game, &character);
    assert_eq!(
        quest_status(&game, FIGHTING_THE_PACK),
        QuestStatus::Active,
        "the kill progress survives the reload"
    );
    assert_eq!(counts(&game, FIGHTING_THE_PACK), [2, 0]);
    let quest_state = game.state().npc_quests.as_ref().unwrap().clone();
    assert_eq!(quest_state.pending_turnin, None);
    assert_eq!(quest_state.pending_offer, None);
    assert_eq!(coins(&game), coins_before, "the reload paid nothing");
    assert_eq!(profile_xp(&game, BRAWLER), xp_before);
    assert_eq!(
        saved(&game),
        save_bytes,
        "the reload round-trips the gameplay payload"
    );

    // Finish the hunt after the reload; the panel still starts from the restored
    // wolves. The first two kills were credited before the reconnect, so three
    // more reach the five-goal.
    for (guid, position) in wolves.iter().copied().take(3) {
        assert!(kill_wolf(&mut game, position, guid, &mut now));
    }
    assert_eq!(count(&game, FIGHTING_THE_PACK, 0), 5);
    let end = open_end_screen(
        &mut game,
        &character,
        CLEETUS,
        FIGHTING_THE_PACK,
        FIGHTING_END_TEXT,
        now + 100,
    );
    assert_end_reward(&end, FIGHTING_THE_PACK, 18, 25);
    finish_quest(
        &mut game,
        &character,
        FIGHTING_THE_PACK,
        BRAWLER,
        18,
        25,
        now + 200,
    );

    // A reconnect after the turn-in does not pay again.
    let mut game = fixture.save_and_reload(&game, &character);
    assert_eq!(
        quest_status(&game, FIGHTING_THE_PACK),
        QuestStatus::Completed
    );
    let after_turnin = saved(&game);
    let after_coins = coins(&game);
    assert_packet_changes_nothing(
        &mut game,
        &character,
        &npc_quest_end_reply(FIGHTING_THE_PACK),
        now + 300,
        "a turn-in replayed after reconnect",
    );
    assert_eq!(saved(&game), after_turnin);
    assert_eq!(coins(&game), after_coins);
}
