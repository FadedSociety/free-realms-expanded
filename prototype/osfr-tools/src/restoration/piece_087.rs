//! Work item 087: friends and presence (roster, online status, requests).
//!
//! Evidence labels follow the worker contract. ORIGINAL means the fact is read from a preserved
//! Free Realms artifact, INFERRED means it comes from the pinned community model with no recovered
//! original serializer, RECONSTRUCTED means this module supplies a bounded local rule, and MISSING
//! is an explicit gap.
//!
//! ORIGINAL facts implemented here:
//! * the friend family is `BaseFriendsPacket` = **74** and the command family is `BaseCommandPacket`
//!   = **26** (`lab/decompiled/server/LibSOE/SOE/BasePackets.cs:43` and
//!   `.../BaseCommandPackets.cs`/`BasePackets.cs`; the client agrees: `FUN_008a2820` at
//!   `pseudocode.c:976253` stores `param_1[1] = 0x1a`).
//! * the nine friend sub-opcodes are read from the shipped client constructors, each of which calls
//!   the shared `BaseFriendPacket` initializer `FUN_00c7d870(<sub>)` immediately before its own
//!   `vftable` store: `FriendMessagePacket` 0 (`pseudocode.c:1748650`), `FriendListPacket` 1
//!   (1749925), `FriendOnlinePacket` 2 (1747660), `FriendOfflinePacket` 3 (1747086),
//!   `FriendUpdatePositionsPacket` 5 (1749760), `FriendAddPacket` 6 (1748612), `FriendRemovePacket`
//!   7 (1747133), `FriendStatusPacket` 9 (1747721) and `FriendRenamePacket` 10 (1747600). The
//!   supplied server's own enum (`lab/decompiled/server/LibSOE/SOE/BaseFriendPackets.cs:3-13`)
//!   agrees on all nine; slots 4 and 8 belong to no friend subclass in either artifact.
//! * the six command sub-opcodes used by the friend requests are read from the client constructors
//!   that call `FUN_008a2820(<sub>, 0)`: `CommandPacketAddFriendRequest` 14 (`pseudocode.c:1035628`),
//!   `CommandPacketRemoveFriendRequest` 15 (1672341), `CommandPacketConfirmFriendRequest` 16
//!   (1378646), `CommandPacketConfirmFriendResponse` 17 (1264146),
//!   `CommandPacketFriendsPositionRequest` 21 (1667324) and `CommandPacketIgnoreRequest` 30
//!   (1672377). The supplied server's `BaseCommandPackets.cs` lists the same numbers.
//! * the roster row the client renders is its own `FriendsList_DS` data source, 18 fields
//!   (`lab/decompiled/client-ui/ScriptsBase.lua:3244-3263`), re-exposed to UI code as the 13
//!   `FriendsListDsColumn` entries (3951-3965). Both are published here as [`ROSTER_FIELDS`] and
//!   [`ROSTER_UI_COLUMNS`].
//! * the supplied server contains exactly one friend handler
//!   (`lab/decompiled/server/Gateway/Gateway.Login/LoginManager.cs:394-407`):
//!   `CommandPacketFriendsPositionRequest` answers with a hard-coded
//!   `PlayerUpdatePacketUpdatePosition` (125) for a fabricated character
//!   `5427660847725618161` at `(-1597.2408, -42.257397, -456.4496)`. It never sends a roster and
//!   never stores a friendship. That canned packet is deliberately not reproduced here.
//!
//! INFERRED (community, pinned; no original serializer recovered):
//! * every byte layout below: `short` family + `byte` sub header, `i32`-length UTF-8 strings, and
//!   the `FriendData`/`FriendStatus`/`FriendMessageType` field orders, from
//!   `lab/references/Sanctuary/src/Sanctuary.Packet/BaseFriendPacket/*.cs` and
//!   `src/Sanctuary.Packet.Common/*.cs`. The same community family (`FreeRealms-Legacy`) is the
//!   pinned reader the other restoration pieces already use for this wire.
//! * the command sub-opcode width: the client stores it in a 32-bit slot, the community writer
//!   emits a 16-bit value. This module emits 16 bits and flags the width as inferred.
//!
//! RECONSTRUCTED (bounded local policy, never present in the supplied server):
//! * who may request whom, self/duplicate/ignore gating, the pending-request lifetime, the mutual
//!   roster write on accept, who receives online/offline/status/rename/position fan-out, and the
//!   defensive decode bounds ([`MAX_ROSTER_ENTRIES`], [`MAX_NAME_BYTES`], [`MAX_POSITION_ENTRIES`]).
//!
//! MISSING / explicit gaps:
//! * the original status code meanings (the community comment claims 110 online / 111 offline) are
//!   carried through verbatim without being interpreted.
//! * the `CommandPacketIgnoreRequest` (30) body, the original request expiry, the teleport-to-friend
//!   rules, the friend cap, per-zone fan-out limits and any persistence format are unrecovered.
//! * `popularity` has no recovered producer; roster rows default it to 0.
//!
//! ponytail: one directory map, one roster map per character, one pending map, one ignore map, and
//! plain byte codecs. No traits, no async, no packets beyond the recovered family.

use std::collections::{BTreeMap, BTreeSet};

/// ORIGINAL: `BasePackets.BaseFriendsPacket` (`BasePackets.cs:43`).
pub const FRIENDS_FAMILY: u16 = 74;
/// ORIGINAL: `BasePackets.BaseCommandPacket` (`BasePackets.cs`; client `param_1[1] = 0x1a`).
pub const COMMAND_FAMILY: u16 = 26;
/// ORIGINAL: `BasePackets.BaseAcquaintancePacket` (`BasePackets.cs:41`); not modelled here.
pub const ACQUAINTANCE_FAMILY: u16 = 72;
/// ORIGINAL: `BasePackets.BaseIgnorePacket` (`BasePackets.cs:70`); the ignore gate is modelled,
/// the ignore wire is not.
pub const IGNORE_FAMILY: u16 = 112;
/// ORIGINAL: friend sub-opcodes with no subclass in the client or the supplied server.
pub const UNUSED_SUB_OPCODES: [u8; 2] = [4, 8];

/// RECONSTRUCTED defensive bound on a decoded roster. No original friend cap was recovered.
pub const MAX_ROSTER_ENTRIES: usize = 128;
/// RECONSTRUCTED defensive bound on a name, matching the 128-byte cleartext name the native
/// character-creation reader already enforces (`src/identity.rs`).
pub const MAX_NAME_BYTES: usize = 128;

/// ORIGINAL: the client `FriendsList_DS` fields (`ScriptsBase.lua:3244-3263`).
pub const ROSTER_FIELDS: [(&str, u8); 18] = [
    ("FriendName", 0),
    ("LocationX", 1),
    ("LocationZ", 2),
    ("Guid", 3),
    ("Online", 4),
    ("Online_Status", 5),
    ("Profile_Name", 6),
    ("Profile_Rank", 7),
    ("Profile_Icon_ID", 8),
    ("Profile_Background_Image_ID", 9),
    ("Popularity", 10),
    ("In_Encounter", 11),
    ("Is_Local", 12),
    ("Status", 13),
    ("HighScore", 14),
    ("Current_Mini_Game_Type", 15),
    ("Current_Mini_Game_Name", 16),
    ("Is_In_Static_Zone", 17),
];

/// ORIGINAL: the roster columns exposed to UI code (`ScriptsBase.lua:3951-3965`).
pub const ROSTER_UI_COLUMNS: [(&str, u8); 13] = [
    ("Name", 0),
    ("X", 1),
    ("Y", 2),
    ("Guid", 3),
    ("Online", 4),
    ("Status", 5),
    ("ProfileName", 6),
    ("ProfileRank", 7),
    ("ProfileIconId", 8),
    ("ProfileBackgroundId", 9),
    ("Popularity", 10),
    ("InEncounter", 11),
    ("IsLocal", 12),
];

/// The client friend subclasses, in ascending sub-opcode order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum FriendSubOpcode {
    Message,
    List,
    Online,
    Offline,
    UpdatePositions,
    Add,
    Remove,
    Status,
    Rename,
}

impl FriendSubOpcode {
    /// Every recovered subclass, ascending.
    pub const ALL: [Self; 9] = [
        Self::Message,
        Self::List,
        Self::Online,
        Self::Offline,
        Self::UpdatePositions,
        Self::Add,
        Self::Remove,
        Self::Status,
        Self::Rename,
    ];

    pub fn code(self) -> u8 {
        match self {
            Self::Message => 0,
            Self::List => 1,
            Self::Online => 2,
            Self::Offline => 3,
            Self::UpdatePositions => 5,
            Self::Add => 6,
            Self::Remove => 7,
            Self::Status => 9,
            Self::Rename => 10,
        }
    }

    pub fn class_name(self) -> &'static str {
        match self {
            Self::Message => "FriendMessagePacket",
            Self::List => "FriendListPacket",
            Self::Online => "FriendOnlinePacket",
            Self::Offline => "FriendOfflinePacket",
            Self::UpdatePositions => "FriendUpdatePositionsPacket",
            Self::Add => "FriendAddPacket",
            Self::Remove => "FriendRemovePacket",
            Self::Status => "FriendStatusPacket",
            Self::Rename => "FriendRenamePacket",
        }
    }

    /// The shipped-client constructor that proves this opcode. Sub-opcode 8 is the community
    /// disagreement: `Sanctuary` assigns it to `FriendMessagePacket` while the client and the
    /// supplied server both put the message subclass at 0 and leave 8 unused.
    pub fn provenance(self) -> &'static str {
        match self {
            Self::Message => "pseudocode.c:1748650 FUN_00c7f960 -> FUN_00c7d870(0)",
            Self::List => "pseudocode.c:1749925 FUN_00c81040 -> FUN_00c7d870(1)",
            Self::Online => "pseudocode.c:1747660 FUN_00c7e910 -> FUN_00c7d870(2)",
            Self::Offline => "pseudocode.c:1747086 FUN_00c7df20 -> FUN_00c7d870(3)",
            Self::UpdatePositions => "pseudocode.c:1749760 FUN_00c80d40 -> FUN_00c7d870(5)",
            Self::Add => "pseudocode.c:1748612 FUN_00c7f8e0 -> FUN_00c7d870(6)",
            Self::Remove => "pseudocode.c:1747133 FUN_00c7dff0 -> FUN_00c7d870(7)",
            Self::Status => "pseudocode.c:1747721 FUN_00c7ea20 -> FUN_00c7d870(9)",
            Self::Rename => "pseudocode.c:1747600 FUN_00c7e810 -> FUN_00c7d870(10)",
        }
    }

    pub fn from_code(code: u8) -> Option<Self> {
        Self::ALL.into_iter().find(|sub| sub.code() == code)
    }
}

/// The friend-related members of the command family (26).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum FriendCommand {
    AddFriendRequest,
    RemoveFriendRequest,
    ConfirmFriendRequest,
    ConfirmFriendResponse,
    FriendsPositionRequest,
    IgnoreRequest,
}

/// ORIGINAL: every friend-related command subclass from the client constructors.
pub const COMMAND_SUB_OPCODES: [FriendCommand; 6] = [
    FriendCommand::AddFriendRequest,
    FriendCommand::RemoveFriendRequest,
    FriendCommand::ConfirmFriendRequest,
    FriendCommand::ConfirmFriendResponse,
    FriendCommand::FriendsPositionRequest,
    FriendCommand::IgnoreRequest,
];

impl FriendCommand {
    pub fn code(self) -> u16 {
        match self {
            Self::AddFriendRequest => 14,
            Self::RemoveFriendRequest => 15,
            Self::ConfirmFriendRequest => 16,
            Self::ConfirmFriendResponse => 17,
            Self::FriendsPositionRequest => 21,
            Self::IgnoreRequest => 30,
        }
    }

    pub fn class_name(self) -> &'static str {
        match self {
            Self::AddFriendRequest => "CommandPacketAddFriendRequest",
            Self::RemoveFriendRequest => "CommandPacketRemoveFriendRequest",
            Self::ConfirmFriendRequest => "CommandPacketConfirmFriendRequest",
            Self::ConfirmFriendResponse => "CommandPacketConfirmFriendResponse",
            Self::FriendsPositionRequest => "CommandPacketFriendsPositionRequest",
            Self::IgnoreRequest => "CommandPacketIgnoreRequest",
        }
    }

    pub fn provenance(self) -> &'static str {
        match self {
            Self::AddFriendRequest => "pseudocode.c:1035628 FUN_008a2820(0xe, 0)",
            Self::RemoveFriendRequest => "pseudocode.c:1672341 FUN_008a2820(0xf, 0)",
            Self::ConfirmFriendRequest => "pseudocode.c:1378646 FUN_008a2820(0x10, 0)",
            Self::ConfirmFriendResponse => "pseudocode.c:1264146 FUN_008a2820(0x11, 0)",
            Self::FriendsPositionRequest => "pseudocode.c:1667324 FUN_008a2820(0x15, 0)",
            Self::IgnoreRequest => "pseudocode.c:1672377 FUN_008a2820(0x1e, 0)",
        }
    }

    pub fn from_code(code: u16) -> Option<Self> {
        COMMAND_SUB_OPCODES
            .into_iter()
            .find(|command| command.code() == code)
    }
}

/// INFERRED: message packet types, numbering from the pinned community enum.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FriendMessageType {
    AddRequestAccepted,
    AddRequestDeclined,
    AddRequestTimedOut,
    NoTeleportToFriendCombat,
    AddRequested,
    NoTeleportFriendOffline,
    TeleportedCombat,
    NoFriendSpam,
    NameChanged,
    NoTeleportToPlayer,
    NoTeleportToFriendMiniGame,
}

impl FriendMessageType {
    pub fn code(self) -> u8 {
        match self {
            Self::AddRequestAccepted => 1,
            Self::AddRequestDeclined => 2,
            Self::AddRequestTimedOut => 3,
            Self::NoTeleportToFriendCombat => 4,
            Self::AddRequested => 5,
            Self::NoTeleportFriendOffline => 6,
            Self::TeleportedCombat => 7,
            Self::NoFriendSpam => 8,
            Self::NameChanged => 9,
            Self::NoTeleportToPlayer => 10,
            Self::NoTeleportToFriendMiniGame => 11,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConfirmStatus {
    Accepted,
    Declined,
    TimedOut,
}

impl ConfirmStatus {
    pub fn from_code(code: i32) -> Option<Self> {
        match code {
            0 => Some(Self::Accepted),
            1 => Some(Self::Declined),
            2 => Some(Self::TimedOut),
            _ => None,
        }
    }

    pub fn code(self) -> i32 {
        match self {
            Self::Accepted => 0,
            Self::Declined => 1,
            Self::TimedOut => 2,
        }
    }

    fn answer_message(self) -> FriendMessageType {
        match self {
            Self::Accepted => FriendMessageType::AddRequestAccepted,
            Self::Declined => FriendMessageType::AddRequestDeclined,
            Self::TimedOut => FriendMessageType::AddRequestTimedOut,
        }
    }
}

/// INFERRED: `NameData` (three locale ids plus the cleartext pair).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NameData {
    pub first_name_id: i32,
    pub last_name_prefix_id: i32,
    pub last_name_suffix_id: i32,
    pub first_name: String,
    pub last_name: String,
}

impl NameData {
    pub fn new(first: &str, last: &str) -> Self {
        Self {
            first_name_id: 0,
            last_name_prefix_id: 0,
            last_name_suffix_id: 0,
            first_name: first.to_string(),
            last_name: last.to_string(),
        }
    }

    pub fn full_name(&self) -> String {
        if self.last_name.trim().is_empty() {
            self.first_name.clone()
        } else {
            format!("{} {}", self.first_name, self.last_name)
        }
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut out = Vec::new();
        encode_name(&mut out, self);
        out
    }
}

impl Default for NameData {
    fn default() -> Self {
        Self::new("", "")
    }
}

/// INFERRED: `FriendStatus`. `status` is carried verbatim; the original codes are unrecovered.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FriendStatus {
    pub status: i32,
    pub current_mini_game_type: i32,
    pub current_mini_game_name_id: i32,
    pub profile_id: i32,
    pub profile_name_id: i32,
    pub profile_icon_id: i32,
    pub profile_background_image_id: i32,
    pub profile_rank: i32,
}

/// INFERRED: one roster row (`FriendData`).
#[derive(Clone, Debug, PartialEq)]
pub struct FriendEntry {
    pub name: NameData,
    pub guid: u64,
    pub online: bool,
    pub in_encounter: bool,
    pub status: FriendStatus,
    pub location_x: f32,
    pub location_z: f32,
    pub popularity: i32,
    pub is_local: bool,
    pub is_in_static_zone: bool,
    /// Written only in the offline branch; no original meaning was recovered.
    pub offline_unknown: u64,
}

/// INFERRED: one position entry (`FriendUpdatePositionsPacket::Entry`).
#[derive(Clone, Debug, PartialEq)]
pub struct PositionEntry {
    pub guid: u64,
    pub name: NameData,
    pub in_encounter: bool,
    pub location_x: f32,
    pub location_z: f32,
}

/// INFERRED: the server-to-client friend packets this piece can produce.
#[derive(Clone, Debug, PartialEq)]
pub enum Notification {
    Message {
        message_type: FriendMessageType,
        guid: u64,
        name: NameData,
        new_name: NameData,
    },
    List {
        friends: Vec<FriendEntry>,
    },
    Online {
        name: NameData,
        guid: u64,
        is_local: bool,
    },
    Offline {
        guid: u64,
    },
    UpdatePositions {
        entries: Vec<PositionEntry>,
    },
    Add {
        entry: FriendEntry,
    },
    Remove {
        guid: u64,
    },
    Status {
        name: NameData,
        guid: u64,
        status: FriendStatus,
    },
    Rename {
        name: NameData,
        guid: u64,
    },
}

impl Notification {
    pub fn sub_opcode(&self) -> FriendSubOpcode {
        match self {
            Self::Message { .. } => FriendSubOpcode::Message,
            Self::List { .. } => FriendSubOpcode::List,
            Self::Online { .. } => FriendSubOpcode::Online,
            Self::Offline { .. } => FriendSubOpcode::Offline,
            Self::UpdatePositions { .. } => FriendSubOpcode::UpdatePositions,
            Self::Add { .. } => FriendSubOpcode::Add,
            Self::Remove { .. } => FriendSubOpcode::Remove,
            Self::Status { .. } => FriendSubOpcode::Status,
            Self::Rename { .. } => FriendSubOpcode::Rename,
        }
    }
}

/// A decoded client request from family 26.
#[derive(Clone, Debug, PartialEq)]
pub enum FriendRequest {
    AddFriend {
        name: String,
    },
    RemoveFriend {
        name: NameData,
    },
    ConfirmFriend {
        guid: u64,
        status: ConfirmStatus,
        name: NameData,
    },
    FriendsPositionRequest,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FriendError {
    Truncated,
    TrailingBytes(usize),
    RosterOutOfBounds(i32),
    StringTooLong(usize),
    InvalidLength(i32),
    InvalidUtf8,
    EmptyName,
    InvalidBool(u8),
    NonFiniteCoordinate,
    InvalidConfirmStatus(i32),
    UnknownSubOpcode(u16),
    /// The opcode is original but its body was never recovered, so this module refuses it.
    UnrecoveredBody(&'static str),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FriendDenial {
    UnknownSender,
    UnknownTarget,
    SelfRequest,
    AlreadyFriends,
    AlreadyPending,
    TargetIgnoresSender,
    NoPendingRequest,
    NotFriends,
    NonFinitePosition,
    InvalidName,
}

/// A packet addressed to one character: either a friend notification or a command packet.
#[derive(Clone, Debug, PartialEq)]
pub enum Packet {
    Friend(Notification),
    Command {
        command: FriendCommand,
        body: Vec<u8>,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct Delivery {
    pub to: u64,
    pub packet: Packet,
}

impl Delivery {
    pub fn friend(to: u64, notification: Notification) -> Self {
        Self {
            to,
            packet: Packet::Friend(notification),
        }
    }

    pub fn command(to: u64, command: FriendCommand, body: Vec<u8>) -> Self {
        Self {
            to,
            packet: Packet::Command { command, body },
        }
    }

    pub fn packet_code(&self) -> u16 {
        match &self.packet {
            Packet::Friend(notification) => notification.sub_opcode().code() as u16,
            Packet::Command { command, .. } => command.code(),
        }
    }

    pub fn message_type(&self) -> Option<u8> {
        match &self.packet {
            Packet::Friend(Notification::Message { message_type, .. }) => Some(message_type.code()),
            _ => None,
        }
    }

    /// The notification this delivery carries, or `None` for a command packet.
    pub fn into_notification(self) -> Option<Notification> {
        match self.packet {
            Packet::Friend(notification) => Some(notification),
            Packet::Command { .. } => None,
        }
    }

    /// The inner packet bytes: family header plus the inferred body.
    pub fn encode(&self) -> Vec<u8> {
        match &self.packet {
            Packet::Friend(notification) => encode_notification(notification),
            Packet::Command { command, body } => {
                let mut out = Vec::with_capacity(4 + body.len());
                out.extend_from_slice(&COMMAND_FAMILY.to_le_bytes());
                out.extend_from_slice(&(command.code() as i16).to_le_bytes());
                out.extend_from_slice(body);
                out
            }
        }
    }
}

/// Encode one friend notification as `u16 family | u8 sub | body` (INFERRED body layouts).
pub fn encode_notification(notification: &Notification) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(&FRIENDS_FAMILY.to_le_bytes());
    out.push(notification.sub_opcode().code());
    match notification {
        Notification::Message {
            message_type,
            guid,
            name,
            new_name,
        } => {
            out.push(message_type.code());
            out.extend_from_slice(&guid.to_le_bytes());
            encode_name(&mut out, name);
            encode_name(&mut out, new_name);
        }
        Notification::List { friends } => out.extend_from_slice(&encode_friend_list(friends)),
        Notification::Online {
            name,
            guid,
            is_local,
        } => {
            encode_name(&mut out, name);
            out.extend_from_slice(&guid.to_le_bytes());
            out.push(u8::from(*is_local));
        }
        Notification::Offline { guid } | Notification::Remove { guid } => {
            out.extend_from_slice(&guid.to_le_bytes());
        }
        Notification::UpdatePositions { entries } => {
            out.extend_from_slice(&(entries.len() as i32).to_le_bytes());
            for entry in entries {
                out.extend_from_slice(&entry.guid.to_le_bytes());
                encode_name(&mut out, &entry.name);
                out.push(u8::from(entry.in_encounter));
                if !entry.in_encounter {
                    out.extend_from_slice(&entry.location_x.to_le_bytes());
                    out.extend_from_slice(&entry.location_z.to_le_bytes());
                }
            }
        }
        Notification::Add { entry } => encode_friend_entry(&mut out, entry),
        Notification::Status { name, guid, status } => {
            encode_name(&mut out, name);
            out.extend_from_slice(&guid.to_le_bytes());
            encode_status(&mut out, status);
        }
        Notification::Rename { name, guid } => {
            encode_name(&mut out, name);
            out.extend_from_slice(&guid.to_le_bytes());
        }
    }
    out
}

/// Encode a friend roster body: `i32 count` then one `FriendData` per row.
pub fn encode_friend_list(friends: &[FriendEntry]) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(&(friends.len() as i32).to_le_bytes());
    for entry in friends {
        encode_friend_entry(&mut out, entry);
    }
    out
}

/// Decode a friend roster body, rejecting malformed or over-bound input.
pub fn decode_friend_list(body: &[u8]) -> Result<Vec<FriendEntry>, FriendError> {
    let mut reader = Reader::new(body);
    let count = reader.i32()?;
    if count < 0 || count as usize > MAX_ROSTER_ENTRIES {
        return Err(FriendError::RosterOutOfBounds(count));
    }
    let mut friends = Vec::with_capacity(count as usize);
    for _ in 0..count {
        friends.push(decode_friend_entry(&mut reader)?);
    }
    reader.end()?;
    Ok(friends)
}

/// Decode a client request from family [`COMMAND_FAMILY`].
pub fn decode_command(sub: u16, body: &[u8]) -> Result<FriendRequest, FriendError> {
    let command = FriendCommand::from_code(sub).ok_or(FriendError::UnknownSubOpcode(sub))?;
    let mut reader = Reader::new(body);
    let request = match command {
        FriendCommand::AddFriendRequest => {
            let name = reader.string()?;
            if name.trim().is_empty() {
                return Err(FriendError::EmptyName);
            }
            FriendRequest::AddFriend { name }
        }
        FriendCommand::RemoveFriendRequest => FriendRequest::RemoveFriend {
            name: reader.name()?,
        },
        FriendCommand::ConfirmFriendResponse => {
            let guid = reader.u64()?;
            let code = reader.i32()?;
            let status =
                ConfirmStatus::from_code(code).ok_or(FriendError::InvalidConfirmStatus(code))?;
            let name = reader.name()?;
            FriendRequest::ConfirmFriend { guid, status, name }
        }
        FriendCommand::FriendsPositionRequest => FriendRequest::FriendsPositionRequest,
        FriendCommand::ConfirmFriendRequest | FriendCommand::IgnoreRequest => {
            return Err(FriendError::UnrecoveredBody(command.class_name()));
        }
    };
    reader.end()?;
    Ok(request)
}

fn encode_name(out: &mut Vec<u8>, name: &NameData) {
    out.extend_from_slice(&name.first_name_id.to_le_bytes());
    out.extend_from_slice(&name.last_name_prefix_id.to_le_bytes());
    out.extend_from_slice(&name.last_name_suffix_id.to_le_bytes());
    encode_string(out, &name.first_name);
    encode_string(out, &name.last_name);
}

fn encode_string(out: &mut Vec<u8>, value: &str) {
    out.extend_from_slice(&(value.len() as i32).to_le_bytes());
    out.extend_from_slice(value.as_bytes());
}

fn encode_status(out: &mut Vec<u8>, status: &FriendStatus) {
    for value in [
        status.status,
        status.current_mini_game_type,
        status.current_mini_game_name_id,
        status.profile_id,
        status.profile_name_id,
        status.profile_icon_id,
        status.profile_background_image_id,
        status.profile_rank,
    ] {
        out.extend_from_slice(&value.to_le_bytes());
    }
}

fn encode_friend_entry(out: &mut Vec<u8>, entry: &FriendEntry) {
    encode_name(out, &entry.name);
    out.extend_from_slice(&entry.guid.to_le_bytes());
    out.push(u8::from(entry.online));
    if entry.online {
        out.push(u8::from(entry.in_encounter));
        encode_status(out, &entry.status);
        if !entry.in_encounter {
            out.extend_from_slice(&entry.location_x.to_le_bytes());
            out.extend_from_slice(&entry.location_z.to_le_bytes());
        }
        out.extend_from_slice(&entry.popularity.to_le_bytes());
        out.push(u8::from(entry.is_local));
    } else {
        out.extend_from_slice(&entry.offline_unknown.to_le_bytes());
    }
    out.push(u8::from(entry.is_in_static_zone));
}

fn decode_friend_entry(reader: &mut Reader<'_>) -> Result<FriendEntry, FriendError> {
    let name = reader.name()?;
    let guid = reader.u64()?;
    let online = reader.boolean()?;
    let mut entry = FriendEntry {
        name,
        guid,
        online,
        in_encounter: false,
        status: FriendStatus::default(),
        location_x: 0.0,
        location_z: 0.0,
        popularity: 0,
        is_local: false,
        is_in_static_zone: false,
        offline_unknown: 0,
    };
    if online {
        entry.in_encounter = reader.boolean()?;
        entry.status = reader.status()?;
        if !entry.in_encounter {
            entry.location_x = reader.f32()?;
            entry.location_z = reader.f32()?;
        }
        entry.popularity = reader.i32()?;
        entry.is_local = reader.boolean()?;
    } else {
        entry.offline_unknown = reader.u64()?;
    }
    entry.is_in_static_zone = reader.boolean()?;
    Ok(entry)
}

/// Bounded little-endian reader shared by the friend codecs.
struct Reader<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> Reader<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    fn take(&mut self, count: usize) -> Result<&'a [u8], FriendError> {
        let end = self
            .offset
            .checked_add(count)
            .ok_or(FriendError::Truncated)?;
        if end > self.bytes.len() {
            return Err(FriendError::Truncated);
        }
        let slice = &self.bytes[self.offset..end];
        self.offset = end;
        Ok(slice)
    }

    fn end(&self) -> Result<(), FriendError> {
        if self.offset == self.bytes.len() {
            Ok(())
        } else {
            Err(FriendError::TrailingBytes(self.bytes.len() - self.offset))
        }
    }

    fn u8(&mut self) -> Result<u8, FriendError> {
        Ok(self.take(1)?[0])
    }

    fn boolean(&mut self) -> Result<bool, FriendError> {
        match self.u8()? {
            0 => Ok(false),
            1 => Ok(true),
            other => Err(FriendError::InvalidBool(other)),
        }
    }

    fn i32(&mut self) -> Result<i32, FriendError> {
        let bytes = self.take(4)?;
        Ok(i32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    fn u64(&mut self) -> Result<u64, FriendError> {
        let bytes = self.take(8)?;
        let mut raw = [0u8; 8];
        raw.copy_from_slice(bytes);
        Ok(u64::from_le_bytes(raw))
    }

    fn f32(&mut self) -> Result<f32, FriendError> {
        let bytes = self.take(4)?;
        let value = f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        if value.is_finite() {
            Ok(value)
        } else {
            Err(FriendError::NonFiniteCoordinate)
        }
    }

    fn string(&mut self) -> Result<String, FriendError> {
        let length = self.i32()?;
        if length < 0 {
            return Err(FriendError::InvalidLength(length));
        }
        let length = length as usize;
        if length > MAX_NAME_BYTES {
            return Err(FriendError::StringTooLong(length));
        }
        let bytes = self.take(length)?;
        String::from_utf8(bytes.to_vec()).map_err(|_| FriendError::InvalidUtf8)
    }

    fn name(&mut self) -> Result<NameData, FriendError> {
        Ok(NameData {
            first_name_id: self.i32()?,
            last_name_prefix_id: self.i32()?,
            last_name_suffix_id: self.i32()?,
            first_name: self.string()?,
            last_name: self.string()?,
        })
    }

    fn status(&mut self) -> Result<FriendStatus, FriendError> {
        Ok(FriendStatus {
            status: self.i32()?,
            current_mini_game_type: self.i32()?,
            current_mini_game_name_id: self.i32()?,
            profile_id: self.i32()?,
            profile_name_id: self.i32()?,
            profile_icon_id: self.i32()?,
            profile_background_image_id: self.i32()?,
            profile_rank: self.i32()?,
        })
    }
}

/// One registered character plus the live state the roster rows read from.
#[derive(Clone, Debug)]
struct Account {
    name: NameData,
    online: bool,
    status: FriendStatus,
    position: Option<(f32, f32, bool)>,
}

/// RECONSTRUCTED roster/request policy over the recovered wire.
///
/// The registry owns no I/O: every mutating call returns the packets the caller must send, so the
/// gateway can keep serializing shared session state. Identity is a `u64` character guid; names are
/// resolved through [`FriendRegistry::register`].
#[derive(Default)]
pub struct FriendRegistry {
    accounts: BTreeMap<u64, Account>,
    rosters: BTreeMap<u64, BTreeMap<u64, FriendEntry>>,
    pending: BTreeSet<(u64, u64)>,
    ignores: BTreeMap<u64, BTreeSet<u64>>,
}

impl FriendRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register or refresh one online character. Unknown targets are rejected by every request.
    pub fn register(&mut self, guid: u64, name: NameData) {
        let account = self.accounts.entry(guid).or_insert_with(|| Account {
            name: name.clone(),
            online: false,
            status: FriendStatus::default(),
            position: None,
        });
        account.name = name.clone();
        for roster in self.rosters.values_mut() {
            if let Some(entry) = roster.get_mut(&guid) {
                entry.name = name.clone();
            }
        }
    }

    pub fn display_name(&self, guid: u64) -> Option<String> {
        self.accounts
            .get(&guid)
            .map(|account| account.name.full_name())
    }

    pub fn is_friend(&self, guid: u64, other: u64) -> bool {
        self.rosters
            .get(&guid)
            .is_some_and(|roster| roster.contains_key(&other))
    }

    pub fn friend_list(&self, guid: u64) -> Result<Vec<FriendEntry>, FriendDenial> {
        if !self.accounts.contains_key(&guid) {
            return Err(FriendDenial::UnknownSender);
        }
        Ok(self
            .rosters
            .get(&guid)
            .map(|roster| roster.values().cloned().collect())
            .unwrap_or_default())
    }

    /// Ignore gating is RECONSTRUCTED; the original ignore wire ([`IGNORE_FAMILY`]) is a gap.
    pub fn ignore(&mut self, guid: u64, target: u64) {
        self.ignores.entry(guid).or_default().insert(target);
    }

    pub fn unignore(&mut self, guid: u64, target: u64) {
        if let Some(ignores) = self.ignores.get_mut(&guid) {
            ignores.remove(&target);
        }
    }

    pub fn add_request(&mut self, from: u64, to: u64) -> Result<Vec<Delivery>, FriendDenial> {
        let sender = self
            .accounts
            .get(&from)
            .ok_or(FriendDenial::UnknownSender)?;
        let sender_name = sender.name.clone();
        let target = self.accounts.get(&to).ok_or(FriendDenial::UnknownTarget)?;
        let target_name = target.name.clone();
        if from == to {
            return Err(FriendDenial::SelfRequest);
        }
        if self.is_friend(from, to) {
            return Err(FriendDenial::AlreadyFriends);
        }
        if self
            .ignores
            .get(&to)
            .is_some_and(|ignores| ignores.contains(&from))
        {
            return Err(FriendDenial::TargetIgnoresSender);
        }
        if self.pending.contains(&(from, to)) || self.pending.contains(&(to, from)) {
            return Err(FriendDenial::AlreadyPending);
        }
        self.pending.insert((from, to));
        Ok(vec![
            Delivery::friend(
                from,
                Notification::Message {
                    message_type: FriendMessageType::AddRequested,
                    guid: to,
                    name: target_name,
                    new_name: NameData::default(),
                },
            ),
            Delivery::command(
                to,
                FriendCommand::ConfirmFriendRequest,
                confirm_request_body(from, &sender_name),
            ),
        ])
    }

    pub fn confirm(
        &mut self,
        responder: u64,
        requester: u64,
        status: ConfirmStatus,
    ) -> Result<Vec<Delivery>, FriendDenial> {
        if !self.accounts.contains_key(&responder) {
            return Err(FriendDenial::UnknownSender);
        }
        if !self.accounts.contains_key(&requester) {
            return Err(FriendDenial::UnknownTarget);
        }
        if !self.pending.remove(&(requester, responder)) {
            return Err(FriendDenial::NoPendingRequest);
        }

        let responder_name = self.accounts[&responder].name.clone();
        let mut deliveries = vec![Delivery::friend(
            requester,
            Notification::Message {
                message_type: status.answer_message(),
                guid: responder,
                name: responder_name,
                new_name: NameData::default(),
            },
        )];

        if status == ConfirmStatus::Accepted {
            // RECONSTRUCTED: both characters are in the world when the request is answered, so
            // the accepted rows start online. An original server never confirmed this rule.
            for guid in [requester, responder] {
                if let Some(account) = self.accounts.get_mut(&guid) {
                    account.online = true;
                }
            }
            let requester_entry = self.build_entry(requester);
            let responder_entry = self.build_entry(responder);
            if let (Some(requester_entry), Some(responder_entry)) =
                (requester_entry, responder_entry)
            {
                self.rosters
                    .entry(responder)
                    .or_default()
                    .insert(requester, requester_entry.clone());
                self.rosters
                    .entry(requester)
                    .or_default()
                    .insert(responder, responder_entry.clone());
                deliveries.push(Delivery::friend(
                    responder,
                    Notification::Add {
                        entry: requester_entry,
                    },
                ));
                deliveries.push(Delivery::friend(
                    requester,
                    Notification::Add {
                        entry: responder_entry,
                    },
                ));
            }
        }

        Ok(deliveries)
    }

    pub fn remove(&mut self, guid: u64, target: u64) -> Result<Vec<Delivery>, FriendDenial> {
        if !self.accounts.contains_key(&guid) {
            return Err(FriendDenial::UnknownSender);
        }
        if !self.is_friend(guid, target) {
            return Err(FriendDenial::NotFriends);
        }
        if let Some(roster) = self.rosters.get_mut(&guid) {
            roster.remove(&target);
        }
        if let Some(roster) = self.rosters.get_mut(&target) {
            roster.remove(&guid);
        }
        Ok(vec![
            Delivery::friend(guid, Notification::Remove { guid: target }),
            Delivery::friend(target, Notification::Remove { guid }),
        ])
    }

    /// Publish a presence change and return the notifications friends must receive.
    pub fn set_presence(&mut self, guid: u64, online: bool) -> Result<Vec<Delivery>, FriendDenial> {
        let account = self
            .accounts
            .get_mut(&guid)
            .ok_or(FriendDenial::UnknownSender)?;
        account.online = online;
        let name = account.name.clone();

        let friends = self.friend_guids(guid);
        let mut deliveries = Vec::new();
        for friend in friends {
            if let Some(roster) = self.rosters.get_mut(&friend) {
                if let Some(entry) = roster.get_mut(&guid) {
                    entry.online = online;
                }
            }
            if !self
                .accounts
                .get(&friend)
                .is_some_and(|account| account.online)
            {
                continue;
            }
            deliveries.push(Delivery::friend(
                friend,
                if online {
                    Notification::Online {
                        name: name.clone(),
                        guid,
                        is_local: true,
                    }
                } else {
                    Notification::Offline { guid }
                },
            ));
        }
        Ok(deliveries)
    }

    pub fn publish_status(
        &mut self,
        guid: u64,
        status: FriendStatus,
    ) -> Result<Vec<Delivery>, FriendDenial> {
        let account = self
            .accounts
            .get_mut(&guid)
            .ok_or(FriendDenial::UnknownSender)?;
        account.status = status;
        let name = account.name.clone();

        let friends = self.friend_guids(guid);
        let mut deliveries = Vec::new();
        for friend in friends {
            if let Some(roster) = self.rosters.get_mut(&friend) {
                if let Some(entry) = roster.get_mut(&guid) {
                    entry.status = status;
                }
            }
            if !self
                .accounts
                .get(&friend)
                .is_some_and(|account| account.online)
            {
                continue;
            }
            deliveries.push(Delivery::friend(
                friend,
                Notification::Status {
                    name: name.clone(),
                    guid,
                    status,
                },
            ));
        }
        Ok(deliveries)
    }

    pub fn update_position(
        &mut self,
        guid: u64,
        x: f32,
        z: f32,
        in_encounter: bool,
    ) -> Result<(), FriendDenial> {
        if !x.is_finite() || !z.is_finite() {
            return Err(FriendDenial::NonFinitePosition);
        }
        let account = self
            .accounts
            .get_mut(&guid)
            .ok_or(FriendDenial::UnknownSender)?;
        account.position = Some((x, z, in_encounter));

        for friend in self.friend_guids(guid) {
            if let Some(roster) = self.rosters.get_mut(&friend) {
                if let Some(entry) = roster.get_mut(&guid) {
                    entry.in_encounter = in_encounter;
                    if !in_encounter {
                        entry.location_x = x;
                        entry.location_z = z;
                    }
                }
            }
        }
        Ok(())
    }

    /// INFERRED: the position reply lists friends whose account is online; an encounter friend is
    /// reported with the flag instead of coordinates.
    pub fn positions_for(&self, guid: u64) -> Result<Vec<PositionEntry>, FriendDenial> {
        if !self.accounts.contains_key(&guid) {
            return Err(FriendDenial::UnknownSender);
        }
        Ok(self
            .friend_guids(guid)
            .into_iter()
            .filter_map(|friend| {
                let account = self.accounts.get(&friend)?;
                if !account.online {
                    return None;
                }
                let (x, z, in_encounter) = account.position.unwrap_or((0.0, 0.0, false));
                Some(PositionEntry {
                    guid: friend,
                    name: account.name.clone(),
                    in_encounter,
                    location_x: x,
                    location_z: z,
                })
            })
            .collect())
    }

    pub fn rename(&mut self, guid: u64, new_name: &str) -> Result<Vec<Delivery>, FriendDenial> {
        if new_name.trim().is_empty() || new_name.len() > MAX_NAME_BYTES {
            return Err(FriendDenial::InvalidName);
        }
        let account = self
            .accounts
            .get_mut(&guid)
            .ok_or(FriendDenial::UnknownSender)?;
        account.name = NameData::new(new_name, "");
        let name = account.name.clone();

        let friends = self.friend_guids(guid);
        let mut deliveries = Vec::new();
        for friend in friends {
            if let Some(roster) = self.rosters.get_mut(&friend) {
                if let Some(entry) = roster.get_mut(&guid) {
                    entry.name = name.clone();
                }
            }
            if !self
                .accounts
                .get(&friend)
                .is_some_and(|account| account.online)
            {
                continue;
            }
            deliveries.push(Delivery::friend(
                friend,
                Notification::Rename {
                    name: name.clone(),
                    guid,
                },
            ));
        }
        Ok(deliveries)
    }

    fn friend_guids(&self, guid: u64) -> Vec<u64> {
        self.rosters
            .get(&guid)
            .map(|roster| roster.keys().copied().collect())
            .unwrap_or_default()
    }

    fn build_entry(&self, friend: u64) -> Option<FriendEntry> {
        let account = self.accounts.get(&friend)?;
        let (location_x, location_z, in_encounter) = account.position.unwrap_or((0.0, 0.0, false));
        Some(FriendEntry {
            name: account.name.clone(),
            guid: friend,
            online: true,
            in_encounter,
            status: account.status,
            location_x,
            location_z,
            popularity: 0,
            is_local: true,
            is_in_static_zone: true,
            offline_unknown: 0,
        })
    }
}

/// INFERRED: `CommandPacketConfirmFriendRequest` body: `u64 guid | NameData | bool`.
fn confirm_request_body(guid: u64, name: &NameData) -> Vec<u8> {
    let mut body = Vec::new();
    body.extend_from_slice(&guid.to_le_bytes());
    encode_name(&mut body, name);
    body.push(0);
    body
}
