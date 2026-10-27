//! T807: where a run takes its startup base from. `--reference DIR` names a captured, hash-checked
//! folder (lab use); without it every startup message is built from `<content>/startup/`
//! (`Gateway::load_gameplay_from_content`), so a release needs no capture.
use super::*;
use osfr_tools::gameplay::GameplayState;

impl Options {
    /// `Some` only when `--reference` named a capture folder.
    pub(crate) fn reference_dir(&self) -> Option<&Path> {
        (!self.reference.as_os_str().is_empty()).then_some(self.reference.as_path())
    }
}

/// The initial character, from the capture folder or the startup content.
pub(crate) fn initial_state(options: &Options) -> io::Result<CharacterState> {
    match (options.reference_dir(), &options.content) {
        (Some(reference), _) => Gateway::reference_state(reference),
        (None, Some(content)) => Gateway::content_state(content),
        (None, None) => Err(io::Error::other("no --reference and no --content")),
    }
}

/// `Gateway::load_gameplay` from the source `options` names.
pub(crate) fn load_gameplay(
    options: &Options,
    character: CharacterState,
    content: &Path,
    saved: Option<GameplayState>,
) -> io::Result<Gateway> {
    match options.reference_dir() {
        Some(reference) => Gateway::load_gameplay(reference, character, content, saved),
        None => Gateway::load_gameplay_from_content(character, content, saved),
    }
}
