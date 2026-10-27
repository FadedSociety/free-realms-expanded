//! Piece 067 (quest prerequisite engine) tests: source-defined requirements,
//! branch availability and invalid acceptance rejection.
//!
//! Evidence boundary, read before the assertions:
//!
//! ORIGINAL (revision-pinned community documentation of the original game):
//! `lab/references/freerealms-wiki/quest-pages.json` states the requirements
//! verbatim, e.g. "Player must complete Quest:Learn to Mine & Smelt Ore in order
//! to do Quest:Your First Weapon" (rev 27815) and "Player must complete
//! Quest:Your First Weapon in order to do Quest:Fighting off the Pack"
//! (rev 28955). The corpus checker in the private recovery checker `piece_067.py` re-derives
//! all 36 requirement rows and fails on drift.
//!
//! ORIGINAL (first-party client locale): 401549 "In order to accept more quests,
//! you must first become a member." and 71311 "Sorry, you don't meet the
//! requirements for that action!" anchor the accept-time gate and its rejection
//! message. `QuestData_DS` carries `MembersOnly` and `JobLevel` client fields.
//!
//! COMMUNITY RECONSTRUCTION (FreeRealms-Legacy, AGPL-3.0, pinned revision):
//! `QuestDefinition.PrerequisiteQuestId` / `ExcludesQuestIds` and the documented
//! ordering "exclusions are checked before the prerequisite" are the basis for
//! branch availability. No first-party exclusion list was recovered; the branch
//! tests use synthetic ids and say so.
//!
//! NOT CLAIMED: any native accept/turn-in run, a recovered numeric quest id for
//! the two wiki pages, or a first-party exclusion table. Those stay gaps.
// The module is not registered in `src/lib.rs` (lead-owned). Including the exact
// source file keeps this target compiling against the real artifact. Integration
// step for the lead: register the module and drop this `#[path]` include.
#[path = "../src/restoration/piece_067.rs"]
#[allow(dead_code)]
mod piece_067;

use piece_067::{
    Availability, Block, OPENING_QUEST_PREREQUISITES, OPENING_UNRESOLVED_PREREQUISITES,
    PlayerFacts, PrerequisiteTable, QuestRow, Rejection, Requirement, TableError, UnmetRequirement,
    opening_table,
};

const DAY_MS: u64 = 86_400_000;

fn row(
    quest: u32,
    prereq: Option<u32>,
    excludes: Vec<u32>,
    requirements: Vec<Requirement>,
) -> QuestRow<u32> {
    QuestRow {
        quest,
        prereq,
        excludes,
        requirements,
        evidence: "synthetic fixture",
    }
}

fn table(rows: Vec<QuestRow<u32>>) -> PrerequisiteTable<u32> {
    PrerequisiteTable::new(rows, Vec::new()).expect("fixture table is valid")
}

fn blocks<K: Clone + std::fmt::Debug + Ord>(
    table: &PrerequisiteTable<K>,
    facts: &PlayerFacts<K>,
    quest: &K,
) -> Vec<Block<K>> {
    match table.availability(facts, quest) {
        Availability::Blocked(blocks) => blocks,
        other => panic!("expected blocked availability, got {other:?}"),
    }
}

#[test]
fn recovered_opening_rows_are_source_bound() {
    assert_eq!(OPENING_QUEST_PREREQUISITES.len(), 2);
    assert_eq!(
        OPENING_UNRESOLVED_PREREQUISITES,
        ["Quest:Learn to Mine & Smelt Ore"]
    );
    let table = opening_table();
    let first = table
        .row(&"Quest:Your First Weapon")
        .expect("recovered row");
    assert_eq!(first.prereq, Some("Quest:Learn to Mine & Smelt Ore"));
    assert!(first.evidence.contains("27815"), "{}", first.evidence);
    let pack = table
        .row(&"Quest:Fighting off the Pack")
        .expect("recovered row");
    assert_eq!(pack.prereq, Some("Quest:Your First Weapon"));
    assert!(pack.evidence.contains("28955"), "{}", pack.evidence);
}

#[test]
fn opening_chain_gates_the_follow_on_quest() {
    let table = opening_table();
    let mut facts = PlayerFacts::new(0);

    // The prerequisite page is absent from the corpus, so its completion is an
    // explicit external gap recorded by id; without it the quest is blocked.
    assert_eq!(
        blocks(&table, &facts, &"Quest:Your First Weapon"),
        vec![Block::Unmet(UnmetRequirement::PrerequisiteNotCompleted {
            prerequisite: "Quest:Learn to Mine & Smelt Ore",
        })]
    );
    assert!(matches!(
        table.accept(&mut facts, &"Quest:Fighting off the Pack"),
        Err(Rejection::Unmet(
            UnmetRequirement::PrerequisiteNotCompleted { .. }
        ))
    ));

    facts.record_completed("Quest:Learn to Mine & Smelt Ore", 0);
    assert!(table.accept(&mut facts, &"Quest:Your First Weapon").is_ok());

    // Accepted but not completed is not enough for the follow-on quest.
    assert!(matches!(
        table.accept(&mut facts, &"Quest:Fighting off the Pack"),
        Err(Rejection::Unmet(
            UnmetRequirement::PrerequisiteNotCompleted { .. }
        ))
    ));
    assert!(matches!(
        table.accept(&mut facts, &"Quest:Your First Weapon"),
        Err(Rejection::AlreadyActive { .. })
    ));

    facts.record_completed("Quest:Your First Weapon", 0);
    assert_eq!(
        table.availability(&facts, &"Quest:Fighting off the Pack"),
        Availability::Available
    );
    assert!(
        table
            .accept(&mut facts, &"Quest:Fighting off the Pack")
            .is_ok()
    );
    assert_eq!(
        table.availability(&facts, &"Quest:Fighting off the Pack"),
        Availability::Active
    );
}

#[test]
fn unknown_quests_are_never_accepted() {
    let table = table(vec![row(1, None, vec![], vec![])]);
    let mut facts = PlayerFacts::new(0);
    assert_eq!(table.availability(&facts, &99), Availability::Unknown);
    assert_eq!(
        table.accept(&mut facts, &99),
        Err(Rejection::UnknownQuest(99))
    );
    assert!(facts.quests.is_empty(), "rejection must not mutate state");
}

#[test]
fn completed_quests_are_rejected_with_the_completed_reason() {
    let table = table(vec![row(1, None, vec![], vec![])]);
    let mut facts = PlayerFacts::new(0);
    facts.record_completed(1, 500);
    assert_eq!(table.availability(&facts, &1), Availability::Completed);
    assert_eq!(
        table.accept(&mut facts, &1),
        Err(Rejection::AlreadyCompleted(1))
    );
}

#[test]
fn branch_exclusion_is_checked_before_the_prerequisite() {
    // Community-reconstructed starter-branch shape: quest 3 is the other side of
    // the choice taken by quest 2, and also carries a prerequisite.
    let table = table(vec![
        row(1, None, vec![], vec![]),
        row(2, Some(1), vec![], vec![]),
        row(3, Some(1), vec![2], vec![]),
    ]);
    let mut facts = PlayerFacts::new(0);
    facts.record_active(2);

    assert_eq!(
        blocks(&table, &facts, &3),
        vec![
            Block::BranchTaken { taken: 2 },
            Block::Unmet(UnmetRequirement::PrerequisiteNotCompleted { prerequisite: 1 }),
        ]
    );
    assert_eq!(
        table.accept(&mut facts, &3),
        Err(Rejection::BranchTaken { quest: 3, taken: 2 })
    );

    // The branch still rules the quest out once the prerequisite is satisfied.
    facts.record_completed(1, 0);
    assert_eq!(
        table.accept(&mut facts, &3),
        Err(Rejection::BranchTaken { quest: 3, taken: 2 })
    );
    // The taken-elsewhere quest stays takeable on its own branch.
    assert_eq!(table.availability(&facts, &2), Availability::Active);
}

#[test]
fn membership_level_and_ride_requirements_gate_acceptance() {
    let table = table(vec![row(
        10,
        None,
        vec![],
        vec![
            Requirement::MembershipActive,
            Requirement::CombatJobLevelAtLeast { level: 8 },
            Requirement::OwnsGlideTrainableRide,
        ],
    )]);
    let mut facts = PlayerFacts::new(0);
    assert_eq!(
        table.accept(&mut facts, &10),
        Err(Rejection::Unmet(UnmetRequirement::MembershipRequired))
    );
    facts.membership_active = true;
    assert_eq!(
        table.accept(&mut facts, &10),
        Err(Rejection::Unmet(UnmetRequirement::CombatJobLevel {
            required: 8,
            actual: 0,
        }))
    );
    facts.best_combat_job_level = 8;
    assert_eq!(
        table.accept(&mut facts, &10),
        Err(Rejection::Unmet(
            UnmetRequirement::GlideTrainableRideRequired
        ))
    );
    facts.owns_glide_trainable_ride = true;
    assert!(table.accept(&mut facts, &10).is_ok());
    assert_eq!(table.availability(&facts, &10), Availability::Active);
}

#[test]
fn event_window_requirement_matches_the_active_event() {
    let table = table(vec![row(
        20,
        None,
        vec![],
        vec![Requirement::EventWindow {
            name: "Winter Party Community Events",
        }],
    )]);
    let mut facts = PlayerFacts::new(0);
    assert_eq!(
        blocks(&table, &facts, &20),
        vec![Block::Unmet(UnmetRequirement::EventNotActive {
            name: "Winter Party Community Events",
        })]
    );
    facts.active_event = Some("Winter Party Community Events".to_string());
    assert!(table.accept(&mut facts, &20).is_ok());
}

#[test]
fn prerequisite_cooldown_counts_from_the_prerequisite_completion() {
    let table = table(vec![
        row(1, None, vec![], vec![]),
        row(
            2,
            Some(1),
            vec![],
            vec![Requirement::PrerequisiteCooldownDays { days: 1 }],
        ),
    ]);
    let mut facts = PlayerFacts::new(1_000);
    facts.record_completed(1, 1_000);

    facts.now_ms = 1_000 + DAY_MS - 1;
    assert_eq!(
        blocks(&table, &facts, &2),
        vec![Block::Unmet(UnmetRequirement::PrerequisiteCooldown {
            required_days: 1,
            remaining_ms: 1,
        })]
    );
    assert!(matches!(
        table.accept(&mut facts, &2),
        Err(Rejection::Unmet(
            UnmetRequirement::PrerequisiteCooldown { .. }
        ))
    ));

    facts.now_ms = 1_000 + DAY_MS;
    assert!(table.accept(&mut facts, &2).is_ok());
}

#[test]
fn cooldown_is_not_reported_while_the_prerequisite_is_incomplete() {
    let table = table(vec![
        row(1, None, vec![], vec![]),
        row(
            2,
            Some(1),
            vec![],
            vec![Requirement::PrerequisiteCooldownDays { days: 1 }],
        ),
    ]);
    let facts = PlayerFacts::new(1_000);
    assert_eq!(
        blocks(&table, &facts, &2),
        vec![Block::Unmet(UnmetRequirement::PrerequisiteNotCompleted {
            prerequisite: 1,
        })]
    );
}

#[test]
fn malformed_tables_are_rejected() {
    assert_eq!(
        PrerequisiteTable::new(
            vec![row(1, None, vec![], vec![]), row(1, None, vec![], vec![])],
            Vec::new()
        )
        .err(),
        Some(TableError::DuplicateQuest(1))
    );
    assert_eq!(
        PrerequisiteTable::new(vec![row(1, Some(1), vec![], vec![])], Vec::new()).err(),
        Some(TableError::SelfPrerequisite(1))
    );
    assert_eq!(
        PrerequisiteTable::new(vec![row(1, None, vec![1], vec![])], Vec::new()).err(),
        Some(TableError::SelfExclusion(1))
    );
    assert_eq!(
        PrerequisiteTable::new(
            vec![
                row(1, None, vec![], vec![]),
                row(2, Some(42), vec![], vec![])
            ],
            Vec::new()
        )
        .err(),
        Some(TableError::UnknownPrerequisite {
            quest: 2,
            prerequisite: 42,
        })
    );
    assert_eq!(
        PrerequisiteTable::new(
            vec![row(1, None, vec![], vec![]), row(2, None, vec![42], vec![])],
            Vec::new()
        )
        .err(),
        Some(TableError::UnknownExclusion {
            quest: 2,
            excluded: 42,
        })
    );
    assert_eq!(
        PrerequisiteTable::new(
            vec![row(1, None, vec![], vec![]), row(2, None, vec![2], vec![])],
            Vec::new()
        )
        .err(),
        Some(TableError::SelfExclusion(2))
    );
    assert_eq!(
        PrerequisiteTable::new(
            vec![
                row(1, None, vec![], vec![]),
                row(2, None, vec![1, 1], vec![])
            ],
            Vec::new()
        )
        .err(),
        Some(TableError::DuplicateExclusion {
            quest: 2,
            excluded: 1,
        })
    );
    assert_eq!(
        PrerequisiteTable::new(
            vec![row(
                1,
                None,
                vec![],
                vec![Requirement::CombatJobLevelAtLeast { level: 0 }],
            )],
            Vec::new()
        )
        .err(),
        Some(TableError::InvalidRequirement {
            quest: 1,
            requirement: Requirement::CombatJobLevelAtLeast { level: 0 },
        })
    );
    assert_eq!(
        PrerequisiteTable::new(
            vec![row(
                1,
                None,
                vec![],
                vec![Requirement::PrerequisiteCooldownDays { days: 0 }],
            )],
            Vec::new()
        )
        .err(),
        Some(TableError::InvalidRequirement {
            quest: 1,
            requirement: Requirement::PrerequisiteCooldownDays { days: 0 },
        })
    );
    assert_eq!(
        PrerequisiteTable::new(
            vec![row(
                1,
                None,
                vec![],
                vec![Requirement::EventWindow { name: "" }],
            )],
            Vec::new()
        )
        .err(),
        Some(TableError::InvalidRequirement {
            quest: 1,
            requirement: Requirement::EventWindow { name: "" },
        })
    );
}

#[test]
fn absent_prerequisite_pages_must_be_declared() {
    let rows = vec![row(1, Some(99), vec![], vec![])];
    assert_eq!(
        PrerequisiteTable::new(rows.clone(), Vec::new()).err(),
        Some(TableError::UnknownPrerequisite {
            quest: 1,
            prerequisite: 99,
        })
    );
    let table = PrerequisiteTable::new(rows, vec![99]).expect("declared gap loads");
    let mut facts = PlayerFacts::new(0);
    assert!(matches!(
        table.accept(&mut facts, &1),
        Err(Rejection::Unmet(
            UnmetRequirement::PrerequisiteNotCompleted { prerequisite: 99 }
        ))
    ));
    facts.record_completed(99, 0);
    assert!(table.accept(&mut facts, &1).is_ok());
}
