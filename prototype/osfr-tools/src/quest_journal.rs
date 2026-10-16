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
//! silent. The 47/14 `ObjectiveTargetUpdate` pin ("Take Me There") is built by `Pin`,
//! `tracked_pin` and `Tracker` below.
//!
//! Nothing here awards or completes a quest. Selection only reads QuestEngine
//! state, and a breadcrumb only makes the regular 49/1 offer; the player still
//! has to accept it and earn its objectives from server events.

use crate::quest_engine::{
    ObjectiveKind, OutboundMessage, QuestDenial, QuestEngine, QuestMessage, QuestObjective,
    QuestOutcome, QuestStatus,
};
use crate::world::OPEN_WORLD_ZONE;
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

/// What the client's "Take Me There" pin needs (47/14 `ObjectiveTargetUpdate`).
///
/// Recovered from the client (FreeRealms.exe). Dispatch `00a91bf0` case 14 reads the body
/// with `00a8db30` -> `00a8b440` and requires it to end exactly, then calls the handler
/// `00a8fab0`. Wire after `u16 47`, `u8 14`: `u8 active`; only when active:
/// `u32 a`, `u32 b`, `u32 c` (handler cache `+0x10/+0x14/+0x18`), `u64 guid`
/// (`+0x20/+0x24`), `u32 name string id` (`+0x28`), `f32 x4` (`+0x30`), `u32 zone id`
/// (`+0x40`). Inactive is just the zero byte (4 bytes in all); active is 48.
/// - `a`, `b`, `c` only identify the tracked task: the handler compares them with its
///   cache to decide whether to restart the path (`009cd6c0` flag), and `c` is the argument
///   of Lua `ObjectiveWindow:SelectTask` (the goals-window row key). The client's 47/6
///   `SelectTaskRequest` (`00a86b30`, sent by `00a976a0`) appears to carry the cached `a`
///   and `b` plus the chosen row id back. The meaning of `a` and `b` is not recovered; this
///   server sends the quest id in `a`, 0 in `b`, and the objective id in `c` (R).
/// - guid: key of the minimap indicator (`00a8e660` -> `MiniMap:SelectIndicator`; a key
///   with no indicator just clears the selection) and `BaseClient.ObjectiveTarget` column 1.
/// - name string id: looked up by `00cb85b0` into column 2; 0 gives an empty name and Lua
///   `MiniMap:SetObjectiveTarget` shows locale 139655 "Objective Destination".
/// - vec4: `x, y, z, w`. Column 3 is `x` and column 4 is `z` (`00cb8810`); the whole vec4
///   is also stored by `009ce260` as the vector `Ui.TakeMeToObjective` (`00c17c70` ->
///   `008a8e50` -> `009cef10` -> `009ce560`) walks the player to, and `009cd6c0` asks for
///   a path to it. The client's own constant for `w` is 1.0 (`0175b420`).
/// - zone id: column 5 ("Zone ID"); no Lua reads it. Sent as the zone of the position.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pin {
    pub quest_id: u32,
    pub objective_id: u32,
    pub guid: u64,
    pub zone_id: u32,
    pub position: [f32; 3],
}

impl Pin {
    /// `None` is the "no pin" message, which clears the client's pin.
    pub fn encode(pin: Option<&Pin>) -> Vec<u8> {
        let mut packet = vec![47, 0, 14];
        let Some(pin) = pin else {
            packet.push(0);
            return packet;
        };
        packet.push(1);
        for dword in [pin.quest_id, 0, pin.objective_id] {
            packet.extend(dword.to_le_bytes());
        }
        packet.extend(pin.guid.to_le_bytes());
        packet.extend(0u32.to_le_bytes());
        for float in [pin.position[0], pin.position[1], pin.position[2], 1.0] {
            packet.extend(float.to_le_bytes());
        }
        packet.extend(pin.zone_id.to_le_bytes());
        packet
    }
}

/// Where the server knows things are. Each lookup answers "unknown" with `None`.
pub struct Places<'a> {
    /// NPC id -> position (zone 0, the one continent; regional labels are not zones).
    pub npc: &'a dyn Fn(u64) -> Option<[f32; 3]>,
    /// Actor kind -> (guid, home position) of a field enemy of that kind.
    pub enemy: &'a dyn Fn(u32) -> Option<(u64, [f32; 3])>,
    /// Reach-POI id -> (zone, position).
    pub poi: &'a dyn Fn(u32) -> Option<(u32, [f32; 3])>,
}

/// The pin for one objective, or `None` when the content gives it no single known place
/// (items, jobs, crafting, minigames, a whole zone). Never a guess (R).
pub fn locate(quest_id: u32, objective: &QuestObjective, places: &Places) -> Option<Pin> {
    let (guid, zone_id, position) = match objective.kind {
        ObjectiveKind::TalkToNpc { npc_id } | ObjectiveKind::InteractNpc { npc_id } => {
            (npc_id, OPEN_WORLD_ZONE, (places.npc)(npc_id)?)
        }
        ObjectiveKind::KillActorKind { actor_kind_id } => {
            let (guid, position) = (places.enemy)(actor_kind_id)?;
            (guid, OPEN_WORLD_ZONE, position)
        }
        ObjectiveKind::ReachPoi { poi_id } => {
            let (zone, position) = (places.poi)(poi_id)?;
            (0, zone, position)
        }
        ObjectiveKind::VisitPosition {
            zone_id, position, ..
        } => (0, zone_id, position),
        _ => return None,
    };
    position.iter().all(|v| v.is_finite()).then_some(Pin {
        quest_id,
        objective_id: objective.id,
        guid,
        zone_id,
        position,
    })
}

/// The tracked-target pin of the character's own selected quest (R): its first open
/// objective that has a place, as long as the place is in the character's current zone.
/// `objectives` are the quest's open objectives (`QuestEngine::current_objectives`).
pub fn tracked_pin(
    selected: Option<u32>,
    objectives: &[&QuestObjective],
    zone_id: u32,
    places: &Places,
) -> Option<Pin> {
    let quest_id = selected?;
    objectives
        .iter()
        .find_map(|objective| locate(quest_id, objective, places))
        .filter(|pin| pin.zone_id == zone_id)
}

/// What the client currently shows. `update` returns the 47/14 to send when the wanted pin
/// differs from it: a changed pin is sent, a vanished pin is cleared, and nothing is sent
/// while there is no pin and none was shown. Not saved: a new session starts empty and the
/// first call sends the pin again.
#[derive(Clone, Debug, Default)]
pub struct Tracker {
    sent: Option<Pin>,
}

impl Tracker {
    pub fn update(&mut self, wanted: Option<Pin>) -> Option<Vec<u8>> {
        if wanted == self.sent {
            return None;
        }
        self.sent = wanted;
        Some(Pin::encode(wanted.as_ref()))
    }
}
