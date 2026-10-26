//! T731: Stillwater Crossing round 3. `3000022` "Introduce Yourself" is the one
//! of the five quests the round-3 list names that a shipped server can run: its
//! giver and its single talk target are both drawn `legacy_data` rows in
//! `data/content/npcs/stillwater_crossing.json`, it has no prerequisite, and its
//! objective is the same one-talk referral shape as the two sibling versions.
//! This slice binds the client's Stillwater offer text (116460) as its
//! `intro_text_string_id`; the other four stay disabled, and the last test
//! proves why through the shipped records and the real menu.
//!
//! Every test drives the production paths in `tests/support/npc_quest_chain.rs`:
//! the 26/8 interact, 26/10 Talk row, 26/6 quest row, 49/1 offer (decoded with
//! the production reader), 49/2 accept, the Talk that credits the step and holds
//! the 49/13, the 26/6 that releases it and the 49/14 turn-in.
//!
//! Client readers (IDA, 2014 client; no new wire): 26/8 constructor `0x008b6bc0`,
//! 26/10 constructor `0x00a36f80`, 26/6 constructor `0x009f5890`; 49/1
//! `QuestInfoPacket` wrapper `0x00c7bb60` over `ReadFields` `0x00c7b7a0`, which
//! reads the 6-byte prefix, the quest id and five card dwords (offer text, title,
//! name, description, unused), a bool and the reward bundle; 49/13
//! `QuestEndPacket::ReadFields` `0x00c7b990` (actor `6..14`, quest `14..18`, end
//! text `18..22`, reward bundle `0x008e7930`); 49/14 constructor `0x00a864c0`.
//! The 49/1 card mapping is the one `npc_interaction.rs` records as natively
//! probe-verified (F267, reader `0x00c7b7a0`).

#[path = "support/npc_quest_chain.rs"]
mod npc_quest_chain;

use npc_quest_chain::{
    ADVENTURER, CHARACTER, Fixture, accept, assert_no_offer, assert_packet_changes_nothing,
    character_at_for, coins, content_dir, content_position, counts, end_screen_reward,
    finish_quest, offer, pending_turnin, profile_xp, quest_end_reply, quest_reply, quest_status,
    read_json, release_end_screen, saved, seed_active_profile, talk,
};
use osfr_tools::{
    character_store::CharacterState, gameplay::Gameplay, npc_interaction::decode_quest_offer_49_1,
    quest_engine::QuestStatus, world::NpcVisibility,
};
use serde_json::Value;

/// "Introduce Yourself" (Stillwater Crossing): giver and target rows in
/// `npcs/stillwater_crossing.json`.
const QUEST: u32 = 3_000_022;
const GIVER: u64 = 9_000_000_227;
const TARGET: u64 = 9_500_000_786;
/// The client's Stillwater offer text: Global.Text 116460, tagged `quest_dialog`
/// in the locale index (zone Stillwater Crossing, hint the talk target). It is
/// the Stillwater wording of the tip already bound as 3000021's intro 94246 and
/// 3000020's intro 104097.
const CLIENT_INTRO: u32 = 116_460;

/// The other four round-3 quests, each still disabled in its job file.
const BRAWLER_QUEST: u32 = 3_030_016;
const BRAWLER_TRAINER: u64 = 9_000_000_038;
const CHEF_QUEST: u32 = 3_050_016;
const CHEF_TRAINER: u64 = 9_000_000_103;
const FISHER_ENTRY: u32 = 3_070_001;
const FISHER_ADVANCE: u32 = 3_070_002;
/// The givers the two fisherman records name; neither id is an `npcs/*.json` row
/// (the guide calls them Message Board objects), so no menu can offer them.
const FISHER_ENTRY_GIVER: u64 = 9_000_000_148;
const FISHER_ADVANCE_GIVER: u64 = 9_000_000_149;
/// The placed fisherman trainer both talk objectives name.
const FISHER_TRAINER: u64 = 9_000_000_181;

const MINER: i32 = 14;
const OTHER: u64 = CHARACTER + 1;

/// The shipped quest record, for the values the tests must not pin (rewards,
/// title, description and end text).
fn record(file: &str, id: u32) -> Value {
    read_json(&content_dir().join("quests").join(file))["records"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["id"].as_u64() == Some(u64::from(id)))
        .cloned()
        .unwrap_or_else(|| panic!("quest {id} is defined in {file}"))
}

fn stillwater_at(guid: u64, npc: u64) -> CharacterState {
    character_at_for(guid, 0, content_position("stillwater_crossing.json", npc))
}

/// A character with `active` selected and the Adventurer ladder unlocked.
fn player(fixture: &Fixture, guid: u64, active: i32) -> Gameplay {
    let mut state = fixture.game_as(guid, None).state();
    seed_active_profile(&mut state, active, &[ADVENTURER]);
    fixture.game_as(guid, Some(state))
}

/// The offer's 26/3 row, 49/1 card and 49/2 accept through the real menu.
fn offered_and_accepted(game: &mut Gameplay, guid: u64, now: &mut u64) -> Value {
    let giver = stillwater_at(guid, GIVER);
    let offered = offer(game, &giver, GIVER, QUEST, *now);
    *now += 2;
    let card = offered
        .replies
        .iter()
        .find_map(|packet| decode_quest_offer_49_1(packet).ok())
        .expect("the 49/1 card decodes");
    let record = record("adventurer.json", QUEST);
    assert_eq!(
        card.name_string_id,
        record["title_string_id"].as_u64().unwrap() as u32,
        "the card title is the record's title"
    );
    assert_eq!(
        card.description_string_id,
        record["description_string_id"].as_u64().unwrap() as u32,
        "the card description is the record's objective text"
    );
    assert_eq!(
        card.intro_text_string_id, CLIENT_INTRO,
        "the card's offer text is the client's Stillwater offer line"
    );
    assert_eq!(
        card.intro_text_string_id,
        record["intro_text_string_id"].as_u64().unwrap() as u32,
        "the record carries the offer text the card sends"
    );
    accept(game, &giver, QUEST, *now);
    *now += 2;
    record
}

/// The Talk that credits the only objective and holds the end screen, the 26/6
/// that releases it (its reward bundle), and the 49/14 that pays it.
fn finished(game: &mut Gameplay, guid: u64, record: &Value, now: &mut u64) {
    let target = stillwater_at(guid, TARGET);
    let talked = talk(game, &target, TARGET, *now);
    assert!(
        talked.changed,
        "the stillwater target's Talk credits the step"
    );
    assert_eq!(counts(game, QUEST), [1]);
    assert_eq!(pending_turnin(game), Some(QUEST));
    *now += 2;
    let end_text = record["end_text_string_id"].as_u64().unwrap_or(0) as u32;
    let end = release_end_screen(game, &target, QUEST, TARGET, end_text, *now);
    *now += 2;
    let reward = &record["rewards"][0];
    let coins_due = reward["coins"].as_u64().unwrap() as u32;
    let xp_due = reward["job_xp"][ADVENTURER.to_string()].as_u64().unwrap() as u32;
    assert_eq!(end_screen_reward(&end), (coins_due, xp_due));
    finish_quest(game, &target, QUEST, ADVENTURER, coins_due, xp_due, *now);
    *now += 2;
}

#[test]
fn introduce_yourself_offers_walks_and_pays_from_its_record() {
    let fixture = Fixture::minimal(CHARACTER);
    let mut game = player(&fixture, CHARACTER, ADVENTURER);
    let (coins_before, xp_before) = (coins(&game), profile_xp(&game, ADVENTURER));
    let mut now = 1_000;
    let record = offered_and_accepted(&mut game, CHARACTER, &mut now);
    assert_eq!(quest_status(&game, QUEST), QuestStatus::Active);
    assert_eq!(counts(&game, QUEST), [0]);

    // The end screen is held at the actor the objective names, and the reward
    // paid is the record's own row.
    finished(&mut game, CHARACTER, &record, &mut now);
    let reward = &record["rewards"][0];
    assert_eq!(
        coins(&game),
        coins_before + reward["coins"].as_u64().unwrap() as u32
    );
    assert_eq!(
        profile_xp(&game, ADVENTURER),
        xp_before + reward["job_xp"][ADVENTURER.to_string()].as_u64().unwrap() as u32
    );
    assert_eq!(quest_status(&game, QUEST), QuestStatus::Completed);

    // A replayed 49/14 and a second offer change nothing.
    assert_packet_changes_nothing(
        &mut game,
        &stillwater_at(CHARACTER, TARGET),
        &quest_end_reply(QUEST),
        now,
        "a replayed 49/14",
    );
    assert_no_offer(
        &mut game,
        &stillwater_at(CHARACTER, GIVER),
        GIVER,
        &mut now,
        "the giver after the turn-in",
    );
}

#[test]
fn every_refused_request_changes_nothing() {
    let fixture = Fixture::minimal(CHARACTER);
    let mut game = player(&fixture, CHARACTER, ADVENTURER);
    let giver = stillwater_at(CHARACTER, GIVER);
    let target = stillwater_at(CHARACTER, TARGET);
    let mut now = 1_000;

    // The target offers nothing before the referral exists.
    assert_no_offer(
        &mut game,
        &target,
        TARGET,
        &mut now,
        "the target before the offer",
    );
    // A 49/2 for a quest the giver never offered is refused.
    assert_packet_changes_nothing(
        &mut game,
        &giver,
        &quest_reply(QUEST, true),
        now,
        "a forged 49/2",
    );

    let record = offered_and_accepted(&mut game, CHARACTER, &mut now);
    // The objective is open: a 49/14 settles nothing and the target's first Talk
    // must still be the one that credits it.
    assert_packet_changes_nothing(
        &mut game,
        &target,
        &quest_end_reply(QUEST),
        now,
        "a 49/14 before the objective",
    );
    assert_eq!(counts(&game, QUEST), [0]);
    finished(&mut game, CHARACTER, &record, &mut now);
    assert_eq!(quest_status(&game, QUEST), QuestStatus::Completed);
}

#[test]
fn progress_survives_reconnect() {
    let fixture = Fixture::minimal(CHARACTER);
    let mut game = player(&fixture, CHARACTER, ADVENTURER);
    let mut now = 1_000;
    let record = offered_and_accepted(&mut game, CHARACTER, &mut now);

    let mut game = fixture.reload_serialized(&game);
    assert_eq!(quest_status(&game, QUEST), QuestStatus::Active);
    assert_eq!(counts(&game, QUEST), [0]);
    finished(&mut game, CHARACTER, &record, &mut now);

    // The completed quest and its reward survive a second reconnect.
    let mut game = fixture.reload_serialized(&game);
    assert_eq!(quest_status(&game, QUEST), QuestStatus::Completed);
    assert_no_offer(
        &mut game,
        &stillwater_at(CHARACTER, GIVER),
        GIVER,
        &mut now,
        "the giver after a reconnect",
    );
}

#[test]
fn two_players_are_independent() {
    let fixture = Fixture::minimal(CHARACTER);
    let mut a = player(&fixture, CHARACTER, ADVENTURER);
    let mut b = player(&fixture, OTHER, ADVENTURER);
    let mut now = 1_000;
    let record = offered_and_accepted(&mut a, CHARACTER, &mut now);
    offered_and_accepted(&mut b, OTHER, &mut now);
    let b_before = saved(&b);

    finished(&mut a, CHARACTER, &record, &mut now);
    assert_eq!(saved(&b), b_before, "A's run changed B");
    assert_eq!(quest_status(&b, QUEST), QuestStatus::Active);
    assert_eq!(counts(&b, QUEST), [0], "A's Talk credited B's step");
}

#[test]
fn xp_goes_to_adventurer_with_another_job_active() {
    let fixture = Fixture::minimal(CHARACTER);
    let mut game = player(&fixture, CHARACTER, MINER);
    let miner_before = profile_xp(&game, MINER);
    let adventurer_before = profile_xp(&game, ADVENTURER);
    let mut now = 1_000;
    let record = offered_and_accepted(&mut game, CHARACTER, &mut now);
    finished(&mut game, CHARACTER, &record, &mut now);
    assert_eq!(
        profile_xp(&game, MINER),
        miner_before,
        "the referral must not pay the active job"
    );
    assert_eq!(
        profile_xp(&game, ADVENTURER),
        adventurer_before
            + record["rewards"][0]["job_xp"][ADVENTURER.to_string()]
                .as_u64()
                .unwrap() as u32
    );
}

#[test]
fn the_four_round_three_quests_without_a_runnable_chain_stay_disabled() {
    // The shipped records: each disabled, and each blocker named by the record
    // it points at.
    for (file, id) in [
        ("brawler.json", BRAWLER_QUEST),
        ("chef.json", CHEF_QUEST),
        ("fisherman.json", FISHER_ENTRY),
        ("fisherman.json", FISHER_ADVANCE),
    ] {
        let row = record(file, id);
        assert_eq!(
            row["enabled"].as_bool(),
            Some(false),
            "quest {id} must stay disabled until its chain can run"
        );
    }
    for (file, id, prerequisite) in [
        ("brawler.json", BRAWLER_QUEST, 3_030_015),
        ("chef.json", CHEF_QUEST, 3_050_015),
        ("fisherman.json", FISHER_ADVANCE, FISHER_ENTRY),
    ] {
        let row = record(file, id);
        assert!(
            row["prerequisite_quest_ids"]
                .as_array()
                .unwrap()
                .iter()
                .any(|value| value.as_u64() == Some(u64::from(prerequisite))),
            "quest {id} keeps its authored prerequisite {prerequisite}"
        );
        let blocking = record(file, prerequisite);
        assert_eq!(
            blocking["enabled"].as_bool(),
            Some(false),
            "quest {id} is unreachable while its prerequisite {prerequisite} is disabled"
        );
    }

    // The two fisherman records name givers that are not authored NPC rows, so
    // no drawable actor can ever open their menu. The placed trainer's Talk
    // offers neither, and a forged accept changes nothing.
    let visibility = NpcVisibility::load(&content_dir()).expect("NPC visibility loads");
    for giver in [FISHER_ENTRY_GIVER, FISHER_ADVANCE_GIVER] {
        assert_eq!(
            visibility.content_key(giver),
            None,
            "giver {giver} must not be a drawn NPC row"
        );
    }

    let fixture = Fixture::minimal(CHARACTER);
    let mut game = player(&fixture, CHARACTER, ADVENTURER);
    let mut now = 1_000;
    // The two trainers whose quests are disabled are still talkable; neither
    // offers anything and the open menu survives the refused Talk.
    for (npc, label) in [
        (BRAWLER_TRAINER, "brawler trainer"),
        (CHEF_TRAINER, "chef trainer"),
    ] {
        assert_no_offer(
            &mut game,
            &stillwater_at(CHARACTER, npc),
            npc,
            &mut now,
            label,
        );
    }
    let trainer = stillwater_at(CHARACTER, FISHER_TRAINER);
    assert_no_offer(
        &mut game,
        &trainer,
        FISHER_TRAINER,
        &mut now,
        "fisherman trainer",
    );
    for quest in [BRAWLER_QUEST, CHEF_QUEST, FISHER_ENTRY, FISHER_ADVANCE] {
        assert_packet_changes_nothing(
            &mut game,
            &trainer,
            &quest_reply(quest, true),
            now,
            &format!("a forged 49/2 for disabled quest {quest}"),
        );
    }
}
