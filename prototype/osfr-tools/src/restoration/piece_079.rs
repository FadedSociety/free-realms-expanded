//! Work item 079: world terrain reader coverage.
//!
//! This module carries the recovered inventory of the preserved Free Realms terrain chunks and the
//! reader-coverage boundary this project measured against them. It is a triage/classification and
//! accounting piece; it deliberately does not decode terrain geometry or collision, because the
//! pinned community reader already decodes the render chunk and because the missing decoders are a
//! recorded gap rather than a licence to invent one.
//!
//! ORIGINAL facts recovered from the preserved corpus (see
//! `notes/Research/Pieces/079 World terrain readers.md` for paths, hashes and the reader survey):
//! * The extraction holds 2,344 `*.gcnk` entries. 2,336 begin with the `GCNK` magic and carry a
//!   little-endian version word: 2 x v2, 51 x v3, 35 x v4, 1,122 x v5 and 1,126 x v6.
//! * The remaining eight are five-byte ASCII stubs containing `hello`. They are the original
//!   `sh_snowball_battle_*` tiles and share one CRC32 (`3610a686`); the pinned community reader
//!   rejects them on magic, which is an asset fact rather than a reader defect. A second community
//!   reader (`soir20/oxide`) special-cases exactly this placeholder.
//! * The applied reader is `ForgeLightToolkit.GcnkFile` revision
//!   `9c06714c5d79d9152c322814bc904abad944e06e` (MIT, pinned under `../lab/references`). Its
//!   declared version window is 0..=6, so every preserved chunk is inside it.
//! * `GcnkFile.LoadCollision` is `// TODO: Bullet Physics` and returns `true` without reading the
//!   decompressed collision section, so no collision is decoded by the applied reader.
//! * The chunk's detail mask ("height map") is skipped behind a `TODO: Height Map` comment; the
//!   per-tile `Unknown7..Unknown13` fields are carried without a recovered meaning, and the eco-data
//!   loop is marked `TODO: FreeRealms PS3`.
//! * Running that reader over all 2,344 chunks decodes 2,277 and aborts on 67: the eight stubs plus
//!   59 chunks (32 x v3, 27 x v4). Every abort is `NotImplementedException` thrown at
//!   `Editor/FileTypes/Gcnk/RuntimeObject.cs:70`, the version <= 4 terrain-object path.
//! * A private lab variant that reads the documented null-terminated terrain object name on that
//!   path recovers 47 of the 59 (all 32 v3 and 15 of the 27 v4). Every recovered v3 chunk then
//!   consumes exactly its declared decompressed chunk length, which is the structural evidence that
//!   the field is a variable-length string there; the name reads back empty in these chunks, so the
//!   name text is not the evidence. The 12 residual chunks (`AssetsW_010.pack`, v4) still overrun
//!   the chunk stream and 13 of the 15 recovered v4 chunks stop early, so the v4 layout stays an
//!   open gap rather than a solved one.
//! * The same variant records that 15 chunks parse without consuming the declared chunk length: 13
//!   of the recovered v4 chunks and two chunks (both v4) the pinned reader already reported `ok`,
//!   which makes "ok" a weaker signal than "consumed the whole declared chunk".
//!
//! RECONSTRUCTED, labelled policy: the classifier below is this item's own triage over the first
//! bytes of a chunk (magic, version, shipped stub) and the coverage accounting derived from it. It
//! exists so the checker and any future reader work agree on what the preserved corpus contains.
//! It is not a terrain decoder and proves no native behaviour.

use std::collections::{BTreeMap, BTreeSet};

/// Magic of a preserved terrain chunk ("GCNK").
pub const GCNK_MAGIC: [u8; 4] = *b"GCNK";
/// The shipped placeholder body, byte-exact: eight `sh_snowball_battle_*` entries are `hello`.
pub const PLACEHOLDER_TEXT: [u8; 5] = *b"hello";

/// Lowest chunk version the pinned community reader accepts (`Version is < 0 or > 6`).
pub const PINNED_READER_MIN_VERSION: i32 = 0;
/// Highest chunk version the pinned community reader accepts.
pub const PINNED_READER_MAX_VERSION: i32 = 6;

/// The pinned `GcnkFile.LoadCollision` is a `TODO: Bullet Physics` placeholder.
pub const COLLISION_DECODED: bool = false;
/// The pinned reader skips the detail mask / height map bytes behind a `TODO: Height Map` comment.
pub const DETAIL_MASK_DECODED: bool = false;

/// Preserved `*.gcnk` entries in the extraction.
pub const PRESERVED_CHUNK_COUNT: usize = 2344;
/// Preserved entries that are five-byte `hello` stubs instead of chunks.
pub const PLACEHOLDER_CHUNK_COUNT: usize = 8;
/// Version histogram of the 2,336 chunks that carry the `GCNK` magic, ascending by version.
pub const VERSION_HISTOGRAM: [(i32, usize); 5] = [(2, 2), (3, 51), (4, 35), (5, 1122), (6, 1126)];

/// Chunks the pinned community reader decodes when run over the whole preserved corpus.
pub const PINNED_SURVEY_OK: usize = 2277;
/// Chunks the pinned reader aborts on: the eight stubs plus [`PINNED_ABORT_CHUNKS`].
pub const PINNED_SURVEY_FAILED: usize = 67;
/// Real chunks (magic `GCNK`, version <= 4) the pinned reader aborts on.
pub const PINNED_ABORT_CHUNKS: usize = 59;
/// Chunks the documented v<=4 name path recovers in the private lab variant.
pub const VARIANT_RECOVERED_CHUNKS: usize = 47;
/// Of those, version 3 chunks; every one consumes its whole declared decompressed chunk.
pub const VARIANT_RECOVERED_V3: usize = 32;
/// Of those, version 4 chunks; only [`VARIANT_RECOVERED_V4_COMPLETE`] consume the whole chunk.
pub const VARIANT_RECOVERED_V4: usize = 15;
/// Recovered version 4 chunks whose parse consumed the declared decompressed chunk length.
pub const VARIANT_RECOVERED_V4_COMPLETE: usize = 2;
/// v4 chunks that still overrun the chunk stream with the name path applied.
pub const VARIANT_RESIDUAL_CHUNKS: usize = 12;
/// Chunks that parse without consuming the declared decompressed chunk length.
pub const SHORT_PARSE_CHUNKS: usize = 15;
/// Of those, chunks the pinned reader reports `ok` although the parse stopped early.
pub const PINNED_SHORT_PARSE_CHUNKS: usize = 2;

/// What the first bytes of a preserved terrain entry say about it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChunkHeader {
    /// A terrain chunk with the `GCNK` magic and its little-endian version word.
    Gcnk { version: i32 },
    /// One of the shipped five-byte `hello` stubs.
    PlaceholderText,
    /// Four bytes that match neither known shape.
    UnknownMagic([u8; 4]),
    /// Fewer than four bytes are available.
    Truncated { bytes: usize },
}

/// Classify a preserved terrain entry from its leading bytes.
///
/// Only the magic, the version word and the shipped stub shape are interpreted; the chunk body is
/// left to a real reader. `bytes` may be the whole file or just its header.
pub fn classify(bytes: &[u8]) -> ChunkHeader {
    if bytes.len() < 4 {
        return ChunkHeader::Truncated { bytes: bytes.len() };
    }
    let magic: [u8; 4] = bytes[..4].try_into().expect("four bytes");
    if magic == GCNK_MAGIC {
        let version = bytes
            .get(4..8)
            .map(|word| i32::from_le_bytes(word.try_into().expect("four bytes")))
            // A chunk shorter than its version word cannot be read by the pinned reader either.
            .unwrap_or(-1);
        return ChunkHeader::Gcnk { version };
    }
    if bytes.len() >= PLACEHOLDER_TEXT.len() && bytes[..PLACEHOLDER_TEXT.len()] == PLACEHOLDER_TEXT
    {
        return ChunkHeader::PlaceholderText;
    }
    ChunkHeader::UnknownMagic(magic)
}

/// Running coverage of the preserved terrain corpus, built from header triage.
#[derive(Clone, Debug, Default)]
pub struct TerrainSurvey {
    total: usize,
    decoded: usize,
    placeholders: usize,
    truncated: usize,
    versions: BTreeMap<i32, usize>,
    unsupported: BTreeSet<i32>,
    unknown_magic: BTreeSet<[u8; 4]>,
}

impl TerrainSurvey {
    /// Account for one preserved entry by its leading bytes.
    pub fn classify(&mut self, bytes: &[u8]) {
        self.total += 1;
        match classify(bytes) {
            ChunkHeader::Gcnk { version } => {
                self.decoded += 1;
                *self.versions.entry(version).or_insert(0) += 1;
                if !(PINNED_READER_MIN_VERSION..=PINNED_READER_MAX_VERSION).contains(&version) {
                    self.unsupported.insert(version);
                }
            }
            ChunkHeader::PlaceholderText => self.placeholders += 1,
            ChunkHeader::UnknownMagic(magic) => {
                self.unknown_magic.insert(magic);
            }
            ChunkHeader::Truncated { .. } => self.truncated += 1,
        }
    }

    /// Entries that carry the `GCNK` magic and a version word.
    pub const fn decoded_chunks(&self) -> usize {
        self.decoded
    }

    /// Shipped five-byte `hello` stubs.
    pub const fn placeholders(&self) -> usize {
        self.placeholders
    }

    /// Every entry seen, including stubs and unknown magic.
    pub const fn total(&self) -> usize {
        self.total
    }

    /// Entries with fewer than four readable bytes.
    pub const fn truncated(&self) -> usize {
        self.truncated
    }

    /// Version histogram of the decoded chunks, ascending.
    pub const fn versions(&self) -> &BTreeMap<i32, usize> {
        &self.versions
    }

    /// Versions outside the pinned reader's declared window, ascending.
    pub fn unsupported_versions(&self) -> Vec<i32> {
        self.unsupported.iter().copied().collect()
    }

    /// Distinct four-byte magics that match neither known shape, sorted.
    pub fn unknown_magic(&self) -> Vec<[u8; 4]> {
        self.unknown_magic.iter().copied().collect()
    }

    /// True when every decoded version sits inside the pinned reader's window.
    pub fn within_pinned_reader(&self) -> bool {
        self.unsupported.is_empty()
    }
}

/// Structure checks over the recovered inventory. The corpus-wide derivation lives in
/// `tests/piece_079.rs` and the private recovery checker `piece_079.py`; this keeps the constants self-consistent.
pub fn validate() -> Result<(), &'static str> {
    let decoded: usize = VERSION_HISTOGRAM.iter().map(|(_, count)| count).sum();
    if decoded + PLACEHOLDER_CHUNK_COUNT != PRESERVED_CHUNK_COUNT {
        return Err("the recovered chunk inventory does not add up");
    }
    if PINNED_SURVEY_OK + PINNED_SURVEY_FAILED != PRESERVED_CHUNK_COUNT {
        return Err("the pinned reader survey does not add up");
    }
    if PLACEHOLDER_CHUNK_COUNT + PINNED_ABORT_CHUNKS != PINNED_SURVEY_FAILED {
        return Err("the pinned reader's aborts are not accounted for");
    }
    if VARIANT_RECOVERED_CHUNKS + VARIANT_RESIDUAL_CHUNKS != PINNED_ABORT_CHUNKS {
        return Err("the variant recovery does not account for every aborted chunk");
    }
    if VARIANT_RECOVERED_V3 + VARIANT_RECOVERED_V4 != VARIANT_RECOVERED_CHUNKS {
        return Err("the variant recovery does not split into the recovered versions");
    }
    if VARIANT_RECOVERED_V4_COMPLETE > VARIANT_RECOVERED_V4 {
        return Err("the recovered v4 split is impossible");
    }
    if VERSION_HISTOGRAM.iter().any(|(version, _)| {
        !(PINNED_READER_MIN_VERSION..=PINNED_READER_MAX_VERSION).contains(version)
    }) {
        return Err("a recovered chunk version is outside the pinned reader's window");
    }
    Ok(())
}
