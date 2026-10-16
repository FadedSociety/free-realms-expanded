//! Piece 071 (NPC dialogue branching) tests: recovered conversation windows, response menus and
//! quest-phase node selection.
//!
//! Evidence boundary, read `notes/Research/Pieces/071 NPC dialogue branching.md` before the
//! assertions:
//!
//! ORIGINAL (first-party supplied inputs, pinned under `../lab`, outside Git):
//! * `lab/decompiled/server/LibSOE/SOE/BaseCommandPackets.cs` is the supplied server's base
//!   command family: `CommandPacketShowDialog = 3`, `CommandPacketEndDialog = 4`,
//!   `PacketDialogResponse = 6`, `CommandPacketPlayDialogEffect = 28`.
//! * The same family is named by two pinned community repositories, both at source level:
//!   `lab/references/FreeRealms-Legacy/src/Sanctuary.Packet.Common/Extensions/PacketReaderExtensions.cs`
//!   (tunnel opcode 26 sub-opcodes 3/4/6/28) and
//!   `lab/references/OpenSourceFreeRealms/ofrserver/Server/LibSOE/Core/PacketCodes.cs`.
//! * Preserved native reader constructors in `lab/decompiled/client/FreeRealms/pseudocode.c`:
//!   `FUN_009f5890` builds `PacketDialogResponse` with opcode 6 and its single i32 defaulted to
//!   `-1`; `FUN_009f5760` builds the field-less `CommandPacketEndDialog` with opcode 4;
//!   `FUN_00a99bd0` builds `CommandPacketShowDialog` with opcode 3; `FUN_00a9ad20` builds
//!   `CommandPacketPlayDialogEffect` with opcode 28.
//! * Supplied UI definition `lab/working/OSFR.Client/OSFR Client/UI/UiModules/Main/wndDialog.xml`
//!   is the `Conversation` window with five response buttons `btn0..btn4`.
//! * Preserved client Lua `lab/decompiled/client-ui/ScriptsBase.lua`:
//!   `DialogHandler:SelectOption` forwards to `Ui.DialogSelect(opt)`, the dialog context quits
//!   through `Ui.DialogEnd()`, `QuestHandler:AddDialogResponseNodes` adds `questYes` as response
//!   option 1 and `questNo` as 2 when the dialog is declinable, and the end screen adds
//!   `questThanks` as response option 3. `Main_wndQuest*_OnUserEvent` maps the response options
//!   back to `QuestScreen.AcceptQuest()`, `Hide()`/`HideEnd()` and `QuestScreen.EndQuest(true)`.
//! * Supplied locale export (`lab/reports/locale-id-to-text.json`, hash in the note): 8125
//!   `Can you help me?` (the default NPC dialog text), 71313 `Yes, I accept!`,
//!   71314 `No! I decline.`, 1911 `Thanks!`.
//! * Pinned wiki revisions (community evidence, CC BY-SA, revision-pinned) 27815
//!   `Quest:Your First Weapon` and 28955 `Quest:Fighting off the Pack` carry the recovered
//!   per-phase NPC lines that are embedded below.
//!
//! RECONSTRUCTED here (labelled, not historical): the definition document schema, the
//! `quests::QuestProgress` -> phase mapping, every bound and every rejection. The original server
//! dialogue database did not ship, so no original node graph is claimed: the two Cleetus
//! definitions are assembled from the pinned wiki lines with their revisions attached, and the
//! phases that the wiki does not record stay absent instead of being invented.
//!
//! The module is not registered in `src/lib.rs` (lead-owned). Including the exact source file
//! keeps this target compiling against the real artifact. Integration step for the lead:
//! register the module and drop this `#[path]` include.

#[path = "../src/quests.rs"]
#[allow(dead_code)]
mod quests;

#[path = "../src/restoration/piece_071.rs"]
#[allow(dead_code)]
mod piece_071;

use piece_071::{
    BASE_COMMAND_PACKET_OPCODE, DEFAULT_NPC_DIALOG_LOCALE_ID, DIALOG_RESPONSE_OPCODE,
    DIALOGUE_SCHEMA_VERSION, DialogueAction, DialogueDefinition, DialogueReply, DialogueText,
    DialogueWindow, END_DIALOG_OPCODE, Evidence, FARNUM_CHAIN_WIKI_REVISIONS,
    MAX_CONVERSATION_RESPONSES, PLAY_DIALOG_EFFECT_OPCODE, QUEST_ACCEPT_RESPONSE,
    QUEST_DECLINE_RESPONSE, QUEST_NO_LOCALE_ID, QUEST_THANKS_LOCALE_ID, QUEST_THANKS_RESPONSE,
    QUEST_YES_LOCALE_ID, QuestPhase, SHOW_DIALOG_OPCODE, UNSET_RESPONSE_INDEX,
    farnum_cleetus_dialogues, quest_phase, respond, select_node, validate,
};
use quests::QuestProgress;

fn progress(accepted: bool, completed: bool, rewarded: bool) -> QuestProgress {
    QuestProgress {
        accepted,
        counts: vec![0],
        completed,
        rewarded,
    }
}

fn literal_text(def: &DialogueDefinition, node_id: &str) -> String {
    let node = def
        .nodes
        .iter()
        .find(|node| node.id == node_id)
        .unwrap_or_else(|| panic!("node {node_id}"));
    node.text.literal.clone().expect("literal node text")
}

#[test]
fn recovered_dialog_opcodes_and_menu_limits_are_pinned() {
    assert_eq!(BASE_COMMAND_PACKET_OPCODE, 26);
    assert_eq!(SHOW_DIALOG_OPCODE, 3);
    assert_eq!(END_DIALOG_OPCODE, 4);
    assert_eq!(DIALOG_RESPONSE_OPCODE, 6);
    assert_eq!(PLAY_DIALOG_EFFECT_OPCODE, 28);
    assert_eq!(UNSET_RESPONSE_INDEX, -1);
    assert_eq!(MAX_CONVERSATION_RESPONSES, 5);
    assert_eq!(DEFAULT_NPC_DIALOG_LOCALE_ID, 8125);
    assert_eq!(QUEST_YES_LOCALE_ID, 71313);
    assert_eq!(QUEST_NO_LOCALE_ID, 71314);
    assert_eq!(QUEST_THANKS_LOCALE_ID, 1911);
    assert_eq!(QUEST_ACCEPT_RESPONSE, 1);
    assert_eq!(QUEST_DECLINE_RESPONSE, 2);
    assert_eq!(QUEST_THANKS_RESPONSE, 3);
    assert_eq!(DIALOGUE_SCHEMA_VERSION, 1);
    assert_eq!(FARNUM_CHAIN_WIKI_REVISIONS, [27815, 28955]);
}

#[test]
fn quest_phase_follows_the_recovered_progress_flags() {
    assert_eq!(quest_phase(None), QuestPhase::NotAccepted);
    assert_eq!(
        quest_phase(Some(&progress(false, false, false))),
        QuestPhase::NotAccepted
    );
    assert_eq!(
        quest_phase(Some(&progress(true, false, false))),
        QuestPhase::InProgress
    );
    assert_eq!(
        quest_phase(Some(&progress(true, true, false))),
        QuestPhase::ReadyToTurnIn
    );
    assert_eq!(
        quest_phase(Some(&progress(true, true, true))),
        QuestPhase::Rewarded
    );
}

#[test]
fn cleetus_definitions_carry_pinned_wiki_revisions_and_unbound_original_ids() {
    let definitions = farnum_cleetus_dialogues();
    assert_eq!(definitions.len(), 2);
    assert_eq!(definitions[0].npc.name, "Cleetus Farnum".to_string());
    assert_eq!(definitions[0].quest_name, "Quest:Your First Weapon");
    assert_eq!(definitions[1].quest_name, "Quest:Fighting off the Pack");
    for definition in &definitions {
        validate(definition).expect("valid recovered definition");
        assert_eq!(definition.schema_version, DIALOGUE_SCHEMA_VERSION);
        // The supplied journal binds 13 opening hub ids without titles; the pinned wiki pages do
        // not carry numeric ids, so every binding stays an explicit gap.
        assert_eq!(definition.original_quest_id, None);
    }
    assert_eq!(
        definitions[0].next_quest_name.as_deref(),
        Some("Quest:Fighting off the Pack")
    );
    assert_eq!(
        definitions[1].next_quest_name.as_deref(),
        Some("Quest:Eviction Notice")
    );

    let revisions: Vec<u64> = definitions
        .iter()
        .flat_map(|definition| definition.nodes.iter())
        .map(|node| match &node.text.evidence {
            Evidence::WikiRevision { revision_id, .. } => *revision_id,
            other => panic!("node text is not wiki-sourced: {other:?}"),
        })
        .collect();
    assert!(
        revisions
            .iter()
            .all(|revision| *revision == 27815 || *revision == 28955)
    );
    assert!(revisions.contains(&27815) && revisions.contains(&28955));
}

#[test]
fn recovered_phase_lines_are_embedded_verbatim() {
    let definitions = farnum_cleetus_dialogues();
    let weapon = &definitions[0];
    assert_eq!(
        literal_text(weapon, "offer"),
        "These Hooligans have been causing us a lot of problems and picking up after a mess is \
         only part of the solution. We need to stop them from breaking things in the first place."
    );
    assert_eq!(
        literal_text(weapon, "objective"),
        "Since you've been so helpful around here I'm going to give you everything you need to \
         craft yourself a better hammer. Use this Anvil to forge a Spunky Scrapper Hammer."
    );
    assert_eq!(
        literal_text(weapon, "turn-in"),
        "That's one mighty hammer, I hope you won't have to put it to good use too often."
    );

    let pack = &definitions[1];
    assert_eq!(
        literal_text(pack, "offer"),
        "Unfortunately now is one of those times to where you'll need to use that hammer. Use it \
         to defeat 5 Hooligan Wolves. Be sure to switch to your Brawler job. Return to me when \
         you are finished."
    );
    assert_eq!(
        literal_text(pack, "turn-in"),
        "We should be having less wolf attacks after the trashing that you gave those Hooligan \
         Wolves."
    );
}

#[test]
fn quest_start_window_offers_the_recovered_accept_and_decline_responses() {
    let definitions = farnum_cleetus_dialogues();
    let offer = select_node(&definitions[0], QuestPhase::NotAccepted).expect("offer node");
    assert_eq!(offer.window, DialogueWindow::QuestStart);
    assert!(offer.declinable);
    assert_eq!(offer.choices.len(), 2);
    assert_eq!(offer.choices[0].response, QUEST_ACCEPT_RESPONSE);
    assert_eq!(offer.choices[0].action, DialogueAction::AcceptQuest);
    assert_eq!(offer.choices[0].label.locale_id, Some(QUEST_YES_LOCALE_ID));
    assert_eq!(offer.choices[1].response, QUEST_DECLINE_RESPONSE);
    assert_eq!(offer.choices[1].action, DialogueAction::Decline);
    assert_eq!(offer.choices[1].label.locale_id, Some(QUEST_NO_LOCALE_ID));

    assert_eq!(
        respond(
            &definitions[0],
            QuestPhase::NotAccepted,
            QUEST_ACCEPT_RESPONSE
        )
        .unwrap(),
        DialogueReply::Accepted
    );
    assert_eq!(
        respond(
            &definitions[0],
            QuestPhase::NotAccepted,
            QUEST_DECLINE_RESPONSE
        )
        .unwrap(),
        DialogueReply::Declined
    );
}

#[test]
fn quest_end_window_turns_in_through_the_recovered_thanks_response() {
    let definitions = farnum_cleetus_dialogues();
    let turn_in = select_node(&definitions[0], QuestPhase::ReadyToTurnIn).expect("turn-in node");
    assert_eq!(turn_in.window, DialogueWindow::QuestEnd);
    assert!(!turn_in.declinable);
    assert_eq!(turn_in.choices.len(), 1);
    assert_eq!(turn_in.choices[0].response, QUEST_THANKS_RESPONSE);
    assert_eq!(turn_in.choices[0].action, DialogueAction::TurnInQuest);
    assert_eq!(
        turn_in.choices[0].label.locale_id,
        Some(QUEST_THANKS_LOCALE_ID)
    );
    assert_eq!(
        respond(
            &definitions[0],
            QuestPhase::ReadyToTurnIn,
            QUEST_THANKS_RESPONSE
        )
        .unwrap(),
        DialogueReply::TurnedIn
    );
}

#[test]
fn objective_line_has_no_menu_and_closes_through_the_end_dialog_command() {
    let definitions = farnum_cleetus_dialogues();
    let objective = select_node(&definitions[0], QuestPhase::InProgress).expect("objective node");
    assert_eq!(objective.window, DialogueWindow::Conversation);
    // The preserved `CommandPacketEndDialog` (opcode 4) carries no fields: a line without a menu
    // is closed by the server, so there is no response index to send back.
    assert!(objective.choices.is_empty());
    let error = respond(&definitions[0], QuestPhase::InProgress, 1).unwrap_err();
    assert!(error.to_string().contains("no response menu"), "{error}");
}

#[test]
fn unrecorded_phases_are_refused_instead_of_invented() {
    let definitions = farnum_cleetus_dialogues();
    // The pinned `Quest:Fighting off the Pack` page records an offer and a turn-in line only.
    assert!(select_node(&definitions[1], QuestPhase::InProgress).is_none());
    assert!(respond(&definitions[1], QuestPhase::InProgress, 1).is_err());

    // A rewarded quest is never re-offered by the journal.
    assert!(select_node(&definitions[0], QuestPhase::Rewarded).is_none());
    assert!(respond(&definitions[0], QuestPhase::Rewarded, QUEST_ACCEPT_RESPONSE).is_err());
}

#[test]
fn unset_stale_and_out_of_range_responses_are_rejected() {
    let definitions = farnum_cleetus_dialogues();
    let unset = respond(
        &definitions[0],
        QuestPhase::NotAccepted,
        UNSET_RESPONSE_INDEX,
    )
    .unwrap_err();
    assert!(unset.to_string().contains("unset"), "{unset}");
    assert!(respond(&definitions[0], QuestPhase::NotAccepted, 0).is_err());
    assert!(respond(&definitions[0], QuestPhase::NotAccepted, 3).is_err());
    // Response 3 belongs to the end window, not to the offer window.
    assert!(
        respond(
            &definitions[0],
            QuestPhase::NotAccepted,
            QUEST_THANKS_RESPONSE
        )
        .is_err()
    );
    // Response 1 on the turn-in node is not the recovered thanks response.
    assert!(respond(&definitions[0], QuestPhase::ReadyToTurnIn, 1).is_err());
}

#[test]
fn validation_rejects_six_options_gaps_duplicates_and_unsourced_text() {
    let definitions = farnum_cleetus_dialogues();

    let mut six = definitions[0].clone();
    let node = six
        .nodes
        .iter_mut()
        .find(|node| node.id == "objective")
        .expect("objective node");
    for response in 1..=MAX_CONVERSATION_RESPONSES as i32 + 1 {
        node.choices.push(piece_071::DialogueChoice {
            response,
            label: DialogueText {
                locale_id: None,
                literal: Some(format!("response {response}")),
                evidence: Evidence::RecoveredClient {
                    symbol: "test".into(),
                },
            },
            action: DialogueAction::Close { effect: None },
        });
    }
    assert!(validate(&six).is_err());

    let mut gapped = definitions[0].clone();
    let node = gapped
        .nodes
        .iter_mut()
        .find(|node| node.id == "offer")
        .expect("offer node");
    node.choices[1].response = 4;
    assert!(validate(&gapped).is_err());

    let mut duplicated = definitions[0].clone();
    let duplicate = duplicated.nodes[1].clone();
    duplicated.nodes.push(duplicate);
    assert!(validate(&duplicated).is_err());

    let mut unsourced = definitions[0].clone();
    unsourced.nodes[0].text.literal = Some("invented line".into());
    unsourced.nodes[0].text.evidence = Evidence::Reconstructed {
        policy: String::new(),
    };
    assert!(validate(&unsourced).is_err());

    let mut two_sources = definitions[0].clone();
    two_sources.nodes[0].text.locale_id = Some(DEFAULT_NPC_DIALOG_LOCALE_ID);
    assert!(validate(&two_sources).is_err());

    // The recovered `questYes` label must come from the supplied locale id, not a copied string.
    let mut wrong_locale = definitions[0].clone();
    wrong_locale.nodes[0].choices[0].label.evidence = Evidence::SuppliedLocale { locale_id: 1 };
    assert!(validate(&wrong_locale).is_err());
}

#[test]
fn validation_keeps_the_recovered_window_menu_shapes() {
    let definitions = farnum_cleetus_dialogues();

    // A declinable offer must keep the recovered yes/no pair.
    let mut undeclared = definitions[0].clone();
    undeclared.nodes[0].declinable = false;
    assert!(validate(&undeclared).is_err());

    // The recovered end window has exactly one response node, option 3.
    let mut wrong_thanks = definitions[0].clone();
    let node = wrong_thanks
        .nodes
        .iter_mut()
        .find(|node| node.id == "turn-in")
        .expect("turn-in node");
    node.choices[0].response = 1;
    assert!(validate(&wrong_thanks).is_err());

    // The recovered quest-start window never carries a third response option.
    let mut third_option = definitions[0].clone();
    let node = third_option
        .nodes
        .iter_mut()
        .find(|node| node.id == "offer")
        .expect("offer node");
    node.choices.push(piece_071::DialogueChoice {
        response: QUEST_THANKS_RESPONSE,
        label: DialogueText {
            locale_id: Some(QUEST_THANKS_LOCALE_ID),
            literal: None,
            evidence: Evidence::SuppliedLocale {
                locale_id: QUEST_THANKS_LOCALE_ID,
            },
        },
        action: DialogueAction::Close { effect: None },
    });
    assert!(validate(&third_option).is_err());

    // A definition without a source for its own node text is refused.
    let mut blank_evidence = definitions[0].clone();
    blank_evidence.nodes[0].text.evidence = Evidence::RecoveredClient {
        symbol: String::new(),
    };
    assert!(validate(&blank_evidence).is_err());
}

#[test]
fn recovered_definition_round_trips_through_json() {
    let definitions = farnum_cleetus_dialogues();
    let encoded = serde_json::to_string(&definitions).expect("serialize");
    let decoded: Vec<DialogueDefinition> = serde_json::from_str(&encoded).expect("deserialize");
    assert_eq!(decoded, definitions);
    for definition in &decoded {
        validate(definition).expect("round-tripped definition stays valid");
    }
}
