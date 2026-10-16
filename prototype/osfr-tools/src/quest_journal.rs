//! Quest journal requests from the client: select a tracked quest (47/12), lock
//! the selection (47/13) and start a breadcrumb quest (49/15).
//!
//! Recovered from the client (FreeRealms.exe, tunneled application packets):
//! - 47/12 `SelectQuestPacket` (ctor `00c79e40`, writer `00c7bc50`, sender
//!   `00c7c250`, caller `00c7c540`): `u16 47`, `u8 12`, `i32 quest id` = 7 bytes.
//!   The client selects locally first and only sends when the id changed.
//! - 47/13 `SelectedQuestLockedPacket` (ctor `00c79f00`, writer `00c7bcb0`,
//!   sender `00c7c360`, caller `00c7c5d0`): `u16 47`, `u8 13`, `u8` = 4 bytes.
//!   The caller stores the client's auto-switch flag (`manager+0x10f8`) and
//!   sends its inversion (`sete`): the wire byte is "locked", so 1 means the
//!   auto-switch flag is clear.
//! - 49/15 `QuestStartBreadcrumbPacket` (ctor `00a86580`, writer `00a91730`,
//!   sender `00a97f90`, Lua `Ui.StartBreadcrumbQuest` `00c18aa0`): the family 49
//!   header (`u16 49`, `u32 15`) then `i32 quest id` = 10 bytes.
//!
//! Every server rule below is reconstructed policy (R): the original server's
//! reply to these requests is unrecovered. The client has already applied the
//! selection and lock locally, so success sends nothing back and refusal is
//! silent. 47/14 `ObjectiveTargetUpdate` (the "tracked target" pin) is not sent
//! here: its reader `00a8b440` is recovered (u8 active, then only when active
//! 12+8+4+16+4 bytes) but which dword is which is not, see the W145 report.
//!
//! Nothing here awards or completes a quest. Selection only reads QuestEngine
//! state, and a breadcrumb only makes the regular 49/1 offer; the player still
//! has to accept it and earn its objectives from server events.

use crate::quest_engine::{
    OutboundMessage, QuestDenial, QuestEngine, QuestMessage, QuestOutcome, QuestStatus,
};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JournalRequest {
    Select {
        quest_id: u32,
    },
    /// Wire byte of 47/13: 1 = selection locked (client auto-switch off).
    Lock {
        locked: bool,
    },
    Breadcrumb {
        quest_id: u32,
    },
}

/// The packet is one of ours but its length or a field is out of range.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Malformed;

/// `Ok(None)`: not a journal request, let the caller keep dispatching.
pub fn decode(inner: &[u8]) -> Result<Option<JournalRequest>, Malformed> {
    Ok(Some(match inner {
        [47, 0, 12, rest @ ..] => JournalRequest::Select {
            quest_id: quest_id(rest)?,
        },
        [47, 0, 13, rest @ ..] => JournalRequest::Lock {
            locked: match rest {
                [0] => false,
                [1] => true,
                _ => return Err(Malformed),
            },
        },
        [49, 0, 15, 0, 0, 0, rest @ ..] => JournalRequest::Breadcrumb {
            quest_id: quest_id(rest)?,
        },
        _ => return Ok(None),
    }))
}

/// Exactly one positive `i32`. The client's "no quest" value is -1 and never a
/// valid request here (R).
fn quest_id(body: &[u8]) -> Result<u32, Malformed> {
    let bytes: [u8; 4] = body.try_into().map_err(|_| Malformed)?;
    u32::try_from(i32::from_le_bytes(bytes))
        .ok()
        .filter(|id| *id != 0)
        .ok_or(Malformed)
}

/// Apply a decoded request. `breadcrumbs` maps each quest the server lets a
/// player start remotely to the NPC that gives it (R); a quest outside it can
/// only be offered by talking to its giver. The caller persists the state when
/// the outcome says `changed` and renders `outbound` as it does for NPC offers.
pub fn handle(
    engine: &mut QuestEngine,
    request: JournalRequest,
    breadcrumbs: &BTreeMap<u32, u64>,
) -> QuestOutcome {
    match request {
        JournalRequest::Select { quest_id } => engine.select_quest(quest_id),
        // The stored flag is the client's auto-switch flag, the inverse of the wire byte.
        JournalRequest::Lock { locked } => engine.set_auto_switch_selected(!locked),
        JournalRequest::Breadcrumb { quest_id } => breadcrumb(engine, quest_id, breadcrumbs),
    }
}

fn breadcrumb(
    engine: &mut QuestEngine,
    quest_id: u32,
    breadcrumbs: &BTreeMap<u32, u64>,
) -> QuestOutcome {
    let status = engine
        .progress(quest_id)
        .map(|progress| progress.status.clone());
    // An owned active quest: the breadcrumb just tracks it.
    if status == Some(QuestStatus::Active) {
        return engine.select_quest(quest_id);
    }
    let Some(&giver) = breadcrumbs.get(&quest_id) else {
        return QuestOutcome {
            denial: Some(QuestDenial::Unavailable),
            ..QuestOutcome::default()
        };
    };
    // Offer still open (window closed, button pressed again): show it again.
    if status == Some(QuestStatus::Offered) && engine.state().pending_offer == Some(quest_id) {
        return QuestOutcome {
            outbound: vec![OutboundMessage::Quest(QuestMessage::QuestInfo { quest_id })],
            ..QuestOutcome::default()
        };
    }
    // Prerequisites, level, job, membership, limit and hollow content are all
    // checked by the engine's regular offer path.
    engine.offer(quest_id, giver)
}
