//! Mystery chest requests, answered before the shared 42 inventory branch (moved out of the
//! `gameplay.rs` hub).
use super::*;

impl Gameplay {
    pub(super) fn handle_mystery_chest(
        &mut self,
        inner: &[u8],
        player_guid: u64,
    ) -> io::Result<Option<Outcome>> {
        let Some(content) = &self.mystery_chests else {
            return Ok(None);
        };
        let Some(request) = mystery_chest::parse_request(inner)? else {
            return Ok(None);
        };
        let before = self.inventory.state().clone();
        let result = content.handle(
            &mut self.inventory,
            &mut self.pending_chest,
            request,
            player_guid,
        )?;
        let mut out = Outcome {
            replies: result.replies,
            changed: result.changed,
            ..Outcome::default()
        };
        self.npc_inventory_events(&before, &mut out)?;
        Ok(Some(out))
    }
}
