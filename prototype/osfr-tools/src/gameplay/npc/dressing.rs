//! rc-0136 F4: world dressing that answers a click. Wild plants and holiday figures are extra
//! spawns with no content NPC row, so a click used to log `unknown_runtime_guid` and answer
//! nothing ("Gathering plants doesn't work at all"). Their `npc_presentation.json` row now says
//! what a click gets (`Click`): a plant gives its harvest item once per regrow time for this
//! player, a figure says its line. Range and busy follow the gathering nodes; a refusal or a
//! spent plant answers one chat line and changes nothing.
//!
//! C10: an Archaeology Node (a dig-site prop with no content NPC row, `dig_sites.json`) starts
//! its hunt on click through the Games window row's own gates (`begin_portal_game`).
use super::*;
use crate::{
    activity_portal::Kind,
    harvest_nodes::{Denied, Gather},
};

/// (R) Click reach for dressing, the gathering nodes' `interact_range`.
const REACH_M: f32 = 12.0;

impl Gameplay {
    pub(super) fn dressing_click_26_8(
        &mut self,
        inner: &[u8],
        character: &CharacterState,
        now_ms: u64,
    ) -> io::Result<Option<Outcome>> {
        let (true, Some(target)) = (inner.starts_with(&[26, 0, 8, 0]), le::u64(inner, 4)) else {
            return Ok(None);
        };
        if self.content_npc_key(target).is_some() {
            return Ok(None);
        }
        let in_reach = |at: [f32; 3]| {
            let reach = (at.iter().zip(character.position))
                .map(|(a, b)| (a - b).powi(2))
                .sum::<f32>();
            reach <= REACH_M * REACH_M
        };
        let dig = (self.npc_interaction.as_ref())
            .and_then(|table| table.dig_site_game(target))
            .zip(self.npc_visibility.as_ref().and_then(|world| world.actor_position(target)));
        if let Some((game, at)) = dig {
            npc_interaction::decode_npc_interact_26_8(inner)
                .map_err(|_| invalid("invalid NPC interaction request"))?;
            if !in_reach(at) {
                return Ok(Some(Outcome {
                    replies: vec![chat::system(&Denied::TooFar.message())],
                    ..Outcome::default()
                }));
            }
            let hunt = Kind::Gather {
                gather: Gather::Artifact,
            };
            return self.begin_portal_game(game, hunt, character, now_ms).map(Some);
        }
        let Some((click, position, name, picked_until)) = self
            .npc_visibility
            .as_ref()
            .and_then(|visibility| visibility.dressing(target))
        else {
            return Ok(None);
        };
        npc_interaction::decode_npc_interact_26_8(inner)
            .map_err(|_| invalid("invalid NPC interaction request"))?;
        let mut out = Outcome::default();
        if !in_reach(position) {
            out.replies
                .push(chat::system(&Denied::TooFar.message()));
            return Ok(Some(out));
        }
        if self.activity_busy() {
            out.replies.push(chat::system(&Denied::Busy.message()));
            return Ok(Some(out));
        }
        let name = name.as_deref().unwrap_or("plant");
        if let Some(line) = click.say_string_id {
            out.replies.push(chat::string_say(target, line));
        }
        let Some(item) = click.item_id else {
            // (R) rc-0136 F4 gather 2: a kind with no served item (Wildroot, Briarapple) still
            // answers, so the click is never silent.
            if click.say_string_id.is_none() {
                out.replies.push(chat::system(&format!(
                    "There is nothing to gather from the {name} right now."
                )));
            }
            return Ok(Some(out));
        };
        if now_ms < picked_until {
            out.replies.push(chat::system(&Denied::Gone.message()));
            return Ok(Some(out));
        }
        let grant = [(item, 1)];
        if !self.inventory.has_definition(item) || !self.inventory.accepts_reward_bundle(&grant) {
            out.replies.push(chat::system(
                "Your bag is full. Make room before gathering.",
            ));
            return Ok(Some(out));
        }
        let mut staged = self.clone();
        if let Err(error) = staged.grant_reward(0, &grant, &mut out) {
            npc_event(serde_json::json!({
                "event": "dressing_refused", "guid": target, "reason": error.to_string()
            }));
            let refused = chat::system("You cannot gather that right now.");
            return Ok(Some(Outcome {
                replies: vec![refused],
                ..Outcome::default()
            }));
        }
        let regrow_ms = u64::from(click.regrow_seconds).saturating_mul(1000);
        if let Some(visibility) = staged.npc_visibility.as_mut() {
            visibility.pick(target, now_ms.saturating_add(regrow_ms));
        }
        out.replies
            .push(chat::system(&format!("You gathered from the {name}.")));
        npc_event(serde_json::json!({
            "event": "dressing_harvest", "guid": target, "item": item
        }));
        *self = staged;
        Ok(Some(out))
    }
}
