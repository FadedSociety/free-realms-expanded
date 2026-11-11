//! GM panel (G1): the Admin Core. One function per staff action, role-gated and audited, shared
//! by every front end: the web panel's HTTP API (`osfr_gateway/admin_http.rs`) and the chat
//! commands that already existed (`realm mute|unmute|kick|ban|unban`, the GM wheel, `realm
//! broadcast`, `!gm players`). Plain types only: the gateway gathers live state and calls in.
//!
//! - [`roles`]: which role may run each action.
//! - [`audit`]: the append-only staff audit log beside the account store.
//! - [`roster`]: the online list, player search and profile reads.
//! - [`moderation`]: the panel's moderation ops on top of `crate::moderation`, and warn.
//! - [`icons`]: item icons read from the player's client install.
use serde::Serialize;

pub mod audit;
pub mod icons;
pub mod moderation;
pub mod roles;
pub mod roster;

/// The staff member acting: the verified account id and the name the audit log shows (the
/// login handle on the panel, the character name in chat).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, serde::Deserialize)]
pub struct Staff {
    pub account_id: u64,
    pub name: String,
}

/// Longest staff-written text kept (a reason or a note), in characters (R).
pub const MAX_TEXT_CHARS: usize = 500;

/// Staff free text (a reason, a note, an announcement): trimmed, bounded and free of control
/// characters, so it is safe in a chat line and a JSON row. `None` or blank gives `None`.
pub fn clean_text(text: Option<&str>) -> Result<Option<String>, &'static str> {
    let Some(text) = text.map(str::trim).filter(|text| !text.is_empty()) else {
        return Ok(None);
    };
    if text.len() > MAX_TEXT_CHARS * 4 || text.chars().count() > MAX_TEXT_CHARS {
        return Err("text is too long");
    }
    if text.chars().any(char::is_control) {
        return Err("text cannot contain control characters");
    }
    Ok(Some(text.to_owned()))
}

#[cfg(test)]
mod tests {
    use super::clean_text;

    #[test]
    fn staff_text_is_trimmed_bounded_and_single_line() {
        assert_eq!(clean_text(None), Ok(None));
        assert_eq!(clean_text(Some("   ")), Ok(None));
        assert_eq!(clean_text(Some("  spam ")), Ok(Some("spam".into())));
        assert!(clean_text(Some("a\nb")).is_err());
        assert!(clean_text(Some(&"x".repeat(501))).is_err());
        assert!(clean_text(Some(&"😀".repeat(500))).is_ok());
    }
}
