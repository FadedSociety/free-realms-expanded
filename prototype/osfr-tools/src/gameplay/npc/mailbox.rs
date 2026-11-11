//! Mail's gameplay side (gm-panel G2). The gateway answers mail itself (`crate::mail::chat`);
//! it needs three things only gameplay has: the reward grant a claim pays through, the item
//! tables a staff letter's attachments are checked against, and the busy rule. A postbox's
//! "Mail Sorting" row comes here as an ordinary 26/6 and starts the town's game through the
//! Games window's own gates (`begin_portal_game`: idle, Postman unlocked, rank floor, payout
//! fitting the bag), so mail sorting keeps exactly one way of starting.
use super::*;
use crate::{activity_portal::Kind, harvest_nodes::Gather};

impl Gameplay {
    /// A claim's grant: coins to the wallet and items through `grant_reward`, the path quests and
    /// redeem codes pay through (bag space, stacking, catalogue), then pets settled as a redeem
    /// settles them. An error grants nothing.
    pub(crate) fn mail_grant(
        &mut self,
        coins: u32,
        items: &[(i32, u32)],
        now_ms: u64,
    ) -> io::Result<Outcome> {
        let mut out = Outcome::default();
        self.grant_reward(coins, items, &mut out)?;
        self.settle_granted_pets(now_ms, &mut out)?;
        Ok(out)
    }

    /// Every attachment row is a catalogue item whose count fits one stack.
    pub(crate) fn mail_items_known(&self, items: &[(i32, u32)]) -> bool {
        items
            .iter()
            .all(|&(item, count)| self.inventory.accepts_reward(item, count))
    }

    /// In an activity (a minigame, a fight, a ride's game): no claim and no dialog then.
    pub(crate) fn mail_busy(&self) -> bool {
        self.activity_busy()
    }

    /// A postbox dialog's Mail Sorting row (`mail::chat::sorting_game`): the dialog closes and the
    /// town's game starts as Play on its Games window row would.
    pub(super) fn mailbox_sorting_26_6(
        &mut self,
        inner: &[u8],
        character: &CharacterState,
        now_ms: u64,
    ) -> io::Result<Option<Outcome>> {
        if !inner.starts_with(&[26, 0, 6, 0]) {
            return Ok(None);
        }
        let Some(game) = npc_interaction::decode_dialogue_response_26_6(inner)
            .ok()
            .and_then(crate::mail::chat::sorting_game)
            .and_then(|game| u32::try_from(game).ok())
        else {
            return Ok(None);
        };
        let gather = Kind::Gather {
            gather: Gather::Mail,
        };
        let mut out = self.begin_portal_game(game, gather, character, now_ms)?;
        out.replies
            .insert(0, npc_interaction::encode_dialogue_close_26_4());
        Ok(Some(out))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mail::chat::{Action, response};

    fn answer(action: Action, value: u32) -> Vec<u8> {
        [vec![26, 0, 6, 0], response(action, value).to_le_bytes().to_vec()].concat()
    }

    /// The postbox's Mail Sorting row is Play on the town's Games window row: the same gates and
    /// the same start, through the ordinary gameplay entry, after the dialog closes.
    #[test]
    fn a_postbox_sorting_row_starts_the_town_game_as_the_games_window_does() {
        let mut game = crate::gameplay::t013_objective_events::game();
        let content = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content");
        game.mail_sorting = Some(crate::mail_sorting::load(&content, None).unwrap());
        let character = CharacterState {
            guid: 1,
            zone_id: 0,
            position: [-1_098.0, 5.0, 225.0],
            rotation: [0.0, 0.0, 1.0],
        };
        // No Postman job: both refuse with the client's missing-job alert and start nothing.
        let row = Kind::Gather {
            gather: Gather::Mail,
        };
        let refused = game.clone().begin_portal_game(376, row, &character, 1_000).unwrap();
        let out = game.handle_in_world(&answer(Action::Sort, 376), &character, 1_000).unwrap();
        assert_eq!(out.replies[0], npc_interaction::encode_dialogue_close_26_4());
        assert_eq!(out.replies[1..], refused.replies[..]);
        assert!(!game.mail_sorting.as_ref().unwrap().active());

        game.profiles
            .as_mut()
            .unwrap()
            .unlock(crate::mail_sorting::POSTMAN_PROFILE_ID)
            .unwrap();
        let row_start = game.clone().begin_portal_game(376, row, &character, 2_000).unwrap();
        assert!(!row_start.replies.is_empty());
        let out = game.handle_in_world(&answer(Action::Sort, 376), &character, 2_000).unwrap();
        assert_eq!(out.replies[1..], row_start.replies[..]);
        assert!(game.mail_sorting.as_ref().unwrap().active(), "the Sanctuary game runs");
    }

    /// Other rows are not this hook's: a game no postbox names, or another mail action.
    #[test]
    fn only_a_postbox_game_row_is_taken() {
        let mut game = crate::gameplay::t013_objective_events::game();
        let character = CharacterState {
            guid: 1,
            zone_id: 0,
            position: [0.0; 3],
            rotation: [0.0, 0.0, 1.0],
        };
        for packet in [answer(Action::Sort, 93), answer(Action::Read, 376), vec![26, 0, 8, 0]] {
            assert!(game.mailbox_sorting_26_6(&packet, &character, 1).unwrap().is_none());
        }
    }
}
