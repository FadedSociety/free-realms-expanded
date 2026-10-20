//! Local gateway file layout beside the character save: Studio savestates, the social list,
//! the W173 chat settings and the per-character state files.
use super::*;

/// Studio savestates sit next to the character save.
pub(super) fn savestate_dir(state_path: &Path) -> PathBuf {
    state_path
        .parent()
        .map(|p| p.join("savestates"))
        .unwrap_or_else(|| PathBuf::from("savestates"))
}

/// W173 (R): the chat settings file sits beside the social list, `<guid>.chat.json`.
pub(super) fn chat_settings_path(social: &Path) -> PathBuf {
    social.with_extension("").with_extension("chat.json")
}

pub(super) fn social_path(state_path: &Path) -> PathBuf {
    // (R) Keep host-managed social data beside the character save.
    state_path.with_extension("social.json")
}

pub(super) fn character_state_path(configured: &Path, guid: u64) -> PathBuf {
    // (R) Store account-selected character state under a local per-character filename.
    configured
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("characters")
        .join(format!("{guid}.json"))
}
