//! T789: accepted ability and item uses feed the quest engine.
//!
//! Both events are server-authoritative facts, not client claims:
//!
//! * An ability id is content resolved from the toolbar slot the client names in 36/10. The
//!   client body is bar id, slot, target mode, then the mode-0 u64 target (`00a342e0`; sender
//!   `00a35150`), so the credited definition and the target actor kind never come from the wire.
//!   `EncounterOutcome::abilities_used` is filled only after every cast refusal (285 energy, 286
//!   unavailable, 287 cooldown, 288 target).
//! * An item is credited only when `item_bar::ItemBar::consume` actually spent it. The confirm
//!   (42/15 `00c10940`) and check (42/16 `00c109f0`) bodies name an owned instance, the
//!   definition comes from the server inventory, and the bar forces a self target (`00a35600`
//!   selector pair 1/2).
use super::*;

impl Gameplay {
    /// Credit one `QuestEvent::UseItem` for the definition of a single-use item actually spent.
    /// `used_definition` is `None` for a boombox press and every refusal, so nothing is credited.
    pub(super) fn emit_item_use(
        &mut self,
        used_definition: Option<i32>,
        out: &mut Outcome,
    ) -> io::Result<()> {
        if let Some(item_id) = used_definition {
            self.npc_quest_event(
                crate::quest_engine::QuestEvent::UseItem {
                    item_id,
                    quantity: 1,
                },
                out,
            )?;
        }
        Ok(())
    }

    /// Credit one `QuestEvent::UseAbility` per accepted cast. An empty list (every other
    /// encounter outcome) touches no engine.
    pub(super) fn emit_ability_uses(
        &mut self,
        abilities_used: Vec<(i32, Option<u32>)>,
        out: &mut Outcome,
    ) -> io::Result<()> {
        for (ability_id, target_actor_kind_id) in abilities_used {
            self.npc_quest_event(
                crate::quest_engine::QuestEvent::UseAbility {
                    ability_id,
                    target_actor_kind_id,
                },
                out,
            )?;
        }
        Ok(())
    }
}
