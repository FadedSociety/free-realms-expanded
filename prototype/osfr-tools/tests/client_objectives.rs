//! W175: the client-observed `45/7 ClientComplete` path and its session registrations.
//!
//! Recovered from the original client:
//! - `45/7` is exactly `u16 45, u8 7, i32 objective id`; `00cffba0` builds it and
//!   `00cffa70` sends it.
//! - `45/10 UIEvent` is `id, i32 byte length, event bytes` (`00c437a0`), `45/11`
//!   FirstMovement is `id, u8 mouse` (`00c43870`), and the `45/6` actor LookAt
//!   form is `id, u64 guid, f32 offset, i32 range, i32 angle` (`00c435c0` and
//!   reader `009b2760`).
//! - The client does not de-duplicate objective registrations (`00c43140`).
//!
//! Everything else here is reconstructed policy: only an open goal of the
//! character's active quest can advance, content must opt in, client ids are
//! unique, and a session registration is written once and re-armed with `45/9`.

use osfr_tools::{
    quest_engine::{
        CharacterEligibility, ClientObjectives, ClientTrigger, ObjectiveKind, OutboundMessage,
        QuestDefinition, QuestEngine, QuestEvent, QuestMessage, QuestObjective, QuestRequirements,
        QuestStatus, decode_client_complete,
    },
    rewards::{RewardDisplay, RewardGrant},
};
use std::collections::BTreeMap;

const CHARACTER: u64 = 42;

fn reward() -> RewardGrant {
    RewardGrant {
        coins: 9,
        items: Vec::new(),
        job_xp: BTreeMap::new(),
        display: RewardDisplay {
            source_guid: 1,
            player_guid: CHARACTER,
            reward_kind: 0,
            icon_id: 0,
            name_id: 0,
            rows: Vec::new(),
        },
    }
}

fn objective(id: u32, kind: ObjectiveKind) -> QuestObjective {
    QuestObjective {
        id,
        required: 1,
        description: format!("Objective {id}"),
        kind,
    }
}

fn quest(id: u32, giver: u64, objectives: Vec<QuestObjective>) -> QuestDefinition {
    QuestDefinition {
        id,
        title: format!("Quest {id}"),
        description: String::new(),
        giver_npc_id: giver,
        turnin_npc_id: giver,
        prerequisite_quest_ids: Vec::new(),
        requirements: QuestRequirements::default(),
        ordered: true,
        objectives,
        reward: reward(),
    }
}

fn engine(definitions: Vec<QuestDefinition>) -> QuestEngine {
    QuestEngine::new(
        CHARACTER,
        definitions,
        CharacterEligibility {
            member: true,
            member_quest_limit: 10,
            nonmember_quest_limit: 10,
            ..Default::default()
        },
        None,
    )
    .unwrap()
}

fn trigger_engine(id: u32, client_id: u32, trigger: ClientTrigger) -> QuestEngine {
    engine(vec![quest(
        id,
        1,
        vec![objective(
            1,
            ObjectiveKind::ClientReported { client_id, trigger },
        )],
    )])
}

fn activate(engine: &mut QuestEngine, id: u32) {
    assert!(engine.offer(id, 1).changed);
    assert!(engine.accept(id).changed);
    assert_eq!(engine.progress(id).unwrap().status, QuestStatus::Active);
}

fn claim(id: i32) -> Vec<u8> {
    let mut packet = vec![45, 0, 7];
    packet.extend(id.to_le_bytes());
    packet
}

fn assert_no_award(outcome: &osfr_tools::quest_engine::QuestOutcome) {
    assert!(outcome.rewards.is_empty());
    assert!(
        outcome
            .outbound
            .iter()
            .all(|message| !matches!(message, OutboundMessage::Reward(_)))
    );
}

#[test]
fn client_completion_is_exact_and_bounded() {
    assert_eq!(decode_client_complete(&claim(73)).unwrap(), Some(73));
    assert_eq!(
        decode_client_complete(&claim(i32::MAX)).unwrap(),
        Some(i32::MAX as u32)
    );
    assert_eq!(
        decode_client_complete(&[45, 0, 10, 1, 0, 0, 0]).unwrap(),
        None
    );
    assert_eq!(
        decode_client_complete(&[46, 0, 7, 1, 0, 0, 0]).unwrap(),
        None
    );
    for bad in [
        vec![45, 0, 7],
        vec![45, 0, 7, 1, 0, 0],
        vec![45, 0, 7, 1, 0, 0, 0, 0],
        claim(0),
        claim(-1),
        claim(i32::MIN),
    ] {
        assert!(decode_client_complete(&bad).is_err(), "{bad:?}");
    }
}

#[test]
fn repeated_forged_and_inactive_claims_change_nothing() {
    let client_id = 600;
    let mut engine = engine(vec![quest(
        200,
        1,
        vec![
            objective(
                1,
                ObjectiveKind::ClientReported {
                    client_id,
                    trigger: ClientTrigger::UiEvent {
                        event: "onStoryBookClose".into(),
                    },
                },
            ),
            objective(2, ObjectiveKind::KillActorKind { actor_kind_id: 9 }),
        ],
    )]);

    // Offered and unknown quests are never open. Neither is an unaccepted goal.
    let before_offer = engine.state().clone();
    for id in [client_id, 999] {
        let outcome = engine
            .on_event(QuestEvent::ClientReported { client_id: id })
            .unwrap();
        assert!(!outcome.changed && outcome.outbound.is_empty());
        assert_eq!(engine.state(), &before_offer);
    }
    assert!(engine.offer(200, 1).changed);
    let before_active = engine.state().clone();
    let outcome = engine
        .on_event(QuestEvent::ClientReported { client_id })
        .unwrap();
    assert!(!outcome.changed && outcome.outbound.is_empty());
    assert_eq!(engine.state(), &before_active);

    assert!(engine.accept(200).changed);
    let outcome = engine
        .on_event(QuestEvent::ClientReported { client_id })
        .unwrap();
    assert!(outcome.changed);
    assert_no_award(&outcome);
    assert_eq!(engine.progress(200).unwrap().counts, [1, 0]);
    let after_claim = engine.state().clone();

    // A replay cannot advance the completed goal. A kill-objective id and a
    // client id from another quest are not aliases. A known non-client goal
    // cannot be advanced by a client completion.
    for id in [client_id, 1, 9, 601] {
        let outcome = engine
            .on_event(QuestEvent::ClientReported { client_id: id })
            .unwrap();
        assert!(!outcome.changed && outcome.outbound.is_empty(), "id {id}");
        assert_eq!(engine.state(), &after_claim, "id {id}");
        assert_no_award(&outcome);
    }
}

#[test]
fn only_open_goals_of_the_character_own_active_quest_are_registered() {
    let mut engine = engine(vec![
        quest(
            300,
            1,
            vec![objective(
                1,
                ObjectiveKind::ClientReported {
                    client_id: 700,
                    trigger: ClientTrigger::FirstMovement { mouse: true },
                },
            )],
        ),
        quest(
            301,
            2,
            vec![objective(
                1,
                ObjectiveKind::ClientReported {
                    client_id: 701,
                    trigger: ClientTrigger::UiEvent {
                        event: "onTakeMeThere".into(),
                    },
                },
            )],
        ),
    ]);
    assert!(engine.offer(301, 2).changed);
    assert!(engine.open_client_objectives().is_empty());
    assert!(engine.accept(301).changed);
    let open = engine.open_client_objectives();
    assert_eq!(open.len(), 1);
    assert_eq!(open[0].0, 701);
    assert!(matches!(
        open[0].1,
        ClientTrigger::UiEvent { event } if event == "onTakeMeThere"
    ));
}

#[test]
fn ui_and_movement_registrations_match_the_recovered_layout_and_are_not_repeated() {
    let ui_event = "onStoryBookClose";
    let mut ui = trigger_engine(
        400,
        701,
        ClientTrigger::UiEvent {
            event: ui_event.into(),
        },
    );
    activate(&mut ui, 400);
    let mut sent = ClientObjectives::default();
    let mut expected = vec![45, 0, 10];
    expected.extend(701u32.to_le_bytes());
    expected.extend((ui_event.len() as u32).to_le_bytes());
    expected.extend(ui_event.as_bytes());
    assert_eq!(sent.update(&ui), vec![expected]);
    assert!(sent.update(&ui).is_empty());

    let mut movement = trigger_engine(401, 702, ClientTrigger::FirstMovement { mouse: true });
    activate(&mut movement, 401);
    let mut sent = ClientObjectives::default();
    let mut expected = vec![45, 0, 11];
    expected.extend(702u32.to_le_bytes());
    expected.push(1);
    assert_eq!(sent.update(&movement), vec![expected]);
    assert!(sent.update(&movement).is_empty());
}

#[test]
fn look_at_registration_matches_the_actor_form() {
    let mut engine = trigger_engine(
        402,
        703,
        ClientTrigger::LookAt {
            guid: 0x1122_3344_5566_7788,
            offset: 1.5,
            range: 12,
            angle_degrees: 45,
        },
    );
    activate(&mut engine, 402);
    let mut sent = ClientObjectives::default();
    let mut expected = vec![45, 0, 6];
    expected.extend(703u32.to_le_bytes());
    expected.extend(0x1122_3344_5566_7788u64.to_le_bytes());
    expected.extend(1.5f32.to_le_bytes());
    expected.extend(12u32.to_le_bytes());
    expected.extend(45u32.to_le_bytes());
    assert_eq!(expected.len(), 27);
    assert_eq!(sent.update(&engine), vec![expected]);
    assert!(sent.update(&engine).is_empty());

    let outcome = engine
        .on_event(QuestEvent::ClientReported { client_id: 703 })
        .unwrap();
    assert!(outcome.changed);
    assert_eq!(engine.progress(402).unwrap().counts, [1]);
    assert_no_award(&outcome);
}

#[test]
fn a_reopened_goal_is_rearmed_without_a_second_registration() {
    let mut engine = trigger_engine(403, 704, ClientTrigger::FirstMovement { mouse: false });
    activate(&mut engine, 403);
    let mut sent = ClientObjectives::default();
    let mut first = vec![45, 0, 11];
    first.extend(704u32.to_le_bytes());
    first.push(0);
    assert_eq!(sent.update(&engine), vec![first]);

    assert!(engine.abandon(403).changed);
    assert!(sent.update(&engine).is_empty());
    activate(&mut engine, 403);
    let mut rearm = vec![45, 0, 9];
    rearm.extend(704u32.to_le_bytes());
    assert_eq!(sent.update(&engine), vec![rearm]);
    assert!(sent.update(&engine).is_empty());
}

#[test]
fn duplicate_and_invalid_client_goals_fail_catalogue_load() {
    let duplicate = vec![
        quest(
            500,
            1,
            vec![objective(
                1,
                ObjectiveKind::ClientReported {
                    client_id: 800,
                    trigger: ClientTrigger::FirstMovement { mouse: false },
                },
            )],
        ),
        quest(
            501,
            2,
            vec![objective(
                1,
                ObjectiveKind::ClientReported {
                    client_id: 800,
                    trigger: ClientTrigger::FirstMovement { mouse: true },
                },
            )],
        ),
    ];
    assert!(
        QuestEngine::new(
            CHARACTER,
            duplicate,
            CharacterEligibility {
                member: true,
                member_quest_limit: 10,
                nonmember_quest_limit: 10,
                ..Default::default()
            },
            None,
        )
        .is_err()
    );

    for trigger in [
        ClientTrigger::LookAt {
            guid: 0,
            offset: 0.0,
            range: 1,
            angle_degrees: 1,
        },
        ClientTrigger::LookAt {
            guid: 1,
            offset: f32::NAN,
            range: 1,
            angle_degrees: 1,
        },
        ClientTrigger::LookAt {
            guid: 1,
            offset: 0.0,
            range: 0,
            angle_degrees: 1,
        },
        ClientTrigger::LookAt {
            guid: 1,
            offset: 0.0,
            range: 1,
            angle_degrees: 0,
        },
        ClientTrigger::UiEvent {
            event: String::new(),
        },
        ClientTrigger::UiEvent {
            event: "bad\nevent".into(),
        },
    ] {
        assert!(
            QuestEngine::new(
                CHARACTER,
                vec![quest(
                    502,
                    1,
                    vec![objective(
                        1,
                        ObjectiveKind::ClientReported {
                            client_id: 801,
                            trigger,
                        },
                    )],
                )],
                CharacterEligibility {
                    member: true,
                    member_quest_limit: 10,
                    nonmember_quest_limit: 10,
                    ..Default::default()
                },
                None,
            )
            .is_err()
        );
    }
}

#[test]
fn a_client_claim_never_awards_and_turn_in_still_needs_the_npc() {
    let mut engine = trigger_engine(503, 802, ClientTrigger::FirstMovement { mouse: false });
    activate(&mut engine, 503);
    let outcome = engine
        .on_event(QuestEvent::ClientReported { client_id: 802 })
        .unwrap();
    assert!(outcome.changed);
    assert!(outcome.outbound.iter().any(|message| {
        matches!(
            message,
            OutboundMessage::Quest(QuestMessage::ObjectiveComplete { .. })
        )
    }));
    assert_eq!(engine.progress(503).unwrap().status, QuestStatus::Active);
    assert!(engine.state().pending_turnin.is_none());
    assert_no_award(&outcome);
}

#[test]
fn another_quests_pending_turn_in_does_not_drop_a_client_completion() {
    let mut engine = engine(vec![
        quest(
            504,
            1,
            vec![objective(
                1,
                ObjectiveKind::KillActorKind { actor_kind_id: 9 },
            )],
        ),
        quest(
            505,
            1,
            vec![objective(
                1,
                ObjectiveKind::ClientReported {
                    client_id: 803,
                    trigger: ClientTrigger::FirstMovement { mouse: false },
                },
            )],
        ),
    ]);
    activate(&mut engine, 504);
    activate(&mut engine, 505);

    let finished = engine
        .on_event(QuestEvent::KillActorKind {
            actor_kind_id: 9,
            count: 1,
        })
        .unwrap();
    assert!(finished.changed);
    let turn_in = engine
        .on_event(QuestEvent::TalkToNpc { npc_id: 1 })
        .unwrap();
    assert!(turn_in.outbound.iter().any(|message| matches!(
        message,
        OutboundMessage::Quest(QuestMessage::QuestEnd { quest_id: 504 })
    )));
    assert_eq!(engine.state().pending_turnin, Some(504));

    // 0x00cffba0 sends 45/7 before setting the client's completed flag, so losing this
    // packet would leave the client waiting forever. A different quest's end screen must
    // not discard the completion.
    let completion = engine
        .on_event(QuestEvent::ClientReported { client_id: 803 })
        .unwrap();
    assert!(completion.changed);
    assert!(completion.outbound.iter().any(|message| matches!(
        message,
        OutboundMessage::Quest(QuestMessage::ObjectiveComplete { quest_id: 505, .. })
    )));
    assert_eq!(engine.progress(505).unwrap().counts, [1]);
    assert_eq!(engine.state().pending_turnin, Some(504));
    assert_no_award(&completion);

    let after = engine.state().clone();
    let replay = engine
        .on_event(QuestEvent::ClientReported { client_id: 803 })
        .unwrap();
    assert!(!replay.changed && replay.outbound.is_empty());
    assert_eq!(engine.state(), &after);
}
