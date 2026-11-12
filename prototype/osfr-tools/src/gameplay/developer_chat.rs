//! The developer `realm` routes, moved out of the opening's chat path (rc-0136, enemy tuning):
//! a realm that runs without the opening content (the shipped `data/content` alone) took
//! its chat through [`Gameplay::handle`], which knows only the ordinary commands, so `realm
//! god`, `realm sim`, `realm battle`, teleports and the rest answered nothing. Both paths
//! now ask [`Gameplay::developer_realm`] first.
use super::*;

impl Gameplay {
    /// A `realm` chat line on the no-opening path: the developer routes when one of them takes
    /// it; `None` hands the line on to [`Gameplay::handle`].
    pub(super) fn developer_chat(
        &mut self,
        inner: &[u8],
        character: &CharacterState,
        now_ms: u64,
    ) -> io::Result<Option<Outcome>> {
        let Some(text) = chat::request(inner, character.guid)? else {
            return Ok(None);
        };
        let lower = text.trim().to_ascii_lowercase();
        let Some(command) = lower.strip_prefix("realm ") else {
            return Ok(None);
        };
        let mut out = Outcome::default();
        let taken = self.developer_realm(&text, command, character, now_ms, &mut out)?;
        Ok(taken.then_some(out))
    }

    /// The developer routes of a `realm <command>` line (`text` is the line as typed, so a
    /// literal name keeps its case): true when one of them answered into `out`.
    pub(super) fn developer_realm(
        &mut self,
        text: &str,
        command: &str,
        character: &CharacterState,
        now_ms: u64,
        out: &mut Outcome,
    ) -> io::Result<bool> {
        let tool = self.may_run(command);
        if tool && self.developer_npx_off(command, out) {
            return Ok(true);
        }
        if tool && self.developer_quest_offer(command, character, out)? {
            return Ok(true);
        }
        if tool && self.developer_teleport(command, character, out)? {
            return Ok(true);
        }
        if tool && command == "here" {
            out.replies
                .push(chat::system(&self.developer_here(character)));
            return Ok(true);
        }
        // Same byte offsets as `command` (ASCII lowercasing keeps lengths): literal names keep
        // their case.
        let raw = &text.trim()["realm ".len()..];
        // L081: every player may redeem a code (no role gate).
        if command.split_whitespace().next() == Some("redeem") {
            let words: Vec<&str> = raw.split_whitespace().collect();
            self.realm_redeem(&words, character, now_ms, out)?;
            return Ok(true);
        }
        if tool && command.split_whitespace().next() == Some("broadcast") {
            crate::world_notice::stage(raw, out);
            return Ok(true);
        }
        Ok(tool && self.developer_cheat(command, raw, character, now_ms, out)?)
    }
}
