//! T787: a `finish_minigame` objective may require facts of the settled run.
//!
//! These tests drive the production `QuestEngine` event path: `QuestEngine::on_event`
//! is the exact call `Gameplay::npc_quest_event` makes, and each settle sends the same
//! two events `Gameplay::credit_minigame` sends (`FinishMinigame`, then
//! `MinigameResult`). Producer wiring and once-per-instance settlement are covered on
//! their real request paths in `src/gameplay/t013_objective_events.rs`,
//! `tests/derby.rs` and `tests/derby_chapter.rs`.

#[path = "support/npc_quest_chain.rs"]
mod npc_quest_chain;

use osfr_tools::{
    content::MinigameRequirement,
    gameplay::Gameplay,
    quest_engine::{
        CharacterEligibility, MinigameResult, ObjectiveKind, OutboundMessage, QuestDefinition,
        QuestEngine, QuestEvent, QuestObjective, QuestOutcome, QuestRequirements, QuestStatus,
    },
    rewards::{RewardDisplay, RewardGrant},
};
use serde_json::{Value, json};
use std::collections::BTreeMap;

const CHARACTER: u64 = 787_001;
const GIVER: u64 = 900_000_787;
/// A shipped cooking game id; the engine does not resolve the id itself.
const MINIGAME: u32 = 46;
/// A shipped enabled quest whose one `finish_minigame` step this file patches.
const LICENSE_TO_DRIVE: u32 = 3_060_005;

fn eligibility() -> CharacterEligibility {
    CharacterEligibility {
        player_level: 10,
        job_levels: BTreeMap::new(),
        member: true,
        nonmember_quest_limit: 8,
        member_quest_limit: 8,
    }
}

fn require(
    min_score: Option<u32>,
    max_elapsed_seconds: Option<u32>,
    max_placement: Option<u8>,
    no_knockout: bool,
) -> MinigameRequirement {
    MinigameRequirement {
        min_score,
        max_elapsed_seconds,
        max_placement,
        no_knockout,
    }
}

fn objective(id: u32, required: u32, kind: ObjectiveKind) -> QuestObjective {
    QuestObjective {
        id,
        required,
        description: format!("Goal {id}"),
        kind,
    }
}

fn definition(id: u32, objectives: Vec<QuestObjective>) -> QuestDefinition {
    QuestDefinition {
        id,
        title: format!("Quest {id}"),
        description: String::new(),
        giver_npc_id: GIVER,
        turnin_npc_id: GIVER,
        prerequisite_quest_ids: Vec::new(),
        requirements: QuestRequirements::default(),
        ordered: false,
        objectives,
        reward: RewardGrant {
            coins: 1,
            items: Vec::new(),
            job_xp: BTreeMap::new(),
            sc: 0,
            display: RewardDisplay {
                source_guid: GIVER,
                player_guid: CHARACTER,
                reward_kind: 0,
                icon_id: 0,
                name_id: 0,
                rows: Vec::new(),
            },
        },
    }
}

fn active(objectives: Vec<QuestObjective>) -> (QuestEngine, Vec<QuestDefinition>) {
    let definitions = vec![definition(7_870_001, objectives)];
    let mut engine = QuestEngine::new(CHARACTER, definitions.clone(), eligibility(), None).unwrap();
    assert!(engine.offer(7_870_001, GIVER).changed);
    assert!(engine.accept(7_870_001).changed);
    (engine, definitions)
}

fn result(
    won: bool,
    elapsed_seconds: Option<u32>,
    score: Option<u32>,
    placement: Option<u8>,
    knocked_out: bool,
) -> MinigameResult {
    MinigameResult {
        won,
        elapsed_seconds,
        score,
        placement,
        knocked_out,
    }
}

/// The same two events `Gameplay::credit_minigame` sends for one settle.
fn settle(engine: &mut QuestEngine, result: MinigameResult) -> QuestOutcome {
    let first = engine
        .on_event(QuestEvent::FinishMinigame {
            minigame_id: MINIGAME,
        })
        .unwrap();
    let second = engine
        .on_event(QuestEvent::MinigameResult {
            minigame_id: MINIGAME,
            result,
        })
        .unwrap();
    let mut merged = first;
    merged.changed |= second.changed;
    merged.messages.extend(second.messages);
    merged.outbound.extend(second.outbound);
    merged.rewards.extend(second.rewards);
    merged.denial = second.denial.or(merged.denial);
    merged
}

fn opcodes(outcome: &QuestOutcome) -> Vec<(u16, u32)> {
    outcome
        .outbound
        .iter()
        .filter_map(|message| match message {
            OutboundMessage::Quest(message) => Some(message.opcode()),
            OutboundMessage::Reward(_) => None,
        })
        .collect()
}

fn counts(engine: &QuestEngine) -> Vec<u32> {
    engine.progress(7_870_001).unwrap().counts.clone()
}

/// Patch the shipped license quest's finish step in the fixture's narrowed catalogue.
fn patch_license_require(fixture: &npc_quest_chain::Fixture, require: Value) {
    fixture.patch_quest("demo_derby_driver.json", LICENSE_TO_DRIVE, |record| {
        let goal = record["objectives"]
            .as_array_mut()
            .expect("license objectives")
            .iter_mut()
            .find(|goal| goal["kind"] == "finish_minigame")
            .expect("the license finish objective");
        goal["require"] = require;
    });
}

/// The split loader owns the requirement bounds: a value outside them rejects the
/// whole catalogue, while the documented upper bounds are accepted.
#[test]
fn out_of_range_requirement_is_rejected_at_load() {
    let fixture = npc_quest_chain::Fixture::new();
    let mut game = Gameplay::load(fixture.path(), None).expect("the fixture content loads");
    for (name, require) in [
        ("zero-score", json!({"min_score": 0})),
        ("large-placement", json!({"max_placement": 9})),
        ("long-elapsed", json!({"max_elapsed_seconds": 86_401})),
    ] {
        patch_license_require(&fixture, require);
        assert!(
            game.bind_npc_quest_catalogue(&fixture.quest_catalogue(), CHARACTER)
                .is_err(),
            "{name} must be rejected by the production split loader"
        );
    }
    patch_license_require(
        &fixture,
        json!({
            "min_score": 1_000_000_000,
            "max_placement": 8,
            "max_elapsed_seconds": 86_400
        }),
    );
    assert!(
        game.bind_npc_quest_catalogue(&fixture.quest_catalogue(), CHARACTER)
            .is_ok(),
        "the documented upper bounds load"
    );
}

/// A result below the authored score, over the elapsed bound, without a placement, or
/// on a lost run leaves the quest state byte-equal and sends nothing.
#[test]
fn result_below_threshold_changes_nothing() {
    let cases = [
        result(true, None, Some(19_999), None, false),
        result(true, Some(91), Some(20_000), None, false),
        result(true, Some(10), Some(20_000), None, false), // no placement reported
        result(false, Some(10), Some(20_000), Some(1), false),
        result(true, Some(10), None, Some(1), false), // score not reported
        result(true, Some(10), Some(20_000), Some(1), true), // knocked out
    ];
    for case in cases {
        let (mut engine, _) = active(vec![objective(
            1,
            1,
            ObjectiveKind::MinigameResult {
                minigame_id: MINIGAME,
                require: require(Some(20_000), Some(90), Some(1), true),
            },
        )]);
        let before = engine.state().clone();
        let outcome = settle(&mut engine, case);
        assert_eq!(
            engine.state(),
            &before,
            "a refused result must change nothing: {case:?}"
        );
        assert!(!outcome.changed);
        assert!(outcome.outbound.is_empty());
        assert!(outcome.messages.is_empty());
    }
}

/// Bounds are inclusive: a score equal to `min_score`, a time equal to
/// `max_elapsed_seconds` and a placement equal to `max_placement` all count, once.
#[test]
fn exact_threshold_credits_once() {
    let (mut engine, _) = active(vec![objective(
        1,
        1,
        ObjectiveKind::MinigameResult {
            minigame_id: MINIGAME,
            require: require(Some(20_000), Some(90), Some(1), true),
        },
    )]);
    let outcome = settle(
        &mut engine,
        result(true, Some(90), Some(20_000), Some(1), false),
    );
    assert_eq!(counts(&engine), vec![1]);
    assert!(outcome.changed);
    assert_eq!(
        opcodes(&outcome),
        vec![(49, 9), (49, 10)],
        "one settle sends one update and one completion"
    );

    // Both events carry the same minigame id; the result objective matches only the
    // result event, so the pair cannot count twice.
    assert_eq!(counts(&engine), vec![1]);
    let before = engine.state().clone();
    let replay = settle(
        &mut engine,
        result(true, Some(90), Some(20_000), Some(1), false),
    );
    assert!(!replay.changed);
    assert!(replay.outbound.is_empty());
    assert_eq!(engine.state(), &before);
}

/// A fact the producer did not report never meets a bound on that fact.
#[test]
fn unreported_fact_does_not_meet_a_bound() {
    for (kind, case) in [
        (
            ObjectiveKind::MinigameResult {
                minigame_id: MINIGAME,
                require: require(Some(1), None, None, false),
            },
            result(true, None, None, None, false),
        ),
        (
            ObjectiveKind::MinigameResult {
                minigame_id: MINIGAME,
                require: require(None, Some(1), None, false),
            },
            result(true, None, Some(1), None, false),
        ),
        (
            ObjectiveKind::MinigameResult {
                minigame_id: MINIGAME,
                require: require(None, None, Some(1), false),
            },
            result(true, Some(1), Some(1), None, false),
        ),
    ] {
        let (mut engine, _) = active(vec![objective(1, 1, kind)]);
        let before = engine.state().clone();
        let outcome = settle(&mut engine, case);
        assert!(!outcome.changed, "a missing fact must not satisfy a bound");
        assert!(outcome.outbound.is_empty());
        assert_eq!(engine.state(), &before);
    }
}

/// `no_knockout` accepts a run with no knock-out and refuses one with a knock-out.
#[test]
fn no_knockout_checks_the_result_flag() {
    let (mut knocked_out, _) = active(vec![objective(
        1,
        1,
        ObjectiveKind::MinigameResult {
            minigame_id: MINIGAME,
            require: require(None, None, None, true),
        },
    )]);
    let before = knocked_out.state().clone();
    let refused = settle(&mut knocked_out, result(true, None, None, None, true));
    assert!(!refused.changed);
    assert_eq!(knocked_out.state(), &before);

    let (mut clean, _) = active(vec![objective(
        1,
        1,
        ObjectiveKind::MinigameResult {
            minigame_id: MINIGAME,
            require: require(None, None, None, true),
        },
    )]);
    let credited = settle(&mut clean, result(true, None, None, None, false));
    assert!(credited.changed);
    assert_eq!(counts(&clean), vec![1]);
}

/// Repeating the result after the objective completed changes nothing.
#[test]
fn result_after_completion_changes_nothing() {
    let (mut engine, _) = active(vec![objective(
        1,
        1,
        ObjectiveKind::MinigameResult {
            minigame_id: MINIGAME,
            require: require(Some(2), None, None, false),
        },
    )]);
    assert!(settle(&mut engine, result(true, None, Some(2), None, false)).changed);
    let completed = engine.state().clone();
    let outcome = settle(&mut engine, result(true, None, Some(2), None, false));
    assert!(!outcome.changed);
    assert!(outcome.outbound.is_empty());
    assert_eq!(engine.state(), &completed);
}

/// The quest counters are the persisted progress: a reload between the result and the
/// turn-in keeps them, and the turn-in still opens. Result facts are never saved.
#[test]
fn reload_between_result_and_turnin_keeps_progress() {
    let (mut engine, definitions) = active(vec![objective(
        1,
        1,
        ObjectiveKind::MinigameResult {
            minigame_id: MINIGAME,
            require: require(Some(20_000), Some(90), None, true),
        },
    )]);
    assert!(
        settle(
            &mut engine,
            result(true, Some(12), Some(20_000), None, false)
        )
        .changed
    );
    let serialized = serde_json::to_vec(engine.state()).unwrap();
    let saved: osfr_tools::quest_engine::QuestState = serde_json::from_slice(&serialized).unwrap();
    assert!(
        !serialized
            .windows(b"knocked_out".len())
            .any(|window| window == b"knocked_out"),
        "result facts are not persisted"
    );
    let mut reloaded =
        QuestEngine::new(CHARACTER, definitions, eligibility(), Some(saved)).unwrap();
    assert_eq!(counts(&reloaded), vec![1]);
    assert_eq!(
        reloaded.progress(7_870_001).unwrap().status,
        QuestStatus::Active
    );
    assert!(reloaded.begin_turnin(7_870_001, GIVER).changed);
}

/// A goal authored without `require` is the id-only kind: it ignores result facts and
/// still credits from the plain finish event exactly as before, and it does not also
/// match the result event.
#[test]
fn no_requirement_goal_behaves_as_before() {
    let (mut engine, _) = active(vec![objective(
        1,
        1,
        ObjectiveKind::FinishMinigame {
            minigame_id: MINIGAME,
        },
    )]);
    // The result event alone never credits the id-only kind.
    let only_result = engine
        .on_event(QuestEvent::MinigameResult {
            minigame_id: MINIGAME,
            result: result(true, Some(1), Some(1), Some(1), false),
        })
        .unwrap();
    assert!(!only_result.changed);
    assert_eq!(counts(&engine), vec![0]);

    let outcome = settle(&mut engine, result(true, Some(1), Some(1), Some(1), false));
    assert_eq!(counts(&engine), vec![1]);
    assert_eq!(opcodes(&outcome), vec![(49, 9), (49, 10)]);
}
