//! T731/F747: Stillwater Crossing round 3. `3000022` "Introduce Yourself" is the
//! one of the five quests the round-3 list names that a shipped server can run: its
//! giver and its single talk target are both drawn `legacy_data` rows in
//! `data/content/npcs/stillwater_crossing.json`, it has no prerequisite, and its
//! objective is the same one-talk referral shape as the two sibling versions.
//! This slice binds the client's Stillwater offer text (116460) as its
//! `intro_text_string_id`; the other four stay disabled, and the last test
//! proves why through the shipped records.
//!
//! Every test drives the production paths in `tests/support/npc_quest_chain.rs`:
//! the 26/8 interact, 26/10 Talk row, 26/6 quest row, 49/1 offer (decoded with
//! the production reader), 49/2 accept, the Talk that credits the step and holds
//! the 49/13, the 26/6 that releases it and the 49/14 turn-in. Card ids, end text
//! and payout are read from the shipped record through `tests/support/quest_records.rs`.
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
#[path = "support/quest_records.rs"]
mod quest_records;

use npc_quest_chain::{
    ADVENTURER, CHARACTER, Fixture, ZoneQuest, ZoneQuests, assert_no_offer,
    assert_packet_changes_nothing, content_dir, counts, profile_xp, quest_end_reply, quest_reply,
    quest_status, saved, zone_player,
};
use osfr_tools::{quest_engine::QuestStatus, world::NpcVisibility};
use quest_records::{RawQuest, read_quests};

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
const CHEF_QUEST: u32 = 3_050_016;
const FISHER_ENTRY: u32 = 3_070_001;
const FISHER_ADVANCE: u32 = 3_070_002;
/// The givers the two fisherman records name; neither id is an `npcs/*.json` row
/// (the guide calls them Message Board objects), so no menu can offer them.
const FISHER_ENTRY_GIVER: u64 = 9_000_000_148;
const FISHER_ADVANCE_GIVER: u64 = 9_000_000_149;

/// Every row these tests click is in `stillwater_crossing.json`, and no NPC has an
/// extra runtime spawn.
const MAP: ZoneQuests = ZoneQuests {
    default_file: "stillwater_crossing.json",
    files: &[],
    runtime: &[],
};

const MINER: i32 = 14;
const OTHER: u64 = CHARACTER + 1;

/// The shipped record, for the values these tests must not pin (rewards, title,
/// description and end text).
fn record(id: u32) -> RawQuest {
    read_quests(&content_dir())
        .into_iter()
        .find(|quest| quest.id == id)
        .unwrap_or_else(|| panic!("the shipped quest record {id}"))
}

/// The quest as the shared harness walks it: card ids, end text and payout (coins,
/// Adventurer XP) read from the shipped record.
fn quest() -> ZoneQuest {
    let shipped = record(QUEST);
    let (coins, xp) = shipped.reward_totals(ADVENTURER);
    ZoneQuest {
        id: QUEST,
        giver: GIVER,
        card: [
            u32::try_from(shipped.title_string_id).expect("title id fits u32"),
            shipped.intro_text_string_id,
            shipped.description_string_id,
        ],
        steps: &[],
        turnin: TARGET,
        end_text: shipped.end_text_string_id,
        coins,
        xp: shipped.reward_stars(),
        profile_xp: xp,
    }
}

#[test]
fn introduce_yourself_offers_walks_and_pays_from_its_record() {
    let fixture = Fixture::minimal(CHARACTER);
    let mut game = zone_player(&fixture, CHARACTER, ADVENTURER);
    let quest = quest();
    assert_eq!(
        record(QUEST).intro_text_string_id,
        CLIENT_INTRO,
        "the record binds the client's Stillwater offer text"
    );
    let mut now = 1_000;
    MAP.accept(&mut game, CHARACTER, &quest, &mut now);
    assert_eq!(quest_status(&game, QUEST), QuestStatus::Active);
    assert_eq!(counts(&game, QUEST), [0]);

    // The one Talk credits the step, its 49/13 pays the record's row, and the end
    // screen is held at the target the objective names.
    MAP.turn_in(&mut game, CHARACTER, &quest, ADVENTURER, &mut now);
    assert_eq!(counts(&game, QUEST), [1]);
    assert_eq!(quest_status(&game, QUEST), QuestStatus::Completed);

    // A replayed 49/14 and a second offer change nothing.
    assert_packet_changes_nothing(
        &mut game,
        &MAP.at(CHARACTER, TARGET),
        &quest_end_reply(QUEST),
        now,
        "a replayed 49/14",
    );
    assert_no_offer(
        &mut game,
        &MAP.at(CHARACTER, GIVER),
        GIVER,
        &mut now,
        "the giver after the turn-in",
    );
}

#[test]
fn every_refused_request_changes_nothing() {
    let fixture = Fixture::minimal(CHARACTER);
    let mut game = zone_player(&fixture, CHARACTER, ADVENTURER);
    let quest = quest();
    let (giver, target) = (MAP.at(CHARACTER, GIVER), MAP.at(CHARACTER, TARGET));
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

    MAP.accept(&mut game, CHARACTER, &quest, &mut now);
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
    MAP.turn_in(&mut game, CHARACTER, &quest, ADVENTURER, &mut now);
    assert_eq!(quest_status(&game, QUEST), QuestStatus::Completed);
}

#[test]
fn progress_survives_reconnect() {
    let fixture = Fixture::minimal(CHARACTER);
    let mut game = zone_player(&fixture, CHARACTER, ADVENTURER);
    let quest = quest();
    let mut now = 1_000;
    MAP.accept(&mut game, CHARACTER, &quest, &mut now);

    let mut game = fixture.reload_serialized(&game);
    assert_eq!(quest_status(&game, QUEST), QuestStatus::Active);
    assert_eq!(counts(&game, QUEST), [0]);
    MAP.turn_in(&mut game, CHARACTER, &quest, ADVENTURER, &mut now);

    // The completed quest and its reward survive a second reconnect.
    let mut game = fixture.reload_serialized(&game);
    assert_eq!(quest_status(&game, QUEST), QuestStatus::Completed);
    assert_no_offer(
        &mut game,
        &MAP.at(CHARACTER, GIVER),
        GIVER,
        &mut now,
        "the giver after a reconnect",
    );
}

#[test]
fn two_players_are_independent() {
    let fixture = Fixture::minimal(CHARACTER);
    let quest = quest();
    let mut a = zone_player(&fixture, CHARACTER, ADVENTURER);
    let mut b = zone_player(&fixture, OTHER, ADVENTURER);
    let mut now = 1_000;
    MAP.accept(&mut a, CHARACTER, &quest, &mut now);
    MAP.accept(&mut b, OTHER, &quest, &mut now);
    let b_before = saved(&b);

    MAP.turn_in(&mut a, CHARACTER, &quest, ADVENTURER, &mut now);
    assert_eq!(saved(&b), b_before, "A's run changed B");
    assert_eq!(quest_status(&b, QUEST), QuestStatus::Active);
    assert_eq!(counts(&b, QUEST), [0], "A's Talk credited B's step");
}

#[test]
fn xp_goes_to_adventurer_with_another_job_active() {
    let fixture = Fixture::minimal(CHARACTER);
    let quest = quest();
    let mut game = zone_player(&fixture, CHARACTER, MINER);
    let miner_before = profile_xp(&game, MINER);
    let adventurer_before = profile_xp(&game, ADVENTURER);
    let mut now = 1_000;
    MAP.accept(&mut game, CHARACTER, &quest, &mut now);
    MAP.turn_in(&mut game, CHARACTER, &quest, ADVENTURER, &mut now);
    assert_eq!(
        profile_xp(&game, MINER),
        miner_before,
        "the referral must not pay the active job"
    );
    assert_eq!(
        profile_xp(&game, ADVENTURER),
        adventurer_before + quest.profile_xp
    );
}

#[test]
fn the_four_round_three_quests_without_a_runnable_chain_stay_disabled() {
    // Each disabled, and each blocker named by the record it points at.
    for (id, prerequisite) in [
        (BRAWLER_QUEST, Some(3_030_015)),
        (CHEF_QUEST, Some(3_050_015)),
        (FISHER_ENTRY, None),
        (FISHER_ADVANCE, Some(FISHER_ENTRY)),
    ] {
        let shipped = record(id);
        assert!(
            !shipped.enabled,
            "quest {id} must stay disabled until its chain can run"
        );
        if let Some(prerequisite) = prerequisite {
            assert!(
                shipped.prerequisite_quest_ids.contains(&prerequisite),
                "quest {id} keeps its authored prerequisite {prerequisite}"
            );
            assert!(
                !record(prerequisite).enabled,
                "quest {id} is unreachable while its prerequisite {prerequisite} is disabled"
            );
        }
    }

    // The two fisherman records name givers that are not authored NPC rows, so no
    // drawable actor can ever open their menu.
    let visibility = NpcVisibility::load(&content_dir()).expect("NPC visibility loads");
    for giver in [FISHER_ENTRY_GIVER, FISHER_ADVANCE_GIVER] {
        assert_eq!(
            visibility.content_key(giver),
            None,
            "giver {giver} must not be a drawn NPC row"
        );
    }
}
