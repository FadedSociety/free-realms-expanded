//! T723/F751: zone quests round 3, Sanctuary. Five of the twenty quests in
//! `lab/references/quest-lines/wiring/round3/sanctuary.json` run end to end on this tip and are
//! enabled in `data/content/quests/{chef,warrior,wizard}.json`: Chef: Reach Level 5, Warrior:
//! Achieve Level 15 and the Wizard line (Reach Level 10, Helping a Friend, Reach Level 15).
//! Become a Postman (3130001) stays disabled: its cited reward is the Postman job unlock, which
//! no quest turn-in can grant (F751 finding 1), and its 49/1 intro and 49/13 end text are
//! unbound. The other fifteen stay disabled; `the_other_fifteen_still_have_a_blocker` lists
//! every blocker and fails when any named blocker lifts.
//!
//! Every test drives the production paths in `tests/support/npc_quest_chain.rs`: the 26/8 ->
//! 26/10 -> 26/6 offer (49/1, decoded with the production reader), the 49/2 accept,
//! `realm level` for the job_level goals, the Talk that holds the 49/13, the 26/6 that
//! releases it and the 49/14 turn-in. The job gate, giver, card ids, objectives, named turn-in,
//! end text and payout are read from the shipped records through
//! `tests/support/quest_records.rs`, never pinned here.
//!
//! Client readers (IDA, 2014 client; no new wire): 49/1 `0x00c7b7a0`, 49/2 `0x00a86400`, 49/9
//! `0x00c7af60`, 49/13 `QuestEndPacket::ReadFields` `0x00c7b990` (actor `6..14`, quest
//! `14..18`, end text `18..22`, one dword, then the reward bundle `0x008e7930`), 49/14
//! `0x00a864c0`. The Jobs window's 147/1 list reader is `0x00be6f20`; its 49/15 request is
//! `Ui.StartBreadcrumbQuest` `0x00c18aa0` -> sender `0x00a97f90`, header writer `0x00a8f860`
//! (`u16 49`, `u32 15`), payload writer `0x00a91730` (the `i32` id), ctor `0x00a86580`
//! (family 49 at `+4`, subtype 15 at `+8`, the quest id at `+0xc`), built by
//! `npc_quest_chain::breadcrumb_start` and decoded by `quest_journal::decode`.

#[path = "support/npc_quest_chain.rs"]
mod npc_quest_chain;
#[path = "support/quest_records.rs"]
mod quest_records;
#[path = "support/quest_target_sets.rs"]
mod quest_target_sets;

use npc_quest_chain::{
    ADVENTURER, CHARACTER, Fixture, ZoneQuest, ZoneQuests, assert_no_offer,
    assert_packet_changes_nothing, breadcrumb_start, character_at_for, coins, content_dir, counts,
    finish_quest, objective_complete, pending_offer, pending_turnin, quest_end_reply, quest_reply,
    quest_status, read_json, realm_level, reference_content, release_end_screen, saved,
    seed_active_profile, select_job, send, talk, zone_player,
};
use osfr_tools::{
    chat, gameplay::Gameplay, quest_engine::QuestStatus, quest_journal::BREADCRUMB_REFUSED,
    world::NpcVisibility,
};
use quest_records::{RawQuest, as_u32, read_quests};
use quest_target_sets::Targets;
use std::collections::BTreeSet;
use std::sync::OnceLock;

const CHEF: i32 = 45;
const WIZARD: i32 = 12;
const CHEF_5: u32 = 3_050_020;
const WARRIOR_15: u32 = 3_150_016;
const WIZARD_10: u32 = 3_160_017;
const FRIEND: u32 = 3_160_019;
const WIZARD_15: u32 = 3_160_024;
const POSTMAN_QUEST: u32 = 3_130_001;
const POSTMAN_PROFILE: i32 = 4;
const SINGLES: [u32; 2] = [CHEF_5, WARRIOR_15];
const WIZARD_LINE: [u32; 3] = [WIZARD_10, FRIEND, WIZARD_15];
const MAP: ZoneQuests = ZoneQuests {
    default_file: "sanctuary.json",
    files: &[],
    runtime: &[],
};
const OTHER: u64 = CHARACTER + 1;

/// The shipped quest records, read once through the shared record reader.
fn shipped_quests() -> &'static [RawQuest] {
    static RECORDS: OnceLock<Vec<RawQuest>> = OnceLock::new();
    RECORDS.get_or_init(|| read_quests(&content_dir()))
}

fn record(id: u32) -> &'static RawQuest {
    shipped_quests()
        .iter()
        .find(|quest| quest.id == id)
        .unwrap_or_else(|| panic!("the shipped quest record {id}"))
}

/// The record's job gate; a player on the Adventurer ladder when the record is ungated.
fn job(id: u32) -> i32 {
    record(id).job.unwrap_or(ADVENTURER)
}

/// The profile the record's job-XP reward names (the Adventurer when it pays none).
fn reward_profile(record: &RawQuest) -> i32 {
    record
        .rewards
        .iter()
        .filter_map(|row| row["job_xp"].as_object())
        .flat_map(|xp| xp.keys())
        .find_map(|profile| profile.parse().ok())
        .unwrap_or(ADVENTURER)
}

/// The last named Talk objective, where the engine opens the 49/13 end screen; the giver
/// when the record's final step is not a Talk.
fn named_turnin(record: &RawQuest) -> u64 {
    record
        .objectives
        .last()
        .filter(|goal| goal["kind"] == "talk_to_npc")
        .and_then(|goal| goal["npc_id"].as_u64())
        .unwrap_or(record.giver_npc_id)
}

/// The quest as the shared harness walks it, every value from the shipped record.
fn quest(id: u32) -> ZoneQuest {
    let record = record(id);
    let (coins, xp) = record.reward_totals(reward_profile(record));
    ZoneQuest {
        id,
        giver: record.giver_npc_id,
        card: [
            u32::try_from(record.title_string_id).expect("quest title id fits the client's u32"),
            record.intro_text_string_id,
            record.description_string_id,
        ],
        steps: &[],
        turnin: named_turnin(record),
        end_text: record.end_text_string_id,
        coins,
        xp,
    }
}

/// Complete every objective but a final Talk, in record order: a rank goal through the
/// developer rank command, a Talk through the NPC menu.
fn walk(game: &mut Gameplay, guid: u64, id: u32, now: &mut u64) {
    let record = record(id);
    let goals = &record.objectives;
    for (index, goal) in goals.iter().enumerate() {
        let goal_id = as_u32(&goal["id"]).unwrap();
        match goal["kind"].as_str().unwrap() {
            "job_level" => {
                let level = as_u32(&goal["level"]).unwrap();
                let reached = realm_level(game, &MAP.at(guid, record.giver_npc_id), level, *now);
                assert!(
                    objective_complete(&reached, id, goal_id),
                    "quest {id}: reaching rank {level} sends the existing 49/10"
                );
            }
            "talk_to_npc" if index + 1 < goals.len() => {
                let npc = goal["npc_id"].as_u64().unwrap();
                assert!(
                    talk(game, &MAP.at(guid, npc), npc, *now).changed,
                    "quest {id}: step {index}"
                );
            }
            // The final Talk is the hand-in: `turn_in` sends it.
            "talk_to_npc" => {}
            other => panic!("quest {id}: unexpected objective kind {other}"),
        }
        *now += 10;
    }
}

#[test]
fn each_quest_is_offered_walked_and_paid_once() {
    for id in SINGLES {
        let fixture = Fixture::new();
        let mut game = zone_player(&fixture, CHARACTER, job(id));
        let mut now = 1_000;
        MAP.accept(&mut game, CHARACTER, &quest(id), &mut now);
        walk(&mut game, CHARACTER, id, &mut now);
        MAP.turn_in(
            &mut game,
            CHARACTER,
            &quest(id),
            reward_profile(record(id)),
            &mut now,
        );

        // Replays and a second offer change nothing.
        let turnin = MAP.at(CHARACTER, named_turnin(record(id)));
        let duplicate = quest_end_reply(id);
        assert_packet_changes_nothing(&mut game, &turnin, &duplicate, now, "a duplicate 49/14");
        let replay = quest_reply(id, true);
        assert_packet_changes_nothing(&mut game, &turnin, &replay, now + 5, "a replayed 49/2");
        let giver = MAP.at(CHARACTER, record(id).giver_npc_id);
        assert_no_offer(
            &mut game,
            &giver,
            record(id).giver_npc_id,
            &mut now,
            "the giver after",
        );
    }
}

#[test]
fn the_wizard_line_runs_in_order_and_each_step_pays() {
    let fixture = Fixture::new();
    let mut game = zone_player(&fixture, CHARACTER, job(WIZARD_10));
    let mut now = 1_000;
    // The follow-ups wait for the level-10 gate: a forged accept before it changes nothing.
    let osgood = MAP.at(CHARACTER, record(WIZARD_10).giver_npc_id);
    for later in [FRIEND, WIZARD_15] {
        let forged = quest_reply(later, true);
        assert_packet_changes_nothing(&mut game, &osgood, &forged, now, "a forged 49/2");
    }
    for id in WIZARD_LINE {
        MAP.accept(&mut game, CHARACTER, &quest(id), &mut now);
        walk(&mut game, CHARACTER, id, &mut now);
        MAP.turn_in(
            &mut game,
            CHARACTER,
            &quest(id),
            reward_profile(record(id)),
            &mut now,
        );
    }
}

#[test]
fn only_the_quests_job_is_offered_and_forged_packets_change_nothing() {
    let fixture = Fixture::minimal(CHARACTER);
    // Another job active: no giver offers its job's quest, and a forged accept is refused.
    for id in [CHEF_5, WARRIOR_15, WIZARD_10, FRIEND, WIZARD_15] {
        let wrong = if job(id) == WIZARD { CHEF } else { WIZARD };
        let mut game = zone_player(&fixture, CHARACTER, wrong);
        let mut now = 1_000;
        let giver = MAP.at(CHARACTER, record(id).giver_npc_id);
        assert_no_offer(
            &mut game,
            &giver,
            record(id).giver_npc_id,
            &mut now,
            "another job",
        );
        let forged = quest_reply(id, true);
        assert_packet_changes_nothing(&mut game, &giver, &forged, now, "a forged 49/2");
        assert_eq!(pending_offer(&game), None);
    }
}

#[test]
fn short_of_the_gate_nothing_completes_or_pays() {
    for id in [CHEF_5, WARRIOR_15, WIZARD_10] {
        let fixture = Fixture::new();
        let mut game = zone_player(&fixture, CHARACTER, job(id));
        let mut now = 1_000;
        MAP.accept(&mut game, CHARACTER, &quest(id), &mut now);
        assert!(counts(&game, id).iter().all(|&count| count == 0));

        // A Talk at the hand-in with the rank goal open neither ends the quest nor offers another.
        let turnin = MAP.at(CHARACTER, named_turnin(record(id)));
        assert_no_offer(
            &mut game,
            &turnin,
            named_turnin(record(id)),
            &mut now,
            "the hand-in",
        );
        assert_eq!(pending_turnin(&game), None);
        let early = quest_end_reply(id);
        assert_packet_changes_nothing(&mut game, &turnin, &early, now, "an early 49/14");
        assert_eq!(quest_status(&game, id), QuestStatus::Active);

        // One rank short of the gate never credits it.
        let gate = as_u32(&record(id).objectives[0]["level"]).unwrap();
        realm_level(&mut game, &turnin, gate - 1, now + 20);
        assert_eq!(counts(&game, id)[0], 0, "one rank short never credits");
        assert_no_offer(
            &mut game,
            &turnin,
            named_turnin(record(id)),
            &mut now,
            "the hand-in",
        );
    }
}

#[test]
fn progress_survives_a_reconnect_and_players_are_independent() {
    let fixture = Fixture::new();
    let mut game = zone_player(&fixture, CHARACTER, job(WIZARD_10));
    let mut other = zone_player(&fixture, OTHER, job(WIZARD_10));
    let mut now = 1_000;
    MAP.accept(&mut game, CHARACTER, &quest(WIZARD_10), &mut now);
    walk(&mut game, CHARACTER, WIZARD_10, &mut now);

    // The second player shares nothing with the first one's progress.
    MAP.accept(&mut other, OTHER, &quest(WIZARD_10), &mut now);
    assert!(counts(&other, WIZARD_10).iter().all(|&count| count == 0));

    let mut game = fixture.reload_serialized(&game);
    assert_eq!(quest_status(&game, WIZARD_10), QuestStatus::Active);
    assert_eq!(counts(&game, WIZARD_10), [1, 0]);

    // The held end screen (49/13 shown, 49/14 not yet sent) also survives a save.
    let osgood = MAP.at(CHARACTER, record(WIZARD_10).giver_npc_id);
    talk(&mut game, &osgood, record(WIZARD_10).giver_npc_id, now);
    release_end_screen(
        &mut game,
        &osgood,
        WIZARD_10,
        record(WIZARD_10).giver_npc_id,
        record(WIZARD_10).end_text_string_id,
        now + 2,
    );
    let mut game = fixture.reload_serialized(&game);
    assert_eq!(pending_turnin(&game), Some(WIZARD_10));
    let paid = quest(WIZARD_10).coins;
    finish_quest(
        &mut game,
        &osgood,
        WIZARD_10,
        reward_profile(record(WIZARD_10)),
        paid,
        0,
        now + 10,
    );

    assert_eq!(quest_status(&other, WIZARD_10), QuestStatus::Active);
    assert!(counts(&other, WIZARD_10).iter().all(|&count| count == 0));
}

/// The Jobs window's Start quest button (49/15) maps original 1964 to Become a Postman.
/// The quest is disabled: its cited reward is the Postman job unlock, no quest-completion
/// path grants a profile (the Postman unlock is the trainer menu's), and its 49/1/49/13
/// texts are unbound.
/// F751 finding 1: the start is refused, pays nothing and unlocks nothing. When a
/// completion path grants profile 4, replace this pin with an end-to-end test that asserts
/// `profiles.unlocked.contains(&POSTMAN_PROFILE)` after the 49/14.
#[test]
fn the_jobs_window_postman_start_is_refused_and_unlocks_no_job() {
    let browser = read_json(&content_dir().join("job_browser.json"));
    let original = browser["records"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["regional_quest_id"].as_u64() == Some(u64::from(POSTMAN_QUEST)))
        .map(|row| row["original_quest_id"].as_i64().unwrap() as i32)
        .expect("the Jobs window maps a Start quest id to Become a Postman");
    let postman = record(POSTMAN_QUEST);
    assert!(
        !postman.enabled,
        "3130001 stays disabled until completion unlocks the job"
    );
    assert!(
        postman.rewards.is_empty(),
        "3130001 must not substitute a coin or item reward for the cited job unlock"
    );
    assert_eq!(
        postman.intro_text_string_id, 0,
        "3130001's 49/1 intro is unbound"
    );
    assert_eq!(
        postman.end_text_string_id, 0,
        "3130001's 49/13 end text is unbound"
    );

    let fixture = Fixture::new();
    let mut game = zone_player(&fixture, CHARACTER, ADVENTURER);
    let felipe = MAP.at(CHARACTER, postman.giver_npc_id);
    let before = saved(&game);
    let coins_before = coins(&game);
    let started = send(&mut game, &felipe, &breadcrumb_start(original), 1_000);
    assert!(
        !started.changed,
        "the disabled quest was started from the Jobs window"
    );
    assert_eq!(started.replies, vec![chat::system(BREADCRUMB_REFUSED)]);
    assert_eq!(saved(&game), before, "the refused 49/15 changed the save");
    assert_eq!(coins(&game), coins_before, "the refused 49/15 paid coins");
    assert!(
        !game
            .state()
            .profiles
            .as_ref()
            .unwrap()
            .unlocked
            .contains(&POSTMAN_PROFILE),
        "the refused 49/15 unlocked the Postman job"
    );
    assert_eq!(pending_offer(&game), None);
    assert!(
        !game
            .state()
            .npc_quests
            .as_ref()
            .unwrap()
            .quests
            .contains_key(&POSTMAN_QUEST),
        "the refused 49/15 added the quest"
    );
    assert_packet_changes_nothing(
        &mut game,
        &felipe,
        &quest_reply(POSTMAN_QUEST, true),
        1_010,
        "a forged accept for the never-offered quest",
    );
}

/// Why a round-3 quest stays disabled. Every reason is checked; any change to one stops
/// the test so the next builder re-checks the rest before wiring the quest.
#[derive(Clone, Copy, Debug)]
enum Blocker {
    /// The giver has no drawn NPC row (absent, a `poi_anchor` row, or unplaced).
    Giver,
    /// A prerequisite is itself disabled, so the quest could never be offered.
    Prerequisite,
    /// The spec's `finish_minigame` id has no server path that reports it
    /// (`tests/support/quest_target_sets.rs` lists every emitter).
    Minigame(u32),
    /// No player can make this job active: the profile has no content at all.
    NoProfile(u32),
    /// The record carries no gate for its intended job, so enabling it would offer the
    /// quest to every player while that job is also not selectable.
    NoJobGate(i32),
    /// The job can be unlocked but the 26/13 switch is refused (no starter outfit).
    Unselectable(i32),
    /// The cited reward is a job unlock and no quest-completion path grants one; a reward
    /// row here would be a substitute for it.
    JobUnlock(i32),
}

use Blocker::{Giver, JobUnlock, Minigame, NoJobGate, NoProfile, Prerequisite, Unselectable};

/// The fifteen round-3 quests that stay disabled. Each row lists every blocker that still
/// holds; the test fails when one lifts, so the next builder re-checks the others before
/// wiring it (some are one content edit away, others need a mechanic the server lacks).
#[test]
fn the_other_fifteen_still_have_a_blocker() {
    const OFF: [(u32, &[Blocker]); 15] = [
        (3_000_039, &[Giver, Prerequisite]),
        (3_020_031, &[Giver, Prerequisite]),
        (3_040_003, &[NoJobGate(120), Unselectable(120)]),
        (3_040_004, &[Minigame(500)]),
        (3_040_009, &[Minigame(504)]),
        (3_040_011, &[Minigame(507), Unselectable(120)]),
        (3_050_018, &[Prerequisite]),
        (3_070_026, &[Giver]),
        (3_120_010, &[NoJobGate(121), NoProfile(121)]),
        (3_130_001, &[JobUnlock(POSTMAN_PROFILE)]),
        (3_130_007, &[Minigame(376)]),
        (3_150_014, &[Prerequisite]),
        (3_150_015, &[Prerequisite]),
        (3_160_016, &[Prerequisite]),
        (3_160_033, &[Prerequisite]),
    ];
    let root = content_dir();
    let visibility = NpcVisibility::load(&root).unwrap();
    let drawn: BTreeSet<u64> = visibility
        .teleport_targets(0)
        .into_iter()
        .filter_map(|target| visibility.content_key(target.guid))
        .collect();
    let targets = Targets::load(&root, &reference_content()).unwrap();
    let quests = read_quests(&root);
    let profile_fixture = Fixture::new();
    for (id, blockers) in OFF {
        let quest = quests
            .iter()
            .find(|quest| quest.id == id)
            .unwrap_or_else(|| panic!("record {id} ships"));
        assert!(!quest.enabled, "quest {id} is enabled, not blocked");
        for &blocker in blockers {
            let held = match blocker {
                Giver => !drawn.contains(&quest.giver_npc_id),
                Prerequisite => quest
                    .prerequisite_quest_ids
                    .iter()
                    .any(|p| quests.iter().any(|q| q.id == *p && !q.enabled)),
                Minigame(game) => !targets.minigames.contains(&game),
                NoProfile(profile) => !targets.profiles.contains(&profile),
                NoJobGate(profile) => quest.job != Some(profile),
                Unselectable(profile) => {
                    let mut state = profile_fixture.game(None).state();
                    seed_active_profile(&mut state, ADVENTURER, &[]);
                    state.profiles.as_mut().unwrap().unlocked.insert(profile);
                    let mut game = profile_fixture.game(Some(state));
                    let at = character_at_for(CHARACTER, 0, [0.0; 3]);
                    let before = saved(&game);
                    let refused = send(&mut game, &at, &select_job(profile), 1_000);
                    assert!(!refused.changed, "quest {id}: job {profile} switched");
                    assert_eq!(
                        saved(&game),
                        before,
                        "quest {id}: the refused switch changed the save"
                    );
                    game.state().profiles.unwrap().active != profile
                }
                JobUnlock(profile) => {
                    !profile_fixture
                        .game(None)
                        .state()
                        .profiles
                        .unwrap()
                        .unlocked
                        .contains(&profile)
                        && quest.rewards.is_empty()
                }
            };
            assert!(
                held,
                "quest {id}: blocker {blocker:?} has lifted; re-check every blocker before wiring it"
            );
        }
    }
}
