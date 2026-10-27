//! Party (group) lifecycle: the original group wire family plus a bounded local party policy.
//!
//! Piece 089, owned paths only. Evidence labels follow the other restoration modules:
//! ORIGINAL is read from the supplied client, the pinned community readers or original retail
//! capture bytes; RECONSTRUCTED is a bounded local policy the lead must replace with native
//! evidence; MISSING is an explicit gap.
//!
//! ORIGINAL facts implemented here:
//! * the party wire family is `BaseGroupPacket` = **40** (`0x28`). The shipped client's base
//!   constructor writes `0x28` into the packet's opcode slot
//!   (`lab/decompiled/client/FreeRealms/pseudocode.c`, `FUN_008a3560` / `FUN_008a35c0`).
//! * the sub-opcode is a **2-byte** little-endian field. `FUN_008a35c0(<sub>)` stores it in the
//!   base packet, and the pinned community reader uses a 2-byte sub-opcode for family 40
//!   (`FreeRealms-Legacy/.../PacketReaderExtensions.cs`; the same convention `src/gateway.rs`
//!   already tunnels).
//! * the sub-opcode table below is read from the constructors themselves: every derived class is
//!   `FUN_008a35c0(<sub>)` immediately before its `::vftable` store. That yields 15 rows. The older
//!   community enum (`OpenSourceFreeRealms/.../PacketCodes.cs`, `BaseGroupPackets`) declares the
//!   same 15 names at the same numbers. The client stays the authority: the private recovery checker `piece_089.py`
//!   re-derives the table from the binary and fails if the two ever disagree.
//! * the envelope is `u16 outer | u8 flag | u32 inner length | u16 opcode | sub | body`, where
//!   `inner length` counts from the opcode (the same convention as piece 020, which the original
//!   retail frames satisfy byte-for-byte).
//! * the retail client ships group UI assets (`GroupWindow`, `GroupPane`, `GroupSmaller`,
//!   `GroupHit`) in the pinned public asset manifest, so the party panels are original client
//!   surfaces driven by this traffic.
//!
//! RECONSTRUCTED here (no original server code for party membership was recovered):
//! * every membership rule: who may invite, capacity accounting including open invites, leader
//!   transfer on leader leave/disconnect, kick authority, and what happens to a party when its last
//!   member leaves or disconnects.
//! * the party-id namespace seeded by [`PartyRegistry::new`] and the opaque [`PlayerId`] width.
//!   Real group ids are server assigned and the identity field width is unrecovered.
//!
//! MISSING / explicit gaps:
//! * no captured family-40 traffic exists. A scan of all 68,968 preserved frame files (65,861 with
//!   a valid envelope, 30 distinct opcodes) finds **zero** opcode-40 frames, so the per-field body
//!   of every group message is opaque here. This module deliberately writes no body.
//! * the original server's invite-timeout, cross-zone and reconnect rules are unrecovered.
//! * no reward, currency, item or progression effect is modelled. Party membership grants nothing.
//!
//! ponytail: two maps (members, invites) and one validation pass per request. No trait objects, no
//! packet writer for an unrecovered body, no tick loop. Add the body codec only when a captured
//! family-40 frame or an original serializer exists.

use std::collections::BTreeMap;

/// ORIGINAL: `BaseGroupPacket` in the supplied client and the pinned community enums.
pub const GROUP_FAMILY: u16 = 40;
/// ORIGINAL: the same family as the client stores it (`param_1[1] = 0x28`).
pub const GROUP_FAMILY_FIELD: i32 = 0x28;
/// ORIGINAL: family 40 carries a 2-byte sub-opcode.
pub const GROUP_SUB_WIDTH: usize = 2;
/// ORIGINAL: how many derived group classes the shipped client constructs.
pub const GROUP_SUBOPCODE_COUNT: usize = 15;

const ENVELOPE_PREFIX: usize = 7;

/// RECONSTRUCTED: opaque player identity. The original width is unrecovered.
pub type PlayerId = u64;

/// ORIGINAL: sub-opcodes of `BaseGroupPacket` (family 40).
///
/// The numbers are read from the client constructors that call `FUN_008a35c0(<sub>)` immediately
/// before their `::vftable` store; `class_name` is that class. Rows 1..13 also appear in the older
/// community enum under the same numbers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PartyMessage {
    Invite,
    InviteReply,
    Leave,
    Accept,
    AcceptReply,
    Kick,
    KickReply,
    GroupUpdate,
    MemberUpdate,
    RenamePlayer,
    AnnounceEncounterReply,
    ChangingJob,
    ChangingJobUpdate,
    MapPing,
    InMinigame,
}

impl PartyMessage {
    /// ORIGINAL: every recovered row, in sub-opcode order.
    pub const ALL: [PartyMessage; GROUP_SUBOPCODE_COUNT] = [
        PartyMessage::Invite,
        PartyMessage::InviteReply,
        PartyMessage::Leave,
        PartyMessage::Accept,
        PartyMessage::AcceptReply,
        PartyMessage::Kick,
        PartyMessage::KickReply,
        PartyMessage::GroupUpdate,
        PartyMessage::MemberUpdate,
        PartyMessage::RenamePlayer,
        PartyMessage::AnnounceEncounterReply,
        PartyMessage::ChangingJob,
        PartyMessage::ChangingJobUpdate,
        PartyMessage::MapPing,
        PartyMessage::InMinigame,
    ];

    /// ORIGINAL: the sub-opcode the client constructor passes to `FUN_008a35c0`.
    pub fn sub(self) -> i32 {
        match self {
            PartyMessage::Invite => 1,
            PartyMessage::InviteReply => 2,
            PartyMessage::Leave => 3,
            PartyMessage::Accept => 4,
            PartyMessage::AcceptReply => 5,
            PartyMessage::Kick => 6,
            PartyMessage::KickReply => 7,
            PartyMessage::GroupUpdate => 8,
            PartyMessage::MemberUpdate => 9,
            PartyMessage::RenamePlayer => 11,
            PartyMessage::AnnounceEncounterReply => 13,
            PartyMessage::ChangingJob => 14,
            PartyMessage::ChangingJobUpdate => 15,
            PartyMessage::MapPing => 16,
            PartyMessage::InMinigame => 17,
        }
    }

    /// ORIGINAL: the client class that owns this sub-opcode.
    pub fn class_name(self) -> &'static str {
        match self {
            PartyMessage::Invite => "GroupPacketGroupInvite",
            PartyMessage::InviteReply => "GroupPacketGroupInviteReply",
            PartyMessage::Leave => "GroupPacketGroupLeave",
            PartyMessage::Accept => "GroupPacketGroupAccept",
            PartyMessage::AcceptReply => "GroupPacketGroupAcceptReply",
            PartyMessage::Kick => "GroupPacketGroupKick",
            PartyMessage::KickReply => "GroupPacketGroupKickReply",
            PartyMessage::GroupUpdate => "GroupPacketGroupUpdate",
            PartyMessage::MemberUpdate => "GroupPacketGroupMemberUpdate",
            PartyMessage::RenamePlayer => "GroupPacketRenamePlayer",
            PartyMessage::AnnounceEncounterReply => "GroupPacketAnnounceEncounterReply",
            PartyMessage::ChangingJob => "GroupChangingJobPacket",
            PartyMessage::ChangingJobUpdate => "GroupChangingJobUpdatePacket",
            PartyMessage::MapPing => "GroupMapPingPacket",
            PartyMessage::InMinigame => "GroupInMinigamePacket",
        }
    }

    /// ORIGINAL: reverse of [`PartyMessage::sub`]; `None` for the gaps the client never declares.
    pub fn from_sub(sub: i32) -> Option<PartyMessage> {
        PartyMessage::ALL.into_iter().find(|row| row.sub() == sub)
    }
}

/// Decoded envelope of one family-40 frame. The body stays opaque.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FamilyHeader<'a> {
    pub outer: u16,
    pub flag: u8,
    pub inner_length: u32,
    pub opcode: u16,
    pub sub: i32,
    pub message: PartyMessage,
    pub body: &'a [u8],
}

/// Every rejection the wire header or the reconstructed policy can produce.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PartyError {
    /// Envelope does not match `u16 | u8 | u32 | u16 | sub` framing.
    MalformedEnvelope,
    /// Envelope is well formed but belongs to another packet family.
    UnsupportedFamily,
    /// Family 40 sub-opcode the client does not declare.
    UnknownSubopcode,
    /// RECONSTRUCTED: capacity below two players.
    InvalidCapacity,
    UnknownParty,
    AlreadyInParty,
    NotPartyMember,
    NotLeader,
    SelfTarget,
    SelfInvite,
    AlreadyInvited,
    NoPendingInvite,
    PartyFull,
}

/// ORIGINAL: parse the shared envelope and the family-40 header; body bytes are preserved as-is.
pub fn parse_family_header(bytes: &[u8]) -> Result<FamilyHeader<'_>, PartyError> {
    if bytes.len() < ENVELOPE_PREFIX + GROUP_SUB_WIDTH {
        return Err(PartyError::MalformedEnvelope);
    }
    let outer = u16::from_le_bytes([bytes[0], bytes[1]]);
    if outer != 5 && outer != 6 {
        return Err(PartyError::MalformedEnvelope);
    }
    let flag = bytes[2];
    if flag > 1 {
        return Err(PartyError::MalformedEnvelope);
    }
    let inner_length = u32::from_le_bytes([bytes[3], bytes[4], bytes[5], bytes[6]]);
    if inner_length as usize != bytes.len() - ENVELOPE_PREFIX {
        return Err(PartyError::MalformedEnvelope);
    }
    let opcode = u16::from_le_bytes([bytes[7], bytes[8]]);
    if opcode != GROUP_FAMILY {
        return Err(PartyError::UnsupportedFamily);
    }
    let sub = i16::from_le_bytes([bytes[9], bytes[10]]) as i32;
    let message = PartyMessage::from_sub(sub).ok_or(PartyError::UnknownSubopcode)?;
    Ok(FamilyHeader {
        outer,
        flag,
        inner_length,
        opcode,
        sub,
        message,
        // `ENVELOPE_PREFIX` covers outer/flag/length; the 2-byte opcode then precedes the sub.
        body: &bytes[ENVELOPE_PREFIX + 2 + GROUP_SUB_WIDTH..],
    })
}

/// What a membership change did to the party.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PartyOutcome {
    /// The player was removed and the party survives under the same leader.
    Left,
    /// The player took the remaining members with them, or a kicked member left.
    LeaderChanged { new_leader: PlayerId },
    /// The party no longer exists.
    Disbanded,
    /// The player was not in a party; disconnect/leave on an absent player is a no-op.
    Absent,
}

#[derive(Debug)]
struct Party {
    leader: PlayerId,
    /// Join order; the oldest remaining member inherits leadership.
    members: Vec<PlayerId>,
}

#[derive(Debug)]
struct PendingInvite {
    party: u64,
    inviter: PlayerId,
}

/// RECONSTRUCTED: bounded party membership for the local server.
///
/// Invariants held after every call (see [`PartyRegistry::audit`]):
/// * every player in `membership` is listed exactly once in that party's `members`;
/// * every party has at least one member and its leader is one of them;
/// * a player holds at most one pending invite and cannot be invited while in a party;
/// * a pending invite never targets a member and never outlives its party.
#[derive(Debug, Default)]
pub struct PartyRegistry {
    capacity: u8,
    next_party_id: u64,
    parties: BTreeMap<u64, Party>,
    membership: BTreeMap<PlayerId, u64>,
    invites: BTreeMap<PlayerId, PendingInvite>,
}

impl PartyRegistry {
    /// RECONSTRUCTED: the caller supplies capacity because the original server cap is unrecovered.
    pub fn new(capacity: u8) -> Result<PartyRegistry, PartyError> {
        if capacity < 2 {
            return Err(PartyError::InvalidCapacity);
        }
        Ok(PartyRegistry {
            capacity,
            next_party_id: 1,
            ..PartyRegistry::default()
        })
    }

    pub fn capacity(&self) -> u8 {
        self.capacity
    }

    /// RECONSTRUCTED: creating a party drops any pending invite the founder was holding.
    pub fn create(&mut self, leader: PlayerId) -> Result<u64, PartyError> {
        if self.membership.contains_key(&leader) {
            return Err(PartyError::AlreadyInParty);
        }
        let id = self.next_party_id;
        self.next_party_id += 1;
        self.invites.remove(&leader);
        self.parties.insert(
            id,
            Party {
                leader,
                members: vec![leader],
            },
        );
        self.membership.insert(leader, id);
        Ok(id)
    }

    /// RECONSTRUCTED: any member may invite; open invites reserve capacity.
    pub fn invite(
        &mut self,
        inviter: PlayerId,
        party: u64,
        target: PlayerId,
    ) -> Result<(), PartyError> {
        let entry = self.parties.get(&party).ok_or(PartyError::UnknownParty)?;
        if !entry.members.contains(&inviter) {
            return Err(PartyError::NotPartyMember);
        }
        if target == inviter {
            return Err(PartyError::SelfInvite);
        }
        if self.membership.contains_key(&target) {
            return Err(PartyError::AlreadyInParty);
        }
        if self.invites.contains_key(&target) {
            return Err(PartyError::AlreadyInvited);
        }
        let reserved = entry.members.len() + self.open_invites(party) as usize;
        if reserved >= self.capacity as usize {
            return Err(PartyError::PartyFull);
        }
        self.invites
            .insert(target, PendingInvite { party, inviter });
        Ok(())
    }

    /// RECONSTRUCTED: accept consumes the invite and joins the party.
    pub fn accept(&mut self, player: PlayerId) -> Result<u64, PartyError> {
        if self.membership.contains_key(&player) {
            return Err(PartyError::AlreadyInParty);
        }
        let invite = self
            .invites
            .get(&player)
            .ok_or(PartyError::NoPendingInvite)?;
        let party = invite.party;
        let entry = self
            .parties
            .get_mut(&party)
            .ok_or(PartyError::UnknownParty)?;
        if entry.members.len() >= self.capacity as usize {
            return Err(PartyError::PartyFull);
        }
        self.invites.remove(&player);
        entry.members.push(player);
        self.membership.insert(player, party);
        Ok(party)
    }

    /// RECONSTRUCTED: declining frees the reserved slot.
    pub fn decline(&mut self, player: PlayerId) -> Result<(), PartyError> {
        self.invites
            .remove(&player)
            .map(|_| ())
            .ok_or(PartyError::NoPendingInvite)
    }

    /// RECONSTRUCTED: only the leader kicks, and never themselves (that is a leave).
    pub fn kick(&mut self, actor: PlayerId, target: PlayerId) -> Result<PartyOutcome, PartyError> {
        let party = self
            .membership
            .get(&actor)
            .copied()
            .ok_or(PartyError::NotPartyMember)?;
        let entry = self.parties.get(&party).ok_or(PartyError::UnknownParty)?;
        if entry.leader != actor {
            return Err(PartyError::NotLeader);
        }
        if actor == target {
            return Err(PartyError::SelfTarget);
        }
        if !entry.members.contains(&target) {
            return Err(PartyError::NotPartyMember);
        }
        self.remove_member(party, target)
    }

    /// RECONSTRUCTED: any member leaves; the leader hands over to the oldest remaining member.
    pub fn leave(&mut self, player: PlayerId) -> Result<PartyOutcome, PartyError> {
        let party = self
            .membership
            .get(&player)
            .copied()
            .ok_or(PartyError::NotPartyMember)?;
        self.remove_member(party, player)
    }

    /// RECONSTRUCTED: a disconnect also clears the player's own pending invite and is idempotent.
    pub fn disconnect(&mut self, player: PlayerId) -> Result<PartyOutcome, PartyError> {
        self.invites.remove(&player);
        match self.leave(player) {
            Ok(outcome) => Ok(outcome),
            Err(PartyError::NotPartyMember) => Ok(PartyOutcome::Absent),
            Err(other) => Err(other),
        }
    }

    pub fn party_of(&self, player: PlayerId) -> Option<u64> {
        self.membership.get(&player).copied()
    }

    pub fn leader_of(&self, party: u64) -> Option<PlayerId> {
        self.parties.get(&party).map(|entry| entry.leader)
    }

    pub fn members(&self, party: u64) -> Option<&[PlayerId]> {
        self.parties
            .get(&party)
            .map(|entry| entry.members.as_slice())
    }

    /// RECONSTRUCTED: `(party, inviter)` for the one open invite a player may hold.
    pub fn pending_for(&self, player: PlayerId) -> Option<(u64, PlayerId)> {
        self.invites
            .get(&player)
            .map(|invite| (invite.party, invite.inviter))
    }

    /// RECONSTRUCTED: open invites targeting a party, in player order.
    pub fn invites_of(&self, party: u64) -> Vec<PlayerId> {
        self.invites
            .iter()
            .filter(|(_, invite)| invite.party == party)
            .map(|(player, _)| *player)
            .collect()
    }

    /// Invariant check used by the tests after every membership change.
    pub fn audit(&self) -> Result<(), PartyError> {
        let mut seen: BTreeMap<PlayerId, u64> = BTreeMap::new();
        for (id, party) in &self.parties {
            if party.members.is_empty() || !party.members.contains(&party.leader) {
                return Err(PartyError::NotPartyMember);
            }
            for member in &party.members {
                if seen.insert(*member, *id).is_some() {
                    return Err(PartyError::AlreadyInParty);
                }
            }
        }
        if seen != self.membership {
            return Err(PartyError::NotPartyMember);
        }
        for (player, invite) in &self.invites {
            if !self.parties.contains_key(&invite.party) || seen.contains_key(player) {
                return Err(PartyError::NoPendingInvite);
            }
        }
        Ok(())
    }

    fn open_invites(&self, party: u64) -> u32 {
        self.invites
            .values()
            .filter(|invite| invite.party == party)
            .count() as u32
    }

    fn remove_member(&mut self, party: u64, player: PlayerId) -> Result<PartyOutcome, PartyError> {
        let entry = self
            .parties
            .get_mut(&party)
            .ok_or(PartyError::UnknownParty)?;
        let position = entry
            .members
            .iter()
            .position(|member| *member == player)
            .ok_or(PartyError::NotPartyMember)?;
        entry.members.remove(position);
        self.membership.remove(&player);
        if entry.members.is_empty() {
            self.parties.remove(&party);
            self.invites.retain(|_, invite| invite.party != party);
            return Ok(PartyOutcome::Disbanded);
        }
        if entry.leader == player {
            let new_leader = entry.members[0];
            entry.leader = new_leader;
            return Ok(PartyOutcome::LeaderChanged { new_leader });
        }
        Ok(PartyOutcome::Left)
    }
}
