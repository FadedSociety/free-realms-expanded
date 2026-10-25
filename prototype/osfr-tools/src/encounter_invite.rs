//! T440 (Shared World slice 5): the ordinary group-encounter invitation. A grouped member's Go
//! offers every other connected, eligible member the original join prompt (41/102); the
//! member's Accept or Decline comes back as 41/103. This module owns both strict codecs and the
//! bounded pending table; routing lives in `bin/osfr_gateway/encounter_invites.rs`.
//!
//! Client layouts (IDA, 2014 client). Both packets share the encounter header: u16 family 41,
//! u16 subtype, i32 word A, i32 word B (reader `008d6690`, writer `00913db0`).
//! * 41/102 inbound: constructor `00a9a0b0`, reader `00a9f920` -> `00a9ceb0`. After the header:
//!   u64 `+0x18` (the client never uses it; default `dword_1BE7B10`), i32 `+0x20` locale string id
//!   of the encounter name (looked up in the client string table, the `minigame` token of the
//!   `GAQEncounterInvite` pop-up through `00a88300` -> `00a0d810` -> `00a0d550`), i32 `+0x24`
//!   countdown handed to `HandlerGroupScreen:ActivateEncounterRequest` (`00a7d830`; zero or less
//!   deactivates). The dispatcher `00aa36c0` case 0x66 passes `a4 = 0`, so the client itself
//!   refuses trailing bytes; the message is exactly 28 bytes.
//! * 41/103 outbound: writer `00a97720` (accept 0 and 1 come from `00a09650` / `00a096a0`),
//!   constructor `008b6da0` -> `008a2fd0(0x67)` leaves word A zero, word B is `client+0xc10` (the
//!   last 41/107 instance), then u64 `+0x18` from the local character object `client+0xc8`
//!   (R: its GUID), then the accept byte at `+0x20`. Field writer `0091b870`, sender `0092a600`.
//!   21 bytes. Word B is NOT zero in real play: 41/107 (dispatcher `00aa36c0` case 0x6b stores
//!   its header word B into `client+0xc10`) is sent on every field-battle start screen
//!   (`encounter/field.rs`) and in the dungeon path, so it holds the last battle instance the
//!   client opened. Nothing in the reply names the invite, so none of its fields is read: the
//!   pending table keyed by the session's own character is the only authority and the run comes
//!   from the server-held row.
//! * `00aa2980` case 9 is the matchmaking-family dispatcher (u64 id plus string id, `00a88330`),
//!   not this packet; `00a979a0` is its reply writer. 41/129 and family 141 stay in matchmaking.

use std::collections::BTreeMap;

/// (R) How long an invite stays answerable. The client countdown below is the same number.
pub const TTL_MS: u64 = 30_000;
/// (R) Unit unverified (the swf `SetEncounterStartTimer`); the server deadline is authoritative.
pub const COUNTDOWN: i32 = (TTL_MS / 1000) as i32;
const INVITE_LEN: usize = 28;
const REPLY_LEN: usize = 21;

/// What the server tells the invitee: 41/102.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InvitePacket {
    pub encounter: u32,
    pub run: i32,
    /// Inviter's character; the client reads and ignores it (R).
    pub inviter: u64,
    pub title_id: i32,
    pub countdown: i32,
}

pub fn encode_invite(p: &InvitePacket) -> Vec<u8> {
    let mut b = vec![41, 0, 102, 0];
    b.extend((p.encounter as i32).to_le_bytes());
    b.extend(p.run.to_le_bytes());
    b.extend(p.inviter.to_le_bytes());
    b.extend(p.title_id.to_le_bytes());
    b.extend(p.countdown.to_le_bytes());
    b
}

/// Strict 41/102 reader (the tests' view of what the client reads).
pub fn decode_invite(b: &[u8]) -> Option<InvitePacket> {
    if b.len() != INVITE_LEN || b[..4] != [41, 0, 102, 0] {
        return None;
    }
    let i32_at = |at: usize| i32::from_le_bytes(b[at..at + 4].try_into().unwrap());
    Some(InvitePacket {
        encounter: u32::try_from(i32_at(4)).ok()?,
        run: i32_at(8),
        inviter: u64::from_le_bytes(b[12..20].try_into().unwrap()),
        title_id: i32_at(20),
        countdown: i32_at(24),
    })
}

/// What the invitee's client sends back: 41/103. Only `accept` is acted on; the other fields
/// are client-controlled and carry no authority (word B is a stale battle instance in real play).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Reply {
    /// Header word A: zero from a real client.
    pub encounter: i32,
    /// Header word B: `client+0xc10`, the last 41/107 instance this client saw.
    pub run: i32,
    /// `client+0xc8` accessor result (R: the local character GUID; unverified, so ignored).
    pub character: u64,
    pub accept: bool,
}

pub fn encode_reply(r: &Reply) -> Vec<u8> {
    let mut b = vec![41, 0, 103, 0];
    b.extend(r.encounter.to_le_bytes());
    b.extend(r.run.to_le_bytes());
    b.extend(r.character.to_le_bytes());
    b.push(u8::from(r.accept));
    b
}

/// Strict 41/103 reader: exact length, and the accept byte must be 0 or 1.
pub fn decode_reply(b: &[u8]) -> Option<Reply> {
    if b.len() != REPLY_LEN || b[..4] != [41, 0, 103, 0] || b[20] > 1 {
        return None;
    }
    Some(Reply {
        encounter: i32::from_le_bytes(b[4..8].try_into().unwrap()),
        run: i32::from_le_bytes(b[8..12].try_into().unwrap()),
        character: u64::from_le_bytes(b[12..20].try_into().unwrap()),
        accept: b[20] == 1,
    })
}

/// One outstanding invite, keyed by invitee in `Invites`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pending {
    pub inviter: u64,
    /// The inviter's World instance id; never taken from a client packet.
    pub run: i32,
    pub encounter: u32,
    pub deadline_ms: u64,
}

/// Bounded pending table: at most one outstanding invite per invitee.
#[derive(Clone, Debug, Default)]
pub struct Invites {
    rows: BTreeMap<u64, Pending>,
}

impl Invites {
    /// Record an invite. False (and nothing changes) while the invitee already has a live one
    /// or the table is at `cap`; an expired row is swept first.
    pub fn offer(&mut self, invitee: u64, row: Pending, now_ms: u64, cap: usize) -> bool {
        self.rows.retain(|_, p| now_ms < p.deadline_ms);
        if self.rows.contains_key(&invitee) || self.rows.len() >= cap {
            return false;
        }
        self.rows.insert(invitee, row);
        true
    }

    /// The invitee's live invite, left in place: an Accept reads it, rechecks, and consumes it
    /// with `answer` only once the join has committed, so a refused Accept burns nothing.
    pub fn live(&self, invitee: u64, now_ms: u64) -> Option<&Pending> {
        self.rows
            .get(&invitee)
            .filter(|row| now_ms < row.deadline_ms)
    }

    /// Consume the invitee's live invite (one answer per invite), else change nothing. The
    /// caller passes the session's own character; no reply field is a key. An expired row is
    /// dropped and answers nothing.
    pub fn answer(&mut self, invitee: u64, now_ms: u64) -> Option<Pending> {
        let row = self.rows.remove(&invitee)?;
        (now_ms < row.deadline_ms).then_some(row)
    }

    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    /// Keep only the rows `keep` accepts (World's per-step sweep: expired, a player gone, or
    /// the inviter no longer in its run).
    pub fn retain(&mut self, mut keep: impl FnMut(u64, &Pending) -> bool) {
        self.rows.retain(|invitee, row| keep(*invitee, row));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pending(inviter: u64, run: i32) -> Pending {
        Pending {
            inviter,
            run,
            encounter: 96,
            deadline_ms: 1_000,
        }
    }
    fn reply(encounter: i32, run: i32, character: u64, accept: bool) -> Reply {
        Reply {
            encounter,
            run,
            character,
            accept,
        }
    }

    #[test]
    fn invite_layout_is_the_client_reader_and_strict() {
        let packet = InvitePacket {
            encounter: 96,
            run: 7,
            inviter: 101,
            title_id: 42056,
            countdown: COUNTDOWN,
        };
        let bytes = encode_invite(&packet);
        assert_eq!(bytes.len(), 28);
        // Reader 00a9ceb0: u64 at +0x18 (bytes 12..20), string id +0x20, countdown +0x24.
        assert_eq!(bytes[20..24], 42056_i32.to_le_bytes());
        assert_eq!(bytes[24..28], 30_i32.to_le_bytes());
        assert_eq!(decode_invite(&bytes), Some(packet));
        assert_eq!(decode_invite(&bytes[..27]), None, "truncated");
        assert_eq!(decode_invite(&[bytes.clone(), vec![0]].concat()), None);
        let mut other = bytes;
        other[2] = 103;
        assert_eq!(decode_invite(&other), None);
    }

    #[test]
    fn reply_layout_is_the_client_writer_and_strict() {
        let bytes = encode_reply(&reply(0, 0, 202, true));
        // Writer 0091b870: header words, u64, then the accept byte last.
        assert_eq!(bytes.len(), 21);
        assert_eq!(bytes[12..20], 202_u64.to_le_bytes());
        assert_eq!(bytes[20], 1);
        assert_eq!(decode_reply(&bytes), Some(reply(0, 0, 202, true)));
        assert_eq!(decode_reply(&bytes[..20]), None, "truncated before accept");
        assert_eq!(decode_reply(&[bytes.clone(), vec![0]].concat()), None);
        let mut bad = bytes;
        bad[20] = 2;
        assert_eq!(decode_reply(&bad), None, "accept flag other than 0 or 1");
    }

    #[test]
    fn one_invite_per_invitee_and_the_table_is_capped() {
        let mut table = Invites::default();
        assert!(table.offer(202, pending(101, 5), 0, 2));
        assert!(
            !table.offer(202, pending(101, 6), 0, 2),
            "no silent replace"
        );
        assert!(table.offer(303, pending(101, 5), 0, 2));
        assert!(!table.offer(404, pending(101, 5), 0, 2), "cap");
        assert_eq!(table.rows.len(), 2);
        // Expiry frees the slot.
        assert!(table.offer(404, pending(101, 9), 1_000, 2));
        assert_eq!(table.rows.len(), 1);
    }

    #[test]
    fn an_answer_consumes_only_the_session_invite_once() {
        let mut table = Invites::default();
        assert!(table.offer(202, pending(101, 5), 0, 8));
        assert_eq!(table.answer(303, 10), None, "no invite for that character");
        assert_eq!(
            table.rows.len(),
            1,
            "another character's answer changes nothing"
        );
        let got = table.answer(202, 10).unwrap();
        assert_eq!((got.inviter, got.run), (101, 5));
        assert_eq!(table.answer(202, 10), None, "replay");
        assert!(table.rows.is_empty());
    }

    #[test]
    fn live_reads_the_invite_without_consuming_it() {
        let mut table = Invites::default();
        assert!(table.offer(202, pending(101, 5), 0, 8));
        assert_eq!(table.live(202, 10).map(|row| row.run), Some(5));
        assert_eq!(
            table.live(202, 10).map(|row| row.run),
            Some(5),
            "still there"
        );
        assert_eq!(table.live(303, 10), None);
        assert_eq!(table.live(202, 1_000), None, "expired");
        assert_eq!(table.rows.len(), 1);
        assert!(table.answer(202, 10).is_some());
    }

    #[test]
    fn an_expired_reply_is_inert_and_the_sweep_clears_rows() {
        let mut table = Invites::default();
        table.offer(202, pending(101, 5), 0, 8);
        assert_eq!(table.answer(202, 1_000), None);
        assert!(table.rows.is_empty());
        table.offer(202, pending(101, 5), 0, 8);
        table.offer(303, pending(404, 6), 0, 8);
        table.retain(|invitee, row| invitee != 303 && row.run == 5);
        assert_eq!(table.rows.len(), 1);
        table.retain(|_, _| false);
        assert!(table.rows.is_empty());
    }
}
