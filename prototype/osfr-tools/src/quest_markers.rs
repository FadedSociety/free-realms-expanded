//! Quest markers: the 35/10 and 35/11 packets and the rule that picks each NPC's marker.
//!
//! The original drew a '!' over an NPC with a quest on offer and a '?' over the NPC to talk to
//! next or hand in to (`notes/Research/Owner Field Notes.md`). This module only builds the
//! bytes and decides the marker; the visibility and quest paths that send them are separate.
//!
//! Client readers (35 dispatch switch `0092F540`, same 4-byte header as 35/3: u16 35, u16
//! subtype; both readers refuse trailing bytes, `a4 = 0` at `009320cc` and `00932220`):
//! - 35/10 case at `009320ac`: ctor `008d06d0`, body `009213e0` -> `0090aa90`, an i32 count
//!   (at most the bytes left) then `008db310` per element.
//! - 35/11 case at `00932200`: ctor `008d07c0`, body `00919d10` -> `008fc330`, an i32 count then
//!   `008db550` per element (u64 guid, i32, i32).
//!
//! Both handlers look the guid up in the client's actor table (`00984960`) and skip a guid it
//! has not spawned, so a caller must send the NPC's spawn first.

use crate::{
    npc_notifications::{self, Notification},
    quest_engine::{QuestEngine, QuestStatus},
};
use serde::Deserialize;
use std::{
    fs,
    io::{self, Read},
    path::Path,
    sync::Arc,
};

/// Optional content next to the other split files. Absent file = no markers.
pub const CONTENT_FILE: &str = "quest_markers.json";
const MAX_FILE: u64 = 64 * 1024;
/// `NotificationImages.txt` keys are small; this bounds a typo, not the table.
const MAX_IMAGE_ID: i32 = 10_000;
pub use crate::npc_notifications::MAX_ENTRIES;

/// What an NPC shows above its head.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QuestMarker {
    /// '!': a quest this player can start here.
    Available,
    /// '?': the NPC this player must talk to or hand in to.
    TurnIn,
}

/// `quest_markers.json`: the `NotificationImages.txt` keys, tunable in data.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct QuestMarkerContent {
    pub schema_version: u32,
    pub available_image_id: i32,
    pub turnin_image_id: i32,
    /// Provenance notes for the content checker; never gameplay input.
    #[serde(default)]
    pub source: Vec<String>,
}

impl QuestMarkerContent {
    pub fn load(path: &Path) -> io::Result<Self> {
        let mut text = String::new();
        fs::File::open(path)?
            .take(MAX_FILE + 1)
            .read_to_string(&mut text)?;
        if text.len() as u64 > MAX_FILE {
            return Err(invalid("quest marker content too large"));
        }
        let content: Self = serde_json::from_str(&text).map_err(|e| invalid(e.to_string()))?;
        content.validate()?;
        Ok(content)
    }

    pub fn validate(&self) -> io::Result<()> {
        if self.schema_version != 1 {
            return Err(invalid("unsupported quest marker content version"));
        }
        for id in [self.available_image_id, self.turnin_image_id] {
            if !(1..=MAX_IMAGE_ID).contains(&id) {
                return Err(invalid(
                    "quest marker image id must be 1 to the local bound",
                ));
            }
        }
        Ok(())
    }

    pub fn image_id(&self, marker: QuestMarker) -> i32 {
        match marker {
            QuestMarker::Available => self.available_image_id,
            QuestMarker::TurnIn => self.turnin_image_id,
        }
    }
}

/// What `Gameplay` keeps for markers; none of it is saved. No `content` (no file bound) means
/// no markers. `rendered` is the engine revision the per-player visibility path last sent
/// markers for (`gameplay/npc_quests.rs`); the shared path keeps that in the gateway session.
#[derive(Clone, Debug, Default)]
pub struct MarkerView {
    pub content: Option<Arc<QuestMarkerContent>>,
    pub rendered: u64,
}

fn invalid(error: impl ToString) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, error.to_string())
}

/// 35/10 `PlayerUpdatePacketAddNotifications` for `(actor guid, image id, name string id)` rows.
/// The guids are runtime actor guids, not content keys. Each row is a plain notification (kind,
/// state and sub-text 0, overhead icon shown, trailing byte 0) in the `008db310` order (R);
/// `None` for more than [`MAX_ENTRIES`] rows.
pub fn encode_add_notifications(entries: &[(u64, i32, i32)]) -> Option<Vec<u8>> {
    let rows: Vec<_> = entries
        .iter()
        .map(|&(guid, image_id, name_id)| {
            let marker = Notification {
                kind: 0,
                image_id,
                state: 0,
                name_id,
                subtext_id: 0,
                hide_overhead: false,
                enabled: false,
            };
            (guid, marker)
        })
        .collect();
    npc_notifications::encode_add(&rows)
}

/// 35/11 `PlayerUpdatePacketRemoveNotifications` for runtime actor guids: an i32 count, then per
/// entry the u64 guid and two i32 zeros (`008db550`). `None` for more than [`MAX_ENTRIES`].
pub fn encode_remove_notifications(guids: &[u64]) -> Option<Vec<u8>> {
    if guids.len() > MAX_ENTRIES {
        return None;
    }
    let mut bytes = vec![35, 0, 11, 0];
    bytes.extend((guids.len() as i32).to_le_bytes());
    for guid in guids {
        bytes.extend(guid.to_le_bytes());
        bytes.extend([0; 8]);
    }
    Some(bytes)
}

/// The marker this player's quest state gives `npc_id` (R rule); reads, never mutates.
/// '?' wins over '!'.
/// - '?': an enabled active quest whose open objective is a Talk to this NPC, one whose
///   objectives are all done and whose turn-in NPC this is (before its end screen opens), or
///   this NPC opened the pending end screen.
/// - '!': any quest this NPC gives passes `can_offer` (enabled, prerequisites, level, job,
///   quest limit, no pending offer elsewhere).
pub fn marker_for(engine: &QuestEngine, npc_id: u64) -> Option<QuestMarker> {
    let talks_to_here = |id: u32| {
        let open = engine.current_objectives(id);
        if open.is_empty() {
            // Done: hand in at the turn-in NPC, unless the end screen already opened
            // somewhere else, where only that actor counts.
            return engine.state().pending_turnin != Some(id)
                && engine.turnin_npc_id(id) == Some(npc_id);
        }
        open.iter().any(|objective| objective.kind.talks_to(npc_id))
    };
    let turn_in = engine.pending_turnin_actor() == Some(npc_id)
        || engine
            .state()
            .quests
            .iter()
            .filter(|(id, progress)| {
                progress.status == QuestStatus::Active && engine.is_enabled(**id)
            })
            .any(|(id, _)| talks_to_here(*id));
    if turn_in {
        Some(QuestMarker::TurnIn)
    } else if engine
        .quest_ids_for_giver(npc_id)
        .any(|id| engine.can_offer(id, npc_id))
    {
        Some(QuestMarker::Available)
    } else {
        None
    }
}
