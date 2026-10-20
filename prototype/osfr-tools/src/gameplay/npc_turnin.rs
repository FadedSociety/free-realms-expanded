//! Saved turn-in replay: a pending 49/13 end screen is re-sent only in a batch that also
//! carries its opener's 35/2 AddNpc, on both the per-session and shared-field view paths.
use super::*;

impl Gameplay {
    /// Reopen a saved pending turn-in only in a batch that also sends its speaker's 35/2
    /// AddNpc. `ShowQuestEndScreen` (0x00A95420) resolves the packet's actor through the
    /// client actor table first and does not retry when it is absent; sending the 49/13
    /// from login synchronization put it before every visibility spawn.
    pub(super) fn append_pending_turnin_screen(
        &self,
        packets: &mut Vec<Vec<u8>>,
    ) -> io::Result<()> {
        let Some(engine) = self.npc_quests.as_ref() else {
            return Ok(());
        };
        let Some(actor_id) = engine.pending_turnin_actor() else {
            return Ok(());
        };
        let Some(quest_id) = engine.state().pending_turnin else {
            return Ok(());
        };
        let Some(runtime_actor) = packets.iter().find_map(|packet| {
            let npc = crate::startup_world::parse_add_npc(packet).ok()?;
            let same_npc = npc.actor_id == actor_id
                || self.content_key_for_turnin(npc.actor_id) == Some(actor_id);
            same_npc.then_some(npc.actor_id)
        }) else {
            return Ok(());
        };
        packets.push(engine.quest_end_packet(quest_id, runtime_actor)?);
        Ok(())
    }

    /// Shared-field path: the same saved end screen, but a packet failure is logged for this
    /// viewer instead of faulting the shared tick.
    pub(super) fn append_shared_pending_end_screen(&self, packets: &mut Vec<Vec<u8>>) {
        if let Err(error) = self.append_pending_turnin_screen(packets) {
            npc_event(serde_json::json!({
                "event": "pending_end_screen_refused",
                "reason": error.to_string()
            }));
        }
    }

    /// Runtime-GUID -> content-key map for a saved turn-in opener. The interaction router's
    /// opening-scene exclusion ([`Self::content_npc_key`]) must not apply here:
    /// `pending_turnin_actor` is saved server state naming the NPC the quest engine saw talked
    /// to, which can be a named Talk-objective target or an opening trigger actor (opening
    /// Talk triggers store their scene actor GUID; see `From<OpeningTrigger> for QuestEvent`).
    /// Excluding opening actors here would lose the spawn match for such a saved opener.
    pub(super) fn content_key_for_turnin(&self, runtime_guid: u64) -> Option<u64> {
        self.npc_visibility
            .as_ref()
            .and_then(|visibility| visibility.content_key(runtime_guid))
    }

    /// Canonical runtime-GUID -> content-key map for a client interaction target.
    /// Opening-scene actors are excluded: they are driven by the opening trigger path, not
    /// the generic NPC router.
    pub(super) fn content_npc_key(&self, runtime_guid: u64) -> Option<u64> {
        if self.opening.as_ref().is_some_and(|opening| {
            opening
                .actors()
                .iter()
                .any(|actor| actor.guid == runtime_guid)
        }) {
            return None;
        }
        self.content_key_for_turnin(runtime_guid)
    }
}
