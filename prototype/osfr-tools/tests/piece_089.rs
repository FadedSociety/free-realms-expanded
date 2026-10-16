//! Integration tests for piece 089 (party lifecycle).
//!
//! The module is included by `#[path]` until the lead registers it in `src/restoration`.
//! Every ORIGINAL check reads the shipped client's own constructor table; every membership
//! rule exercised below is explicitly RECONSTRUCTED local policy.

#[path = "../src/restoration/piece_089.rs"]
mod piece_089;

use piece_089::{
    GROUP_FAMILY, GROUP_FAMILY_FIELD, GROUP_SUB_WIDTH, GROUP_SUBOPCODE_COUNT, PartyError,
    PartyMessage, PartyOutcome, PartyRegistry, parse_family_header,
};

fn envelope(opcode: u16, sub: i32, body: &[u8]) -> Vec<u8> {
    let mut inner = Vec::new();
    inner.extend_from_slice(&opcode.to_le_bytes());
    inner.extend_from_slice(&(sub as i16).to_le_bytes());
    inner.extend_from_slice(body);
    let mut frame = Vec::new();
    frame.extend_from_slice(&5u16.to_le_bytes());
    frame.push(1);
    frame.extend_from_slice(&(inner.len() as u32).to_le_bytes());
    frame.extend_from_slice(&inner);
    frame
}

#[test]
fn subopcode_table_matches_client_constructors() {
    assert_eq!(GROUP_FAMILY, 40);
    assert_eq!(GROUP_FAMILY_FIELD, 0x28);
    assert_eq!(GROUP_SUB_WIDTH, 2);
    let rows = [
        (1, "GroupPacketGroupInvite"),
        (2, "GroupPacketGroupInviteReply"),
        (3, "GroupPacketGroupLeave"),
        (4, "GroupPacketGroupAccept"),
        (5, "GroupPacketGroupAcceptReply"),
        (6, "GroupPacketGroupKick"),
        (7, "GroupPacketGroupKickReply"),
        (8, "GroupPacketGroupUpdate"),
        (9, "GroupPacketGroupMemberUpdate"),
        (11, "GroupPacketRenamePlayer"),
        (13, "GroupPacketAnnounceEncounterReply"),
        (14, "GroupChangingJobPacket"),
        (15, "GroupChangingJobUpdatePacket"),
        (16, "GroupMapPingPacket"),
        (17, "GroupInMinigamePacket"),
    ];
    assert_eq!(GROUP_SUBOPCODE_COUNT, rows.len());
    for (sub, class_name) in rows {
        let message = PartyMessage::from_sub(sub).expect("recovered sub-opcode");
        assert_eq!(message.sub(), sub);
        assert_eq!(message.class_name(), class_name);
    }
    // The community enum stops at 13; the shipped client also declares 14..17.
    assert!(PartyMessage::from_sub(10).is_none());
    assert!(PartyMessage::from_sub(12).is_none());
    assert!(PartyMessage::from_sub(18).is_none());
}

#[test]
fn parses_invite_frame_header_and_keeps_body_opaque() {
    let frame = envelope(GROUP_FAMILY, 1, &[0xde, 0xad, 0xbe, 0xef]);
    let header = parse_family_header(&frame).expect("valid group envelope");
    assert_eq!(header.outer, 5);
    assert_eq!(header.flag, 1);
    assert_eq!(header.opcode, GROUP_FAMILY);
    assert_eq!(header.message, PartyMessage::Invite);
    // No body codec exists: the bytes are preserved, never interpreted.
    assert_eq!(header.body, &[0xde, 0xad, 0xbe, 0xef]);
}

#[test]
fn rejects_other_families_and_unknown_subopcodes() {
    let launch = envelope(175, 1, &[]);
    assert_eq!(
        parse_family_header(&launch).unwrap_err(),
        PartyError::UnsupportedFamily
    );
    let unknown = envelope(GROUP_FAMILY, 10, &[]);
    assert_eq!(
        parse_family_header(&unknown).unwrap_err(),
        PartyError::UnknownSubopcode
    );
}

#[test]
fn rejects_malformed_envelope() {
    let mut frame = envelope(GROUP_FAMILY, 1, &[]);
    let len = frame.len();
    frame[len - 1] = 0; // keep one stray byte: inner length no longer matches
    frame.push(0);
    assert_eq!(
        parse_family_header(&frame).unwrap_err(),
        PartyError::MalformedEnvelope
    );
    assert_eq!(
        parse_family_header(&[5, 0, 1]).unwrap_err(),
        PartyError::MalformedEnvelope
    );
}

#[test]
fn invite_accept_and_leader_membership() {
    let mut registry = PartyRegistry::new(4).expect("capacity");
    let party = registry.create(1).expect("create");
    assert_eq!(registry.leader_of(party), Some(1));
    assert_eq!(registry.members(party).unwrap(), &[1]);

    registry.invite(1, party, 2).expect("invite");
    assert_eq!(registry.pending_for(2), Some((party, 1)));
    // A pending invite is not membership.
    assert_eq!(registry.party_of(2), None);

    assert_eq!(registry.accept(2).expect("accept"), party);
    assert_eq!(registry.party_of(2), Some(party));
    assert_eq!(registry.members(party).unwrap(), &[1, 2]);
    assert_eq!(registry.leader_of(party), Some(1));
    registry.audit().expect("invariants");
}

#[test]
fn duplicate_accept_and_uninvited_accept_are_rejected() {
    let mut registry = PartyRegistry::new(4).expect("capacity");
    let party = registry.create(1).expect("create");
    registry.invite(1, party, 2).expect("invite");
    registry.accept(2).expect("accept");
    assert_eq!(registry.accept(2).unwrap_err(), PartyError::AlreadyInParty);
    assert_eq!(registry.accept(3).unwrap_err(), PartyError::NoPendingInvite);
    registry.audit().expect("invariants");
}

#[test]
fn invite_rejects_self_duplicate_and_cross_party_members() {
    let mut registry = PartyRegistry::new(4).expect("capacity");
    let first = registry.create(1).expect("create");
    let second = registry.create(9).expect("create");
    registry.invite(first, first, 2).expect("invite");

    assert_eq!(
        registry.invite(first, first, 1).unwrap_err(),
        PartyError::SelfInvite
    );
    assert_eq!(
        registry.invite(first, first, 2).unwrap_err(),
        PartyError::AlreadyInvited
    );
    assert_eq!(
        registry.invite(first, first, 9).unwrap_err(),
        PartyError::AlreadyInParty
    );
    assert_eq!(
        registry.invite(1, second, 3).unwrap_err(),
        PartyError::NotPartyMember
    );
    assert_eq!(
        registry.invite(first, 404, 3).unwrap_err(),
        PartyError::UnknownParty
    );
    registry.audit().expect("invariants");
}

#[test]
fn capacity_counts_members_and_open_invites() {
    let mut registry = PartyRegistry::new(2).expect("capacity");
    assert_eq!(registry.capacity(), 2);
    let party = registry.create(1).expect("create");
    registry.invite(1, party, 2).expect("invite");
    assert_eq!(registry.invites_of(party), vec![2]);
    // The open invite already reserves the second slot.
    assert_eq!(
        registry.invite(1, party, 3).unwrap_err(),
        PartyError::PartyFull
    );
    registry.accept(2).expect("accept");
    assert!(registry.invites_of(party).is_empty());
    assert_eq!(
        registry.invite(2, party, 3).unwrap_err(),
        PartyError::PartyFull
    );
    registry.audit().expect("invariants");
}

#[test]
fn decline_releases_the_slot() {
    let mut registry = PartyRegistry::new(2).expect("capacity");
    let party = registry.create(1).expect("create");
    registry.invite(1, party, 2).expect("invite");
    registry.decline(2).expect("decline");
    assert_eq!(registry.pending_for(2), None);
    registry.invite(1, party, 3).expect("invite after decline");
    assert_eq!(
        registry.decline(4).unwrap_err(),
        PartyError::NoPendingInvite
    );
    registry.audit().expect("invariants");
}

#[test]
fn only_the_leader_kicks_and_never_self() {
    let mut registry = PartyRegistry::new(4).expect("capacity");
    let party = registry.create(1).expect("create");
    registry.invite(1, party, 2).expect("invite");
    registry.accept(2).expect("accept");
    registry.invite(1, party, 3).expect("invite");
    registry.accept(3).expect("accept");

    assert_eq!(registry.kick(2, 3).unwrap_err(), PartyError::NotLeader);
    assert_eq!(registry.kick(1, 1).unwrap_err(), PartyError::SelfTarget);
    assert_eq!(registry.kick(1, 8).unwrap_err(), PartyError::NotPartyMember);
    assert_eq!(registry.kick(1, 3).expect("kick"), PartyOutcome::Left);
    assert_eq!(registry.party_of(3), None);
    assert_eq!(registry.members(party).unwrap(), &[1, 2]);
    registry.audit().expect("invariants");
}

#[test]
fn leaving_member_is_not_re_addable_while_invited_elsewhere() {
    let mut registry = PartyRegistry::new(4).expect("capacity");
    let party = registry.create(1).expect("create");
    registry.invite(1, party, 2).expect("invite");
    registry.accept(2).expect("accept");
    assert_eq!(registry.leave(2).expect("leave"), PartyOutcome::Left);
    assert_eq!(registry.party_of(2), None);
    registry.invite(1, party, 2).expect("re-invite");
    assert_eq!(registry.accept(2).expect("accept"), party);
    registry.audit().expect("invariants");
}

#[test]
fn leader_leave_transfers_to_the_oldest_member() {
    let mut registry = PartyRegistry::new(4).expect("capacity");
    let party = registry.create(1).expect("create");
    for target in [2, 3] {
        registry.invite(1, party, target).expect("invite");
        registry.accept(target).expect("accept");
    }
    assert_eq!(
        registry.leave(1).expect("leader leave"),
        PartyOutcome::LeaderChanged { new_leader: 2 }
    );
    assert_eq!(registry.leader_of(party), Some(2));
    assert_eq!(registry.members(party).unwrap(), &[2, 3]);
    registry.audit().expect("invariants");
}

#[test]
fn last_member_leave_disbands_and_drops_open_invites() {
    let mut registry = PartyRegistry::new(4).expect("capacity");
    let party = registry.create(1).expect("create");
    registry.invite(1, party, 2).expect("invite");
    assert_eq!(registry.leave(1).expect("leave"), PartyOutcome::Disbanded);
    assert_eq!(registry.party_of(1), None);
    assert_eq!(registry.pending_for(2), None);
    assert_eq!(registry.leader_of(party), None);
    assert_eq!(registry.accept(2).unwrap_err(), PartyError::NoPendingInvite);
    assert_eq!(registry.leave(1).unwrap_err(), PartyError::NotPartyMember);
    registry.audit().expect("invariants");
}

#[test]
fn disconnect_transfers_leadership_then_disbands() {
    let mut registry = PartyRegistry::new(4).expect("capacity");
    let party = registry.create(1).expect("create");
    registry.invite(1, party, 2).expect("invite");
    registry.accept(2).expect("accept");
    assert_eq!(
        registry.disconnect(1).expect("leader disconnect"),
        PartyOutcome::LeaderChanged { new_leader: 2 }
    );
    assert_eq!(registry.leader_of(party), Some(2));
    assert_eq!(
        registry.disconnect(2).expect("last disconnect"),
        PartyOutcome::Disbanded
    );
    assert_eq!(registry.leader_of(party), None);
    // Disconnecting an uninvolved or already-absent player is idempotent.
    assert_eq!(
        registry.disconnect(2).expect("idempotent"),
        PartyOutcome::Absent
    );
    registry.audit().expect("invariants");
}

#[test]
fn capacity_must_allow_a_pair() {
    assert_eq!(
        PartyRegistry::new(1).unwrap_err(),
        PartyError::InvalidCapacity
    );
    assert_eq!(
        PartyRegistry::new(0).unwrap_err(),
        PartyError::InvalidCapacity
    );
}
