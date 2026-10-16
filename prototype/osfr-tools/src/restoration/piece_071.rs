//! Work item 071: NPC dialogue branching -- recovered conversation menus and quest-phase nodes.
//!
//! The original server dialogue database did not ship in either supplied archive, so this module
//! owns the recovered *client* dialogue contract plus the phase-selection policy, and refuses to
//! invent the node graph that has no source.
//!
//! ORIGINAL facts used here (private evidence under `../lab`, outside Git; full paths and the
//! extraction commands are in `notes/Research/Pieces/071 NPC dialogue branching.md`):
//!
//! * Base command family opcodes, agreed by the supplied server
//!   (`lab/decompiled/server/LibSOE/SOE/BaseCommandPackets.cs`) and two pinned community
//!   repositories (`FreeRealms-Legacy` `PacketReaderExtensions.cs`, `OpenSourceFreeRealms`
//!   `PacketCodes.cs`): `CommandPacketShowDialog = 3`, `CommandPacketEndDialog = 4`,
//!   `PacketDialogResponse = 6`, `CommandPacketPlayDialogEffect = 28`, all carried inside tunnel
//!   opcode 26 (`BaseCommandPacket`).
//! * Preserved native reader constructors in `lab/decompiled/client/FreeRealms/pseudocode.c`:
//!   `FUN_009f5890` builds `PacketDialogResponse` with opcode 6 and one i32 defaulted to `-1`;
//!   `FUN_009f5760` builds the field-less `CommandPacketEndDialog` (opcode 4); `FUN_00a99bd0`
//!   builds `CommandPacketShowDialog` (opcode 3); `FUN_00a9ad20` builds
//!   `CommandPacketPlayDialogEffect` (opcode 28). The exact `ShowDialog` payload layout is not
//!   recovered and is deliberately not modelled here.
//! * Supplied UI definition `UI/UiModules/Main/wndDialog.xml` (`Conversation`, buttons
//!   `btn0..btn4`) bounds a plain conversation to five response options.
//! * Preserved client Lua `lab/decompiled/client-ui/ScriptsBase.lua`: `DialogHandler:SelectOption`
//!   forwards the clicked response to `Ui.DialogSelect(opt)` and quits through `Ui.DialogEnd()`;
//!   `QuestHandler:AddDialogResponseNodes` adds `questYes` as option 1 and `questNo` as option 2
//!   only while the dialog is declinable; the end screen adds exactly one option, `questThanks`
//!   (=3); `Main_wndQuestStart_swfQuestStart_OnUserEvent` maps 1 to
//!   `QuestScreen.AcceptQuest()`, 2 to a plain hide, and the end-screen handler maps 3 to
//!   `QuestScreen.EndQuest(true)`.
//! * Supplied locale export: 8125 `Can you help me?` (the default NPC line), 71313
//!   `Yes, I accept!`, 71314 `No! I decline.`, 1911 `Thanks!`.
//! * Pinned wiki revisions 27815 (`Quest:Your First Weapon`) and 28955
//!   (`Quest:Fighting off the Pack`) carry the per-phase Cleetus Farnum lines embedded in
//!   [`farnum_cleetus_dialogues`]. Those two pages are community evidence, CC BY-SA, with the
//!   revision id attached to every embedded line.
//!
//! RECONSTRUCTED here (labelled, not historical): the definition schema, the
//! [`quests::QuestProgress`] -> [`QuestPhase`] mapping, the window-shape validation and every
//! rejection. Phases the pinned pages do not record stay absent so the caller gets
//! `Err`/`None` instead of a fabricated line.
//!
//! Integration step for the lead: register the module in `src/lib.rs` and drive
//! [`select_node`]/[`respond`] from the NPC interaction path. The module never mutates quest
//! state, never grants rewards and never writes a packet; the returned [`DialogueReply`] is an
//! instruction for the existing quest/gameplay owners.

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::io;

use crate::quests::QuestProgress;

/// Schema version of the definition document this module understands.
pub const DIALOGUE_SCHEMA_VERSION: i64 = 1;

/// Tunnel opcode that carries the whole base command family (supplied `BaseCommandPacket`).
pub const BASE_COMMAND_PACKET_OPCODE: u16 = 26;
/// Server-to-client dialog display command.
pub const SHOW_DIALOG_OPCODE: u16 = 3;
/// Client-declined/server-closed dialog command; the preserved constructor adds no fields.
pub const END_DIALOG_OPCODE: u16 = 4;
/// Client-to-server response index packet. `FUN_009f5890` defaults its single i32 to -1.
pub const DIALOG_RESPONSE_OPCODE: u16 = 6;
/// Server-to-client dialog effect command.
pub const PLAY_DIALOG_EFFECT_OPCODE: u16 = 28;
/// The preserved `PacketDialogResponse` default: no option chosen yet.
pub const UNSET_RESPONSE_INDEX: i32 = -1;
/// `wndDialog.xml` supplies response buttons `btn0..btn4`.
pub const MAX_CONVERSATION_RESPONSES: usize = 5;
/// Recovered Lua fallback for an empty NPC dialog line (`Ui.GetString(8125)`).
pub const DEFAULT_NPC_DIALOG_LOCALE_ID: i32 = 8125;
/// Recovered `questYes` label locale.
pub const QUEST_YES_LOCALE_ID: i32 = 71313;
/// Recovered `questNo` label locale.
pub const QUEST_NO_LOCALE_ID: i32 = 71314;
/// Recovered `questThanks` label locale, the single end-screen option.
pub const QUEST_THANKS_LOCALE_ID: i32 = 1911;
/// `AddResponseNode(Ui.GetString("questYes"), 300, 17, 1)`.
pub const QUEST_ACCEPT_RESPONSE: i32 = 1;
/// `AddResponseNode(Ui.GetString("questNo"), 302, 18, 2)`.
pub const QUEST_DECLINE_RESPONSE: i32 = 2;
/// `AddResponseNode(Ui.GetString("questThanks"), 300, 17, 3)`.
pub const QUEST_THANKS_RESPONSE: i32 = 3;
/// Pinned wiki revisions behind [`farnum_cleetus_dialogues`].
pub const FARNUM_CHAIN_WIKI_REVISIONS: [u64; 2] = [27815, 28955];

const MAX_NODES: usize = 32;
const MAX_NODE_ID: usize = 64;
const MAX_TEXT: usize = 4096;
const MAX_EFFECT: usize = 128;

fn invalid(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.into())
}

fn missing(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::NotFound, message.into())
}

/// The quest phase an NPC conversation is served for, derived from the restored quest engine's
/// progress record only. This mirrors the recovered accept / objective / turn-in / rewarded
/// sequence and adds no new state.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QuestPhase {
    NotAccepted,
    InProgress,
    ReadyToTurnIn,
    Rewarded,
}

/// Derive the dialogue phase from the restored engine's progress. A quest with no record is not
/// accepted; `rewarded` wins over `completed` because a rewarded quest is never offered again.
pub fn quest_phase(progress: Option<&QuestProgress>) -> QuestPhase {
    match progress {
        None => QuestPhase::NotAccepted,
        Some(progress) if !progress.accepted => QuestPhase::NotAccepted,
        Some(progress) if !progress.completed => QuestPhase::InProgress,
        Some(progress) if !progress.rewarded => QuestPhase::ReadyToTurnIn,
        Some(_) => QuestPhase::Rewarded,
    }
}

/// Which recovered client window serves a node. The window fixes the response option set.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DialogueWindow {
    /// `Main.wndDialog`/`Main.wndFlashDialog` conversation: options 1..=5.
    Conversation,
    /// `Main.wndQuestStart` quest offer: option 1 accept, option 2 decline.
    QuestStart,
    /// `Main.wndQuestEnd` completion: option 3 turn in.
    QuestEnd,
}

/// Where a text came from. Every dialogue line must carry one; there is no default.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Evidence {
    /// Community wiki transcription with its pinned revision.
    WikiRevision {
        page: String,
        revision_id: u64,
        section: String,
    },
    /// A locale id from the supplied client export.
    SuppliedLocale { locale_id: i32 },
    /// A symbol recovered from the preserved client binary or Lua.
    RecoveredClient { symbol: String },
    /// Explicit local policy, never presented as original content.
    Reconstructed { policy: String },
}

impl Evidence {
    fn is_blank(&self) -> bool {
        match self {
            Evidence::WikiRevision { page, section, .. } => {
                page.trim().is_empty() || section.trim().is_empty()
            }
            Evidence::SuppliedLocale { locale_id } => *locale_id <= 0,
            Evidence::RecoveredClient { symbol } => symbol.trim().is_empty(),
            Evidence::Reconstructed { policy } => policy.trim().is_empty(),
        }
    }
}

/// One recovered dialogue line: exactly one of a locale id or a literal transcription.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DialogueText {
    pub locale_id: Option<i32>,
    pub literal: Option<String>,
    pub evidence: Evidence,
}

fn locale_text(locale_id: i32) -> DialogueText {
    DialogueText {
        locale_id: Some(locale_id),
        literal: None,
        evidence: Evidence::SuppliedLocale { locale_id },
    }
}

fn wiki_text(page: &str, revision_id: u64, section: &str, literal: &str) -> DialogueText {
    DialogueText {
        locale_id: None,
        literal: Some(literal.to_string()),
        evidence: Evidence::WikiRevision {
            page: page.to_string(),
            revision_id,
            section: section.to_string(),
        },
    }
}

/// What a response option does when the client sends its index back.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum DialogueAction {
    /// `QuestScreen.AcceptQuest()`.
    AcceptQuest,
    /// Plain hide, the recovered decline path.
    Decline,
    /// `QuestScreen.EndQuest(true)`.
    TurnInQuest,
    /// `Ui.DialogEnd()`; an effect name is optional and only names a caller-side hook.
    Close { effect: Option<String> },
    /// Continue inside the same conversation.
    Open { node: String },
}

/// One response option. `response` is the recovered 1-based option id the client sends back.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DialogueChoice {
    pub response: i32,
    pub label: DialogueText,
    pub action: DialogueAction,
}

/// One phase's dialogue: the line the NPC says plus the menu that line is shown with.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DialogueNode {
    pub id: String,
    pub phase: QuestPhase,
    pub window: DialogueWindow,
    pub text: DialogueText,
    pub declinable: bool,
    /// Response options in the recovered 1-based order. An empty list is a line with no menu:
    /// the client shows the text and the server closes the dialog
    /// ([`END_DIALOG_OPCODE`]), which is the preserved `CommandPacketEndDialog` shape.
    pub choices: Vec<DialogueChoice>,
}

/// The NPC a definition belongs to, with the source that names it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NpcRef {
    pub name: String,
    pub evidence: Evidence,
}

/// A recovered NPC conversation for one quest. `original_quest_id` stays `None` while the
/// original numeric binding is unproven.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DialogueDefinition {
    pub schema_version: i64,
    pub npc: NpcRef,
    pub quest_name: String,
    pub original_quest_id: Option<u32>,
    pub next_quest_name: Option<String>,
    pub nodes: Vec<DialogueNode>,
}

/// What the caller should do after a validated response.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DialogueReply {
    Accepted,
    Declined,
    TurnedIn,
    Closed,
    Advanced { node: String },
}

fn valid_node_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= MAX_NODE_ID
        && id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
}

fn validate_text(text: &DialogueText, what: &str) -> io::Result<()> {
    if text.evidence.is_blank() {
        return Err(invalid(format!("{what}: evidence is empty")));
    }
    match (&text.locale_id, &text.literal) {
        (Some(_), Some(_)) => Err(invalid(format!(
            "{what}: a dialogue text carries either a locale id or a literal, not both"
        ))),
        (None, None) => Err(invalid(format!(
            "{what}: a dialogue text needs a locale id or a literal"
        ))),
        (Some(locale_id), None) => {
            if *locale_id <= 0 {
                return Err(invalid(format!("{what}: locale id must be positive")));
            }
            if let Evidence::SuppliedLocale { locale_id: claimed } = text.evidence {
                if claimed != *locale_id {
                    return Err(invalid(format!(
                        "{what}: evidence locale {claimed} does not match text locale {locale_id}"
                    )));
                }
            }
            Ok(())
        }
        (None, Some(literal)) => {
            if literal.trim().is_empty() {
                return Err(invalid(format!("{what}: literal text is empty")));
            }
            if literal.len() > MAX_TEXT {
                return Err(invalid(format!(
                    "{what}: literal text exceeds {MAX_TEXT} bytes"
                )));
            }
            if literal.contains('\r') || literal.contains('\n') {
                return Err(invalid(format!(
                    "{what}: literal text must be a single line"
                )));
            }
            Ok(())
        }
    }
}

fn validate_choice(
    choice: &DialogueChoice,
    node: &DialogueNode,
    node_ids: &BTreeSet<&str>,
) -> io::Result<()> {
    if choice.response <= 0 {
        return Err(invalid(format!(
            "node {}: response {} must be a positive option id",
            node.id, choice.response
        )));
    }
    if (choice.response as usize) > MAX_CONVERSATION_RESPONSES {
        return Err(invalid(format!(
            "node {}: response {} exceeds the recovered {MAX_CONVERSATION_RESPONSES}-option menu",
            node.id, choice.response
        )));
    }
    validate_text(
        &choice.label,
        &format!("node {} response {}", node.id, choice.response),
    )?;

    match &choice.action {
        DialogueAction::AcceptQuest => {
            if node.window != DialogueWindow::QuestStart || choice.response != QUEST_ACCEPT_RESPONSE
            {
                return Err(invalid(format!(
                    "node {}: accept is only the recovered quest-start option 1",
                    node.id
                )));
            }
        }
        DialogueAction::Decline => {
            if node.window != DialogueWindow::QuestStart
                || choice.response != QUEST_DECLINE_RESPONSE
            {
                return Err(invalid(format!(
                    "node {}: decline is only the recovered quest-start option 2",
                    node.id
                )));
            }
        }
        DialogueAction::TurnInQuest => {
            if node.window != DialogueWindow::QuestEnd || choice.response != QUEST_THANKS_RESPONSE {
                return Err(invalid(format!(
                    "node {}: turn-in is only the recovered end-screen option 3",
                    node.id
                )));
            }
        }
        DialogueAction::Close { effect } => {
            if node.window == DialogueWindow::QuestStart {
                return Err(invalid(format!(
                    "node {}: the quest-start window only carries accept/decline",
                    node.id
                )));
            }
            if let Some(effect) = effect {
                if effect.trim().is_empty() || effect.len() > MAX_EFFECT {
                    return Err(invalid(format!("node {}: invalid effect name", node.id)));
                }
            }
        }
        DialogueAction::Open { node: target } => {
            if !valid_node_id(target) || !node_ids.contains(target.as_str()) {
                return Err(invalid(format!(
                    "node {}: open target {target:?} is not a node in this definition",
                    node.id
                )));
            }
            if *target == node.id {
                return Err(invalid(format!("node {}: cannot open itself", node.id)));
            }
        }
    }
    Ok(())
}

/// Validate a definition against the recovered window shapes. Errors mean the definition can not
/// be served by the restored client contract.
pub fn validate(definition: &DialogueDefinition) -> io::Result<()> {
    if definition.schema_version != DIALOGUE_SCHEMA_VERSION {
        return Err(invalid(format!(
            "unsupported dialogue schema version {}",
            definition.schema_version
        )));
    }
    if definition.npc.name.trim().is_empty() {
        return Err(invalid("npc name is empty"));
    }
    if definition.npc.evidence.is_blank() {
        return Err(invalid("npc evidence is empty"));
    }
    if definition.quest_name.trim().is_empty() {
        return Err(invalid("quest name is empty"));
    }
    if definition.nodes.is_empty() || definition.nodes.len() > MAX_NODES {
        return Err(invalid(format!(
            "a definition carries 1..={MAX_NODES} nodes, found {}",
            definition.nodes.len()
        )));
    }

    for node in &definition.nodes {
        if !valid_node_id(&node.id) {
            return Err(invalid(format!("invalid node id {:?}", node.id)));
        }
    }
    let ids: BTreeSet<&str> = definition
        .nodes
        .iter()
        .map(|node| node.id.as_str())
        .collect();
    if ids.len() != definition.nodes.len() {
        let mut seen = BTreeSet::new();
        for node in &definition.nodes {
            if !seen.insert(node.id.as_str()) {
                return Err(invalid(format!("duplicate node id {:?}", node.id)));
            }
        }
        return Err(invalid("duplicate node id"));
    }

    let mut phases = BTreeSet::new();
    for node in &definition.nodes {
        if !phases.insert(node.phase) {
            return Err(invalid(format!(
                "node {:?}: a second definition of phase {:?} is already present",
                node.id, node.phase
            )));
        }
    }

    for node in &definition.nodes {
        validate_text(&node.text, &format!("node {}", node.id))?;
        if node.choices.len() > MAX_CONVERSATION_RESPONSES {
            return Err(invalid(format!(
                "node {}: {} options exceed the recovered {MAX_CONVERSATION_RESPONSES}-option menu",
                node.id,
                node.choices.len()
            )));
        }
        for choice in &node.choices {
            validate_choice(choice, node, &ids)?;
        }
        // The recovered windows each hand the client a fixed option set: a conversation numbers
        // its buttons 1..=n, the quest offer is exactly accept 1 / decline 2, and the end screen
        // builds exactly the thanks option 3.
        let responses: Vec<i32> = node.choices.iter().map(|choice| choice.response).collect();
        let expected: Vec<i32> = match node.window {
            DialogueWindow::Conversation => (1..=node.choices.len() as i32).collect(),
            DialogueWindow::QuestStart => vec![QUEST_ACCEPT_RESPONSE, QUEST_DECLINE_RESPONSE],
            DialogueWindow::QuestEnd => vec![QUEST_THANKS_RESPONSE],
        };
        if responses != expected {
            return Err(invalid(format!(
                "node {}: the recovered menu for {:?} is {expected:?}, found {responses:?}",
                node.id, node.window
            )));
        }
    }

    // Window shapes, after every node id is known so `open` targets could be checked above.
    for node in &definition.nodes {
        let declines = node
            .choices
            .iter()
            .filter(|choice| choice.action == DialogueAction::Decline)
            .count();
        match node.window {
            DialogueWindow::QuestStart => {
                if node.phase != QuestPhase::NotAccepted {
                    return Err(invalid(format!(
                        "node {}: the quest-start window only serves an unaccepted quest",
                        node.id
                    )));
                }
                let accepts = node
                    .choices
                    .iter()
                    .filter(|choice| choice.action == DialogueAction::AcceptQuest)
                    .count();
                if accepts != 1 || !node.declinable || declines != 1 {
                    return Err(invalid(format!(
                        "node {}: the recovered offer carries option 1 accept and option 2 decline",
                        node.id
                    )));
                }
            }
            DialogueWindow::QuestEnd => {
                if node.phase != QuestPhase::ReadyToTurnIn || node.declinable {
                    return Err(invalid(format!(
                        "node {}: the end screen only serves a completed, unclaimed quest",
                        node.id
                    )));
                }
                if node.choices.is_empty() {
                    return Err(invalid(format!(
                        "node {}: the end screen always builds its thanks response",
                        node.id
                    )));
                }
            }
            DialogueWindow::Conversation => {
                if node.declinable {
                    return Err(invalid(format!(
                        "node {}: a plain conversation has no recovered decline option",
                        node.id
                    )));
                }
            }
        }
        if node.window != DialogueWindow::QuestStart && declines != 0 {
            return Err(invalid(format!(
                "node {}: only the quest-start window declines",
                node.id
            )));
        }
    }
    Ok(())
}

/// The single node serving this phase, or `None` when the recovered evidence has no line for it.
pub fn select_node(definition: &DialogueDefinition, phase: QuestPhase) -> Option<&DialogueNode> {
    definition.nodes.iter().find(|node| node.phase == phase)
}

/// Resolve a client response index against the node serving `phase`.
///
/// The response index arrives from `dialogResponse`/`responseOption` through
/// `Ui.DialogSelect(opt)` or the quest windows. `-1` is the preserved client's unset default and
/// is rejected; an index that the recovered menu for this phase does not define is rejected too.
pub fn respond(
    definition: &DialogueDefinition,
    phase: QuestPhase,
    response: i32,
) -> io::Result<DialogueReply> {
    validate(definition)?;
    if response == UNSET_RESPONSE_INDEX {
        return Err(invalid("dialogue response is unset (-1)"));
    }
    if response <= 0 {
        return Err(invalid(format!(
            "dialogue response {response} is not a menu option"
        )));
    }
    let node = select_node(definition, phase).ok_or_else(|| {
        missing(format!(
            "no recovered dialogue node for {:?} of {}",
            phase, definition.quest_name
        ))
    })?;
    if node.choices.is_empty() {
        return Err(invalid(format!(
            "node {} carries no response menu; the caller closes the dialog",
            node.id
        )));
    }
    let choice = node
        .choices
        .iter()
        .find(|choice| choice.response == response)
        .ok_or_else(|| {
            invalid(format!(
                "response {response} is not defined by node {}",
                node.id
            ))
        })?;
    Ok(match &choice.action {
        DialogueAction::AcceptQuest => DialogueReply::Accepted,
        DialogueAction::Decline => DialogueReply::Declined,
        DialogueAction::TurnInQuest => DialogueReply::TurnedIn,
        DialogueAction::Close { .. } => DialogueReply::Closed,
        DialogueAction::Open { node } => DialogueReply::Advanced { node: node.clone() },
    })
}

const WEAPON_PAGE: &str = "Quest:Your First Weapon";
const PACK_PAGE: &str = "Quest:Fighting off the Pack";
const WEAPON_REVISION: u64 = FARNUM_CHAIN_WIKI_REVISIONS[0];
const PACK_REVISION: u64 = FARNUM_CHAIN_WIKI_REVISIONS[1];

/// The two recovered Cleetus Farnum conversations of the opening farm arc.
///
/// Every line is a pinned wiki transcription of the original NPC text; the phases the two pages
/// do not record are absent on purpose. `Quest:Fighting off the Pack` records an offer and a
/// turn-in line only, so it has no in-progress node -- an explicit gap, not a missing feature.
pub fn farnum_cleetus_dialogues() -> Vec<DialogueDefinition> {
    let npc = |page: &str, revision_id: u64| NpcRef {
        name: "Cleetus Farnum".to_string(),
        evidence: Evidence::WikiRevision {
            page: page.to_string(),
            revision_id,
            section: "to-start".to_string(),
        },
    };

    let weapon = DialogueDefinition {
        schema_version: DIALOGUE_SCHEMA_VERSION,
        npc: npc(WEAPON_PAGE, WEAPON_REVISION),
        quest_name: WEAPON_PAGE.to_string(),
        original_quest_id: None,
        next_quest_name: Some(PACK_PAGE.to_string()),
        nodes: vec![
            DialogueNode {
                id: "offer".to_string(),
                phase: QuestPhase::NotAccepted,
                window: DialogueWindow::QuestStart,
                text: wiki_text(
                    WEAPON_PAGE,
                    WEAPON_REVISION,
                    "steps",
                    "These Hooligans have been causing us a lot of problems and picking up after a \
                     mess is only part of the solution. We need to stop them from breaking things \
                     in the first place.",
                ),
                declinable: true,
                choices: quest_offer_choices(),
            },
            DialogueNode {
                id: "objective".to_string(),
                phase: QuestPhase::InProgress,
                window: DialogueWindow::Conversation,
                text: wiki_text(
                    WEAPON_PAGE,
                    WEAPON_REVISION,
                    "steps",
                    "Since you've been so helpful around here I'm going to give you everything you \
                     need to craft yourself a better hammer. Use this Anvil to forge a Spunky \
                     Scrapper Hammer.",
                ),
                declinable: false,
                choices: Vec::new(),
            },
            DialogueNode {
                id: "turn-in".to_string(),
                phase: QuestPhase::ReadyToTurnIn,
                window: DialogueWindow::QuestEnd,
                text: wiki_text(
                    WEAPON_PAGE,
                    WEAPON_REVISION,
                    "steps",
                    "That's one mighty hammer, I hope you won't have to put it to good use too \
                     often.",
                ),
                declinable: false,
                choices: vec![DialogueChoice {
                    response: QUEST_THANKS_RESPONSE,
                    label: locale_text(QUEST_THANKS_LOCALE_ID),
                    action: DialogueAction::TurnInQuest,
                }],
            },
        ],
    };

    let pack = DialogueDefinition {
        schema_version: DIALOGUE_SCHEMA_VERSION,
        npc: npc(PACK_PAGE, PACK_REVISION),
        quest_name: PACK_PAGE.to_string(),
        original_quest_id: None,
        next_quest_name: Some("Quest:Eviction Notice".to_string()),
        nodes: vec![
            DialogueNode {
                id: "offer".to_string(),
                phase: QuestPhase::NotAccepted,
                window: DialogueWindow::QuestStart,
                text: wiki_text(
                    PACK_PAGE,
                    PACK_REVISION,
                    "steps",
                    "Unfortunately now is one of those times to where you'll need to use that \
                     hammer. Use it to defeat 5 Hooligan Wolves. Be sure to switch to your Brawler \
                     job. Return to me when you are finished.",
                ),
                declinable: true,
                choices: quest_offer_choices(),
            },
            DialogueNode {
                id: "turn-in".to_string(),
                phase: QuestPhase::ReadyToTurnIn,
                window: DialogueWindow::QuestEnd,
                text: wiki_text(
                    PACK_PAGE,
                    PACK_REVISION,
                    "steps",
                    "We should be having less wolf attacks after the trashing that you gave those \
                     Hooligan Wolves.",
                ),
                declinable: false,
                choices: vec![DialogueChoice {
                    response: QUEST_THANKS_RESPONSE,
                    label: locale_text(QUEST_THANKS_LOCALE_ID),
                    action: DialogueAction::TurnInQuest,
                }],
            },
        ],
    };

    vec![weapon, pack]
}

fn quest_offer_choices() -> Vec<DialogueChoice> {
    vec![
        DialogueChoice {
            response: QUEST_ACCEPT_RESPONSE,
            label: locale_text(QUEST_YES_LOCALE_ID),
            action: DialogueAction::AcceptQuest,
        },
        DialogueChoice {
            response: QUEST_DECLINE_RESPONSE,
            label: locale_text(QUEST_NO_LOCALE_ID),
            action: DialogueAction::Decline,
        },
    ]
}
