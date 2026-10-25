//! Housing rating directory stub (T517): the client's 174/16 `CandidateInfoRequest` probe on
//! every house entry, answered with an empty 174/17 `CandidateInfoReply`. Codec only; the
//! dispatch branch lives in `gameplay::housing`.
//!
//! Family 174 uses a **u8** subtype: the header reader `0x00c833b0` reads an i16 family and then
//! one u8 subtype, and the base constructor `0x00aa5ca0` stores family 174 plus the caller's
//! subtype (`0x00aae280` passes 16, the reply constructor `0x00c856c0` passes 17). Never reuse
//! family 127's u16-subtype reader here.
//!
//! Client evidence (2014 client, checked with IDA):
//! * request constructor `0x00aae280`: IStrings at +0xc and +0x1c, the global u64 at +0x30;
//!   request writer `0x00abfe70` writes the header through `0x00aba870` (i16 family, u8 subtype),
//!   both strings through `0x00911c30` (u32 length plus UTF-8 bytes) and the u64s at +0x30 and
//!   +0x38. 2 + 1 + (4 + 7) + 4 + 8 + 8 = 34 bytes.
//! * reply reader `0x00c85530`: header through `0x00c833b0`, u64 correlation, then the list
//!   reader `0x00c84e90` (i32 count). Each record reader `0x00c83d90` reads three i32-length
//!   strings, an f32 at record +0x30, an i32 at +0x34 and two bools at +0x38/+0x39.
//! * The record's semantic labels (candidate id, owner name, house name) and the two request
//!   u64s' meanings (owner guid, correlation) are reconstructed (R); the field order is the
//!   client's.
//!
//! Server rules are stricter than the client's reader: strings are at most `MAX_STRING` bytes and
//! valid UTF-8, a negative length or count is refused, and the body must be consumed exactly.
use std::io;

/// Family opcode; base constructor `0x00aa5ca0` stores 174.
pub const FAMILY: u16 = 174;
/// 174/16 request subtype; request constructor `0x00aae280` passes 16.
pub const CANDIDATE_INFO_REQUEST: u8 = 16;
/// 174/17 reply subtype; reply constructor `0x00c856c0` passes 17.
pub const CANDIDATE_INFO_REPLY: u8 = 17;
/// The system string the housing request carries; dispatcher `0x00c869e0` falls back to it (R).
pub const HOUSING_SYSTEM: &str = "Housing";

/// Bound on every wire string here; longer strings are refused before a string allocation.
pub const MAX_STRING: usize = 256;

/// Every record of `0x00c83d90` needs at least three empty strings, an f32, an i32 and two bools.
const MIN_RECORD: usize = 3 * 4 + 4 + 4 + 1 + 1;

fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

fn header(subtype: u8) -> [u8; 3] {
    [FAMILY as u8, (FAMILY >> 8) as u8, subtype]
}

/// 174/16 `RatingPacketCandidateInfoRequest` as the client writer `0x00abfe70` lays it out.
#[derive(Clone, Debug, PartialEq)]
pub struct CandidateInfoRequest {
    /// System the request is about; the client sends `Housing` (writer `0x00abfe70` +0xc).
    pub system: String,
    /// Candidate being inspected; empty on house entry (writer `0x00abfe70` +0x1c).
    pub candidate_id: String,
    /// u64 at +0x30 (writer `0x00abfe70`); reconstructed as the owner guid (R).
    pub owner_guid: u64,
    /// u64 at +0x38 (writer `0x00abfe70`); the reply carries it back (R).
    pub correlation: u64,
}

impl CandidateInfoRequest {
    /// Decode one inner application packet: header, two strings, two u64s, no trailing bytes.
    pub fn decode(inner: &[u8]) -> Option<Self> {
        let mut reader = Reader::new(inner);
        if reader.u16()? != FAMILY || reader.u8()? != CANDIDATE_INFO_REQUEST {
            return None;
        }
        let request = Self {
            system: reader.string()?,
            candidate_id: reader.string()?,
            owner_guid: reader.u64()?,
            correlation: reader.u64()?,
        };
        reader.finish().then_some(request)
    }
}

/// One row of the 174/17 candidate list, in client reader `0x00c83d90` order.
#[derive(Clone, Debug, PartialEq)]
pub struct CandidateRatingInfo {
    /// First string (record reader `0x00c83d90` +4); reconstructed as the candidate id (R).
    pub candidate_id: String,
    /// Second string (record reader `0x00c83d90` +0x10); reconstructed as the owner name (R).
    pub owner_name: String,
    /// Third string (record reader `0x00c83d90` +0x20); reconstructed as the house name (R).
    pub house_name: String,
    /// f32 at +0x30 (record reader `0x00c83d90`); the dispatcher hands it to the UI as the rating.
    pub rating: f32,
    /// i32 at +0x34 (record reader `0x00c83d90`): the vote count.
    pub votes: i32,
    /// Bool at +0x38 (record reader `0x00c83d90`): the player has already rated this candidate.
    pub has_rating: bool,
    /// Bool at +0x39 (record reader `0x00c83d90`): the player may vote.
    pub can_vote: bool,
}

/// 174/17 `RatingPacketCandidateInfoReply`: the echoed correlation and the candidate list.
#[derive(Clone, Debug, PartialEq)]
pub struct CandidateInfoReply {
    /// u64 after the header (reader `0x00c85530`); the request's second u64 (R).
    pub correlation: u64,
    /// i32-counted list (`0x00c84e90`); empty until the directory briefs fill it.
    pub candidates: Vec<CandidateRatingInfo>,
}

impl CandidateInfoReply {
    /// Encode the inner application packet the client dispatcher `0x00c869e0` reads.
    pub fn encode_inner(&self) -> io::Result<Vec<u8>> {
        let mut out = header(CANDIDATE_INFO_REPLY).to_vec();
        out.extend_from_slice(&self.correlation.to_le_bytes());
        let count = i32::try_from(self.candidates.len())
            .map_err(|_| invalid("rating candidate count out of range"))?;
        out.extend_from_slice(&count.to_le_bytes());
        for candidate in &self.candidates {
            let strings = [
                &candidate.candidate_id,
                &candidate.owner_name,
                &candidate.house_name,
            ];
            for text in strings {
                if text.len() > MAX_STRING {
                    return Err(invalid("rating candidate string longer than 256 bytes"));
                }
                out.extend_from_slice(&(text.len() as i32).to_le_bytes());
                out.extend_from_slice(text.as_bytes());
            }
            out.extend_from_slice(&candidate.rating.to_le_bytes());
            out.extend_from_slice(&candidate.votes.to_le_bytes());
            out.push(u8::from(candidate.has_rating));
            out.push(u8::from(candidate.can_vote));
        }
        Ok(out)
    }

    /// Decode a reply body; the server only encodes replies, tests round-trip them through this.
    pub fn decode(inner: &[u8]) -> Option<Self> {
        let mut reader = Reader::new(inner);
        if reader.u16()? != FAMILY || reader.u8()? != CANDIDATE_INFO_REPLY {
            return None;
        }
        let correlation = reader.u64()?;
        let count = usize::try_from(reader.i32()?).ok()?;
        // A count the remaining body cannot hold is refused before the allocation.
        if count > reader.remaining() / MIN_RECORD {
            return None;
        }
        let mut candidates = Vec::with_capacity(count);
        for _ in 0..count {
            candidates.push(CandidateRatingInfo {
                candidate_id: reader.string()?,
                owner_name: reader.string()?,
                house_name: reader.string()?,
                rating: reader.f32()?,
                votes: reader.i32()?,
                has_rating: reader.bool()?,
                can_vote: reader.bool()?,
            });
        }
        reader.finish().then_some(Self {
            correlation,
            candidates,
        })
    }
}

struct Reader<'a> {
    data: &'a [u8],
    offset: usize,
}

impl<'a> Reader<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data, offset: 0 }
    }

    fn remaining(&self) -> usize {
        self.data.len() - self.offset
    }

    fn take(&mut self, count: usize) -> Option<&'a [u8]> {
        let end = self.offset.checked_add(count)?;
        let bytes = self.data.get(self.offset..end)?;
        self.offset = end;
        Some(bytes)
    }

    fn u8(&mut self) -> Option<u8> {
        Some(self.take(1)?[0])
    }

    fn u16(&mut self) -> Option<u16> {
        Some(u16::from_le_bytes(self.take(2)?.try_into().ok()?))
    }

    fn i32(&mut self) -> Option<i32> {
        Some(i32::from_le_bytes(self.take(4)?.try_into().ok()?))
    }

    fn u64(&mut self) -> Option<u64> {
        Some(u64::from_le_bytes(self.take(8)?.try_into().ok()?))
    }

    fn f32(&mut self) -> Option<f32> {
        Some(f32::from_le_bytes(self.take(4)?.try_into().ok()?))
    }

    fn bool(&mut self) -> Option<bool> {
        match self.u8()? {
            0 => Some(false),
            1 => Some(true),
            _ => None,
        }
    }

    fn string(&mut self) -> Option<String> {
        let length = usize::try_from(self.i32()?)
            .ok()
            .filter(|&n| n <= MAX_STRING)?;
        let text = std::str::from_utf8(self.take(length)?).ok()?;
        Some(text.to_owned())
    }

    fn finish(&self) -> bool {
        self.offset == self.data.len()
    }
}
