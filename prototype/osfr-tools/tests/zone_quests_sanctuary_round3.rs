//! T723: zone quests round 3, Sanctuary. Six of the twenty quests in
//! `lab/references/quest-lines/wiring/round3/sanctuary.json` run end to end on this tip and are
//! enabled in `data/content/quests/{chef,postman,warrior,wizard}.json`: Chef: Reach Level 5,
//! Become a Postman, Warrior: Achieve Level 15 and the Wizard line (Reach Level 10, Helping a
//! Friend, Reach Level 15). The other fourteen stay disabled;
//! `the_other_fourteen_still_have_a_blocker` names why and fails when a blocker lifts, so the
//! next builder wires that quest.
//!
//! Every test drives the production paths in `tests/support/npc_quest_chain.rs`: the 26/8 ->
//! 26/10 -> 26/6 offer (49/1, decoded with the production reader), the 49/2 accept,
//! `realm level` for the job_level goals, the Talk that holds the 49/13, the 26/6 that
//! releases it and the 49/14 turn-in. Objectives, text ids, the hand-in end text and the
//! rewards are read from the shipped quest records, never pinned here.
//!
//! Client readers (IDA, 2014 client; no new wire): 49/1 `0x00c7b7a0`, 49/2 `0x00a86400`, 49/9
//! `0x00c7af60`, 49/13 `QuestEndPacket::ReadFields` `0x00c7b990` (actor `6..14`, quest
//! `14..18`, end text `18..22`, one dword, then the reward bundle `0x008e7930`), 49/14
//! `0x00a864c0`, 49/15 Start quest (Jobs window) `0x00be6f20` list reader.

#[path = "support/npc_quest_chain.rs"]
mod npc_quest_chain;
#[path = "support/quest_records.rs"]
mod quest_records;
#[path = "support/quest_target_sets.rs"]
mod quest_target_sets;

use npc_quest_chain::{
    accept, assert_no_offer, assert_packet_changes_nothing, character_at_for, content_dir, counts,
    finish_quest, objective_complete, pending_offer, pending_turnin, quest_end_reply, quest_reply,
    quest_status, read_json, reference_content, release_end_screen, seed_active_profile,
    select_job, send, talk, zone_player, Fixture, ZoneQuest, ZoneQuests, ADVENTURER, CHARACTER,
};
use osfr_tools::{
    account_store::AccountRole,
    character_store::CharacterState,
    chat,
    gameplay::{Gameplay, Outcome},
    quest_engine::QuestStatus,
    world::NpcVisibility,
};
use quest_target_sets::Targets;
use serde_json::Value;
use std::collections::BTreeSet;

const BORDON: u64 = 9_000_000_028;
const FELIPE: u64 = 9_000_000_086;
const WHITTINGTON: u64 = 9_000_000_205;
const OSGOOD: u64 = 9_000_000_164;
const SANDBUR: u64 = 9_000_000_190;
const MAP: ZoneQuests = ZoneQuests {
    default_file: "sanctuary.json",
    files: &[],
    runtime: &[],
};
const OTHER: u64 = CHARACTER + 1;
const CHEF: i32 = 45;
const WARRIOR: i32 = 32;
const WIZARD: i32 = 12;

/// One enabled quest: its record file, giver, hand-in NPC and the job that must be active.
struct Spec {
    id: u32,
    file: &'static str,
    giver: u64,
    turnin: u64,
    job: i32,
}

const CHEF_5: Spec = Spec {
    id: 3_050_020,
    file: "chef.json",
    giver: BORDON,
    turnin: BORDON,
    job: CHEF,
};
const POSTMAN: Spec = Spec {
    id: 3_130_001,
    file: "postman.json",
    giver: FELIPE,
    turnin: FELIPE,
    job: ADVENTURER,
};
const WARRIOR_15: Spec = Spec {
    id: 3_150_016,
    file: "warrior.json",
    giver: WHITTINGTON,
    turnin: WHITTINGTON,
    job: WARRIOR,
};
const WIZARD_10: Spec = Spec {
    id: 3_160_017,
    file: "wizard.json",
    giver: OSGOOD,
    turnin: OSGOOD,
    job: WIZARD,
};
const FRIEND: Spec = Spec {
    id: 3_160_019,
    file: "wizard.json",
    giver: OSGOOD,
    turnin: SANDBUR,
    job: WIZARD,
};
const WIZARD_15: Spec = Spec {
    id: 3_160_024,
    file: "wizard.json",
    giver: OSGOOD,
    turnin: OSGOOD,
    job: WIZARD,
};
const SINGLES: [&Spec; 3] = [&CHEF_5, &POSTMAN, &WARRIOR_15];
const WIZARD_LINE: [&Spec; 3] = [&WIZARD_10, &FRIEND, &WIZARD_15];

impl Spec {
    /// The shipped quest record, for every value the tests must not pin.
    fn record(&self) -> Value {
        read_json(&content_dir().join("quests").join(self.file))["records"]
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row["id"].as_u64() == Some(u64::from(self.id)))
            .cloned()
            .unwrap_or_else(|| panic!("record {} ships", self.id))
    }

    fn text(&self, key: &str) -> u32 {
        self.record()[key].as_u64().unwrap_or(0) as u32
    }

    /// The coins and per-profile job XP of the record's one reward row.
    fn reward(&self) -> (u32, Vec<(i32, u32)>) {
        let row = self.record()["rewards"][0].clone();
        let xp = row["job_xp"]
            .as_object()
            .unwrap()
            .iter()
            .map(|(profile, xp)| (profile.parse().unwrap(), xp.as_u64().unwrap() as u32))
            .collect();
        (row["coins"].as_u64().unwrap() as u32, xp)
    }

    /// The quest as the shared harness walks it: card ids, hand-in text and rewards from the record.
    fn quest(&self) -> ZoneQuest {
        let (coins, xp) = self.reward();
        ZoneQuest {
            id: self.id,
            giver: self.giver,
            card: [
                "title_string_id",
                "intro_text_string_id",
                "description_string_id",
            ]
            .map(|key| self.text(key)),
            steps: &[],
            turnin: self.turnin,
            end_text: self.text("end_text_string_id"),
            coins,
            xp: xp.iter().map(|(_, xp)| xp).sum(),
        }
    }

    /// The profile the record's job XP is paid to (Adventurer when it pays none).
    fn profile(&self) -> i32 {
        self.reward()
            .1
            .first()
            .map_or(ADVENTURER, |(profile, _)| *profile)
    }

    /// Complete every objective but a final Talk, in record order: a rank goal through the
    /// developer rank command, a Talk through the NPC menu.
    fn walk(&self, game: &mut Gameplay, guid: u64, now: &mut u64) {
        let goals = self.record()["objectives"].as_array().unwrap().clone();
        for (index, goal) in goals.iter().enumerate() {
            let id = goal["id"].as_u64().unwrap() as u32;
            match goal["kind"].as_str().unwrap() {
                "job_level" => {
                    let level = goal["level"].as_u64().unwrap() as u32;
                    let reached = realm_level(game, &MAP.at(guid, self.giver), level, *now);
                    assert!(
                        objective_complete(&reached, self.id, id),
                        "quest {}: reaching rank {level} sends the existing 49/10",
                        self.id
                    );
                }
                "talk_to_npc" if index + 1 < goals.len() => {
                    let npc = goal["npc_id"].as_u64().unwrap();
                    assert!(
                        talk(game, &MAP.at(guid, npc), npc, *now).changed,
                        "quest {}: step {index}",
                        self.id
                    );
                }
                // The final Talk is the hand-in: `turn_in` sends it.
                "talk_to_npc" => {}
                other => panic!("quest {}: unexpected objective kind {other}", self.id),
            }
            *now += 10;
        }
    }
}

/// `realm level` (the job_level goals) is routed by the opening-aware chat handler, so every
/// quest here uses the full fixture, as T663 does.
fn realm_level(game: &mut Gameplay, at: &CharacterState, level: u32, now: u64) -> Outcome {
    game.set_dev_tools(true);
    game.set_role(AccountRole::Admin);
    send(
        game,
        at,
        &chat::local_request(&format!("realm level {level}")),
        now,
    )
}

#[test]
fn each_quest_is_offered_walked_and_paid_once() {
    for spec in SINGLES {
        let fixture = Fixture::new();
        let mut game = zone_player(&fixture, CHARACTER, spec.job);
        let mut now = 1_000;
        MAP.accept(&mut game, CHARACTER, &spec.quest(), &mut now);
        spec.walk(&mut game, CHARACTER, &mut now);
        MAP.turn_in(
            &mut game,
            CHARACTER,
            &spec.quest(),
            spec.profile(),
            &mut now,
        );

        // Replays and a second offer change nothing.
        let turnin = MAP.at(CHARACTER, spec.turnin);
        let duplicate = quest_end_reply(spec.id);
        assert_packet_changes_nothing(&mut game, &turnin, &duplicate, now, "a duplicate 49/14");
        let replay = quest_reply(spec.id, true);
        assert_packet_changes_nothing(&mut game, &turnin, &replay, now + 5, "a replayed 49/2");
        let giver = MAP.at(CHARACTER, spec.giver);
        assert_no_offer(&mut game, &giver, spec.giver, &mut now, "the giver after");
    }
}

#[test]
fn the_wizard_line_runs_in_order_and_each_step_pays() {
    let fixture = Fixture::new();
    let mut game = zone_player(&fixture, CHARACTER, WIZARD_10.job);
    let mut now = 1_000;
    // The follow-ups wait for the level-10 gate: a forged accept before it changes nothing.
    let osgood = MAP.at(CHARACTER, OSGOOD);
    for later in [&FRIEND, &WIZARD_15] {
        let forged = quest_reply(later.id, true);
        assert_packet_changes_nothing(&mut game, &osgood, &forged, now, "a forged 49/2");
    }
    for spec in WIZARD_LINE {
        MAP.accept(&mut game, CHARACTER, &spec.quest(), &mut now);
        spec.walk(&mut game, CHARACTER, &mut now);
        MAP.turn_in(
            &mut game,
            CHARACTER,
            &spec.quest(),
            spec.profile(),
            &mut now,
        );
    }
}

#[test]
fn only_the_quests_job_is_offered_and_forged_packets_change_nothing() {
    let fixture = Fixture::minimal(CHARACTER);
    // Another job active: no giver offers its job's quest, and a forged accept is refused.
    for spec in [&CHEF_5, &WARRIOR_15, &WIZARD_10, &FRIEND, &WIZARD_15] {
        let wrong = if spec.job == WIZARD { CHEF } else { WIZARD };
        let mut game = zone_player(&fixture, CHARACTER, wrong);
        let mut now = 1_000;
        let giver = MAP.at(CHARACTER, spec.giver);
        assert_no_offer(&mut game, &giver, spec.giver, &mut now, "another job");
        let forged = quest_reply(spec.id, true);
        assert_packet_changes_nothing(&mut game, &giver, &forged, now, "a forged 49/2");
        assert_eq!(pending_offer(&game), None);
    }
}

#[test]
fn short_of_the_gate_nothing_completes_or_pays() {
    for spec in [&CHEF_5, &WARRIOR_15, &WIZARD_10] {
        let fixture = Fixture::new();
        let mut game = zone_player(&fixture, CHARACTER, spec.job);
        let mut now = 1_000;
        MAP.accept(&mut game, CHARACTER, &spec.quest(), &mut now);
        assert!(counts(&game, spec.id).iter().all(|&count| count == 0));

        // A Talk at the hand-in with the rank goal open neither ends the quest nor offers another.
        let turnin = MAP.at(CHARACTER, spec.turnin);
        assert_no_offer(&mut game, &turnin, spec.turnin, &mut now, "the hand-in");
        assert_eq!(pending_turnin(&game), None);
        let early = quest_end_reply(spec.id);
        assert_packet_changes_nothing(&mut game, &turnin, &early, now, "an early 49/14");
        assert_eq!(quest_status(&game, spec.id), QuestStatus::Active);

        // One rank short of the gate never credits it.
        let gate = spec.record()["objectives"][0]["level"].as_u64().unwrap() as u32;
        realm_level(&mut game, &turnin, gate - 1, now + 20);
        assert_eq!(counts(&game, spec.id)[0], 0, "one rank short never credits");
        assert_no_offer(&mut game, &turnin, spec.turnin, &mut now, "the hand-in");
    }
}

#[test]
fn progress_survives_a_reconnect_and_players_are_independent() {
    let fixture = Fixture::new();
    let mut game = zone_player(&fixture, CHARACTER, WIZARD_10.job);
    let mut other = zone_player(&fixture, OTHER, WIZARD_10.job);
    let mut now = 1_000;
    MAP.accept(&mut game, CHARACTER, &WIZARD_10.quest(), &mut now);
    WIZARD_10.walk(&mut game, CHARACTER, &mut now);

    // The second player shares nothing with the first one's progress.
    MAP.accept(&mut other, OTHER, &WIZARD_10.quest(), &mut now);
    assert!(counts(&other, WIZARD_10.id).iter().all(|&count| count == 0));

    let mut game = fixture.reload_serialized(&game);
    assert_eq!(quest_status(&game, WIZARD_10.id), QuestStatus::Active);
    assert_eq!(counts(&game, WIZARD_10.id), [1, 0]);

    // The held end screen (49/13 shown, 49/14 not yet sent) also survives a save.
    let osgood = MAP.at(CHARACTER, OSGOOD);
    talk(&mut game, &osgood, OSGOOD, now);
    let end_text = WIZARD_10.text("end_text_string_id");
    release_end_screen(&mut game, &osgood, WIZARD_10.id, OSGOOD, end_text, now + 2);
    let mut game = fixture.reload_serialized(&game);
    assert_eq!(pending_turnin(&game), Some(WIZARD_10.id));
    let (paid, _) = WIZARD_10.reward();
    finish_quest(&mut game, &osgood, WIZARD_10.id, WIZARD, paid, 0, now + 10);

    assert_eq!(quest_status(&other, WIZARD_10.id), QuestStatus::Active);
    assert!(counts(&other, WIZARD_10.id).iter().all(|&count| count == 0));
}

/// The Jobs window's Start quest button (49/15) offers "Become a Postman": `job_browser.json`
/// maps original quest 1964 to this record.
#[test]
fn become_a_postman_starts_from_the_jobs_window() {
    let browser = read_json(&content_dir().join("job_browser.json"));
    let original = browser["records"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["regional_quest_id"].as_u64() == Some(u64::from(POSTMAN.id)))
        .map(|row| row["original_quest_id"].as_i64().unwrap() as i32)
        .expect("the Jobs window maps a Start quest id to Become a Postman");
    let fixture = Fixture::new();
    let mut game = zone_player(&fixture, CHARACTER, POSTMAN.job);
    let mut now = 1_000;
    let felipe = MAP.at(CHARACTER, FELIPE);
    let start = [&[49u8, 0, 15, 0, 0, 0][..], &original.to_le_bytes()].concat();
    assert!(send(&mut game, &felipe, &start, now).changed);
    assert_eq!(pending_offer(&game), Some(POSTMAN.id));
    now += 10;
    accept(&mut game, &felipe, POSTMAN.id, now);
    now += 10;
    POSTMAN.walk(&mut game, CHARACTER, &mut now);
    MAP.turn_in(
        &mut game,
        CHARACTER,
        &POSTMAN.quest(),
        POSTMAN.profile(),
        &mut now,
    );
}

/// Why a round-3 quest stays disabled.
enum Blocker {
    /// The giver has no drawn NPC row (absent, or a `poi_anchor` row).
    Giver,
    /// A prerequisite is itself disabled, so the quest could never be offered.
    Prerequisite,
    /// The spec's `finish_minigame` id has no server path that reports it
    /// (`tests/support/quest_target_sets.rs` lists every emitter).
    Minigame(u32),
    /// No player can make this job active: the profile has no content at all.
    NoProfile(u32),
    /// The job can be unlocked but the 26/13 switch is refused (no starter outfit).
    Unselectable(i32),
}

/// The fourteen round-3 quests that stay disabled each still have a blocker the server cannot
/// lift from data alone. When one lifts this test fails, and that quest is ready to wire.
#[test]
fn the_other_fourteen_still_have_a_blocker() {
    use Blocker::{Giver, Minigame, NoProfile, Prerequisite, Unselectable};
    const OFF: [(u32, Blocker); 14] = [
        (3_000_039, Giver),
        (3_020_031, Giver),
        (3_040_003, Unselectable(120)),
        (3_040_004, Minigame(500)),
        (3_040_009, Minigame(504)),
        (3_040_011, Minigame(507)),
        (3_050_018, Prerequisite),
        (3_070_026, Giver),
        (3_120_010, NoProfile(121)),
        (3_130_007, Minigame(376)),
        (3_150_014, Prerequisite),
        (3_150_015, Prerequisite),
        (3_160_016, Prerequisite),
        (3_160_033, Prerequisite),
    ];
    let root = content_dir();
    let visibility = NpcVisibility::load(&root).unwrap();
    let drawn: BTreeSet<u64> = visibility
        .teleport_targets(0)
        .into_iter()
        .filter_map(|target| visibility.content_key(target.guid))
        .collect();
    let targets = Targets::load(&root, &reference_content()).unwrap();
    let quests = quest_records::read_quests(&root);
    for (id, blocker) in OFF {
        let quest = quests
            .iter()
            .find(|quest| quest.id == id)
            .unwrap_or_else(|| panic!("record {id} ships"));
        assert!(!quest.enabled, "quest {id} is enabled, not blocked");
        let blocked = match blocker {
            Giver => !drawn.contains(&quest.giver_npc_id),
            Prerequisite => quest
                .prerequisite_quest_ids
                .iter()
                .any(|p| quests.iter().any(|q| q.id == *p && !q.enabled)),
            Minigame(game) => !targets.minigames.contains(&game),
            NoProfile(profile) => !targets.profiles.contains(&profile),
            Unselectable(profile) => {
                let fixture = Fixture::new();
                let mut state = fixture.game(None).state();
                seed_active_profile(&mut state, ADVENTURER, &[]);
                state.profiles.as_mut().unwrap().unlocked.insert(profile);
                let mut game = fixture.game(Some(state));
                let at = character_at_for(CHARACTER, 0, [0.0; 3]);
                send(&mut game, &at, &select_job(profile), 1_000);
                game.state().profiles.unwrap().active != profile
            }
        };
        assert!(blocked, "quest {id}: its blocker has lifted, wire it");
    }
}
