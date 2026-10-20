//! `realm broadcast <text>`: one server-owned system line for every online session.
//! The text is validated here and encoded by the existing 15/3 `ChatPacketDebugChat`
//! writer (`chat::system`), so no second packet shape or per-recipient encoder exists.
//! Evidence (IDA, client 1.910.1.530630): receive dispatcher `0x00b14350` reads the shared
//! two-u16 header (`0x00b0f550`) and, for subtype 3, constructs `0x00b0f9d0` and reads the
//! body with `0x00b0f5f0` (called from `0x00b10a20`): two u16 header words, a u32 byte
//! length, that many text bytes, then one flag byte. See
//! notes/Build Guides/Gameplay/Social (Friends, Guilds, Chat).md mode 3.
use std::fmt;

/// Longest announcement in characters (R: local bound; no recovered native limit).
pub const MAX_BROADCAST_CHARS: usize = 200;
/// Hard pre-check bound. A larger line is refused before counting characters or
/// naming recipients, so no packet or per-recipient allocation scales with it.
pub const MAX_BROADCAST_BYTES: usize = 2048;

const COMMAND: &str = "broadcast";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NoticeError {
    /// The command is not `broadcast <text>`.
    Syntax,
    Empty,
    TooManyChars,
    TooManyBytes,
    ControlCharacter,
}

impl fmt::Display for NoticeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Syntax => write!(f, "Usage: realm broadcast <message>."),
            Self::Empty => write!(f, "Broadcast needs a message: realm broadcast <message>."),
            Self::TooManyChars => write!(
                f,
                "Broadcast is too long: {MAX_BROADCAST_CHARS} characters maximum."
            ),
            Self::TooManyBytes => write!(
                f,
                "Broadcast is too long: {MAX_BROADCAST_BYTES} bytes maximum."
            ),
            Self::ControlCharacter => {
                write!(f, "Broadcast cannot contain control characters.")
            }
        }
    }
}

/// Parse the raw `realm` command (everything after `realm `) and return the trimmed
/// announcement. The literal text is kept for the packet; a refusal never echoes input.
pub fn parse(command: &str) -> Result<&str, NoticeError> {
    let rest = command
        .get(..COMMAND.len())
        .filter(|word| word.eq_ignore_ascii_case(COMMAND))
        .and_then(|_| command.get(COMMAND.len()..))
        .filter(|rest| rest.is_empty() || rest.starts_with(char::is_whitespace))
        .ok_or(NoticeError::Syntax)?;
    let text = rest.trim();
    if text.is_empty() {
        return Err(NoticeError::Empty);
    }
    if text.len() > MAX_BROADCAST_BYTES {
        return Err(NoticeError::TooManyBytes);
    }
    if text.chars().count() > MAX_BROADCAST_CHARS {
        return Err(NoticeError::TooManyChars);
    }
    if text
        .chars()
        .any(|character| u32::from(character) < 0x20 || character == '\u{7f}')
    {
        return Err(NoticeError::ControlCharacter);
    }
    Ok(text)
}

/// The one 15/3 system-line encoder; markup escaping stays in `chat::system`.
pub fn encode(text: &str) -> Vec<u8> {
    crate::chat::system(text)
}

/// Stage a `realm` line that starts with `broadcast`: the notice on success, one system
/// refusal otherwise. Both realm dispatch paths call this so the rule lives in one place;
/// a refusal never echoes the input.
pub fn stage(raw: &str, out: &mut crate::gameplay::Outcome) {
    match parse(raw) {
        Ok(text) => out.broadcast = Some(text.to_owned()),
        Err(error) => out.replies.push(crate::chat::system(&error.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn command(text: &str) -> String {
        format!("broadcast {text}")
    }

    #[test]
    fn boundary_is_two_hundred_characters_either_side() {
        // Literal values: the bounds are pinned by the spec, so a changed constant fails here.
        assert_eq!(MAX_BROADCAST_CHARS, 200);
        assert_eq!(MAX_BROADCAST_BYTES, 2048);
        let accepted = "a".repeat(200);
        assert_eq!(parse(&command(&accepted)), Ok(accepted.as_str()));
        let refused = "a".repeat(201);
        assert_eq!(parse(&command(&refused)), Err(NoticeError::TooManyChars));
    }

    #[test]
    fn multibyte_text_counts_characters_but_never_bytes_first() {
        let smiles = "😀".repeat(200);
        assert_eq!(smiles.len(), 800);
        assert_eq!(parse(&command(&smiles)), Ok(smiles.as_str()));
        // 2,049 ASCII bytes is refused before a 2,049-character count.
        let huge = "a".repeat(2049);
        assert_eq!(parse(&command(&huge)), Err(NoticeError::TooManyBytes));
        // 201 four-byte characters are 804 bytes: the character bound still refuses.
        let too_many = "😀".repeat(201);
        assert_eq!(parse(&command(&too_many)), Err(NoticeError::TooManyChars));
    }

    #[test]
    fn empty_whitespace_and_missing_text_are_refused() {
        assert_eq!(parse("broadcast"), Err(NoticeError::Empty));
        assert_eq!(parse("broadcast    "), Err(NoticeError::Empty));
        assert_eq!(parse("broadcastful"), Err(NoticeError::Syntax));
        assert_eq!(parse("say hello"), Err(NoticeError::Syntax));
    }

    #[test]
    fn control_characters_are_refused_and_inner_spaces_are_trimmed() {
        for bad in [
            "hi\r\nthere",
            "hi\nthere",
            "hi\rthere",
            "hi\tthere",
            "hi\0there",
            "hi\u{7f}there",
        ] {
            assert_eq!(
                parse(&command(bad)),
                Err(NoticeError::ControlCharacter),
                "{bad:?}"
            );
        }
        assert_eq!(parse("broadcast   spaced  text  "), Ok("spaced  text"));
        assert_eq!(parse("BROADCAST Hello"), Ok("Hello"));
    }

    #[test]
    fn encode_is_the_existing_escaped_system_line() {
        let packet = encode("<hunt>&");
        assert_eq!(packet, crate::chat::system("<hunt>&"));
        assert_eq!(&packet[..4], &[15, 0, 3, 0]);
        assert_eq!(&packet[8..packet.len() - 1], b"&lt;hunt&gt;&amp;");
        assert_eq!(packet[packet.len() - 1], 1);
    }
}
