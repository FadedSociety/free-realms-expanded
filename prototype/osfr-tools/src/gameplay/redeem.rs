//! L081: `realm redeem [code]` for every player and `realm code ...` for admins: the GM panel's
//! `!redeem` and `!code` (Sanctuary PR #163 protocol). Codes live in `crate::redeem_codes`.
use super::*;
use crate::{
    account_store::{AccountStoreLock, STAFF_LOCK_WAIT},
    redeem_codes::{self, Codes},
};

use redeem_codes::{random_code, today};

impl Gameplay {
    // gifts-codes: what the Claim window's gift rows need from the session (`crate::gifts::claim`).

    /// This character's own claim rows (achievement rewards), encoded for the 108/2 list.
    pub(crate) fn gift_claim_rows(&self) -> Vec<Vec<u8>> {
        self.progression_extras.claim_rows()
    }

    /// The folder `gift_catalog.json` is read from: the one the season table comes from.
    pub(crate) fn gift_content_dir(&self) -> PathBuf {
        crate::content::npc_content(&self.content_dir)
    }

    pub(crate) fn gift_presentation(&self, item: i32) -> Option<crate::inventory::ItemPresentation> {
        self.inventory.presentation(item).ok()
    }

    /// The serialized `ClientItemDefinition` a 108/6 carries.
    pub(crate) fn gift_definition(&self, item: i32) -> Option<Vec<u8>> {
        self.inventory.definition_record(item).map(<[u8]>::to_vec)
    }

    pub(crate) fn gift_member(&self) -> bool {
        self.game_rules.membership.is_active_at(current_unix_seconds())
    }

    /// A gift's grant through mail's reward path (stacking, the item tables, pets settled), held
    /// to the bag: no new shown stack past `BAG_STACKS` (or a legacy bag's own count). Run it on
    /// a copy; an error leaves that copy to be dropped.
    pub(crate) fn gift_grant(
        &mut self,
        coins: u32,
        items: &[(i32, u32)],
        now_ms: u64,
    ) -> io::Result<Vec<Vec<u8>>> {
        let room = (self.inventory.state().bag_stacks()).max(crate::inventory::BAG_STACKS);
        let granted = self.mail_grant(coins, items, now_ms)?;
        if self.inventory.state().bag_stacks() > room {
            return Err(io::Error::other("your bag is full, make room first"));
        }
        Ok(granted.replies)
    }

    pub(super) fn realm_redeem(
        &mut self,
        words: &[&str],
        character: &CharacterState,
        now_ms: u64,
        out: &mut Outcome,
    ) -> io::Result<bool> {
        let Some(path) = redeem_codes::store() else {
            out.replies
                .push(chat::system("This realm has no redeem codes."));
            return Ok(true);
        };
        // A bare `!redeem` (the gold hub tree) opens the panel's code box.
        let Some(code) = words.get(1) else {
            out.replies
                .push(chat::system(&crate::gm_panel::line(&["redeem", "open"])));
            return Ok(true);
        };
        let _lock = AccountStoreLock::acquire_within(path, STAFF_LOCK_WAIT)?;
        let mut codes = Codes::load(path)?;
        let member = self
            .game_rules
            .membership
            .is_active_at(current_unix_seconds());
        match codes.redeem(code, character.guid, member, today()) {
            Err(text) => out.replies.push(chat::system(text)),
            Ok((coins, items)) => {
                // The grant runs first: a refused grant (a full bag) records no use.
                self.grant_reward(coins, &items, out)?;
                self.settle_granted_pets(now_ms, out)?;
                codes.save(path)?;
                let name = redeem_codes::normalize(code);
                out.replies
                    .push(chat::system(&format!("Redeemed code {name}.")));
            }
        }
        Ok(true)
    }

    pub(super) fn realm_code(&mut self, words: &[&str], out: &mut Outcome) -> io::Result<bool> {
        let Some(path) = redeem_codes::store() else {
            out.replies
                .push(chat::system("This realm has no redeem codes."));
            return Ok(true);
        };
        let _lock = AccountStoreLock::acquire_within(path, STAFF_LOCK_WAIT)?;
        let mut codes = Codes::load(path)?;
        match codes.admin(&words[1..], today(), random_code) {
            Ok((lines, changed)) => {
                if changed {
                    codes.save(path)?;
                }
                out.replies
                    .extend(lines.iter().map(|line| chat::system(line)));
            }
            Err(text) => out.replies.push(chat::system(&text)),
        }
        Ok(true)
    }
}
