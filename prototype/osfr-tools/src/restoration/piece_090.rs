//! Work item 090: guild lifecycle - recovered protocol plus a bounded create/member/role model.
//!
//! # Recovered original facts
//!
//! * Families and ids. `lab/decompiled/server/LibSOE/SOE/BasePackets.cs` numbers the tunneled inner
//!   families `BaseGuildPacket = 129` and `BaseAdminGuildPacket = 131`; `BaseGuildPackets.cs` lists
//!   the 28 guild sub-packets below and `BaseAdminGuildPackets.cs` lists
//!   `AdminGuildServerVersionPacket = 41`.
//! * Client confirmation. In `lab/decompiled/client/FreeRealms/pseudocode.c` the native
//!   `BaseGuildPacket` constructor writes `param_1[1] = 0x81`, `BaseAdminGuildPacket` writes `0x83`,
//!   and all 28 guild classes own a constructor calling `FUN_00a09b70(<sub>)` with the same id the
//!   server enum uses. The shipped client binary therefore confirms the whole id table, not just the
//!   server's copy of it.
//! * Client schema. `lab/decompiled/client-ui/ScriptsBase.lua` binds `BaseClient.Guilds` to
//!   (GuildGuid, Name, MessageOfTheDay, CanCreateGuild) and `BaseClient.GuildMembers` to 15 columns;
//!   the client's own column-name tables in the same binary agree (`Guild Guid`, `Name`, `MOTD`,
//!   `CanCreateGuild?`, `CanRenameGuild?` and `Guid`..`Is Friend`).
//! * Wire framing. Guild packets are tunneled SOE payloads: a little-endian `u16` main opcode, a
//!   little-endian `u16` sub opcode, then the body. Strings are a little-endian `i32` UTF-8 byte
//!   length followed by the bytes (the original server's `AddHost*` writers and the pinned community
//!   `PacketWriter` agree on host order).
//!
//! # Community evidence (recorded, not original)
//!
//! The body field order for the 13 sub-packets the pinned AGPL-3.0 `lab/references/Sanctuary` server
//! implements, the four hardcoded role ids (1 Leader, 2 Officer, 3 Member, 4 Recruit), the `Recruit`
//! role handed to a joining member, the invite/promote permission split, the leader-succession
//! ordering (lowest role id, then lowest character id) and the 100-member default all come from that
//! server. `GuildMemberLocationRequest` (15) is *not* empty in the native client - its constructor
//! allocates a `SoeUtil::Array<StrongType<u64, SoeUtilSignatureCharacterGuid,0>,0,1>` member - so this
//! module preserves an unmodelled body instead of asserting the community shape.
//!
//! # Reconstructed policy (labelled, local)
//!
//! `GuildStore` is an offline model over already-known guild state. Its monotonic guild ids, the
//! caller-supplied invite deadline, the `GuildEvent` list and the "promote never mints a second
//! Leader" rule are local policy: the pinned community handler does allow an Officer to be promoted to
//! Leader, which would leave two leaders, and the leader transfer route was never recovered. Every
//! numeric limit that is not recovered stays a caller argument. Nothing here sends a packet, grants a
//! rank, or moves currency, items or experience.
//!
//! # Explicit gaps
//!
//! * Sub-packets 2 (`GuildDeletePacket`), 11 (`GuildMotdUpdatePacket`) and 26
//!   (`GuildInviteCancelPacket`) are client-bound in the native image but no body was recovered, so
//!   the parser refuses them instead of guessing.
//! * `GuildDataFullPacket` (22) keeps unrecovered `Unknown`/`Unknown2` fields and an unproven member
//!   dictionary shape, so it is deliberately not encoded.
//! * The guild-creation rank gate, the paid-rename token economy and the MOTD body remain unrecovered.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Tunneled inner opcode of the guild family (`BaseGuildPacket`).
pub const GUILD_MAIN: u16 = 129;
/// Tunneled inner opcode of the admin guild family (`BaseAdminGuildPacket`).
pub const ADMIN_GUILD_MAIN: u16 = 131;

/// Longest string this module will read out of a guild body.
pub const MAX_WIRE_STRING_LEN: usize = 256;
/// Largest unmodelled body kept for `GuildMemberLocationRequest`.
pub const MAX_TRAILING_BODY: usize = 4096;

/// Community name rule: guild names normalize to 3..=32 characters.
pub const NAME_MIN_LEN: usize = 3;
pub const NAME_MAX_LEN: usize = 32;

/// Upper bound the store accepts for a member cap; the recovered default is far lower.
pub const MAX_MEMBERS_LIMIT: usize = 1000;
/// Bound on simultaneously pending invites per player.
pub const MAX_PENDING_INVITES: usize = 8;

/// `GuildMemberStatusUpdatePacket` update types (community).
pub const MEMBER_UPDATE_JOINED: i32 = 1;
pub const MEMBER_UPDATE_REMOVED: i32 = 2;
pub const MEMBER_UPDATE_QUIT: i32 = 3;
pub const MEMBER_UPDATE_PROMOTED: i32 = 4;
pub const MEMBER_UPDATE_DEMOTED: i32 = 5;
pub const MEMBER_UPDATE_ONLINE: i32 = 6;
pub const MEMBER_UPDATE_NONE: i32 = 7;

/// `GuildPaidRenameCheckReplyPacket` results (community numbering).
pub const PAID_RENAME_NOT_LEADER: i32 = 1;
pub const PAID_RENAME_INVALID: i32 = 2;
pub const PAID_RENAME_TAKEN: i32 = 3;
pub const PAID_RENAME_ALREADY_PENDING: i32 = 4;
pub const PAID_RENAME_SUCCESS: i32 = 5;

/// Every recovered guild sub-opcode, in enum id order.
pub const SUB_OPCODES: [(u16, &str); 28] = [
    (1, "GuildCreatePacket"),
    (2, "GuildDeletePacket"),
    (3, "GuildInvitePacket"),
    (4, "GuildPromotePacket"),
    (5, "GuildDemotePacket"),
    (6, "GuildRemovePacket"),
    (7, "GuildQuitPacket"),
    (8, "GuildInviteAcceptPacket"),
    (9, "GuildInviteDeclinePacket"),
    (10, "GuildInviteTimeOutPacket"),
    (11, "GuildMotdUpdatePacket"),
    (13, "GuildRenameGuildPacket"),
    (14, "GuildNameRequestPacket"),
    (15, "GuildMemberLocationRequest"),
    (16, "GuildPaidRenameCheckRequestPacket"),
    (17, "GuildMemberStatusUpdatePacket"),
    (18, "GuildNameUpdatePacket"),
    (19, "GuildNameAcceptedPacket"),
    (20, "GuildNameRejectedPacket"),
    (21, "GuildRoleNameChangedPacket"),
    (22, "GuildDataFullPacket"),
    (24, "GuildInviteNotificationPacket"),
    (25, "GuildMemberLocationUpdatePacket"),
    (26, "GuildInviteCancelPacket"),
    (29, "GuildDeleteNotificationPacket"),
    (30, "GuildPaidRenameCheckReplyPacket"),
    (31, "GuildPlayerStatusUpdatePacket"),
    (32, "GuildCanCreateGuildPacket"),
];

/// Recovered name of a guild sub-opcode, or `None` when the original enum never numbered it.
pub fn sub_opcode_name(sub: u16) -> Option<&'static str> {
    SUB_OPCODES
        .iter()
        .find(|(id, _)| *id == sub)
        .map(|(_, name)| *name)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum GuildRole {
    Leader,
    Officer,
    Member,
    Recruit,
}

impl GuildRole {
    /// Community-hardcoded role ids: 1 Leader, 2 Officer, 3 Member, 4 Recruit.
    pub const fn id(self) -> i32 {
        match self {
            GuildRole::Leader => 1,
            GuildRole::Officer => 2,
            GuildRole::Member => 3,
            GuildRole::Recruit => 4,
        }
    }

    pub fn from_id(id: i32) -> Option<Self> {
        match id {
            1 => Some(GuildRole::Leader),
            2 => Some(GuildRole::Officer),
            3 => Some(GuildRole::Member),
            4 => Some(GuildRole::Recruit),
            _ => None,
        }
    }

    const fn step_up(self) -> Option<Self> {
        match self {
            GuildRole::Recruit => Some(GuildRole::Member),
            GuildRole::Member => Some(GuildRole::Officer),
            GuildRole::Officer | GuildRole::Leader => None,
        }
    }

    const fn step_down(self) -> Option<Self> {
        match self {
            GuildRole::Leader => Some(GuildRole::Officer),
            GuildRole::Officer => Some(GuildRole::Member),
            GuildRole::Member => Some(GuildRole::Recruit),
            GuildRole::Recruit => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GuildError {
    /// Fewer bytes than the recovered body needs.
    Incomplete,
    /// Bytes left over after the recovered body.
    TrailingBytes,
    BadMainOpcode,
    UnknownSubtype,
    /// The sub-packet is client-bound but no body was recovered for it.
    UnrecoveredBody {
        sub: u16,
    },
    /// The sub-packet travels server-to-client, so the server must not accept it.
    ServerBoundSubtype {
        sub: u16,
    },
    OversizedString,
    BadUtf8,
    NameTooShort,
    NameTooLong,
    IllegalNameCharacter,
    NameTaken,
    AlreadyInGuild,
    NotInGuild,
    NoInviteForPlayer,
    InviteExpired,
    TooManyPendingInvites,
    GuildFull,
    NotPermitted,
    TargetNotMember,
    SelfTarget,
    CantPromoteAbove,
    CannotPromoteToLeader,
    AtMinRank,
    StateTooLarge,
}

/// One recovered client-bound guild request body.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ClientGuildRequest {
    Create {
        name: String,
        temporary_name: String,
        locale: String,
    },
    Invite {
        player_guid: u64,
        guild_guid: u64,
        player_name: String,
    },
    Promote {
        player_guid: u64,
        guild_guid: u64,
    },
    Demote {
        player_guid: u64,
        guild_guid: u64,
    },
    Remove {
        player_guid: u64,
        guild_guid: u64,
    },
    Quit {
        guid: u64,
    },
    InviteAccept {
        player_guid: u64,
    },
    InviteDecline {
        player_guid: u64,
    },
    InviteTimeout {
        player_guid: u64,
    },
    Rename {
        guid: u64,
        name: String,
        locale: String,
        is_non_china_environment: bool,
    },
    NameRequest {
        guid: u64,
    },
    /// The native client owns an extra character-GUID array the community class does not model.
    MemberLocationRequest {
        trailing: Vec<u8>,
    },
    PaidRenameCheck {
        guid: u64,
        name: String,
    },
}

/// The 4-byte header of a guild body.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GuildHeader {
    pub main: u16,
    pub sub: u16,
}

struct Cursor<'a> {
    bytes: &'a [u8],
    at: usize,
}

impl<'a> Cursor<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Cursor { bytes, at: 0 }
    }

    fn take(&mut self, count: usize) -> Result<&'a [u8], GuildError> {
        let end = self.at.checked_add(count).ok_or(GuildError::Incomplete)?;
        let part = self.bytes.get(self.at..end).ok_or(GuildError::Incomplete)?;
        self.at = end;
        Ok(part)
    }

    fn u16(&mut self) -> Result<u16, GuildError> {
        Ok(u16::from_le_bytes(self.take(2)?.try_into().unwrap()))
    }

    fn i32(&mut self) -> Result<i32, GuildError> {
        Ok(i32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }

    fn u64(&mut self) -> Result<u64, GuildError> {
        Ok(u64::from_le_bytes(self.take(8)?.try_into().unwrap()))
    }

    fn bool(&mut self) -> Result<bool, GuildError> {
        Ok(self.take(1)?[0] != 0)
    }

    fn string(&mut self) -> Result<String, GuildError> {
        let length = i32::from_le_bytes(self.take(4)?.try_into().unwrap());
        if length < 0 {
            return Err(GuildError::OversizedString);
        }
        let length = length as usize;
        if length > MAX_WIRE_STRING_LEN {
            return Err(GuildError::OversizedString);
        }
        let raw = self.take(length)?;
        std::str::from_utf8(raw)
            .map(str::to_owned)
            .map_err(|_| GuildError::BadUtf8)
    }

    fn rest(&mut self) -> &'a [u8] {
        let rest = &self.bytes[self.at..];
        self.at = self.bytes.len();
        rest
    }

    fn done(&self) -> Result<(), GuildError> {
        if self.at == self.bytes.len() {
            Ok(())
        } else {
            Err(GuildError::TrailingBytes)
        }
    }
}

/// Recovered main/sub header of a guild body.
pub fn guild_header(inner: &[u8]) -> Result<GuildHeader, GuildError> {
    let mut cursor = Cursor::new(inner);
    let main = cursor.u16()?;
    if main != GUILD_MAIN {
        return Err(GuildError::BadMainOpcode);
    }
    Ok(GuildHeader {
        main,
        sub: cursor.u16()?,
    })
}

/// Parse one tunneled guild payload (`u16` main, `u16` sub, body) into a client request.
///
/// A sub-packet with no recovered body is refused, never guessed. Session identity stays with the
/// caller: the GUIDs inside the body are advisory and never select a character or a guild on their own.
pub fn parse_client_request(inner: &[u8]) -> Result<ClientGuildRequest, GuildError> {
    let header = guild_header(inner)?;
    let mut cursor = Cursor::new(inner);
    cursor.take(4)?;
    let request = match header.sub {
        1 => ClientGuildRequest::Create {
            name: cursor.string()?,
            temporary_name: cursor.string()?,
            locale: cursor.string()?,
        },
        3 => ClientGuildRequest::Invite {
            player_guid: cursor.u64()?,
            guild_guid: cursor.u64()?,
            player_name: cursor.string()?,
        },
        4 => ClientGuildRequest::Promote {
            player_guid: cursor.u64()?,
            guild_guid: cursor.u64()?,
        },
        5 => ClientGuildRequest::Demote {
            player_guid: cursor.u64()?,
            guild_guid: cursor.u64()?,
        },
        6 => ClientGuildRequest::Remove {
            player_guid: cursor.u64()?,
            guild_guid: cursor.u64()?,
        },
        7 => ClientGuildRequest::Quit {
            guid: cursor.u64()?,
        },
        8 => ClientGuildRequest::InviteAccept {
            player_guid: cursor.u64()?,
        },
        9 => ClientGuildRequest::InviteDecline {
            player_guid: cursor.u64()?,
        },
        10 => ClientGuildRequest::InviteTimeout {
            player_guid: cursor.u64()?,
        },
        13 => ClientGuildRequest::Rename {
            guid: cursor.u64()?,
            name: cursor.string()?,
            locale: cursor.string()?,
            is_non_china_environment: cursor.bool()?,
        },
        14 => ClientGuildRequest::NameRequest {
            guid: cursor.u64()?,
        },
        15 => {
            let trailing = cursor.rest();
            if trailing.len() > MAX_TRAILING_BODY {
                return Err(GuildError::OversizedString);
            }
            return Ok(ClientGuildRequest::MemberLocationRequest {
                trailing: trailing.to_vec(),
            });
        }
        16 => ClientGuildRequest::PaidRenameCheck {
            guid: cursor.u64()?,
            name: cursor.string()?,
        },
        2 | 11 | 26 => return Err(GuildError::UnrecoveredBody { sub: header.sub }),
        17 | 18 | 19 | 20 | 21 | 22 | 23 | 24 | 25 | 29 | 30 | 31 | 32 => {
            return Err(GuildError::ServerBoundSubtype { sub: header.sub });
        }
        _ => return Err(GuildError::UnknownSubtype),
    };
    cursor.done()?;
    Ok(request)
}

/// Recovered `GuildMemberStatusUpdatePacket` name block.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NameData {
    pub first_name_id: i32,
    pub last_name_prefix_id: i32,
    pub last_name_suffix_id: i32,
    pub first_name: String,
    pub last_name: String,
}

/// Recovered `GuildMemberStatusUpdatePacket` body.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MemberStatus {
    pub guild_guid: u64,
    pub member_guid: u64,
    pub name: NameData,
    pub role: i32,
    pub online: bool,
    pub update_type: i32,
    pub world_id: i32,
    pub unknown: i32,
    pub profile_id: i32,
    pub profile_rank: i32,
}

fn push_header(packet: &mut Vec<u8>, sub: u16) {
    packet.extend_from_slice(&GUILD_MAIN.to_le_bytes());
    packet.extend_from_slice(&sub.to_le_bytes());
}

fn push_string(packet: &mut Vec<u8>, value: &str) {
    packet.extend_from_slice(&(value.len() as i32).to_le_bytes());
    packet.extend_from_slice(value.as_bytes());
}

fn push_guid(packet: &mut Vec<u8>, value: u64) {
    packet.extend_from_slice(&value.to_le_bytes());
}

/// `GuildErrorPacket` (23): a locale key the client resolves to its own text.
pub fn guild_error(message_name: &str) -> Vec<u8> {
    let mut packet = Vec::new();
    push_header(&mut packet, 23);
    push_string(&mut packet, message_name);
    packet
}

/// `GuildCanCreateGuildPacket` (32).
pub fn can_create_guild(can_create_guild: bool) -> Vec<u8> {
    let mut packet = Vec::new();
    push_header(&mut packet, 32);
    packet.push(u8::from(can_create_guild));
    packet
}

/// `GuildPlayerStatusUpdatePacket` (31).
pub fn player_status(player_guid: u64, guild_guid: u64, is_in_guild: bool) -> Vec<u8> {
    let mut packet = Vec::new();
    push_header(&mut packet, 31);
    push_guid(&mut packet, player_guid);
    push_guid(&mut packet, guild_guid);
    packet.push(u8::from(is_in_guild));
    packet
}

/// `GuildNameUpdatePacket` (18).
pub fn name_update(guid: u64, name: &str) -> Vec<u8> {
    let mut packet = Vec::new();
    push_header(&mut packet, 18);
    push_guid(&mut packet, guid);
    push_string(&mut packet, name);
    packet
}

/// `GuildPaidRenameCheckReplyPacket` (30).
pub fn paid_rename_reply(guid: u64, name: &str, result: i32) -> Vec<u8> {
    let mut packet = Vec::new();
    push_header(&mut packet, 30);
    push_guid(&mut packet, guid);
    push_string(&mut packet, name);
    packet.extend_from_slice(&result.to_le_bytes());
    packet
}

/// `GuildMemberStatusUpdatePacket` (17).
pub fn member_status(status: &MemberStatus) -> Vec<u8> {
    let mut packet = Vec::new();
    push_header(&mut packet, 17);
    push_guid(&mut packet, status.guild_guid);
    push_guid(&mut packet, status.member_guid);
    packet.extend_from_slice(&status.name.first_name_id.to_le_bytes());
    packet.extend_from_slice(&status.name.last_name_prefix_id.to_le_bytes());
    packet.extend_from_slice(&status.name.last_name_suffix_id.to_le_bytes());
    push_string(&mut packet, &status.name.first_name);
    push_string(&mut packet, &status.name.last_name);
    packet.extend_from_slice(&status.role.to_le_bytes());
    packet.push(u8::from(status.online));
    for value in [
        status.update_type,
        status.world_id,
        status.unknown,
        status.profile_id,
        status.profile_rank,
    ] {
        packet.extend_from_slice(&value.to_le_bytes());
    }
    packet
}

/// Kind of change the store just applied; the caller turns it into the 17/31/23 replies.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GuildEvent {
    Created {
        guild: u64,
        creator: u64,
    },
    GuildRenamed {
        guild: u64,
        name: String,
    },
    MemberJoined {
        guild: u64,
        member: u64,
        role: GuildRole,
    },
    MemberLeft {
        guild: u64,
        member: u64,
    },
    MemberRemoved {
        guild: u64,
        member: u64,
        actor: u64,
    },
    RoleChanged {
        guild: u64,
        member: u64,
        role: GuildRole,
        promoted: bool,
    },
    GuildDisbanded {
        guild: u64,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GuildMembership {
    pub role: GuildRole,
    pub online: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Guild {
    pub guid: u64,
    pub name: String,
    pub max_members: usize,
    pub members: BTreeMap<u64, GuildMembership>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PendingInvite {
    pub guild: u64,
    pub inviter: u64,
    pub sent_tick: u64,
}

/// Bounded guild state: guilds, memberships and outstanding invites.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GuildStore {
    max_members: usize,
    next_guild_id: u64,
    guilds: BTreeMap<u64, Guild>,
    invites: BTreeMap<u64, PendingInvite>,
}

impl GuildStore {
    /// `max_members` is the per-guild cap written into every new guild (the community default is 100).
    pub fn new(max_members: usize) -> Self {
        GuildStore {
            max_members: max_members.clamp(1, MAX_MEMBERS_LIMIT),
            next_guild_id: 1,
            guilds: BTreeMap::new(),
            invites: BTreeMap::new(),
        }
    }

    pub fn next_guild_id(&self) -> u64 {
        self.next_guild_id
    }

    pub fn guild_of(&self, player: u64) -> Option<u64> {
        self.guilds
            .iter()
            .find(|(_, guild)| guild.members.contains_key(&player))
            .map(|(guid, _)| *guid)
    }

    pub fn role_of(&self, player: u64) -> Option<GuildRole> {
        let guild = self.guild_of(player)?;
        self.guilds
            .get(&guild)?
            .members
            .get(&player)
            .map(|row| row.role)
    }

    pub fn guild_name(&self, guild: u64) -> Option<&str> {
        self.guilds.get(&guild).map(|row| row.name.as_str())
    }

    pub fn member_count(&self, guild: u64) -> usize {
        self.guilds.get(&guild).map_or(0, |row| row.members.len())
    }

    pub fn members(&self, guild: u64) -> Option<&BTreeMap<u64, GuildMembership>> {
        self.guilds.get(&guild).map(|row| &row.members)
    }

    /// Guid of the current leader, or `None` while the guild does not exist.
    pub fn leader_of(&self, guild: u64) -> Option<u64> {
        self.guilds
            .get(&guild)?
            .members
            .iter()
            .find(|(_, row)| row.role == GuildRole::Leader)
            .map(|(guid, _)| *guid)
    }

    pub fn pending_invite(&self, player: u64) -> Option<PendingInvite> {
        self.invites.get(&player).copied()
    }

    /// Community name gate: whitespace-collapsed, 3..=32 characters, letters/digits/space/'/-.
    pub fn validate_name(raw: &str) -> Result<String, GuildError> {
        let name = raw.split_whitespace().collect::<Vec<_>>().join(" ");
        if name.chars().count() < NAME_MIN_LEN {
            return Err(GuildError::NameTooShort);
        }
        if name.chars().count() > NAME_MAX_LEN {
            return Err(GuildError::NameTooLong);
        }
        if !name
            .chars()
            .all(|c| c.is_alphanumeric() || c == ' ' || c == '\'' || c == '-')
        {
            return Err(GuildError::IllegalNameCharacter);
        }
        Ok(name)
    }

    // ponytail: linear duplicate-name scan; the store is a bounded local guild table, index it only
    // if a real shard ever holds thousands of guilds.
    fn name_taken(&self, name: &str, except: Option<u64>) -> bool {
        let lowered = name.to_lowercase();
        self.guilds
            .iter()
            .any(|(guid, guild)| guild.name.to_lowercase() == lowered && Some(*guid) != except)
    }

    pub fn create_guild(&mut self, creator: u64, raw_name: &str) -> Result<GuildEvent, GuildError> {
        if self.guild_of(creator).is_some() {
            return Err(GuildError::AlreadyInGuild);
        }
        let name = Self::validate_name(raw_name)?;
        if self.name_taken(&name, None) {
            return Err(GuildError::NameTaken);
        }
        let guid = self.next_guild_id;
        self.next_guild_id = guid.checked_add(1).ok_or(GuildError::StateTooLarge)?;
        let mut members = BTreeMap::new();
        members.insert(
            creator,
            GuildMembership {
                role: GuildRole::Leader,
                online: true,
            },
        );
        self.guilds.insert(
            guid,
            Guild {
                guid,
                name,
                max_members: self.max_members,
                members,
            },
        );
        Ok(GuildEvent::Created {
            guild: guid,
            creator,
        })
    }

    pub fn invite(&mut self, inviter: u64, invitee: u64, tick: u64) -> Result<(), GuildError> {
        let guild_guid = self.guild_of(inviter).ok_or(GuildError::NotInGuild)?;
        let role = self.role_of(inviter).ok_or(GuildError::NotInGuild)?;
        if !matches!(role, GuildRole::Leader | GuildRole::Officer) {
            return Err(GuildError::NotPermitted);
        }
        let guild = self.guilds.get(&guild_guid).ok_or(GuildError::NotInGuild)?;
        if guild.members.len() >= guild.max_members {
            return Err(GuildError::GuildFull);
        }
        if self.guild_of(invitee).is_some() {
            return Err(GuildError::AlreadyInGuild);
        }
        if self.invites.contains_key(&invitee) {
            // The community server drops a duplicate invite silently; refresh nothing.
            return Ok(());
        }
        let outstanding = self
            .invites
            .values()
            .filter(|row| row.guild == guild_guid)
            .count();
        if outstanding >= MAX_PENDING_INVITES {
            return Err(GuildError::TooManyPendingInvites);
        }
        self.invites.insert(
            invitee,
            PendingInvite {
                guild: guild_guid,
                inviter,
                sent_tick: tick,
            },
        );
        Ok(())
    }

    /// `timeout_ticks` is the caller's clock window; no historical invite lifetime is baked in.
    pub fn accept_invite(
        &mut self,
        player: u64,
        guild_guid: u64,
        tick: u64,
        timeout_ticks: u64,
    ) -> Result<GuildEvent, GuildError> {
        if self.guild_of(player).is_some() {
            return Err(GuildError::AlreadyInGuild);
        }
        let invite = self
            .invites
            .get(&player)
            .copied()
            .ok_or(GuildError::NoInviteForPlayer)?;
        if invite.guild != guild_guid {
            return Err(GuildError::NoInviteForPlayer);
        }
        if tick.saturating_sub(invite.sent_tick) > timeout_ticks {
            self.invites.remove(&player);
            return Err(GuildError::InviteExpired);
        }
        let guild = self
            .guilds
            .get_mut(&guild_guid)
            .ok_or(GuildError::NotInGuild)?;
        if guild.members.len() >= guild.max_members {
            return Err(GuildError::GuildFull);
        }
        let role = GuildRole::Recruit;
        guild
            .members
            .insert(player, GuildMembership { role, online: true });
        self.invites.remove(&player);
        Ok(GuildEvent::MemberJoined {
            guild: guild_guid,
            member: player,
            role,
        })
    }

    pub fn decline_invite(&mut self, player: u64, guild_guid: u64) -> Result<(), GuildError> {
        match self.invites.get(&player) {
            Some(invite) if invite.guild == guild_guid => {
                self.invites.remove(&player);
                Ok(())
            }
            _ => Err(GuildError::NoInviteForPlayer),
        }
    }

    fn membership(&self, player: u64) -> Result<(u64, GuildRole), GuildError> {
        let guild = self.guild_of(player).ok_or(GuildError::TargetNotMember)?;
        let role = self.role_of(player).ok_or(GuildError::TargetNotMember)?;
        Ok((guild, role))
    }

    fn can_manage(actor: GuildRole, target: GuildRole) -> bool {
        match actor {
            GuildRole::Leader => true,
            // Lower id outranks higher: an officer may only touch Member or Recruit.
            GuildRole::Officer => target.id() > GuildRole::Officer.id(),
            GuildRole::Member | GuildRole::Recruit => false,
        }
    }

    pub fn promote(&mut self, actor: u64, target: u64) -> Result<GuildEvent, GuildError> {
        let guild_guid = self.guild_of(actor).ok_or(GuildError::NotInGuild)?;
        let actor_role = self.role_of(actor).ok_or(GuildError::NotInGuild)?;
        let (target_guild, target_role) = self.membership(target)?;
        if target_guild != guild_guid {
            return Err(GuildError::TargetNotMember);
        }
        if actor == target {
            return Err(GuildError::SelfTarget);
        }
        if !Self::can_manage(actor_role, target_role) {
            return Err(GuildError::CantPromoteAbove);
        }
        let new_role = target_role
            .step_up()
            .ok_or(GuildError::CannotPromoteToLeader)?;
        if actor_role == GuildRole::Officer && new_role.id() < GuildRole::Officer.id() {
            return Err(GuildError::CantPromoteAbove);
        }
        self.set_role(guild_guid, target, new_role)?;
        Ok(GuildEvent::RoleChanged {
            guild: guild_guid,
            member: target,
            role: new_role,
            promoted: true,
        })
    }

    pub fn demote(&mut self, actor: u64, target: u64) -> Result<GuildEvent, GuildError> {
        let guild_guid = self.guild_of(actor).ok_or(GuildError::NotInGuild)?;
        let actor_role = self.role_of(actor).ok_or(GuildError::NotInGuild)?;
        let (target_guild, target_role) = self.membership(target)?;
        if target_guild != guild_guid {
            return Err(GuildError::TargetNotMember);
        }
        if !Self::can_manage(actor_role, target_role) {
            return Err(GuildError::CantPromoteAbove);
        }
        if actor == target {
            return Err(GuildError::SelfTarget);
        }
        let new_role = target_role.step_down().ok_or(GuildError::AtMinRank)?;
        self.set_role(guild_guid, target, new_role)?;
        Ok(GuildEvent::RoleChanged {
            guild: guild_guid,
            member: target,
            role: new_role,
            promoted: false,
        })
    }

    pub fn remove(&mut self, actor: u64, target: u64) -> Result<GuildEvent, GuildError> {
        let guild_guid = self.guild_of(actor).ok_or(GuildError::NotInGuild)?;
        let actor_role = self.role_of(actor).ok_or(GuildError::NotInGuild)?;
        let (target_guild, target_role) = self.membership(target)?;
        if target_guild != guild_guid {
            return Err(GuildError::TargetNotMember);
        }
        if actor == target {
            return Err(GuildError::SelfTarget);
        }
        if !Self::can_manage(actor_role, target_role) {
            return Err(GuildError::CantPromoteAbove);
        }
        let guild = self
            .guilds
            .get_mut(&guild_guid)
            .ok_or(GuildError::NotInGuild)?;
        guild.members.remove(&target);
        self.invites.remove(&target);
        Ok(GuildEvent::MemberRemoved {
            guild: guild_guid,
            member: target,
            actor,
        })
    }

    /// Leave the guild. The last member deletes it; a leaving leader hands the rank to the senior
    /// remaining member (lowest role id, then lowest character id, as the community server orders it).
    pub fn quit(&mut self, player: u64) -> Result<GuildEvent, GuildError> {
        let guild_guid = self.guild_of(player).ok_or(GuildError::NotInGuild)?;
        let was_leader = self.role_of(player) == Some(GuildRole::Leader);
        let guild = self
            .guilds
            .get_mut(&guild_guid)
            .ok_or(GuildError::NotInGuild)?;
        guild.members.remove(&player);
        self.invites.remove(&player);
        if guild.members.is_empty() {
            self.guilds.remove(&guild_guid);
            return Ok(GuildEvent::GuildDisbanded { guild: guild_guid });
        }
        if was_leader {
            if let Some((heir, _)) = guild
                .members
                .iter()
                .min_by_key(|(guid, row)| (row.role.id(), **guid))
                .map(|(guid, row)| (*guid, row.role))
            {
                if let Some(row) = guild.members.get_mut(&heir) {
                    row.role = GuildRole::Leader;
                }
            }
        }
        Ok(GuildEvent::MemberLeft {
            guild: guild_guid,
            member: player,
        })
    }

    pub fn rename(&mut self, actor: u64, raw_name: &str) -> Result<GuildEvent, GuildError> {
        let guild_guid = self.guild_of(actor).ok_or(GuildError::NotInGuild)?;
        if self.role_of(actor) != Some(GuildRole::Leader) {
            return Err(GuildError::NotPermitted);
        }
        let name = Self::validate_name(raw_name)?;
        if self.name_taken(&name, Some(guild_guid)) {
            return Err(GuildError::NameTaken);
        }
        let guild = self
            .guilds
            .get_mut(&guild_guid)
            .ok_or(GuildError::NotInGuild)?;
        guild.name = name.clone();
        Ok(GuildEvent::GuildRenamed {
            guild: guild_guid,
            name,
        })
    }

    fn set_role(&mut self, guild: u64, player: u64, role: GuildRole) -> Result<(), GuildError> {
        let guild = self.guilds.get_mut(&guild).ok_or(GuildError::NotInGuild)?;
        let row = guild
            .members
            .get_mut(&player)
            .ok_or(GuildError::TargetNotMember)?;
        row.role = role;
        Ok(())
    }

    pub fn to_json(&self) -> Result<String, GuildError> {
        serde_json::to_string(self).map_err(|_| GuildError::StateTooLarge)
    }

    pub fn from_json(json: &str) -> Result<Self, GuildError> {
        let store: GuildStore =
            serde_json::from_str(json).map_err(|_| GuildError::StateTooLarge)?;
        if store.max_members == 0 || store.max_members > MAX_MEMBERS_LIMIT {
            return Err(GuildError::StateTooLarge);
        }
        Ok(store)
    }
}
