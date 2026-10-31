//! L081: `realm redeem [code]` for every player and `realm code ...` for admins: the GM panel's
//! `!redeem` and `!code` (Sanctuary PR #163 protocol). Codes live in `crate::redeem_codes`.
use super::*;
use crate::{
    account_store::{AccountStoreLock, STAFF_LOCK_WAIT},
    redeem_codes::{self, Codes},
};

fn today() -> i64 {
    (current_unix_seconds() / 86_400) as i64
}

/// `FR-XXXX-XXXX` from the OS random source (letters and digits without 0/O/1/I).
fn random_code() -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
    let mut bytes = [0u8; 8];
    let _ = getrandom::getrandom(&mut bytes);
    let pick: String = bytes
        .iter()
        .map(|b| ALPHABET[usize::from(*b) % ALPHABET.len()] as char)
        .collect();
    format!("FR-{}-{}", &pick[..4], &pick[4..])
}

impl Gameplay {
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
