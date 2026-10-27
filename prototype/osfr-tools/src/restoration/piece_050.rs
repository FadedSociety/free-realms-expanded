//! Work item 050: character naming/select lifecycle.
//!
//! ORIGINAL facts implemented here (provenance in
//! `notes/Research/Pieces/050 Character naming/select lifecycle.md` and re-extracted by
//! the private recovery checker `piece_050.py`):
//!
//! * `decompiled/client/FreeRealms/pseudocode.c` constructs `PacketCheckNameRequest` with
//!   opcode 0xd2 (210) and `PacketCheckNameReply` with 0xd3 (211). Both are length-prefixed
//!   ASCII strings; the request carries first name, last name and three glossary definitions.
//! * `ClientAccountManager::OnRequestCheckName` switches the reply result code onto the literal
//!   message keys `Char_Name_Taken_PS3` (2), `Char_Name_Profane_PS3` (3),
//!   `Char_Name_Incorrect_Length_PS3` (5), `Char_Server_Failure_PS3` (6), `First_Name_Short_PS3`
//!   (7), `Last_Name_Short_PS3` (8), `First_Name_Long_PS3` (9), `Last_Name_Long_PS3` (10),
//!   `-1 | 4 -> Char_Name_Invalid_PS3`, `1 -> available`, default `Char_Unable_Log_Server_PS3`.
//!   It also walks a suggestion list whose rows carry name/prefix/suffix.
//! * `Resources/CharacterCreate/CharacterCreateNames.txt` composes preset names:
//!   `character.name.format = [*char_firstname*] [*char_lastname*]` over PART_ID 1 first names
//!   and `character.lastname.format = [*char_lastname_prefix*][*char_lastname_suffix*]` over
//!   PART_ID 2 prefix/suffix rows.
//! * `CharacterDeleteRequest`/`CharacterDeleteReply` are opcodes 9/10 (byte opcode, u64 entity
//!   key, i32 status, length-prefixed payload) per the pinned community serializers.
//!
//! RECONSTRUCTED, labelled at each call site: the character set/length window reused from
//! `identity::valid_name`, the case-insensitive duplicate comparison, and the local delete gate.
//! UNRECOVERED: the short/long thresholds behind result codes 7-10, a profanity list, the reply
//! suggestion rows, and any native delete acceptance.

use std::{
    collections::BTreeMap,
    fmt,
    io::{self, ErrorKind},
};

pub const CHECK_NAME_REQUEST_OPCODE: u16 = 210;
pub const CHECK_NAME_REPLY_OPCODE: u16 = 211;
pub const CHARACTER_DELETE_REQUEST_OPCODE: u8 = 9;
pub const CHARACTER_DELETE_REPLY_OPCODE: u8 = 10;

/// Locally reconstructed per-part window, identical to `identity::valid_name`, so the create and
/// check-name paths cannot disagree. The original thresholds behind result codes 7-10 are not
/// recovered; only this window is enforced.
pub const NAME_PART_MAX_CHARS: usize = 32;
/// Wire/reader bound used for every name string on the check-name packet.
pub const NAME_FIELD_MAX_BYTES: usize = 128;
/// Definition strings are longer than names; 512 bytes matches the existing login reader.
pub const NAME_DEFINITION_MAX_BYTES: usize = 512;

/// Recovered client text for result 5 (`Char_Name_Incorrect_Length_PS3`, locale 432109).
pub const INCORRECT_LENGTH_MESSAGE: &str = "The name you entered is not the right length.  Your first and last name must contain at least 15 characters.";
/// Recovered client text for result 7 (`First_Name_Short_PS3`, locale 432111).
pub const FIRST_NAME_SHORT_MESSAGE: &str =
    "The First Name you entered is too short.  Make sure it contains at least two characters.";
/// Recovered client text for result 8 (`Last_Name_Short_PS3`, locale 432112).
pub const LAST_NAME_SHORT_MESSAGE: &str =
    "The Last Name you entered is too short.  Make sure it contains at least two characters.";
/// Recovered client text for result 9 (`First_Name_Long_PS3`, locale 432097).
pub const FIRST_NAME_LONG_MESSAGE: &str =
    "The Custom name you have entered is invalid.  The first name must be less than 15 characters.";
/// Recovered client text for result 10 (`Last_Name_Long_PS3`, locale 432096).
pub const LAST_NAME_LONG_MESSAGE: &str =
    "The Custom name you have entered is invalid.  The last name must be less than 15 characters.";
/// Client default branch for a result code it does not know (`Char_Unable_Log_Server_PS3`).
pub const UNKNOWN_RESULT_MESSAGE_KEY: &str = "Char_Unable_Log_Server_PS3";
/// Custom-name thresholds recovered from the four length messages above.
pub const CUSTOM_NAME_PART_MIN_CHARS: usize = 2;
/// "must be less than 15 characters" for each part of a custom name.
pub const CUSTOM_NAME_PART_MAX_CHARS: usize = 14;
/// "Your first and last name must contain at least 15 characters" for a custom name.
pub const CUSTOM_NAME_COMBINED_MIN_CHARS: usize = 15;
/// Shortest name the shipped preset tables can compose: a 3 character first name plus a 2
/// character prefix and a 3 character suffix, canonicalised to the lexicographically first
/// witness (`Ari` + `Ox` + `arm`), 9 characters. Preset names bypass the custom-name combined
/// length rule because these tables were offered, which is why `classify_name` (the
/// production-matching shape rule) and `classify_custom_name` are separate.
pub const SHORTEST_PRESET_NAME: &str = "Ari Oxarm";
pub const SHORTEST_PRESET_NAME_LENGTH: usize = 9;

fn invalid(message: &'static str) -> io::Error {
    io::Error::new(ErrorKind::InvalidData, message)
}

/// `-1` and `4` share one client message; `4` is the canonical wire value.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NameResultCode {
    Available,
    Taken,
    Profane,
    Invalid,
    IncorrectLength,
    ServiceFailure,
    FirstNameTooShort,
    LastNameTooShort,
    FirstNameTooLong,
    LastNameTooLong,
}

impl NameResultCode {
    pub fn code(self) -> i32 {
        match self {
            Self::Available => 1,
            Self::Taken => 2,
            Self::Profane => 3,
            Self::Invalid => 4,
            Self::IncorrectLength => 5,
            Self::ServiceFailure => 6,
            Self::FirstNameTooShort => 7,
            Self::LastNameTooShort => 8,
            Self::FirstNameTooLong => 9,
            Self::LastNameTooLong => 10,
        }
    }

    pub fn from_code(code: i32) -> Option<Self> {
        Some(match code {
            1 => Self::Available,
            2 => Self::Taken,
            3 => Self::Profane,
            4 | -1 => Self::Invalid,
            5 => Self::IncorrectLength,
            6 => Self::ServiceFailure,
            7 => Self::FirstNameTooShort,
            8 => Self::LastNameTooShort,
            9 => Self::FirstNameTooLong,
            10 => Self::LastNameTooLong,
            _ => return None,
        })
    }

    /// Original message key from the client switch; empty means the client reports success.
    pub fn message_key(self) -> &'static str {
        match self {
            Self::Available => "",
            Self::Taken => "Char_Name_Taken_PS3",
            Self::Profane => "Char_Name_Profane_PS3",
            Self::Invalid => "Char_Name_Invalid_PS3",
            Self::IncorrectLength => "Char_Name_Incorrect_Length_PS3",
            Self::ServiceFailure => "Char_Server_Failure_PS3",
            Self::FirstNameTooShort => "First_Name_Short_PS3",
            Self::LastNameTooShort => "Last_Name_Short_PS3",
            Self::FirstNameTooLong => "First_Name_Long_PS3",
            Self::LastNameTooLong => "Last_Name_Long_PS3",
        }
    }

    /// The original client only acts on the recovered codes; anything else shows the
    /// `UNKNOWN_RESULT_MESSAGE_KEY` failure text.
    pub fn is_recovered_code(code: i32) -> bool {
        Self::from_code(code).is_some()
    }
}

struct Slice<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> Slice<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    fn take(&mut self, n: usize) -> io::Result<&'a [u8]> {
        let end = self
            .offset
            .checked_add(n)
            .filter(|end| *end <= self.bytes.len())
            .ok_or_else(|| invalid("truncated check-name packet"))?;
        let slice = &self.bytes[self.offset..end];
        self.offset = end;
        Ok(slice)
    }

    fn u8(&mut self) -> io::Result<u8> {
        Ok(self.take(1)?[0])
    }

    fn u16(&mut self) -> io::Result<u16> {
        Ok(u16::from_le_bytes(self.take(2)?.try_into().unwrap()))
    }

    fn u32(&mut self) -> io::Result<u32> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }

    fn i32(&mut self) -> io::Result<i32> {
        Ok(i32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }

    fn u64(&mut self) -> io::Result<u64> {
        Ok(u64::from_le_bytes(self.take(8)?.try_into().unwrap()))
    }

    fn string(&mut self, max: usize) -> io::Result<String> {
        let length = self.u32()? as usize;
        if length > max {
            return Err(invalid("check-name string exceeds bound"));
        }
        let bytes = self.take(length)?;
        std::str::from_utf8(bytes)
            .map(str::to_owned)
            .map_err(|_| invalid("check-name string is not UTF-8"))
    }

    fn end(&self) -> io::Result<()> {
        if self.offset == self.bytes.len() {
            Ok(())
        } else {
            Err(invalid("trailing bytes in check-name packet"))
        }
    }
}

fn put_string(out: &mut Vec<u8>, value: &str) {
    out.extend((value.len() as u32).to_le_bytes());
    out.extend(value.as_bytes());
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NameCheckRequest {
    pub first_name: String,
    pub last_name: String,
    pub name_definition: String,
    pub last_prefix_definition: String,
    pub last_suffix_definition: String,
}

impl NameCheckRequest {
    pub fn decode(bytes: &[u8]) -> io::Result<Self> {
        let mut r = Slice::new(bytes);
        if r.u16()? != CHECK_NAME_REQUEST_OPCODE {
            return Err(invalid("not a check-name request"));
        }
        let value = Self {
            first_name: r.string(NAME_FIELD_MAX_BYTES)?,
            last_name: r.string(NAME_FIELD_MAX_BYTES)?,
            name_definition: r.string(NAME_DEFINITION_MAX_BYTES)?,
            last_prefix_definition: r.string(NAME_DEFINITION_MAX_BYTES)?,
            last_suffix_definition: r.string(NAME_DEFINITION_MAX_BYTES)?,
        };
        r.end()?;
        Ok(value)
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut out = CHECK_NAME_REQUEST_OPCODE.to_le_bytes().to_vec();
        for value in [
            &self.first_name,
            &self.last_name,
            &self.name_definition,
            &self.last_prefix_definition,
            &self.last_suffix_definition,
        ] {
            put_string(&mut out, value);
        }
        out
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NameCheckReply {
    pub first_name: String,
    pub last_name: String,
    pub result: NameResultCode,
}

impl NameCheckReply {
    pub fn encode(&self) -> Vec<u8> {
        let mut out = CHECK_NAME_REPLY_OPCODE.to_le_bytes().to_vec();
        put_string(&mut out, &self.first_name);
        put_string(&mut out, &self.last_name);
        out.extend(self.result.code().to_le_bytes());
        out
    }

    pub fn decode(bytes: &[u8]) -> io::Result<Self> {
        let mut r = Slice::new(bytes);
        if r.u16()? != CHECK_NAME_REPLY_OPCODE {
            return Err(invalid("not a check-name reply"));
        }
        let first_name = r.string(NAME_FIELD_MAX_BYTES)?;
        let last_name = r.string(NAME_FIELD_MAX_BYTES)?;
        let code = r.i32()?;
        r.end()?;
        let result = NameResultCode::from_code(code)
            .ok_or_else(|| invalid("unrecovered check-name result code"))?;
        Ok(Self {
            first_name,
            last_name,
            result,
        })
    }
}

/// Duplicate check for a concrete display name. Compared case-insensitively; the exact-match rule
/// in the pinned community handler and the local create path are both satisfied by this stricter
/// reconstruction.
pub fn classify_name(first: &str, last: &str, existing: &[&str]) -> NameResultCode {
    if !name_shape_valid(first, last) {
        return NameResultCode::Invalid;
    }
    let display = if last.is_empty() {
        first.to_owned()
    } else {
        format!("{first} {last}")
    };
    if existing
        .iter()
        .any(|name| name.eq_ignore_ascii_case(&display))
    {
        NameResultCode::Taken
    } else {
        NameResultCode::Available
    }
}

/// Same window as `identity::valid_name`: both parts trimmed, first part non-empty, letters plus
/// space/hyphen/apostrophe only, at most `NAME_PART_MAX_CHARS` characters and
/// `NAME_FIELD_MAX_BYTES` bytes per part.
pub fn name_shape_valid(first: &str, last: &str) -> bool {
    !first.trim().is_empty()
        && first == first.trim()
        && last == last.trim()
        && first.chars().count() <= NAME_PART_MAX_CHARS
        && last.chars().count() <= NAME_PART_MAX_CHARS
        && name_characters_valid(first)
        && name_characters_valid(last)
}

fn name_characters_valid(value: &str) -> bool {
    value.len() <= NAME_FIELD_MAX_BYTES
        && value
            .chars()
            .all(|c| c.is_alphabetic() || matches!(c, ' ' | '-' | '\''))
}

/// Custom-name policy reconstructed from the four recovered length messages: letters plus
/// space/hyphen/apostrophe, each part 2-14 characters, and the composed display name at least 15
/// characters. Blank first or last names report the recovered short-name codes.
///
/// This is deliberately separate from `classify_name`: the production create path accepts the
/// wider `identity::valid_name` window, and the shipped preset tables offer 9 character names, so
/// adopting the recovered custom thresholds has to be a lead decision with native evidence.
pub fn classify_custom_name(first: &str, last: &str, existing: &[&str]) -> NameResultCode {
    if first != first.trim()
        || last != last.trim()
        || !name_characters_valid(first)
        || !name_characters_valid(last)
    {
        return NameResultCode::Invalid;
    }
    if first.chars().count() < CUSTOM_NAME_PART_MIN_CHARS {
        return NameResultCode::FirstNameTooShort;
    }
    if last.chars().count() < CUSTOM_NAME_PART_MIN_CHARS {
        return NameResultCode::LastNameTooShort;
    }
    if first.chars().count() > CUSTOM_NAME_PART_MAX_CHARS {
        return NameResultCode::FirstNameTooLong;
    }
    if last.chars().count() > CUSTOM_NAME_PART_MAX_CHARS {
        return NameResultCode::LastNameTooLong;
    }
    let display = format!("{first} {last}");
    if display.chars().count() < CUSTOM_NAME_COMBINED_MIN_CHARS {
        return NameResultCode::IncorrectLength;
    }
    if existing
        .iter()
        .any(|name| name.eq_ignore_ascii_case(&display))
    {
        NameResultCode::Taken
    } else {
        NameResultCode::Available
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CatalogueError {
    Empty,
    MissingFormat(&'static str),
    MalformedRow { line: usize },
    DuplicateId(String),
}

impl fmt::Display for CatalogueError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "empty name catalogue"),
            Self::MissingFormat(id) => write!(f, "missing format row {id}"),
            Self::MalformedRow { line } => write!(f, "malformed name row on line {line}"),
            Self::DuplicateId(id) => write!(f, "duplicate glossary id {id}"),
        }
    }
}

impl std::error::Error for CatalogueError {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ResolveError {
    MissingFirstName,
    UnknownDefinition(String),
    InconsistentPreset,
    IncompletePreset,
}

impl fmt::Display for ResolveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingFirstName => write!(f, "no first name or name definition"),
            Self::UnknownDefinition(id) => write!(f, "unknown glossary id {id}"),
            Self::InconsistentPreset => {
                write!(f, "typed name does not match the submitted definition")
            }
            Self::IncompletePreset => write!(f, "last name needs both prefix and suffix"),
        }
    }
}

impl std::error::Error for ResolveError {}

pub const FIRST_NAME_FORMAT_ID: &str = "character.name.format";
pub const LAST_NAME_FORMAT_ID: &str = "character.lastname.format";

/// Preset-name glossary from `CharacterCreateNames.txt`. PART_ID 1 rows are preset first names and
/// PART_ID 2 rows are last-name components; the `character.*.format` rows carry PART_ID 0.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NameCatalogue {
    entries: BTreeMap<String, String>,
    first_name_format: String,
    last_name_format: String,
}

impl NameCatalogue {
    pub fn parse(text: &str) -> Result<Self, CatalogueError> {
        let mut entries = BTreeMap::new();
        let mut first_name_format = None;
        let mut last_name_format = None;
        for (index, raw) in text.lines().enumerate() {
            let line = raw.trim_end_matches('\r');
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let line_number = index + 1;
            let fields: Vec<&str> = line.split('^').collect();
            if fields.len() < 3 {
                return Err(CatalogueError::MalformedRow { line: line_number });
            }
            let id = fields[0];
            let value = fields[1];
            let part: u32 = fields[2]
                .parse()
                .map_err(|_| CatalogueError::MalformedRow { line: line_number })?;
            match (part, id) {
                (0, FIRST_NAME_FORMAT_ID) => first_name_format = Some(value.to_owned()),
                (0, LAST_NAME_FORMAT_ID) => last_name_format = Some(value.to_owned()),
                (1 | 2, _) => {
                    if entries.insert(id.to_owned(), value.to_owned()).is_some() {
                        return Err(CatalogueError::DuplicateId(id.to_owned()));
                    }
                }
                _ => return Err(CatalogueError::MalformedRow { line: line_number }),
            }
        }
        if entries.is_empty() {
            return Err(CatalogueError::Empty);
        }
        Ok(Self {
            entries,
            first_name_format: first_name_format
                .ok_or(CatalogueError::MissingFormat(FIRST_NAME_FORMAT_ID))?,
            last_name_format: last_name_format
                .ok_or(CatalogueError::MissingFormat(LAST_NAME_FORMAT_ID))?,
        })
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn first_name_format(&self) -> &str {
        &self.first_name_format
    }

    pub fn last_name_format(&self) -> &str {
        &self.last_name_format
    }

    pub fn lookup(&self, glossary_id: &str) -> Option<&str> {
        self.entries.get(glossary_id).map(String::as_str)
    }

    /// Resolve a check-name request into the concrete name the original format rows describe.
    /// Typed values win only when they agree with a submitted definition; a request with empty
    /// definitions is the custom-name path.
    pub fn resolve(&self, request: &NameCheckRequest) -> Result<ResolvedName, ResolveError> {
        let preset_first = self.definition(&request.name_definition)?;
        let first_name = match (request.first_name.is_empty(), preset_first) {
            (false, Some(preset)) => {
                if preset != request.first_name {
                    return Err(ResolveError::InconsistentPreset);
                }
                request.first_name.clone()
            }
            (false, None) => request.first_name.clone(),
            (true, Some(preset)) => preset.to_owned(),
            (true, None) => return Err(ResolveError::MissingFirstName),
        };

        let prefix = self.definition(&request.last_prefix_definition)?;
        let suffix = self.definition(&request.last_suffix_definition)?;
        let last_name = match (prefix, suffix) {
            (Some(prefix), Some(suffix)) => {
                let composed = format!("{prefix}{suffix}");
                if !request.last_name.is_empty() && request.last_name != composed {
                    return Err(ResolveError::InconsistentPreset);
                }
                composed
            }
            (None, None) => request.last_name.clone(),
            _ => return Err(ResolveError::IncompletePreset),
        };

        Ok(ResolvedName {
            first_name,
            last_name,
        })
    }

    fn definition<'a>(&'a self, id: &str) -> Result<Option<&'a str>, ResolveError> {
        if id.is_empty() {
            return Ok(None);
        }
        self.entries
            .get(id)
            .map(|value| Some(value.as_str()))
            .ok_or_else(|| ResolveError::UnknownDefinition(id.to_owned()))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedName {
    pub first_name: String,
    pub last_name: String,
}

impl ResolvedName {
    pub fn display_name(&self) -> String {
        if self.last_name.is_empty() {
            self.first_name.clone()
        } else {
            format!("{} {}", self.first_name, self.last_name)
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CharacterDeleteRequest {
    pub entity_key: u64,
}

impl CharacterDeleteRequest {
    pub fn decode(bytes: &[u8]) -> io::Result<Self> {
        let mut r = Slice::new(bytes);
        if r.u8()? != CHARACTER_DELETE_REQUEST_OPCODE {
            return Err(invalid("not a character delete request"));
        }
        let entity_key = r.u64()?;
        r.end()?;
        Ok(Self { entity_key })
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut out = vec![CHARACTER_DELETE_REQUEST_OPCODE];
        out.extend(self.entity_key.to_le_bytes());
        out
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CharacterDeleteReply {
    pub entity_key: u64,
    pub status: i32,
}

impl CharacterDeleteReply {
    pub fn encode(&self) -> Vec<u8> {
        let mut out = vec![CHARACTER_DELETE_REPLY_OPCODE];
        out.extend(self.entity_key.to_le_bytes());
        out.extend(self.status.to_le_bytes());
        out.extend(0u32.to_le_bytes());
        out
    }

    pub fn decode(bytes: &[u8]) -> io::Result<Self> {
        let mut r = Slice::new(bytes);
        if r.u8()? != CHARACTER_DELETE_REPLY_OPCODE {
            return Err(invalid("not a character delete reply"));
        }
        let entity_key = r.u64()?;
        let status = r.i32()?;
        let payload_length = r.u32()? as usize;
        r.take(payload_length)?;
        r.end()?;
        Ok(Self { entity_key, status })
    }
}

/// Local delete gate: the requested key must be the saved character, and the separate gameplay
/// save must be absent or already coordinated before a delete is allowed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeleteDecision {
    Allowed,
    UnknownKey,
    GameplaySavePresent,
}

pub fn delete_decision(
    requested: u64,
    saved: Option<u64>,
    gameplay_save_present: bool,
) -> DeleteDecision {
    match saved {
        Some(key) if key == requested => {
            if gameplay_save_present {
                DeleteDecision::GameplaySavePresent
            } else {
                DeleteDecision::Allowed
            }
        }
        _ => DeleteDecision::UnknownKey,
    }
}
