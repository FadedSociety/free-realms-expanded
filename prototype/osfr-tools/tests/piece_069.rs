//! Piece 069 (quest turn-in transactions) tests: the exactly-once turn-in boundary.
//!
//! Evidence boundary, read the note before the assertions:
//!
//! ORIGINAL (first-party inputs, pinned under `../lab`, outside Git):
//! * `lab/decompiled/server/LibSOE/SOE/BaseQuestPackets.cs` is the supplied server's
//!   quest family: `QuestCompletePacket = 4`, `CompletedQuestCountUpdatePacket = 12`,
//!   `QuestEndPacket = 13`, `QuestEndReplyPacket = 14`, `QuestAddPacket = 3`,
//!   `QuestObjectiveCompletePacket = 10`.
//! * `lab/runs/client-baseline-20261008T171251Z/analysis/native-cooking-finish-turnin-proof.json`
//!   records the native turn-in frames: two objective packets (opcodes 9/10, quest 910004),
//!   one `QuestCompletePacket` (opcode 4) at frame 4071, one `CompletedQuestCountUpdatePacket`
//!   (opcode 12, count 4) at frame 4072, one wallet update 110->120, one Chef XP update 0->25,
//!   and `repeat_Mae_no_reward_progress_packet: true` for the second selection.
//! * `lab/runs/client-baseline-20261008T193318Z/analysis-live/native-bartle-turnin-nextquest-proof.json`
//!   records one choice, one wallet value, one XP value, one quest removal, one next-quest add
//!   and `reward_emitted_once: true`.
//!
//! RECONSTRUCTED here (labelled, not historical): the write-ahead ledger, its record schema,
//! every bound and every rejection. No original server source for a turn-in transaction was
//! recovered, and the native client has not accepted any artifact this module produces.
//!
//! The module is not registered in `src/lib.rs` (lead-owned). Including the exact source file
//! keeps this target compiling against the real artifact. Integration step for the lead:
//! register the module and drop this `#[path]` include.

#[path = "../src/restoration/piece_069.rs"]
#[allow(dead_code)]
mod piece_069;

use piece_069::{
    COMPLETED_QUEST_COUNT_UPDATE_PACKET, MAX_LEDGER_RECORDS, QUEST_ADD_PACKET,
    QUEST_COMPLETE_PACKET, QUEST_END_PACKET, QUEST_END_REPLY_PACKET,
    QUEST_OBJECTIVE_COMPLETE_PACKET, QUEST_PACKET_FAMILY, RECOVERED_TURN_IN_PACKETS,
    TurnInDecision, TurnInLedger, TurnInReceipt, TurnInRecovery, TurnInRejection, TurnInRequest,
    TurnInReward, TurnInStaged,
};
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

const QUEST: u32 = 910004;

fn ledger_path(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "osfr-piece-069-{}-{name}.jsonl",
        std::process::id()
    ))
}

fn progress(accepted: bool, completed: bool, rewarded: bool) -> piece_069::QuestProgress {
    piece_069::QuestProgress {
        accepted,
        counts: vec![1],
        completed,
        rewarded,
    }
}

fn state(progress: piece_069::QuestProgress) -> piece_069::QuestState {
    let mut quests = BTreeMap::new();
    quests.insert(QUEST, progress);
    piece_069::QuestState {
        version: 1,
        quests,
        trial_best_ms: None,
        trial_completions: 0,
    }
}

fn request(attempt: u32, request_id: u64) -> TurnInRequest {
    TurnInRequest {
        quest_id: QUEST,
        attempt,
        request_id,
    }
}

fn reward() -> TurnInReward {
    TurnInReward {
        coins: 10,
        experience: 25,
        items: vec![(35, 1)],
    }
}

/// The recovered packet family and turn-in order, bound to the decompiled enum.
#[test]
fn recovered_packet_constants_match_the_decompiled_quest_family() {
    assert_eq!(QUEST_PACKET_FAMILY, 0x0031);
    assert_eq!(QUEST_COMPLETE_PACKET, 4);
    assert_eq!(COMPLETED_QUEST_COUNT_UPDATE_PACKET, 12);
    assert_eq!(QUEST_END_PACKET, 13);
    assert_eq!(QUEST_END_REPLY_PACKET, 14);
    assert_eq!(QUEST_ADD_PACKET, 3);
    assert_eq!(QUEST_OBJECTIVE_COMPLETE_PACKET, 10);
    assert_eq!(RECOVERED_TURN_IN_PACKETS, [4, 12]);
}

#[test]
fn a_completed_turn_in_applies_once_then_replays_without_a_second_grant() {
    let path = ledger_path("apply-once");
    let _ = fs::remove_file(&path);
    let ready = progress(true, true, false);

    let mut ledger = TurnInLedger::open(&path).unwrap();
    let staged = match ledger
        .consider(request(1, 7), Some(&ready), &reward())
        .unwrap()
    {
        TurnInDecision::Apply(staged) => staged,
        other => panic!("expected a staged turn-in, got {other:?}"),
    };
    assert_eq!(staged.reward(), &reward());
    let receipt = ledger.commit(staged).unwrap();
    assert_eq!(
        receipt,
        TurnInReceipt {
            quest_id: QUEST,
            attempt: 1,
            request_id: 7,
            settled_from_state: false,
        }
    );
    assert_eq!(ledger.records(), 2, "one intent plus one applied record");

    // The exact same request and the post-turn-in progress must never grant again.
    let rewarded = progress(true, true, true);
    let repeat = ledger
        .consider(request(1, 7), Some(&rewarded), &reward())
        .unwrap();
    assert!(matches!(repeat, TurnInDecision::Replay(_)));
    assert_eq!(ledger.records(), 2, "a repeat writes no journal record");

    // A fresh request id for an already rewarded quest also replays.
    let repeat = ledger
        .consider(request(1, 8), Some(&rewarded), &reward())
        .unwrap();
    assert!(matches!(repeat, TurnInDecision::Replay(_)));
    assert_eq!(ledger.records(), 2);
}

#[test]
fn a_crash_between_intent_and_snapshot_replays_exactly_once() {
    let path = ledger_path("crash-before-snapshot");
    let _ = fs::remove_file(&path);
    let ready = progress(true, true, false);

    let mut ledger = TurnInLedger::open(&path).unwrap();
    let _ = match ledger
        .consider(request(1, 7), Some(&ready), &reward())
        .unwrap()
    {
        TurnInDecision::Apply(staged) => staged,
        other => panic!("expected a staged turn-in, got {other:?}"),
    };
    drop(ledger); // crash: the intent is durable, the snapshot never landed

    let mut recovered = TurnInLedger::open(&path).unwrap();
    let actions = recovered
        .recover(&state(progress(true, true, false)))
        .unwrap();
    let mut grants = Vec::new();
    for action in actions {
        match action {
            TurnInRecovery::Replay(staged) => grants.push(staged),
            other => panic!("expected a replay, got {other:?}"),
        }
    }
    assert_eq!(grants.len(), 1, "the reward is re-derived exactly once");
    assert_eq!(grants[0].reward(), &reward());
    recovered.commit(grants.remove(0)).unwrap();

    // A second crash/restart after the replayed snapshot is settled grants nothing.
    drop(recovered);
    let mut after = TurnInLedger::open(&path).unwrap();
    assert!(
        after
            .recover(&state(progress(true, true, true)))
            .unwrap()
            .is_empty()
    );
    assert!(matches!(
        after
            .consider(request(1, 7), Some(&progress(true, true, true)), &reward())
            .unwrap(),
        TurnInDecision::Replay(_)
    ));
}

#[test]
fn a_crash_after_the_snapshot_but_before_the_applied_record_settles_without_a_grant() {
    let path = ledger_path("crash-after-snapshot");
    let _ = fs::remove_file(&path);
    let ready = progress(true, true, false);

    let mut ledger = TurnInLedger::open(&path).unwrap();
    let _ = match ledger
        .consider(request(1, 7), Some(&ready), &reward())
        .unwrap()
    {
        TurnInDecision::Apply(staged) => staged,
        other => panic!("expected a staged turn-in, got {other:?}"),
    };
    drop(ledger); // crash: the caller persisted the reward with the snapshot

    let mut recovered = TurnInLedger::open(&path).unwrap();
    let actions = recovered
        .recover(&state(progress(true, true, true)))
        .unwrap();
    assert_eq!(actions.len(), 1);
    match &actions[0] {
        TurnInRecovery::Settled(receipt) => {
            assert!(receipt.settled_from_state);
            assert_eq!(receipt.quest_id, QUEST);
            assert_eq!(receipt.attempt, 1);
        }
        other => panic!("expected a settlement without a grant, got {other:?}"),
    }
    assert!(
        recovered
            .recover(&state(progress(true, true, true)))
            .unwrap()
            .is_empty()
    );
}

#[test]
fn stale_and_invalid_requests_are_rejected_without_touching_the_journal() {
    let path = ledger_path("stale");
    let _ = fs::remove_file(&path);
    let ready = progress(true, true, false);

    let mut ledger = TurnInLedger::open(&path).unwrap();
    let staged = match ledger
        .consider(request(2, 7), Some(&ready), &reward())
        .unwrap()
    {
        TurnInDecision::Apply(staged) => staged,
        other => panic!("expected a staged turn-in, got {other:?}"),
    };
    ledger.commit(staged).unwrap();
    let before = fs::read(&path).unwrap();

    let cases = [
        (
            request(0, 9),
            Some(ready.clone()),
            TurnInRejection::InvalidRequest,
        ),
        (
            request(1, 9),
            Some(ready.clone()),
            TurnInRejection::StaleAttempt { recorded: 2 },
        ),
        (
            request(2, 8),
            Some(ready.clone()),
            TurnInRejection::StaleAttempt { recorded: 2 },
        ),
        (
            request(3, 9),
            Some(progress(false, false, false)),
            TurnInRejection::NotAccepted,
        ),
        (
            request(3, 9),
            Some(progress(true, false, false)),
            TurnInRejection::ObjectivesIncomplete,
        ),
        // The caller resolves progress by the requested quest id; a quest it cannot resolve is
        // unknown, and no other quest's progress may stand in for it.
        (request(3, 9), None, TurnInRejection::UnknownQuest),
        (
            TurnInRequest {
                quest_id: 4242,
                attempt: 1,
                request_id: 9,
            },
            None,
            TurnInRejection::UnknownQuest,
        ),
    ];
    for (request, progress, expected) in cases {
        let decision = ledger
            .consider(request, progress.as_ref(), &reward())
            .unwrap();
        match decision {
            TurnInDecision::Reject(rejection) => assert_eq!(rejection, expected),
            other => panic!("expected {expected:?}, got {other:?}"),
        }
        assert_eq!(
            fs::read(&path).unwrap(),
            before,
            "a rejected request must not mutate the journal"
        );
    }

    // The exact recorded request, and any request on an already rewarded quest, replay instead of
    // rejecting or granting again.
    for (request, progress) in [
        (request(2, 7), ready.clone()),
        (request(2, 7), progress(true, true, true)),
        (request(9, 9), progress(true, true, true)),
    ] {
        let decision = ledger
            .consider(request, Some(&progress), &reward())
            .unwrap();
        match decision {
            TurnInDecision::Replay(receipt) => {
                assert_eq!(receipt.quest_id, QUEST);
                assert_eq!(receipt.attempt, 2);
                assert!(!receipt.settled_from_state);
            }
            other => panic!("expected a replay, got {other:?}"),
        }
        assert_eq!(fs::read(&path).unwrap(), before);
    }
}

#[test]
fn a_rewarded_quest_without_a_receipt_is_refused_rather_than_replayed() {
    let path = ledger_path("pre-ledger");
    let _ = fs::remove_file(&path);
    let mut ledger = TurnInLedger::open(&path).unwrap();
    let decision = ledger
        .consider(request(1, 7), Some(&progress(true, true, true)), &reward())
        .unwrap();
    assert!(matches!(
        decision,
        TurnInDecision::Reject(TurnInRejection::AlreadyRewarded)
    ));
}

#[test]
fn a_new_attempt_on_a_recorded_quest_is_refused_until_repeat_semantics_are_recovered() {
    // Neither native proof shows the same quest id turned in twice, so a later attempt on a
    // recorded quest is refused instead of silently granting a second bundle.
    let path = ledger_path("repeat-quest");
    let _ = fs::remove_file(&path);
    let mut ledger = TurnInLedger::open(&path).unwrap();
    let staged = match ledger
        .consider(request(1, 7), Some(&progress(true, true, false)), &reward())
        .unwrap()
    {
        TurnInDecision::Apply(staged) => staged,
        other => panic!("expected a staged turn-in, got {other:?}"),
    };
    ledger.commit(staged).unwrap();

    let decision = ledger
        .consider(
            request(2, 11),
            Some(&progress(true, true, false)),
            &reward(),
        )
        .unwrap();
    assert!(matches!(
        decision,
        TurnInDecision::Reject(TurnInRejection::StaleAttempt { recorded: 1 })
    ));
    assert_eq!(ledger.records(), 2);
}

#[test]
fn malformed_or_unbounded_journals_are_refused() {
    let cases = [
        "{\"schema\":1,\"kind\":\"applied\",\"quest_id\":910004,\"attempt\":1,\"request_id\":7,\"reward\":null}\n",
        "{\"schema\":1,\"kind\":\"intent\",\"quest_id\":910004,\"attempt\":1,\"request_id\":7,\"reward\":{\"coins\":10,\"experience\":25,\"items\":[]}}\n{\"schema\":1,\"kind\":\"intent\",\"quest_id\":910004,\"attempt\":1,\"request_id\":7,\"reward\":{\"coins\":10,\"experience\":25,\"items\":[]}}\n",
        "{\"schema\":1,\"kind\":\"intent\",\"quest_id\":910004,\"attempt\":1,\"request_id\":7,\"reward\":{\"coins\":10,\"experience\":25,\"items\":[]},\"extra\":1}\n",
        "{\"schema\":9,\"kind\":\"intent\",\"quest_id\":910004,\"attempt\":1,\"request_id\":7,\"reward\":{\"coins\":10,\"experience\":25,\"items\":[]}}\n",
    ];
    for (index, body) in cases.iter().enumerate() {
        let path = ledger_path(&format!("malformed-{index}"));
        fs::write(&path, body).unwrap();
        assert!(
            TurnInLedger::open(&path).is_err(),
            "journal variant {index} must be refused"
        );
    }

    let path = ledger_path("oversized");
    fs::write(&path, vec![b'x'; piece_069::MAX_LEDGER_BYTES + 1]).unwrap();
    assert!(TurnInLedger::open(&path).is_err());
    assert!(MAX_LEDGER_RECORDS >= 64);

    for name in [
        "malformed-0",
        "malformed-1",
        "malformed-2",
        "malformed-3",
        "oversized",
    ] {
        let _ = fs::remove_file(ledger_path(name));
    }
}

#[test]
fn staged_turn_ins_are_typed_to_the_quest_that_produced_them() {
    let path = ledger_path("typed");
    let _ = fs::remove_file(&path);
    let mut ledger = TurnInLedger::open(&path).unwrap();
    let staged: TurnInStaged = match ledger
        .consider(request(1, 7), Some(&progress(true, true, false)), &reward())
        .unwrap()
    {
        TurnInDecision::Apply(staged) => staged,
        other => panic!("expected a staged turn-in, got {other:?}"),
    };
    assert_eq!(staged.quest_id(), QUEST);

    // Re-considering the same staged request returns the same intent and writes no second record.
    let again = ledger
        .consider(request(1, 7), Some(&progress(true, true, false)), &reward())
        .unwrap();
    match again {
        TurnInDecision::Apply(again) => assert_eq!(again, staged),
        other => panic!("expected the same staged turn-in, got {other:?}"),
    }
    assert_eq!(ledger.records(), 1);

    let receipt = ledger.commit(staged).unwrap();
    assert_eq!(ledger.receipt(QUEST), Some(receipt));
    assert_eq!(ledger.receipt(4242), None);
}
