//! W175 (Next 10 brief #5, second half): the 49/15 breadcrumb crosswalk.
//!
//! The client sends the ORIGINAL quest id of a world-map point of interest. The regional
//! catalogue does not use those ids, so `data/content/breadcrumb_quests.json` maps them to
//! regional quests and anything unlisted is refused (R). These tests pin the loader, the
//! shipped file and the journal behaviour behind it; the gateway route is covered by the
//! unit tests in `gameplay.rs` (see `lab/runs/claude/W175/apply.py`).

use osfr_tools::{
    quest_engine::{
        CharacterEligibility, ObjectiveKind, OutboundMessage, QuestDefinition, QuestDenial,
        QuestEngine, QuestMessage, QuestObjective, QuestOutcome, QuestRequirements, QuestStatus,
    },
    quest_journal::{Crosswalk, JournalRequest, handle, load_crosswalk, translate},
    rewards::{RewardDisplay, RewardGrant},
};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU32, Ordering},
};

const CHARACTER: u64 = 900;
const CANARY_COINS: u32 = 88_001;

fn bundled_content() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content")
}

struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        static SERIAL: AtomicU32 = AtomicU32::new(0);
        let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!(
            "w175-{}-{}",
            std::process::id(),
            SERIAL.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&dir).unwrap();
        Self(dir)
    }
    fn write(&self, document: &Value) -> &Path {
        fs::write(self.0.join("breadcrumb_quests.json"), document.to_string()).unwrap();
        &self.0
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn record(original: u32, regional: Option<u32>) -> Value {
    json!({
        "original_quest_id": original,
        "poi_ids": [7],
        "poi_name": "Synthetic",
        "regional_quest_id": regional,
        "source": [{"kind": "reconstructed", "evidence": "(R) synthetic test row"}]
    })
}

fn file(records: Vec<Value>) -> Value {
    json!({"schema_version": 1, "records": records})
}

#[test]
fn the_shipped_crosswalk_loads_and_points_at_real_quests() {
    let content = bundled_content();
    let crosswalk = load_crosswalk(&content).unwrap();
    let document: Value =
        serde_json::from_slice(&fs::read(content.join("breadcrumb_quests.json")).unwrap()).unwrap();
    let rows = document["records"].as_array().unwrap();
    let originals: BTreeSet<u32> = rows
        .iter()
        .map(|row| row["original_quest_id"].as_u64().unwrap() as u32)
        .collect();
    assert_eq!(
        originals,
        BTreeSet::from([
            1799, 1801, 1802, 1803, 1804, 1805, 1806, 1807, 1808, 1809, 1810, 1811, 1812, 1813,
            1814, 1815, 1880, 1881,
        ])
    );
    assert_eq!(rows.len(), 18);
    let mut points = BTreeSet::new();
    for row in rows {
        for point in row["poi_ids"].as_array().unwrap() {
            assert!(points.insert(point.as_u64().unwrap() as u32));
        }
    }
    // The startup snapshot names 43 POIs carrying these 18 non-zero original quest ids.
    assert_eq!(points.len(), 43);
    // Only title/activity matches are mapped. 1811 is authored below; the other three
    // targets stay hollow/disabled, so the regular offer path still refuses them.
    assert_eq!(
        crosswalk,
        BTreeMap::from([
            (1805, 3_050_009),
            (1808, 3_050_013),
            (1811, 3_050_004),
            (1881, 3_140_001),
        ])
    );
    let mut catalogue = BTreeSet::new();
    let mut offerable = BTreeMap::new();
    let mut cooking_starter = None;
    for entry in fs::read_dir(content.join("quests")).unwrap() {
        let document: Value =
            serde_json::from_slice(&fs::read(entry.unwrap().path()).unwrap()).unwrap();
        for quest in document["records"].as_array().unwrap() {
            if let Some(id) = quest["id"].as_u64() {
                catalogue.insert(id);
                offerable.insert(
                    id,
                    (
                        quest["enabled"].as_bool().unwrap_or(true),
                        quest["objectives"].as_array().map_or(0, Vec::len),
                    ),
                );
                if id == 3_050_004 {
                    cooking_starter = Some(quest.clone());
                }
            }
        }
    }
    for (original, regional) in &crosswalk {
        assert!(*original > 0 && *original <= i32::MAX as u32);
        assert!(
            catalogue.contains(&u64::from(*regional)),
            "original {original} maps to missing quest {regional}"
        );
        // The two id spaces must not overlap, or a regional id could pass as an original one.
        assert!(!catalogue.contains(&u64::from(*original)));
    }
    // The Cooking Table starter is reachable, not merely a title-to-id mapping:
    // it has an ordered journey to the trainer table and a content-defined 45/11 goal.
    let starter = cooking_starter.expect("Cooking 101 exists");
    assert_ne!(starter["enabled"], json!(false));
    assert_eq!(starter["objectives"][0]["kind"], "visit_position");
    assert_eq!(starter["objectives"][1]["kind"], "client_reported");
    assert_eq!(
        starter["objectives"][1]["trigger"],
        json!({"type": "first_movement", "mouse": false})
    );
    // The crosswalk entries that reach a real, enabled quest: the Cooking starter (1811)
    // and, since T542, the Soccer Star POI 1881 -> 3140001 (the same record the Jobs window
    // maps 1965 to). Every other target must stay hollow, so a mapping to an unbuilt record
    // cannot be started from a POI.
    const SUPPORTED: [u32; 2] = [1811, 1881];
    for (original, regional) in &crosswalk {
        if !SUPPORTED.contains(original) {
            let (enabled, objectives) = offerable[&u64::from(*regional)];
            assert!(
                !enabled || objectives == 0,
                "unsupported mapping {original} -> {regional} must stay unavailable"
            );
        }
    }
}

#[test]
fn every_shipped_row_names_its_points_and_says_how_it_was_decided() {
    let document: Value = serde_json::from_slice(
        &fs::read(bundled_content().join("breadcrumb_quests.json")).unwrap(),
    )
    .unwrap();
    for row in document["records"].as_array().unwrap() {
        let kinds: Vec<_> = row["source"]
            .as_array()
            .unwrap()
            .iter()
            .map(|tag| tag["kind"].as_str().unwrap())
            .collect();
        assert!(kinds.contains(&"legacy-data") && kinds.contains(&"client"));
        // Every id-to-quest decision is ours, so it carries an (R) tag.
        assert!(row["source"].as_array().unwrap().iter().any(|tag| {
            tag["kind"] == "reconstructed" && tag["evidence"].as_str().unwrap().starts_with("(R)")
        }));
    }
}

#[test]
fn a_missing_file_is_an_empty_crosswalk() {
    let scratch = Scratch::new();
    assert!(load_crosswalk(&scratch.0).unwrap().is_empty());
}

#[test]
fn unmapped_rows_are_kept_out_of_the_map() {
    let scratch = Scratch::new();
    let dir = scratch.write(&file(vec![record(1807, Some(10)), record(1808, None)]));
    assert_eq!(load_crosswalk(dir).unwrap(), BTreeMap::from([(1807, 10)]));
}

#[test]
fn malformed_crosswalks_are_errors_not_silent_empties() {
    let mut extra = record(1807, Some(10));
    extra["surprise"] = json!(1);
    let mut no_source = record(1807, Some(10));
    no_source["source"] = json!([]);
    let mut no_points = record(1807, Some(10));
    no_points["poi_ids"] = json!([]);
    let mut zero_point = record(1807, Some(10));
    zero_point["poi_ids"] = json!([0]);
    let mut duplicate_point = record(1807, Some(10));
    duplicate_point["poi_ids"] = json!([7, 7]);
    let mut no_name = record(1807, Some(10));
    no_name["poi_name"] = json!("");
    let mut missing_r = record(1807, Some(10));
    missing_r["source"][0]["evidence"] = json!("not marked reconstructed");
    let bad = [
        file(vec![record(1807, Some(10)), record(1807, Some(11))]),
        file(vec![record(0, Some(10))]),
        file(vec![record(1807, Some(0))]),
        file(vec![record(u32::MAX, Some(10))]),
        file(vec![record(1807, Some(u32::MAX))]),
        file(vec![extra]),
        file(vec![no_source]),
        file(vec![no_points]),
        file(vec![zero_point]),
        file(vec![duplicate_point]),
        file(vec![no_name]),
        file(vec![missing_r]),
        json!({"schema_version": 2, "records": []}),
        json!({"records": []}),
        json!({"schema_version": 1, "records": [], "extra": 1}),
        json!([1, 2]),
    ];
    for document in bad {
        let scratch = Scratch::new();
        assert!(
            load_crosswalk(scratch.write(&document)).is_err(),
            "accepted {document}"
        );
    }
    let scratch = Scratch::new();
    fs::write(scratch.0.join("breadcrumb_quests.json"), b"{ not json").unwrap();
    assert!(load_crosswalk(&scratch.0).is_err());
}

#[test]
fn only_breadcrumb_starts_are_rewritten() {
    let crosswalk: Crosswalk = BTreeMap::from([(1807, 10)]);
    assert_eq!(
        translate(JournalRequest::Breadcrumb { quest_id: 1807 }, &crosswalk),
        Some(JournalRequest::Breadcrumb { quest_id: 10 })
    );
    // An unlisted original id, and a regional id used as if it were an original one.
    for id in [1808, 10, 3_000_021] {
        assert_eq!(
            translate(JournalRequest::Breadcrumb { quest_id: id }, &crosswalk),
            None
        );
    }
    // Select and lock carry ids the server sent, so they pass through untouched.
    for request in [
        JournalRequest::Select { quest_id: 5 },
        JournalRequest::Lock { locked: true },
    ] {
        assert_eq!(translate(request, &crosswalk), Some(request));
    }
}

fn quest(id: u32, giver: u64, objectives: usize) -> QuestDefinition {
    QuestDefinition {
        id,
        title: format!("Quest {id}"),
        description: String::new(),
        giver_npc_id: giver,
        turnin_npc_id: giver,
        prerequisite_quest_ids: Vec::new(),
        requirements: QuestRequirements::default(),
        ordered: false,
        objectives: (1..=objectives as u32)
            .map(|n| QuestObjective {
                id: n,
                required: 2,
                description: format!("Goal {n}"),
                kind: ObjectiveKind::KillActorKind { actor_kind_id: 5 },
            })
            .collect(),
        reward: RewardGrant {
            coins: CANARY_COINS,
            items: Vec::new(),
            job_xp: BTreeMap::new(),
            sc: 0,
            display: RewardDisplay {
                source_guid: 1,
                player_guid: CHARACTER,
                reward_kind: 0,
                icon_id: 0,
                name_id: 0,
                rows: Vec::new(),
            },
        },
    }
}

/// 10 plain, 11 behind 10, 12 hollow, 13 needs level 50, 14 plain, 15 plain.
fn engine(limit: usize) -> QuestEngine {
    let mut after_ten = quest(11, 2, 1);
    after_ten.prerequisite_quest_ids = vec![10];
    let mut high_level = quest(13, 5, 1);
    high_level.requirements.player_level = Some(50);
    let definitions = vec![
        quest(10, 1, 1),
        after_ten,
        quest(12, 3, 0),
        high_level,
        quest(14, 4, 1),
        quest(15, 6, 1),
    ];
    let eligibility = CharacterEligibility {
        player_level: 10,
        job_levels: BTreeMap::new(),
        member: true,
        nonmember_quest_limit: limit,
        member_quest_limit: limit,
    };
    QuestEngine::new(CHARACTER, definitions, eligibility, None).unwrap()
}

/// Original ids 1807..1813 map to 10..13 (11, 12, 13 are the refusal cases); 1814 is unmapped.
fn crosswalk() -> Crosswalk {
    BTreeMap::from([(1807, 10), (1808, 11), (1809, 12), (1810, 13), (1811, 14)])
}

fn givers(engine: &QuestEngine, crosswalk: &Crosswalk) -> BTreeMap<u32, u64> {
    crosswalk
        .values()
        .filter_map(|id| Some((*id, engine.giver_npc_id(*id)?)))
        .collect()
}

/// What the gateway route does with a 49/15: translate, then the journal.
fn start(engine: &mut QuestEngine, crosswalk: &Crosswalk, original: u32) -> Option<QuestOutcome> {
    let request = translate(JournalRequest::Breadcrumb { quest_id: original }, crosswalk)?;
    Some(handle(engine, request, &givers(engine, crosswalk)))
}

fn offered(outcome: &QuestOutcome) -> Option<u32> {
    outcome.outbound.iter().find_map(|message| match message {
        OutboundMessage::Quest(QuestMessage::QuestInfo { quest_id }) => Some(*quest_id),
        _ => None,
    })
}

/// A refusal changes nothing: same state, no reward, no offer, no completion.
fn assert_untouched(engine: &QuestEngine, before: &osfr_tools::quest_engine::QuestState) {
    assert_eq!(engine.state(), before);
    assert!(
        engine
            .state()
            .quests
            .values()
            .all(|progress| progress.status != QuestStatus::Completed
                && progress.counts.iter().all(|count| *count == 0))
    );
}

fn assert_no_award(outcome: &QuestOutcome) {
    assert!(outcome.rewards.is_empty());
    assert!(
        outcome
            .outbound
            .iter()
            .all(|message| !matches!(message, OutboundMessage::Reward(_)))
    );
}

#[test]
fn a_listed_original_id_offers_its_regional_quest_once() {
    let (mut engine, crosswalk) = (engine(5), crosswalk());
    let outcome = start(&mut engine, &crosswalk, 1807).unwrap();
    assert!(outcome.changed);
    assert_eq!(offered(&outcome), Some(10));
    assert_no_award(&outcome);
    assert_eq!(engine.state().pending_offer, Some(10));
    assert_eq!(engine.progress(10).unwrap().status, QuestStatus::Offered);

    // The window closed and the button was pressed again: shown again, nothing new made.
    let before = engine.state().clone();
    let again = start(&mut engine, &crosswalk, 1807).unwrap();
    assert!(!again.changed);
    assert_eq!(offered(&again), Some(10));
    assert_eq!(engine.state(), &before);

    // A burst of presses still leaves one record, and accepting it is the player's choice.
    for _ in 0..50 {
        start(&mut engine, &crosswalk, 1807);
    }
    assert_eq!(engine.state().quests.len(), 1);
    assert_eq!(engine.progress(10).unwrap().status, QuestStatus::Offered);
    assert!(engine.accept(10).changed);
    assert_eq!(engine.progress(10).unwrap().status, QuestStatus::Active);
    assert_eq!(engine.progress(10).unwrap().counts, [0]);
}

#[test]
fn an_unlisted_original_id_is_refused_and_changes_nothing() {
    let (mut engine, crosswalk) = (engine(5), crosswalk());
    start(&mut engine, &crosswalk, 1807).unwrap();
    let before = engine.state().clone();
    // 1814 is an original id nobody mapped; 10 and 14 are regional ids used as originals;
    // 0 and -1 never reach here (the decoder refuses them), the rest are forged.
    for id in [1814, 10, 14, 3_000_021, 1, i32::MAX as u32] {
        assert!(start(&mut engine, &crosswalk, id).is_none(), "id {id}");
        assert_eq!(engine.state(), &before);
    }
}

#[test]
fn a_listed_id_the_engine_will_not_offer_is_refused_with_a_denial_and_changes_nothing() {
    let crosswalk = crosswalk();
    // 1808 -> 11: its prerequisite 10 is not done.
    // 1809 -> 12: hollow content.
    // 1810 -> 13: needs level 50.
    for (original, denial) in [
        (1808, QuestDenial::Prerequisite),
        (1809, QuestDenial::Unavailable),
        (1810, QuestDenial::Level),
    ] {
        let mut engine = engine(5);
        let before = engine.state().clone();
        let outcome = start(&mut engine, &crosswalk, original).unwrap();
        assert_eq!(outcome.denial, Some(denial), "original {original}");
        assert!(!outcome.changed && offered(&outcome).is_none());
        assert_no_award(&outcome);
        assert_untouched(&engine, &before);
    }
}

#[test]
fn a_full_quest_log_refuses_the_start_and_says_so() {
    let (mut engine, crosswalk) = (engine(1), crosswalk());
    // 15 fills the single slot through its own giver, as a normal offer would.
    assert!(engine.offer(15, 6).changed);
    assert!(engine.accept(15).changed);
    let before = engine.state().clone();
    let outcome = start(&mut engine, &crosswalk, 1811).unwrap();
    assert_eq!(outcome.denial, Some(QuestDenial::Limit));
    assert!(offered(&outcome).is_none());
    assert_no_award(&outcome);
    assert_eq!(engine.state(), &before);
    // The member limit notice (47/15) is the only thing sent.
    assert!(
        outcome
            .outbound
            .contains(&OutboundMessage::Quest(QuestMessage::QuestLimit {
                member: true
            }))
    );
}

#[test]
fn a_listed_id_for_a_quest_already_done_or_held_changes_nothing() {
    let (mut engine, crosswalk) = (engine(5), crosswalk());
    start(&mut engine, &crosswalk, 1807).unwrap();
    assert!(engine.accept(10).changed);
    // Active: the breadcrumb just tracks it (selecting changes only the selection).
    let outcome = start(&mut engine, &crosswalk, 1807).unwrap();
    assert!(outcome.changed);
    assert_eq!(engine.state().selected_quest_id, Some(10));
    assert_eq!(engine.progress(10).unwrap().status, QuestStatus::Active);
    assert_eq!(engine.progress(10).unwrap().counts, [0]);
    assert_no_award(&outcome);
    // Pressing it again changes nothing at all.
    let before = engine.state().clone();
    let outcome = start(&mut engine, &crosswalk, 1807).unwrap();
    assert!(!outcome.changed);
    assert_eq!(engine.state(), &before);
}
