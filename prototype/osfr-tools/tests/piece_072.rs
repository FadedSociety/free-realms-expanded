//! Piece 072 (starting-area historical coverage) tests: the recovered client
//! objective (`45/10` registration, `45/7` completion) contract used by the
//! original Hero Journal close micro-tutorial, plus the recovered NPX4.0
//! directed-play prompt evidence table.
//!
//! Evidence boundary, read before the assertions:
//!
//! ORIGINAL (first-party client inputs; hashes pinned in the lab bundle):
//! * `decompiled/client-ui/ScriptsBase.lua` `StoryBook:Hide` re-enables
//!   movement/keyboard, and only inside `Ui.IsInNPXTutorial()` kills
//!   `FtesNpxBookTab/Sticker/Quest/Open` and emits
//!   `Ui.OnObjectiveEvent("onStoryBookClose")` (line 14722, the only occurrence
//!   of that event string in the file).
//! * `Resources/CodeStringMappings.txt`: `FtesNpxBookExit^5100079^`,
//!   `FtesNpxDppCamera1/2/3^10258/10259/10260^`,
//!   `FtesNpxDppMovement1/2/3^10261/10262/10263^`.
//! * The matching locale rows: 5100079 "Click the X to close the Hero's
//!   Journal.", 10258-10260 the three Magic Sphere look prompts, 10261-10263
//!   the obstacle WASD/Space prompts.
//! * `FirstTimeEvent` callbacks play sound definitions 18264-18266 (camera) and
//!   18267-18269 (movement); `FtesNpxBookExit` sets `shouldAllowHide` and plays
//!   no sound.
//! * Recovered wire shapes: S2C `45/10 UIEvent` =
//!   `u16(45), u8(10), i32 objectiveID, i32 string byte length, string bytes`;
//!   C2S `45/7 ClientComplete` = `u16(45), u8(7), i32 objectiveID`, exactly
//!   7 bytes. `45/8 ClientClear` and `45/9 ClientCompleteFailed` are recovered
//!   as messages with those meanings; their byte layout is NOT recovered, so
//!   this module models their state effect only and never encodes them.
//!
//! RECONSTRUCTED (labelled, not historical): the session-scoped registry, the
//! zone/phase guard, the once-only completion ledger, the event-name length
//! bound, and the objective id itself (the original Hero Journal objective id is
//! unrecovered and stays `None` until a capture proves it).
//!
//! NOT CLAIMED: the original objective id, camera globe / movement barrier
//! bindings, the original server ordering of the directed-play prompts, or any
//! native journal-close run. Those stay gaps.
// The module is not registered in `src/lib.rs` (lead-owned). Including the exact
// source file keeps this target compiling against the real artifact. Integration
// step for the lead: register the module and drop this `#[path]` include.
#[path = "../src/restoration/piece_072.rs"]
#[allow(dead_code)]
mod piece_072;

use piece_072::{
    CLIENT_COMPLETE_BYTES, ClientObjectiveRegistry, InputHint, JOURNAL_CLOSE_INSTRUCTION_LOCALE,
    ORIGINAL_TUTORIAL_GATES, ObjectiveError, STORY_BOOK_CLOSE_EVENT, STORY_BOOK_CLOSE_OBJECTIVE_ID,
    ServerBinding, TutorialPrompt, UI_EVENT_CLIENT_CLEAR, UI_EVENT_CLIENT_COMPLETE,
    UI_EVENT_CLIENT_COMPLETE_FAILED, UI_EVENT_OPCODE, UI_EVENT_REGISTER, ZonePhase,
    encode_ui_event_registration,
};

fn scope() -> ZonePhase {
    ZonePhase {
        zone_id: 0,
        phase: 1,
    }
}

/// Synthetic objective id for tests only; the original id stays unrecovered.
const TEST_OBJECTIVE_ID: i32 = 910_001_00;

fn completion_packet(objective_id: i32) -> Vec<u8> {
    let mut packet = Vec::with_capacity(CLIENT_COMPLETE_BYTES);
    packet.extend((UI_EVENT_OPCODE as u16).to_le_bytes());
    packet.push(UI_EVENT_CLIENT_COMPLETE);
    packet.extend(objective_id.to_le_bytes());
    packet
}

#[test]
fn recovered_message_types_and_length() {
    assert_eq!(UI_EVENT_OPCODE, 45);
    assert_eq!(UI_EVENT_REGISTER, 10);
    assert_eq!(UI_EVENT_CLIENT_COMPLETE, 7);
    assert_eq!(UI_EVENT_CLIENT_CLEAR, 8);
    assert_eq!(UI_EVENT_CLIENT_COMPLETE_FAILED, 9);
    assert_eq!(CLIENT_COMPLETE_BYTES, 7);
    assert_eq!(STORY_BOOK_CLOSE_EVENT, "onStoryBookClose");
    assert_eq!(JOURNAL_CLOSE_INSTRUCTION_LOCALE, 5100079);
    // The original Hero Journal objective id is unrecovered.
    assert_eq!(STORY_BOOK_CLOSE_OBJECTIVE_ID, None);
    assert_eq!(
        completion_packet(TEST_OBJECTIVE_ID).len(),
        CLIENT_COMPLETE_BYTES
    );
}

#[test]
fn registration_packet_matches_the_recovered_layout() {
    let packet = encode_ui_event_registration(TEST_OBJECTIVE_ID, STORY_BOOK_CLOSE_EVENT).unwrap();
    let event = STORY_BOOK_CLOSE_EVENT.as_bytes();
    assert_eq!(&packet[0..2], &(UI_EVENT_OPCODE as u16).to_le_bytes());
    assert_eq!(packet[2], UI_EVENT_REGISTER);
    assert_eq!(&packet[3..7], &TEST_OBJECTIVE_ID.to_le_bytes());
    assert_eq!(&packet[7..11], &(event.len() as i32).to_le_bytes());
    assert_eq!(&packet[11..], event);
    assert_eq!(packet.len(), 11 + event.len());
}

#[test]
fn registration_rejects_invalid_ids_and_event_names() {
    assert_eq!(
        encode_ui_event_registration(0, STORY_BOOK_CLOSE_EVENT).unwrap_err(),
        ObjectiveError::InvalidObjectiveId(0)
    );
    assert_eq!(
        encode_ui_event_registration(TEST_OBJECTIVE_ID, "").unwrap_err(),
        ObjectiveError::EmptyEventName
    );
    let long = "e".repeat(piece_072::MAX_EVENT_NAME_BYTES + 1);
    assert_eq!(
        encode_ui_event_registration(TEST_OBJECTIVE_ID, &long).unwrap_err(),
        ObjectiveError::EventNameTooLong
    );
    assert_eq!(
        encode_ui_event_registration(TEST_OBJECTIVE_ID, "bad\nevent").unwrap_err(),
        ObjectiveError::InvalidEventName
    );
}

#[test]
fn client_completion_requires_exactly_seven_bytes() {
    let good = completion_packet(TEST_OBJECTIVE_ID);
    assert_eq!(
        piece_072::decode_client_completion(&good).unwrap(),
        TEST_OBJECTIVE_ID
    );

    let short = &good[..CLIENT_COMPLETE_BYTES - 1];
    assert_eq!(
        piece_072::decode_client_completion(short).unwrap_err(),
        ObjectiveError::WrongPacketLength(short.len())
    );

    let mut trailing = good.clone();
    trailing.push(0);
    assert_eq!(
        piece_072::decode_client_completion(&trailing).unwrap_err(),
        ObjectiveError::WrongPacketLength(CLIENT_COMPLETE_BYTES + 1)
    );

    let mut wrong_opcode = good.clone();
    wrong_opcode[0] = 46;
    assert_eq!(
        piece_072::decode_client_completion(&wrong_opcode).unwrap_err(),
        ObjectiveError::WrongMessageType
    );

    let mut wrong_kind = good.clone();
    wrong_kind[2] = UI_EVENT_REGISTER;
    assert_eq!(
        piece_072::decode_client_completion(&wrong_kind).unwrap_err(),
        ObjectiveError::WrongMessageType
    );

    let zero = completion_packet(0);
    assert_eq!(
        piece_072::decode_client_completion(&zero).unwrap_err(),
        ObjectiveError::InvalidObjectiveId(0)
    );
}

#[test]
fn registry_accepts_a_registered_completion_exactly_once() {
    let mut registry = ClientObjectiveRegistry::new(scope());
    registry
        .register(TEST_OBJECTIVE_ID, STORY_BOOK_CLOSE_EVENT, scope())
        .unwrap();
    assert_eq!(registry.outstanding(), vec![TEST_OBJECTIVE_ID]);

    let completed = registry
        .complete(&completion_packet(TEST_OBJECTIVE_ID), scope())
        .unwrap();
    assert_eq!(completed.objective_id, TEST_OBJECTIVE_ID);
    assert_eq!(completed.event, STORY_BOOK_CLOSE_EVENT);
    assert_eq!(registry.outstanding(), vec![TEST_OBJECTIVE_ID]);

    assert_eq!(
        registry
            .complete(&completion_packet(TEST_OBJECTIVE_ID), scope())
            .unwrap_err(),
        ObjectiveError::AlreadyCompleted(TEST_OBJECTIVE_ID)
    );
}

#[test]
fn registry_rejects_unknown_and_stale_objectives() {
    let mut registry = ClientObjectiveRegistry::new(scope());
    assert_eq!(
        registry
            .complete(&completion_packet(TEST_OBJECTIVE_ID), scope())
            .unwrap_err(),
        ObjectiveError::UnknownObjective(TEST_OBJECTIVE_ID)
    );

    registry
        .register(TEST_OBJECTIVE_ID, STORY_BOOK_CLOSE_EVENT, scope())
        .unwrap();
    let dropped = registry.change_scope(ZonePhase {
        zone_id: 0,
        phase: 2,
    });
    assert_eq!(dropped, vec![TEST_OBJECTIVE_ID]);
    assert_eq!(registry.outstanding(), Vec::<i32>::new());
    assert_eq!(
        registry
            .complete(&completion_packet(TEST_OBJECTIVE_ID), registry.scope())
            .unwrap_err(),
        ObjectiveError::UnknownObjective(TEST_OBJECTIVE_ID)
    );
}

#[test]
fn registry_rejects_a_completion_from_another_zone_or_phase() {
    let mut registry = ClientObjectiveRegistry::new(scope());
    registry
        .register(TEST_OBJECTIVE_ID, STORY_BOOK_CLOSE_EVENT, scope())
        .unwrap();
    let elsewhere = ZonePhase {
        zone_id: 40,
        phase: 1,
    };
    assert_eq!(
        registry
            .complete(&completion_packet(TEST_OBJECTIVE_ID), elsewhere)
            .unwrap_err(),
        ObjectiveError::WrongScope {
            expected: scope(),
            observed: elsewhere
        }
    );
    // The refused packet must not consume the objective.
    assert!(
        registry
            .complete(&completion_packet(TEST_OBJECTIVE_ID), scope())
            .is_ok()
    );
}

#[test]
fn registration_is_scope_bound_and_conflicts_are_refused() {
    let mut registry = ClientObjectiveRegistry::new(scope());
    let elsewhere = ZonePhase {
        zone_id: 40,
        phase: 1,
    };
    assert_eq!(
        registry
            .register(TEST_OBJECTIVE_ID, STORY_BOOK_CLOSE_EVENT, elsewhere)
            .unwrap_err(),
        ObjectiveError::WrongScope {
            expected: scope(),
            observed: elsewhere
        }
    );
    registry
        .register(TEST_OBJECTIVE_ID, STORY_BOOK_CLOSE_EVENT, scope())
        .unwrap();
    // Re-registering the same objective with the same event is idempotent.
    registry
        .register(TEST_OBJECTIVE_ID, STORY_BOOK_CLOSE_EVENT, scope())
        .unwrap();
    assert_eq!(
        registry
            .register(TEST_OBJECTIVE_ID, "otherEvent", scope())
            .unwrap_err(),
        ObjectiveError::ConflictingRegistration(TEST_OBJECTIVE_ID)
    );
}

#[test]
fn clear_and_complete_failed_allow_a_retry_while_clear_removes_the_objective() {
    let mut registry = ClientObjectiveRegistry::new(scope());
    registry
        .register(TEST_OBJECTIVE_ID, STORY_BOOK_CLOSE_EVENT, scope())
        .unwrap();
    registry
        .complete(&completion_packet(TEST_OBJECTIVE_ID), scope())
        .unwrap();

    // 45/9 clears the completed flag while the objective stays enabled.
    registry.complete_failed(TEST_OBJECTIVE_ID).unwrap();
    // A second 45/9 has nothing to clear and is refused.
    assert_eq!(
        registry.complete_failed(TEST_OBJECTIVE_ID).unwrap_err(),
        ObjectiveError::NotCompleted(TEST_OBJECTIVE_ID)
    );
    // The cleared objective can be completed again.
    assert!(
        registry
            .complete(&completion_packet(TEST_OBJECTIVE_ID), scope())
            .is_ok()
    );

    registry.clear(TEST_OBJECTIVE_ID).unwrap();
    assert_eq!(
        registry
            .complete(&completion_packet(TEST_OBJECTIVE_ID), scope())
            .unwrap_err(),
        ObjectiveError::UnknownObjective(TEST_OBJECTIVE_ID)
    );
    assert_eq!(
        registry.clear(TEST_OBJECTIVE_ID).unwrap_err(),
        ObjectiveError::UnknownObjective(TEST_OBJECTIVE_ID)
    );
}

#[test]
fn a_restart_starts_from_an_empty_session_registry() {
    let mut previous = ClientObjectiveRegistry::new(scope());
    previous
        .register(TEST_OBJECTIVE_ID, STORY_BOOK_CLOSE_EVENT, scope())
        .unwrap();
    previous
        .complete(&completion_packet(TEST_OBJECTIVE_ID), scope())
        .unwrap();

    let mut restarted = ClientObjectiveRegistry::new(scope());
    assert_eq!(restarted.outstanding(), Vec::<i32>::new());
    assert_eq!(
        restarted
            .complete(&completion_packet(TEST_OBJECTIVE_ID), scope())
            .unwrap_err(),
        ObjectiveError::UnknownObjective(TEST_OBJECTIVE_ID)
    );
    // The restarted session can register and complete the same objective again.
    restarted
        .register(TEST_OBJECTIVE_ID, STORY_BOOK_CLOSE_EVENT, scope())
        .unwrap();
    assert!(
        restarted
            .complete(&completion_packet(TEST_OBJECTIVE_ID), scope())
            .is_ok()
    );
}

#[test]
fn original_tutorial_gate_table_matches_the_recovered_evidence() {
    assert_eq!(ORIGINAL_TUTORIAL_GATES.len(), 7);
    let expected: [(TutorialPrompt, &str, u32, Option<u32>, InputHint); 7] = [
        (
            TutorialPrompt::JournalClose,
            "FtesNpxBookExit",
            JOURNAL_CLOSE_INSTRUCTION_LOCALE,
            None,
            InputHint::CloseJournal,
        ),
        (
            TutorialPrompt::DirectedPlayCamera1,
            "FtesNpxDppCamera1",
            10258,
            Some(18264),
            InputHint::LookWithRightMouse,
        ),
        (
            TutorialPrompt::DirectedPlayCamera2,
            "FtesNpxDppCamera2",
            10259,
            Some(18265),
            InputHint::LookWithRightMouse,
        ),
        (
            TutorialPrompt::DirectedPlayCamera3,
            "FtesNpxDppCamera3",
            10260,
            Some(18266),
            InputHint::LookWithRightMouse,
        ),
        (
            TutorialPrompt::DirectedPlayMovement1,
            "FtesNpxDppMovement1",
            10261,
            Some(18267),
            InputHint::Move,
        ),
        (
            TutorialPrompt::DirectedPlayMovement2,
            "FtesNpxDppMovement2",
            10262,
            Some(18268),
            InputHint::MoveAndJump,
        ),
        (
            TutorialPrompt::DirectedPlayMovement3,
            "FtesNpxDppMovement3",
            10263,
            Some(18269),
            InputHint::MoveAndJump,
        ),
    ];
    for (gate, (prompt, callback, text_locale_id, sound_id, input_hint)) in
        ORIGINAL_TUTORIAL_GATES.iter().zip(expected)
    {
        assert_eq!(gate.prompt, prompt);
        assert_eq!(gate.callback, callback);
        assert_eq!(gate.text_locale_id, text_locale_id);
        assert_eq!(gate.sound_id, sound_id);
        assert_eq!(gate.input_hint, input_hint);
    }
}

#[test]
fn only_the_journal_close_gate_has_a_recovered_objective_event() {
    let journal = ORIGINAL_TUTORIAL_GATES
        .iter()
        .find(|gate| gate.prompt == TutorialPrompt::JournalClose)
        .expect("journal close row");
    assert_eq!(
        journal.server_binding,
        ServerBinding::ObjectiveEventRecovered
    );
    assert_eq!(journal.objective_event, Some(STORY_BOOK_CLOSE_EVENT));
    for gate in ORIGINAL_TUTORIAL_GATES
        .iter()
        .filter(|gate| gate.prompt != TutorialPrompt::JournalClose)
    {
        assert_eq!(gate.server_binding, ServerBinding::Unrecovered);
        assert_eq!(gate.objective_event, None);
    }
}
