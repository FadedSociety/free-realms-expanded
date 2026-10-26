//! T503: quest-only pickup nodes. A click (26/8) on a `quest_nodes.json` site credits one
//! counted `collect_node` objective of an active quest, hides the node for this player and
//! schedules its return after the pool's respawn time.
//!
//! The admission rule is reconstructed behaviour (R) from Lxzxrus Free Realms Server dac754e
//! (Ariel's 2026-10-16 playtest): a pickup is limited to a player on the matching Collect
//! step. Every refusal answers one chat line before anything is staged, so it changes nothing.
use super::*;

impl Gameplay {
    /// A click on a quest pickup node, or `None` when `target` is not one (the gather and NPC
    /// paths then run unchanged). The caller has already checked the 26/8 header and read the
    /// target guid.
    pub(super) fn pick_up_quest_node(
        &mut self,
        inner: &[u8],
        character: &CharacterState,
        target: u64,
        now_ms: u64,
    ) -> io::Result<Option<Outcome>> {
        let Some(node_type) = self
            .gathering_nodes
            .as_ref()
            .and_then(|nodes| nodes.pickup_kind(target))
        else {
            return Ok(None);
        };
        npc_interaction::decode_npc_interact_26_8(inner)
            .map_err(|_| invalid("invalid node interaction request"))?;
        let mut out = Outcome::default();
        let needs = self
            .npc_quests
            .as_ref()
            .is_some_and(|quests| quests.wants_node(node_type));
        let busy = self.activity_busy();
        let denied = self.gathering_nodes.as_ref().and_then(|nodes| {
            nodes
                .admit_pickup(target, character.zone_id, character.position, busy, needs)
                .err()
        });
        if let Some(denied) = denied {
            out.replies.push(chat::system(&denied.message()));
            return Ok(Some(out));
        }
        let mut staged = self.clone();
        let hide = staged
            .gathering_nodes
            .as_mut()
            .and_then(|nodes| nodes.consume(target, now_ms));
        staged.npc_quest_event(
            crate::quest_engine::QuestEvent::CollectNode {
                node_type: node_type.to_owned(),
            },
            &mut out,
        )?;
        if let (Some(hide), Some(visibility)) = (hide, staged.npc_visibility.as_mut()) {
            apply_node_changes(visibility, vec![hide]);
            out.replies.extend(visibility.refresh()?);
        }
        out.changed = true;
        *self = staged;
        Ok(Some(out))
    }
}
