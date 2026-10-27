//! T533: the owner's pet treasure chest in `Gameplay`: the world tick of the hunt and the click
//! (26/8) that opens the chest. The hunt, the chest and its contents live in `crate::pets`; the
//! payout goes through the one reward path (`rewards::apply_grant`, adopted by
//! `adopt_reward_state`), so the bag limit and the atomic staging are the ordinary ones.
use super::*;
use crate::{
    pets::PetBusy,
    rewards::{GrantFailure, apply_grant},
};

const BAG_FULL: &str = "Your bag is full. Make room before opening the treasure.";
const BUSY: &str = "Finish the active activity before opening the treasure.";
const CANNOT: &str = "You cannot open that treasure right now.";

impl Gameplay {
    /// The hunt's world tick. The find marks a save (the pet's cooldown); the chest goes to the
    /// owner alone (`replies`), the sniff and the dig also to the viewers (`nearby`).
    pub(super) fn tick_pet_treasure(
        &mut self,
        character: &CharacterState,
        now_ms: u64,
        out: &mut Outcome,
    ) {
        let busy = PetBusy {
            minigame_active: self.blocks(|gates| gates.pets),
        };
        let ground = terrain_height(self.terrain.clone(), character.zone_id);
        let wire = self.pets.treasure_tick(
            busy,
            character.position,
            ground,
            now_ms,
            current_unix_seconds(),
        );
        out.changed |= wire.changed;
        out.replies.extend(wire.private);
        out.nearby.extend(wire.shared);
    }

    /// 26/8 on the player's own chest. A click on anything else is `None` (the NPC router goes on).
    /// Every refusal is one chat line and changes nothing; the grant and the chest's removal commit
    /// together on a staged copy, so a chest pays once and a full bag keeps it.
    pub(in crate::gameplay) fn pet_treasure_26_8(
        &mut self,
        inner: &[u8],
        character: &CharacterState,
        now_ms: u64,
    ) -> io::Result<Option<Outcome>> {
        let Ok(request) = npc_interaction::decode_npc_interact_26_8(inner) else {
            return Ok(None);
        };
        let guid = request.target;
        if !self.pets.is_treasure_chest(guid) {
            return Ok(None);
        }
        let refuse = |text: &str| {
            Ok(Some(Outcome {
                replies: vec![chat::system(text)],
                ..Outcome::default()
            }))
        };
        if self.activity_busy() {
            return refuse(BUSY);
        }
        let grant = match self
            .pets
            .open_treasure(guid, character.guid, character.position, now_ms)
        {
            Ok(grant) => grant,
            Err(refusal) => return refuse(refusal.message()),
        };
        let (next, replies) = match apply_grant(
            &self.reward_snapshot(),
            &grant,
            &format!("pet-treasure-{guid}"),
        ) {
            Ok(paid) => paid,
            Err(GrantFailure::BagFull) => return refuse(BAG_FULL),
            Err(error) => {
                eprintln!("pet treasure {guid} payout refused: {error}");
                return refuse(CANNOT);
            }
        };
        let mut staged = self.clone();
        let mut out = Outcome::default();
        out.replies.extend(replies);
        staged.adopt_reward_state(next, &mut out)?;
        out.replies.extend(staged.pets.clear_treasure());
        *self = staged;
        Ok(Some(out))
    }
}
