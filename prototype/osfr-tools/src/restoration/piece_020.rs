//! Multiplayer activity grouping: activity-launch membership for the local server.
//!
//! Piece 020, owned paths only. Evidence labels follow the other restoration modules:
//! ORIGINAL is read from the supplied client, the decompiled community server model, the pinned
//! community readers or original retail capture bytes; RECONSTRUCTED is a bounded local policy the
//! lead must replace with native evidence; MISSING is an explicit gap.
//!
//! ORIGINAL facts implemented here:
//! * the multiplayer grouping wire family is `ClientActivityLaunchBasePacket` = **175** (`0xaf`).
//!   The client constructor writes `0xaf` into the packet's opcode slot
//!   (`lab/decompiled/client/FreeRealms/pseudocode.c`, `FUN_009c0bc0` / `FUN_009c0c30`).
//! * every derived launch packet is `FUN_009c0c30(<sub>, a, b)` immediately before its
//!   `ClientActivityLaunchPacket*::vftable` store, so the sub-opcode table below is read from the
//!   shipped client, not translated from a comment. `UpdateMemberState` is 9 there; only the older
//!   community enum duplicates 8 for `RemoveMember` and `UpdateMemberState`.
//! * the launch sub-opcode is a 4-byte little-endian int on the wire; the same reader implementation
//!   uses a 2-byte sub-opcode for family 40/141 and a 1-byte sub-opcode for 39/147/167
//!   (`FreeRealms-Legacy/.../PacketReaderExtensions.cs`), which the original capture frames in
//!   `lab/reports/original-minigame-packets/` confirm for 39/141/167.
//! * the envelope is `u16 outer | u8 flag | u32 inner length | u16 opcode | sub | body`, where
//!   `inner length` counts from the opcode, byte-for-byte as the four capture frames above.
//! * the community corpus documents that groups are invite based and are what players use to play
//!   activities together (`freerealms-wiki` page "Groups").
//!
//! RECONSTRUCTED here (no original server code for launch membership was recovered):
//! * every membership rule: who may invite, ready gating, owner-only kick/launch/cancel, capacity
//!   accounting, what happens to a formation when the owner disconnects, and how an instance closes.
//! * the instance-id namespace seeded by [`ActivityGroupRegistry::new`]. Real instance ids are
//!   server assigned and unrecovered.
//!
//! MISSING / explicit gaps:
//! * no captured 175 traffic and no community serializer exist, so the per-field payload body of the
//!   launch messages is opaque here. This module deliberately does not write one.
//! * the matchmaking family (141) and the activity catalogue (147/167) are known but not modelled;
//!   their state machines belong to the catalogue and queue pieces, not to launch membership.
//! * no reward, currency, item or progression effect is modelled. Grouping grants nothing.
//!
//! ponytail: one map of groups, one map of instance rosters, one validation pass per request. No
//! trait objects, no packet writer for an unrecovered body, no tick loop. Add the launch body codec
//! only when a captured 175 frame or an original serializer exists.

use std::collections::BTreeMap;

/// ORIGINAL: `ClientActivityLaunchBasePacket` in the supplied client and the pinned packet enums.
pub const ACTIVITY_LAUNCH_FAMILY: u16 = 175;
/// ORIGINAL: the same family as the client stores it (`param_1[1] = 0xaf`).
pub const ACTIVITY_LAUNCH_FAMILY_FIELD: i32 = 0xaf;
/// ORIGINAL: `BaseMiniGamePacket`; a 1-byte sub-opcode, capture-confirmed.
pub const MINIGAME_FAMILY: u16 = 39;
/// ORIGINAL: `BaseGroupPacket`; a 2-byte sub-opcode (`GroupInMinigamePacket` = 17 lives here).
pub const GROUP_FAMILY: u16 = 40;
/// ORIGINAL: `BaseMatchmakingPacket`; a 2-byte sub-opcode, capture-confirmed for `ListQueues*`.
pub const MATCHMAKING_FAMILY: u16 = 141;
/// ORIGINAL: `ActivityManagerBasePacket`; a 1-byte sub-opcode.
pub const ACTIVITY_MANAGER_FAMILY: u16 = 147;
/// ORIGINAL: `BaseActivityServicePacket`; a 1-byte sub-opcode, capture-confirmed.
pub const ACTIVITY_SERVICE_FAMILY: u16 = 167;
/// ORIGINAL: `BaseActivityServicePacket.BaseActivityPacket` = 1 (its own sub-opcode follows).
pub const ACTIVITY_SERVICE_ACTIVITY_PACKET: u8 = 1;

const ENVELOPE_PREFIX: usize = 7;

/// How a fact in this module is supported.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Evidence {
    /// Read from the supplied client, the pinned community sources, or original capture bytes.
    Original,
    /// A bounded local rule with no recovered original.
    Reconstructed,
    /// Known to be missing; never presented as implemented.
    Missing,
}

/// ORIGINAL: sub-opcodes of `ClientActivityLaunchBasePacket` (family 175).
///
/// The numbers are read from the client constructors that call `FUN_009c0c30(<sub>, a, b)`;
/// `class_name` is the class the same constructor instantiates. The older community enum
/// (`OpenSourceFreeRealms/.../PacketCodes.cs`) agrees on every row except that it duplicates 8.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LaunchMessage {
    InviteDetails,
    InviteMemberReply,
    InviteMemberResult,
    PlayerHasBeenAccepted,
    ActivityLaunched,
    ActivityCanceled,
    AddMember,
    RemoveMember,
    UpdateMemberState,
    KickMemberEvent,
    OnMatchmakingStart,
    OnMatchmakingLaunch,
    OnMatchmakingCancel,
    InviteResponse,
    MemberReady,
    MemberQuit,
    InviteMemberRequest,
    KickMemberRequest,
    OwnerLaunchRequest,
    OwnerMatchmakingRequest,
}

impl LaunchMessage {
    /// Every recovered sub-opcode, in ascending order.
    pub const ALL: [LaunchMessage; 20] = [
        LaunchMessage::InviteDetails,
        LaunchMessage::InviteMemberReply,
        LaunchMessage::InviteMemberResult,
        LaunchMessage::PlayerHasBeenAccepted,
        LaunchMessage::ActivityLaunched,
        LaunchMessage::ActivityCanceled,
        LaunchMessage::AddMember,
        LaunchMessage::RemoveMember,
        LaunchMessage::UpdateMemberState,
        LaunchMessage::KickMemberEvent,
        LaunchMessage::OnMatchmakingStart,
        LaunchMessage::OnMatchmakingLaunch,
        LaunchMessage::OnMatchmakingCancel,
        LaunchMessage::InviteResponse,
        LaunchMessage::MemberReady,
        LaunchMessage::MemberQuit,
        LaunchMessage::InviteMemberRequest,
        LaunchMessage::KickMemberRequest,
        LaunchMessage::OwnerLaunchRequest,
        LaunchMessage::OwnerMatchmakingRequest,
    ];

    pub fn sub_opcode(self) -> i32 {
        match self {
            LaunchMessage::InviteDetails => 1,
            LaunchMessage::InviteMemberReply => 2,
            LaunchMessage::InviteMemberResult => 3,
            LaunchMessage::PlayerHasBeenAccepted => 4,
            LaunchMessage::ActivityLaunched => 5,
            LaunchMessage::ActivityCanceled => 6,
            LaunchMessage::AddMember => 7,
            LaunchMessage::RemoveMember => 8,
            LaunchMessage::UpdateMemberState => 9,
            LaunchMessage::KickMemberEvent => 13,
            LaunchMessage::OnMatchmakingStart => 14,
            LaunchMessage::OnMatchmakingLaunch => 15,
            LaunchMessage::OnMatchmakingCancel => 16,
            LaunchMessage::InviteResponse => 17,
            LaunchMessage::MemberReady => 18,
            LaunchMessage::MemberQuit => 19,
            LaunchMessage::InviteMemberRequest => 20,
            LaunchMessage::KickMemberRequest => 21,
            LaunchMessage::OwnerLaunchRequest => 22,
            LaunchMessage::OwnerMatchmakingRequest => 23,
        }
    }

    /// The client class that carries this sub-opcode, as it is named in the supplied binary.
    pub fn class_name(self) -> &'static str {
        match self {
            LaunchMessage::InviteDetails => "ClientActivityLaunchPacketInviteDetails",
            LaunchMessage::InviteMemberReply => "ClientActivityLaunchPacketInviteMemberReply",
            LaunchMessage::InviteMemberResult => "ClientActivityLaunchPacketInviteMemberResult",
            LaunchMessage::PlayerHasBeenAccepted => {
                "ClientActivityLaunchPacketPlayerHasBeenAccepted"
            }
            LaunchMessage::ActivityLaunched => "ClientActivityLaunchPacketActivityLaunched",
            LaunchMessage::ActivityCanceled => "ClientActivityLaunchPacketActivityCanceled",
            LaunchMessage::AddMember => "ClientActivityLaunchPacketAddMember",
            LaunchMessage::RemoveMember => "ClientActivityLaunchPacketRemoveMember",
            LaunchMessage::UpdateMemberState => "ClientActivityLaunchPacketUpdateMemberState",
            LaunchMessage::KickMemberEvent => "ClientActivityLaunchPacketKickMemberEvent",
            LaunchMessage::OnMatchmakingStart => "ClientActivityLaunchPacketOnMatchmakingStart",
            LaunchMessage::OnMatchmakingLaunch => "ClientActivityLaunchPacketOnMatchmakingLaunch",
            LaunchMessage::OnMatchmakingCancel => "ClientActivityLaunchPacketOnMatchmakingCancel",
            LaunchMessage::InviteResponse => "ClientActivityLaunchPacketInviteResponse",
            LaunchMessage::MemberReady => "ClientActivityLaunchPacketMemberReady",
            LaunchMessage::MemberQuit => "ClientActivityLaunchPacketMemberQuit",
            LaunchMessage::InviteMemberRequest => "ClientActivityLaunchPacketInviteMemberRequest",
            LaunchMessage::KickMemberRequest => "ClientActivityLaunchPacketKickMemberRequest",
            LaunchMessage::OwnerLaunchRequest => "ClientActivityLaunchPacketOwnerLaunchRequest",
            LaunchMessage::OwnerMatchmakingRequest => {
                "ClientActivityLaunchPacketOwnerMatchmakingRequest"
            }
        }
    }

    pub fn evidence(self) -> Evidence {
        Evidence::Original
    }

    pub fn from_sub(sub: i32) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|message| message.sub_opcode() == sub)
    }
}

/// ORIGINAL: the sub-opcode width, in bytes, of a grouping family.
///
/// `None` means the family is not part of the recovered grouping surface.
pub fn family_sub_width(opcode: u16) -> Option<usize> {
    match opcode {
        MINIGAME_FAMILY | ACTIVITY_MANAGER_FAMILY | ACTIVITY_SERVICE_FAMILY => Some(1),
        GROUP_FAMILY | MATCHMAKING_FAMILY => Some(2),
        ACTIVITY_LAUNCH_FAMILY => Some(4),
        _ => None,
    }
}

/// A parsed grouping envelope. `body` is deliberately opaque: no original serializer for the
/// activity-launch payload was recovered, so this module never invents its fields.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FamilyHeader {
    /// `u16` outer discriminator; every original capture frame uses 5 or 6.
    pub outer: u16,
    /// Opaque tunnel flag; every original capture frame uses 0 or 1.
    pub flag: u8,
    /// `u32` inner length, counted from the family opcode exactly as the captures write it.
    pub inner_length: u32,
    /// Family opcode (`39`, `40`, `141`, `147`, `167`, `175`).
    pub opcode: u16,
    /// Sub-opcode, widened from 1, 2 or 4 little-endian bytes per [`family_sub_width`].
    pub sub: i32,
    /// Everything after the sub-opcode.
    pub body: Vec<u8>,
}

/// Parse the recovered grouping envelope, rejecting anything that does not match it exactly.
pub fn parse_family_header(bytes: &[u8]) -> Result<FamilyHeader, GroupError> {
    if bytes.len() < ENVELOPE_PREFIX + 2 {
        return Err(GroupError::MalformedEnvelope);
    }
    let outer = u16::from_le_bytes([bytes[0], bytes[1]]);
    if outer != 5 && outer != 6 {
        return Err(GroupError::MalformedEnvelope);
    }
    let flag = bytes[2];
    if flag > 1 {
        return Err(GroupError::MalformedEnvelope);
    }
    let inner_length = u32::from_le_bytes([bytes[3], bytes[4], bytes[5], bytes[6]]);
    if inner_length as usize != bytes.len() - ENVELOPE_PREFIX {
        return Err(GroupError::MalformedEnvelope);
    }
    let opcode = u16::from_le_bytes([bytes[7], bytes[8]]);
    let width = family_sub_width(opcode).ok_or(GroupError::UnsupportedFamily)?;
    let start = ENVELOPE_PREFIX + 2;
    if bytes.len() < start + width {
        return Err(GroupError::MalformedEnvelope);
    }
    let sub = match width {
        1 => bytes[start] as i32,
        2 => i16::from_le_bytes([bytes[start], bytes[start + 1]]) as i32,
        4 => i32::from_le_bytes([
            bytes[start],
            bytes[start + 1],
            bytes[start + 2],
            bytes[start + 3],
        ]),
        _ => return Err(GroupError::UnsupportedFamily),
    };
    Ok(FamilyHeader {
        outer,
        flag,
        inner_length,
        opcode,
        sub,
        body: bytes[start + width..].to_vec(),
    })
}

/// A character guid. The wire uses 64-bit character guids throughout the original captures.
pub type PlayerId = u64;

/// RECONSTRUCTED membership states. `AddMember`/`RemoveMember`/`UpdateMemberState` prove that a
/// member carries a state on the wire; the exact original state vocabulary was not recovered.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MemberState {
    /// Invited, not yet accepted.
    Invited,
    /// In the launch group.
    Accepted,
    /// In the launch group and ready for the owner to launch.
    Ready,
}

/// RECONSTRUCTED phase. `ActivityLaunched` (5) and `ActivityCanceled` (6) are the recovered ends of
/// a launch; a formed group becomes exactly one instance.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GroupPhase {
    Forming,
    Launched { instance_id: i32 },
}

/// One member of a launch group.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Member {
    pub player: PlayerId,
    pub state: MemberState,
}

/// One launch group for one activity id.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActivityGroup {
    pub activity_id: i32,
    pub owner: PlayerId,
    /// RECONSTRUCTED: the original cap per activity was not recovered, so it is supplied by the
    /// caller and enforced for every member, invited or accepted.
    pub capacity: usize,
    pub phase: GroupPhase,
    pub members: Vec<Member>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct InstanceRoster {
    activity_id: i32,
    members: Vec<PlayerId>,
}

/// What the caller should tell the clients after a request.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Reply {
    GroupOpened {
        activity_id: i32,
        owner: PlayerId,
    },
    InviteSent {
        activity_id: i32,
        target: PlayerId,
    },
    InviteDeclined {
        activity_id: i32,
        player: PlayerId,
    },
    MemberAdded {
        activity_id: i32,
        player: PlayerId,
    },
    MemberStateUpdated {
        activity_id: i32,
        player: PlayerId,
        state: MemberState,
    },
    MemberRemoved {
        activity_id: i32,
        player: PlayerId,
        kicked: bool,
    },
    Launched {
        activity_id: i32,
        instance_id: i32,
        members: Vec<PlayerId>,
    },
    Canceled {
        activity_id: i32,
    },
    MemberDisconnected {
        activity_id: i32,
        player: PlayerId,
        instance_id: Option<i32>,
    },
    InstanceClosed {
        activity_id: i32,
        instance_id: i32,
    },
}

/// Invalid-membership and malformed-wire rejections. Every one of these is a refusal, never a
/// silent success or a fabricated state change.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GroupError {
    /// The envelope or a sub-opcode is not the recovered shape.
    MalformedEnvelope,
    /// The family opcode is not part of the recovered grouping surface.
    UnsupportedFamily,
    /// No group exists for that activity id.
    UnknownActivity,
    /// A group already exists for that activity id.
    AlreadyOpen,
    /// The sender is not a member of that group.
    NotAMember,
    /// The member exists but has not accepted the invitation yet.
    NotAccepted,
    /// The target was already invited.
    AlreadyInvited,
    /// The target is already in the group.
    AlreadyMember,
    /// The group is at capacity.
    GroupFull,
    /// Only the owner may do that.
    NotOwner,
    /// The owner cannot be kicked out of their own launch.
    CannotKickOwner,
    /// At least one accepted member is not ready.
    NotReady,
    /// At least one invited member has not answered yet.
    PendingInvite,
    /// The group already launched; membership changes must go through the instance.
    AlreadyLaunched,
    /// No instance exists with that id.
    UnknownInstance,
    /// The sender is not a member of that instance.
    NotInInstance,
    /// The reconstructed instance-id namespace is exhausted.
    InstanceSpaceExhausted,
}

/// A bounded, evidence-labelled activity-grouping state machine.
///
/// RECONSTRUCTED policy summary: a group is opened by its owner; any member may invite; the invitee
/// accepts or declines; the owner launches once every invited member has answered and every member
/// is ready; the owner alone kicks, launches and cancels; a launched instance keeps a roster and
/// closes when its last member leaves; the owner disconnecting before launch cancels the formation.
#[derive(Clone, Debug)]
pub struct ActivityGroupRegistry {
    groups: BTreeMap<i32, ActivityGroup>,
    instances: BTreeMap<i32, InstanceRoster>,
    next_instance_id: i32,
}

impl ActivityGroupRegistry {
    /// `first_instance_id` seeds the RECONSTRUCTED local instance namespace.
    pub fn new(first_instance_id: i32) -> Self {
        Self {
            groups: BTreeMap::new(),
            instances: BTreeMap::new(),
            next_instance_id: first_instance_id,
        }
    }

    /// The owner forms a launch group for one activity.
    pub fn open(
        &mut self,
        activity_id: i32,
        owner: PlayerId,
        capacity: usize,
    ) -> Result<Reply, GroupError> {
        if self.groups.contains_key(&activity_id) {
            return Err(GroupError::AlreadyOpen);
        }
        self.groups.insert(
            activity_id,
            ActivityGroup {
                activity_id,
                owner,
                capacity,
                phase: GroupPhase::Forming,
                members: vec![Member {
                    player: owner,
                    state: MemberState::Accepted,
                }],
            },
        );
        Ok(Reply::GroupOpened { activity_id, owner })
    }

    /// Invite another player. Any current member may invite; the group never exceeds its capacity.
    pub fn invite(
        &mut self,
        activity_id: i32,
        actor: PlayerId,
        target: PlayerId,
    ) -> Result<Reply, GroupError> {
        let group = self.group_mut(activity_id)?;
        if group.phase != GroupPhase::Forming {
            return Err(GroupError::AlreadyLaunched);
        }
        if index_of(group, actor).is_none() {
            return Err(GroupError::NotAMember);
        }
        match index_of(group, target).map(|index| group.members[index].state) {
            Some(MemberState::Invited) => return Err(GroupError::AlreadyInvited),
            Some(_) => return Err(GroupError::AlreadyMember),
            None => {}
        }
        if group.members.len() >= group.capacity {
            return Err(GroupError::GroupFull);
        }
        group.members.push(Member {
            player: target,
            state: MemberState::Invited,
        });
        Ok(Reply::InviteSent {
            activity_id,
            target,
        })
    }

    /// The invitee accepts and becomes a member of the launch group.
    pub fn accept(&mut self, activity_id: i32, player: PlayerId) -> Result<Reply, GroupError> {
        let group = self.group_mut(activity_id)?;
        if group.phase != GroupPhase::Forming {
            return Err(GroupError::AlreadyLaunched);
        }
        let Some(index) = index_of(group, player) else {
            return Err(GroupError::NotAMember);
        };
        match group.members[index].state {
            MemberState::Invited => {
                group.members[index].state = MemberState::Accepted;
                Ok(Reply::MemberAdded {
                    activity_id,
                    player,
                })
            }
            _ => Err(GroupError::AlreadyMember),
        }
    }

    /// The invitee declines; the invitation is dropped.
    pub fn decline(&mut self, activity_id: i32, player: PlayerId) -> Result<Reply, GroupError> {
        let group = self.group_mut(activity_id)?;
        if group.phase != GroupPhase::Forming {
            return Err(GroupError::AlreadyLaunched);
        }
        let Some(index) = index_of(group, player) else {
            return Err(GroupError::NotAMember);
        };
        if group.members[index].state != MemberState::Invited {
            return Err(GroupError::AlreadyMember);
        }
        group.members.remove(index);
        Ok(Reply::InviteDeclined {
            activity_id,
            player,
        })
    }

    /// Mark a member ready (or not ready). RECONSTRUCTED: allowed while forming and after launch,
    /// because `UpdateMemberState` is a live state message, but it never changes membership.
    pub fn set_ready(
        &mut self,
        activity_id: i32,
        player: PlayerId,
        ready: bool,
    ) -> Result<Reply, GroupError> {
        let group = self.group_mut(activity_id)?;
        let Some(index) = index_of(group, player) else {
            return Err(GroupError::NotAMember);
        };
        let state = match (group.members[index].state, ready) {
            (MemberState::Invited, _) => return Err(GroupError::NotAccepted),
            (_, true) => MemberState::Ready,
            (_, false) => MemberState::Accepted,
        };
        group.members[index].state = state;
        Ok(Reply::MemberStateUpdated {
            activity_id,
            player,
            state,
        })
    }

    /// A member leaves the group. The owner leaving cancels the formation.
    pub fn quit(&mut self, activity_id: i32, player: PlayerId) -> Result<Reply, GroupError> {
        let (phase, owner, index) = {
            let group = self.group(activity_id)?;
            (
                group.phase,
                group.owner,
                index_of(group, player).ok_or(GroupError::NotAMember)?,
            )
        };
        if phase != GroupPhase::Forming {
            return Err(GroupError::AlreadyLaunched);
        }
        if player == owner {
            self.groups.remove(&activity_id);
            return Ok(Reply::Canceled { activity_id });
        }
        let group = self.group_mut(activity_id)?;
        group.members.remove(index);
        if group.members.is_empty() {
            self.groups.remove(&activity_id);
        }
        Ok(Reply::MemberRemoved {
            activity_id,
            player,
            kicked: false,
        })
    }

    /// The owner removes another member.
    pub fn kick(
        &mut self,
        activity_id: i32,
        actor: PlayerId,
        target: PlayerId,
    ) -> Result<Reply, GroupError> {
        let group = self.group_mut(activity_id)?;
        if group.phase != GroupPhase::Forming {
            return Err(GroupError::AlreadyLaunched);
        }
        if index_of(group, actor).is_none() {
            return Err(GroupError::NotAMember);
        }
        if actor != group.owner {
            return Err(GroupError::NotOwner);
        }
        if target == group.owner {
            return Err(GroupError::CannotKickOwner);
        }
        let Some(index) = index_of(group, target) else {
            return Err(GroupError::NotAMember);
        };
        group.members.remove(index);
        Ok(Reply::MemberRemoved {
            activity_id,
            player: target,
            kicked: true,
        })
    }

    /// The owner launches: every invite must be answered and every member ready.
    pub fn launch(&mut self, activity_id: i32, actor: PlayerId) -> Result<Reply, GroupError> {
        let instance_id = self.next_instance_id;
        let next_id = instance_id
            .checked_add(1)
            .ok_or(GroupError::InstanceSpaceExhausted)?;
        let members = {
            let group = self.group_mut(activity_id)?;
            if group.phase != GroupPhase::Forming {
                return Err(GroupError::AlreadyLaunched);
            }
            if index_of(group, actor).is_none() {
                return Err(GroupError::NotAMember);
            }
            if actor != group.owner {
                return Err(GroupError::NotOwner);
            }
            if group
                .members
                .iter()
                .any(|member| member.state == MemberState::Invited)
            {
                return Err(GroupError::PendingInvite);
            }
            if group
                .members
                .iter()
                .any(|member| member.state != MemberState::Ready)
            {
                return Err(GroupError::NotReady);
            }
            let members: Vec<PlayerId> = group.members.iter().map(|member| member.player).collect();
            group.phase = GroupPhase::Launched { instance_id };
            members
        };
        self.next_instance_id = next_id;
        self.instances.insert(
            instance_id,
            InstanceRoster {
                activity_id,
                members: members.clone(),
            },
        );
        Ok(Reply::Launched {
            activity_id,
            instance_id,
            members,
        })
    }

    /// The owner cancels a formation before it launches.
    pub fn cancel(&mut self, activity_id: i32, actor: PlayerId) -> Result<Reply, GroupError> {
        let (phase, owner, is_member) = {
            let group = self.group(activity_id)?;
            (group.phase, group.owner, index_of(group, actor).is_some())
        };
        if phase != GroupPhase::Forming {
            return Err(GroupError::AlreadyLaunched);
        }
        if !is_member {
            return Err(GroupError::NotAMember);
        }
        if actor != owner {
            return Err(GroupError::NotOwner);
        }
        self.groups.remove(&activity_id);
        Ok(Reply::Canceled { activity_id })
    }

    /// Drop a disconnected player from every group and instance they belong to.
    ///
    /// RECONSTRUCTED: before launch the owner's disconnect cancels the formation; after launch a
    /// disconnect only leaves the instance roster, and the instance closes with its last member.
    pub fn disconnect(&mut self, player: PlayerId) -> Vec<Reply> {
        let activity_ids: Vec<i32> = self
            .groups
            .iter()
            .filter(|(_, group)| index_of(group, player).is_some())
            .map(|(activity_id, _)| *activity_id)
            .collect();
        let mut replies = Vec::new();
        for activity_id in activity_ids {
            let Some((phase, is_owner)) = self
                .groups
                .get(&activity_id)
                .map(|group| (group.phase, group.owner == player))
            else {
                continue;
            };
            match phase {
                GroupPhase::Launched { instance_id } => {
                    replies.push(Reply::MemberDisconnected {
                        activity_id,
                        player,
                        instance_id: Some(instance_id),
                    });
                    let mut emptied = false;
                    if let Some(roster) = self.instances.get_mut(&instance_id) {
                        roster.members.retain(|member| *member != player);
                        emptied = roster.members.is_empty();
                    }
                    if emptied {
                        self.instances.remove(&instance_id);
                        replies.push(Reply::InstanceClosed {
                            activity_id,
                            instance_id,
                        });
                    }
                    let group = self
                        .groups
                        .get_mut(&activity_id)
                        .expect("group checked above");
                    group.members.retain(|member| member.player != player);
                    if group.members.is_empty() {
                        self.groups.remove(&activity_id);
                    }
                }
                GroupPhase::Forming => {
                    replies.push(Reply::MemberDisconnected {
                        activity_id,
                        player,
                        instance_id: None,
                    });
                    if is_owner {
                        self.groups.remove(&activity_id);
                        replies.push(Reply::Canceled { activity_id });
                        continue;
                    }
                    let group = self
                        .groups
                        .get_mut(&activity_id)
                        .expect("group checked above");
                    group.members.retain(|member| member.player != player);
                    if group.members.is_empty() {
                        self.groups.remove(&activity_id);
                    }
                }
            }
        }
        replies
    }

    /// The state of one player in one group.
    pub fn member_state(&self, activity_id: i32, player: PlayerId) -> Option<MemberState> {
        let group = self.groups.get(&activity_id)?;
        index_of(group, player).map(|index| group.members[index].state)
    }

    /// The full roster of one group, in join order.
    pub fn members(&self, activity_id: i32) -> Result<Vec<PlayerId>, GroupError> {
        Ok(self
            .group(activity_id)?
            .members
            .iter()
            .map(|member| member.player)
            .collect())
    }

    pub fn phase(&self, activity_id: i32) -> Option<GroupPhase> {
        self.groups.get(&activity_id).map(|group| group.phase)
    }

    /// The instance roster, live for as long as the instance has a member.
    pub fn instance_members(&self, instance_id: i32) -> Option<&[PlayerId]> {
        self.instances
            .get(&instance_id)
            .map(|roster| roster.members.as_slice())
    }

    /// Which instance a player currently belongs to.
    pub fn instance_of(&self, player: PlayerId) -> Option<i32> {
        self.instances
            .iter()
            .find(|(_, roster)| roster.members.contains(&player))
            .map(|(instance_id, _)| *instance_id)
    }

    /// The membership check the minigame families must run before trusting an instance id.
    pub fn authorize_instance(&self, instance_id: i32, player: PlayerId) -> Result<(), GroupError> {
        let roster = self
            .instances
            .get(&instance_id)
            .ok_or(GroupError::UnknownInstance)?;
        if roster.members.contains(&player) {
            Ok(())
        } else {
            Err(GroupError::NotInInstance)
        }
    }

    /// The activity id an instance was created from.
    pub fn instance_activity(&self, instance_id: i32) -> Option<i32> {
        self.instances
            .get(&instance_id)
            .map(|roster| roster.activity_id)
    }

    /// A stable JSON snapshot in the same shape as the other restoration modules.
    pub fn state(&self) -> serde_json::Value {
        let groups: Vec<serde_json::Value> = self
            .groups
            .values()
            .map(|group| {
                let members: Vec<serde_json::Value> = group
                    .members
                    .iter()
                    .map(|member| {
                        serde_json::json!({
                            "player": member.player,
                            "state": match member.state {
                                MemberState::Invited => "invited",
                                MemberState::Accepted => "accepted",
                                MemberState::Ready => "ready",
                            },
                        })
                    })
                    .collect();
                serde_json::json!({
                    "activity_id": group.activity_id,
                    "owner": group.owner,
                    "capacity": group.capacity,
                    "phase": match group.phase {
                        GroupPhase::Forming => "forming",
                        GroupPhase::Launched { .. } => "launched",
                    },
                    "instance_id": match group.phase {
                        GroupPhase::Forming => serde_json::Value::Null,
                        GroupPhase::Launched { instance_id } => serde_json::json!(instance_id),
                    },
                    "members": members,
                })
            })
            .collect();
        let instances: Vec<serde_json::Value> = self
            .instances
            .iter()
            .map(|(instance_id, roster)| {
                serde_json::json!({
                    "id": instance_id,
                    "activity_id": roster.activity_id,
                    "members": roster.members,
                })
            })
            .collect();
        serde_json::json!({ "groups": groups, "instances": instances })
    }

    fn group(&self, activity_id: i32) -> Result<&ActivityGroup, GroupError> {
        self.groups
            .get(&activity_id)
            .ok_or(GroupError::UnknownActivity)
    }

    fn group_mut(&mut self, activity_id: i32) -> Result<&mut ActivityGroup, GroupError> {
        self.groups
            .get_mut(&activity_id)
            .ok_or(GroupError::UnknownActivity)
    }
}

fn index_of(group: &ActivityGroup, player: PlayerId) -> Option<usize> {
    group
        .members
        .iter()
        .position(|member| member.player == player)
}
