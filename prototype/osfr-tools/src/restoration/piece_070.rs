//! Work item 070: writer-verified serialization of the quest-journal root tail.
//!
//! The opening-journal writer (`crate::opening_journal`) still copies the last eight bytes of the
//! captured self payload into every rebuilt journal. Those eight bytes are the two unnamed i32
//! fields the supplied compiled writer emits after `completed count / selected quest / select flag`
//! (`Gateway.Login.ClientPcData` quest block, decompiled `lab/decompiled/server/Gateway/
//! Gateway.Login/ClientPcData.cs`) and that the preserved native reader stores at quest-manager
//! `+0x1100`/`+0x1104` (`FUN_00940c60`; the shorter variant `FUN_00940e60` writes `+0x1100` too).
//! No preserved function reads either offset back from the quest manager, so the pair is
//! write-only presentation-neutral state.
//!
//! This module owns that tail as typed, validated data plus the field writer, so the remaining
//! captured bytes become writer output. The byte layout is verified against the supplied compiled
//! serializer in the private recovery checker `piece_070.py`; the two values are emitted for reconstructed
//! campaigns because every preserved self input declares `10`/`30`.
//!
//! ORIGINAL: field order and native destinations, the captured reference tail `241, 2513, true,
//! 10, 30`, and the write-only pair.
//! RECONSTRUCTED: `RECOVERED_FIELD_0X1100`/`RECOVERED_FIELD_0X1104` as the default emitted values
//! for reconstructed campaigns; the semantics of both fields stay unknown.
//!
//! Integration step for the lead: register the module and replace `OpeningJournal::root_unknown`
//! with [`JournalRootTail`] (`from_span` for the loaded reference, `opening_campaign(...).write(...)`
//! when rebuilding). This file adds no I/O, grants nothing and touches no shared manifest.

use std::io;
use std::ops::Range;

/// Exact root-tail size: completed i32, selected i32, select bool, two unnamed i32.
pub const TAIL_BYTES: usize = 17;
/// Bound copied from the opening-journal reference loader; spans are exact journal slices.
pub const MAX_SPAN_BYTES: usize = 2 * 1024 * 1024;
pub const MAX_QUESTS: usize = 256;
pub const MAX_OBJECTIVES: usize = 64;
const MAX_REWARD_ENTRIES: usize = 256;
const MAX_REWARD_STRING: usize = 65_536;

/// Quest-manager `+0x1100` (wire `ClientQuestData.Unknown4`). Captured as `10` in every preserved
/// self input; the field itself is write-only in the preserved client.
pub const RECOVERED_FIELD_0X1100: i32 = 10;
/// Quest-manager `+0x1104` (wire `ClientQuestData.Unknown5`). Captured as `30` in every preserved
/// self input; the field itself is write-only in the preserved client.
pub const RECOVERED_FIELD_0X1104: i32 = 30;

fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

/// Typed quest-journal root tail, in the supplied writer's field order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct JournalRootTail {
    completed_quests: i32,
    selected_quest_id: i32,
    auto_select_selected_quest: bool,
    unknown_0x1100: i32,
    unknown_0x1104: i32,
}

impl JournalRootTail {
    /// Tail for a reconstructed campaign: `selected_quest_id` is `-1` when nothing is selected.
    /// The two unnamed fields take the recovered captured values.
    pub fn opening_campaign(
        completed_quests: i32,
        selected_quest_id: i32,
        auto_select_selected_quest: bool,
    ) -> io::Result<Self> {
        Self::with_recovered_fields(
            completed_quests,
            selected_quest_id,
            auto_select_selected_quest,
            RECOVERED_FIELD_0X1100,
            RECOVERED_FIELD_0X1104,
        )
    }

    /// Explicit unnamed-field values. Used when re-serializing a span whose declared values differ
    /// from the recovered constants (for example the compiled-writer oracle).
    pub fn with_recovered_fields(
        completed_quests: i32,
        selected_quest_id: i32,
        auto_select_selected_quest: bool,
        unknown_0x1100: i32,
        unknown_0x1104: i32,
    ) -> io::Result<Self> {
        let tail = Self {
            completed_quests,
            selected_quest_id,
            auto_select_selected_quest,
            unknown_0x1100,
            unknown_0x1104,
        };
        tail.validate()?;
        Ok(tail)
    }

    pub fn completed_quests(&self) -> i32 {
        self.completed_quests
    }

    pub fn selected_quest_id(&self) -> i32 {
        self.selected_quest_id
    }

    pub fn auto_select_selected_quest(&self) -> bool {
        self.auto_select_selected_quest
    }

    pub fn unknown_0x1100(&self) -> i32 {
        self.unknown_0x1100
    }

    pub fn unknown_0x1104(&self) -> i32 {
        self.unknown_0x1104
    }

    /// Domain checks only; the two unnamed fields accept any i32 because their semantics are
    /// unrecovered, and the select flag is a plain bool native accepts in both states.
    pub fn validate(&self) -> io::Result<()> {
        if self.completed_quests < 0 {
            return Err(invalid("journal completed count must not be negative"));
        }
        if self.selected_quest_id < -1 || self.selected_quest_id == 0 {
            return Err(invalid(
                "journal selected quest id must be -1 (none) or a positive quest id",
            ));
        }
        Ok(())
    }

    pub fn to_bytes(&self) -> io::Result<[u8; TAIL_BYTES]> {
        self.validate()?;
        let mut bytes = [0u8; TAIL_BYTES];
        bytes[0..4].copy_from_slice(&self.completed_quests.to_le_bytes());
        bytes[4..8].copy_from_slice(&self.selected_quest_id.to_le_bytes());
        bytes[8] = u8::from(self.auto_select_selected_quest);
        bytes[9..13].copy_from_slice(&self.unknown_0x1100.to_le_bytes());
        bytes[13..17].copy_from_slice(&self.unknown_0x1104.to_le_bytes());
        Ok(bytes)
    }

    pub fn write(&self, out: &mut Vec<u8>) -> io::Result<()> {
        out.extend(self.to_bytes()?);
        Ok(())
    }

    pub fn parse(bytes: &[u8]) -> io::Result<Self> {
        if bytes.len() != TAIL_BYTES {
            return Err(invalid("journal tail is not exactly 17 bytes"));
        }
        let flag = match bytes[8] {
            0 => false,
            1 => true,
            _ => return Err(invalid("invalid journal select flag")),
        };
        let tail = Self {
            completed_quests: i32::from_le_bytes(bytes[0..4].try_into().unwrap()),
            selected_quest_id: i32::from_le_bytes(bytes[4..8].try_into().unwrap()),
            auto_select_selected_quest: flag,
            unknown_0x1100: i32::from_le_bytes(bytes[9..13].try_into().unwrap()),
            unknown_0x1104: i32::from_le_bytes(bytes[13..17].try_into().unwrap()),
        };
        tail.validate()?;
        Ok(tail)
    }

    /// Byte range of the root tail in an exact journal span. Walks the recovered record layout with
    /// the same bounds the opening-journal loader uses, so a malformed or unrelated span fails
    /// instead of yielding a plausible-looking offset.
    pub fn locate(span: &[u8]) -> io::Result<Range<usize>> {
        if span.len() > MAX_SPAN_BYTES {
            return Err(invalid("journal span exceeds size limit"));
        }
        let mut reader = Reader {
            bytes: span,
            offset: 0,
        };
        for _ in 0..reader.count(MAX_QUESTS)? {
            reader.take(16)?;
            reader.boolean()?;
            reader.take(12)?;
            reader.boolean()?;
            let progress = f32::from_le_bytes(reader.take(4)?.try_into().unwrap());
            if !progress.is_finite() {
                return Err(invalid("nonfinite journal quest progress"));
            }
            reader.reward()?;
            for _ in 0..reader.count(MAX_OBJECTIVES)? {
                reader.take(16)?;
                reader.boolean()?;
                reader.reward()?;
                reader.take(16)?;
                reader.boolean()?;
                reader.take(4)?;
            }
            reader.take(4)?;
            reader.boolean()?;
            reader.boolean()?;
        }
        let start = reader.offset;
        Self::parse(reader.take(TAIL_BYTES)?)?;
        if reader.offset != span.len() {
            return Err(invalid("journal span has trailing bytes"));
        }
        Ok(start..span.len())
    }

    /// Parse the tail of an exact journal span; validates the whole span structure first.
    pub fn from_span(span: &[u8]) -> io::Result<Self> {
        let range = Self::locate(span)?;
        Self::parse(&span[range])
    }

    /// Replace only the root tail of an exact journal span with writer output.
    pub fn replace_in_span(span: &[u8], tail: &JournalRootTail) -> io::Result<Vec<u8>> {
        let range = Self::locate(span)?;
        let mut result = span.to_vec();
        result[range].copy_from_slice(&tail.to_bytes()?);
        Ok(result)
    }
}

struct Reader<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> Reader<'a> {
    fn take(&mut self, length: usize) -> io::Result<&'a [u8]> {
        let end = self
            .offset
            .checked_add(length)
            .ok_or_else(|| invalid("journal range overflow"))?;
        let value = self
            .bytes
            .get(self.offset..end)
            .ok_or_else(|| invalid("truncated journal span"))?;
        self.offset = end;
        Ok(value)
    }

    fn integer(&mut self) -> io::Result<i32> {
        Ok(i32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }

    fn count(&mut self, limit: usize) -> io::Result<usize> {
        let value = self.integer()?;
        if value < 0 || value as usize > limit {
            return Err(invalid("invalid journal array count"));
        }
        Ok(value as usize)
    }

    fn boolean(&mut self) -> io::Result<bool> {
        match self.take(1)?[0] {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(invalid("invalid journal bool")),
        }
    }

    /// Reward bundle in the supplied writer's order: bool, 9*i32, 2*i64, 2*i32, entries, i32.
    /// Only recovered entry type 1 (item) is accepted; other types fail explicitly.
    fn reward(&mut self) -> io::Result<()> {
        let extended_item = self.boolean()?;
        self.take(60)?;
        for _ in 0..self.count(MAX_REWARD_ENTRIES)? {
            if self.integer()? != 1 {
                return Err(invalid("unsupported journal reward entry type"));
            }
            self.boolean()?;
            self.take(24)?;
            let length = self.count(MAX_REWARD_STRING)?;
            self.take(length)?;
            self.take(4)?;
            self.boolean()?;
            if extended_item {
                self.take(4)?;
            }
        }
        self.take(4)?;
        Ok(())
    }
}
