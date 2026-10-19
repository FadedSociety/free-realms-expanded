//! Per-character friends and ignore lists, presence fanout, and volatile groups.
//!
//! Client facts come from the readers and senders cited beside each packet builder. Server-side
//! membership, persistence format, capacity, invite expiry, and ignore enforcement are (R).
//! Unknown group fields stay opaque; member rows retain recovered client-constructor defaults.

use crate::{character_store, social_chat::IgnoreLists};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::File,
    io::{self, Read},
    path::Path,
};

const SOCIAL_SCHEMA_VERSION: u32 = 1;
const MAX_SOCIAL_FILE_BYTES: usize = 1 << 20;
// (R) The client length-bounds names but exposes no maximum; this local cap protects serializers.
const MAX_NAME_BYTES: usize = 128;

pub const FRIEND_FAMILY: u16 = 74;
pub const COMMAND_FAMILY: u16 = 26;
pub const IGNORE_FAMILY: u16 = 112;
pub const GROUP_FAMILY: u16 = 40;

/// (R) Map values share the 1,000,000 magnitude bound the other position checks use.
pub const MAX_MAP_COORD: i32 = 1_000_000;
/// (R) One 74/5 update never carries more than this many friends (entries are 21 bytes).
pub const MAX_POSITION_ENTRIES: usize = 512;
/// (R) Default lifetime of an unanswered group invite, overridable by server configuration.
pub const DEFAULT_GROUP_INVITE_TTL_MS: u64 = 60_000;
const MIN_GROUP_INVITE_TTL_MS: u64 = 1_000;
const MAX_GROUP_INVITE_TTL_MS: u64 = 600_000;

/// The original client has no recovered friend-list cap; this module adds no cap.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SocialName {
    pub first: String,
    pub last: String,
}

impl SocialName {
    pub fn new(first: impl Into<String>, last: impl Into<String>) -> Result<Self, SocialError> {
        let name = Self {
            first: first.into(),
            last: last.into(),
        };
        name.validate()?;
        Ok(name)
    }

    pub fn display(&self) -> String {
        [self.first.trim(), self.last.trim()]
            .into_iter()
            .filter(|part| !part.is_empty())
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// (R) Require a first name and reject control characters or parts over the local 128-byte cap.
    fn validate(&self) -> Result<(), SocialError> {
        if self.first.trim().is_empty()
            || self.first.len() > MAX_NAME_BYTES
            || self.last.len() > MAX_NAME_BYTES
            || self
                .first
                .chars()
                .chain(self.last.chars())
                .any(char::is_control)
        {
            return Err(SocialError::InvalidName);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CharacterIdentity {
    pub account_id: u64,
    pub character_guid: u64,
    pub name: SocialName,
}

impl CharacterIdentity {
    fn validate(&self) -> Result<(), SocialError> {
        if self.account_id == 0 || self.character_guid == 0 {
            return Err(SocialError::InvalidIdentity);
        }
        self.name.validate()
    }
}

/// Account/name storage implements this lookup before calling social request handlers.
pub trait CharacterLookup {
    fn find_by_name(&self, name: &SocialName) -> Option<CharacterIdentity>;
}

/// Persisted document for exactly one character. Pending requests and groups are volatile.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SocialLists {
    pub account_id: u64,
    pub character_guid: u64,
    pub name: SocialName,
    pub friends: BTreeMap<u64, SocialName>,
    pub ignored: BTreeMap<u64, SocialName>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SocialDocument {
    schema_version: u32,
    profile: SocialLists,
}

impl SocialLists {
    pub fn new(identity: CharacterIdentity) -> Result<Self, SocialError> {
        identity.validate()?;
        Ok(Self {
            account_id: identity.account_id,
            character_guid: identity.character_guid,
            name: identity.name,
            friends: BTreeMap::new(),
            ignored: BTreeMap::new(),
        })
    }

    pub fn identity(&self) -> CharacterIdentity {
        CharacterIdentity {
            account_id: self.account_id,
            character_guid: self.character_guid,
            name: self.name.clone(),
        }
    }

    pub fn save(&self, path: &Path) -> io::Result<()> {
        self.validate().map_err(invalid_data)?;
        if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent)?;
        }
        let bytes = serde_json::to_vec_pretty(&SocialDocument {
            schema_version: SOCIAL_SCHEMA_VERSION,
            profile: self.clone(),
        })
        .map_err(invalid_data)?;
        if bytes.len() > MAX_SOCIAL_FILE_BYTES {
            return Err(invalid_data("social list exceeds 1 MiB"));
        }
        character_store::replace_bytes(path, &bytes)
    }

    pub fn load(path: &Path, account_id: u64, character_guid: u64) -> io::Result<Self> {
        let mut file = File::open(path)?;
        let mut bytes = Vec::new();
        Read::by_ref(&mut file)
            .take((MAX_SOCIAL_FILE_BYTES + 1) as u64)
            .read_to_end(&mut bytes)?;
        if bytes.len() > MAX_SOCIAL_FILE_BYTES {
            return Err(invalid_data("social list exceeds 1 MiB"));
        }
        let document: SocialDocument = serde_json::from_slice(&bytes).map_err(invalid_data)?;
        if document.schema_version != SOCIAL_SCHEMA_VERSION {
            return Err(invalid_data("unsupported social list schema version"));
        }
        document.profile.validate().map_err(invalid_data)?;
        if document.profile.account_id != account_id
            || document.profile.character_guid != character_guid
        {
            return Err(invalid_data("social list belongs to another character"));
        }
        Ok(document.profile)
    }

    fn validate(&self) -> Result<(), SocialError> {
        CharacterIdentity {
            account_id: self.account_id,
            character_guid: self.character_guid,
            name: self.name.clone(),
        }
        .validate()?;
        for (guid, name) in self.friends.iter().chain(&self.ignored) {
            if *guid == 0 || *guid == self.character_guid {
                return Err(SocialError::InvalidIdentity);
            }
            name.validate()?;
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(i8)]
pub enum FriendResultCode {
    Accepted = 1,
    Declined = 2,
    TimedOut = 3,
    CannotTeleportCombat = 4,
    RequestSent = 5,
    FriendOffline = 6,
    TeleportedCombat = 7,
    NoFriendSpam = 8,
    NameChanged = 9,
    CannotTeleportPlayer = 10,
    CannotTeleportMiniGame = 11,
}

impl FriendResultCode {
    pub const ALL: [Self; 11] = [
        Self::Accepted,
        Self::Declined,
        Self::TimedOut,
        Self::CannotTeleportCombat,
        Self::RequestSent,
        Self::FriendOffline,
        Self::TeleportedCombat,
        Self::NoFriendSpam,
        Self::NameChanged,
        Self::CannotTeleportPlayer,
        Self::CannotTeleportMiniGame,
    ];

    pub const fn wire(self) -> i8 {
        self as i8
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SocialMessage {
    /// Character guid is the routing key; the lead maps it to the live session.
    pub recipient_guid: u64,
    /// Family payload, before the gateway's tunnel envelope.
    pub bytes: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SocialError {
    InvalidIdentity,
    InvalidName,
    MissingProfile,
    UnknownName,
    SelfFriend,
    AlreadyFriend,
    NotFriend,
    IgnoredByTarget,
    AlreadyIgnored,
    NotIgnored,
    NoPendingFriendRequest,
    InvalidGroupLimit,
    InvalidInviteTtl,
    NotInGroup,
    InvalidPosition,
}

impl std::fmt::Display for SocialError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::InvalidIdentity => "social identity is invalid",
            Self::InvalidName => "social name is invalid",
            Self::MissingProfile => "social profile is not loaded",
            Self::UnknownName => "character name was not found",
            Self::SelfFriend => "cannot add yourself as a friend",
            Self::AlreadyFriend => "character is already a friend",
            Self::NotFriend => "character is not a friend",
            Self::IgnoredByTarget => "target ignores the requester",
            Self::AlreadyIgnored => "character is already ignored",
            Self::NotIgnored => "character is not ignored",
            Self::NoPendingFriendRequest => "friend request is not pending",
            Self::InvalidGroupLimit => "group limit must be at least two",
            Self::InvalidInviteTtl => "group invite lifetime must be 1000..=600000 ms",
            Self::NotInGroup => "character is not in a group",
            Self::InvalidPosition => "map position is not finite or is out of bounds",
        })
    }
}

impl std::error::Error for SocialError {}

#[derive(Clone, Debug)]
struct PendingFriendRequest {
    requester_guid: u64,
    target_guid: u64,
}

#[derive(Clone, Debug)]
struct Group {
    leader_guid: u64,
    members: Vec<u64>,
}

#[derive(Clone, Copy, Debug)]
struct PendingGroupInvite {
    group_id: u32,
    inviter_guid: u64,
    created_ms: u64,
}

/// Presence is runtime-only. The caller sends login packets after self-data has initialized the UI.
#[derive(Clone, Debug)]
pub struct SocialService {
    profiles: BTreeMap<u64, SocialLists>,
    online: BTreeSet<u64>,
    friend_requests: BTreeMap<(u64, u64), PendingFriendRequest>,
    max_group_members: usize,
    /// Host policy copied with the service; never persisted and not part of the saved profile.
    group_invite_ttl_ms: u64,
    next_group_id: u32,
    groups: BTreeMap<u32, Group>,
    member_group: BTreeMap<u64, u32>,
    group_invites: BTreeMap<u64, PendingGroupInvite>,
    /// Volatile 40/14 state: members whose job picker is open (HUD flag at member +0xc4).
    changing_job: BTreeSet<u64>,
}

impl SocialService {
    /// Group capacity is reconstructed policy (R), supplied by the host configuration.
    pub fn new(max_group_members: usize) -> Result<Self, SocialError> {
        if max_group_members < 2 {
            return Err(SocialError::InvalidGroupLimit);
        }
        Ok(Self {
            profiles: BTreeMap::new(),
            online: BTreeSet::new(),
            friend_requests: BTreeMap::new(),
            max_group_members,
            group_invite_ttl_ms: DEFAULT_GROUP_INVITE_TTL_MS,
            next_group_id: 1,
            groups: BTreeMap::new(),
            member_group: BTreeMap::new(),
            group_invites: BTreeMap::new(),
            changing_job: BTreeSet::new(),
        })
    }

    pub fn group_invite_ttl_ms(&self) -> u64 {
        self.group_invite_ttl_ms
    }

    /// Set the bounded server policy before the server accepts sessions.
    pub fn set_group_invite_ttl(&mut self, ttl_ms: u64) -> Result<(), SocialError> {
        if !(MIN_GROUP_INVITE_TTL_MS..=MAX_GROUP_INVITE_TTL_MS).contains(&ttl_ms) {
            return Err(SocialError::InvalidInviteTtl);
        }
        self.group_invite_ttl_ms = ttl_ms;
        Ok(())
    }

    pub fn register(&mut self, profile: SocialLists) -> Result<(), SocialError> {
        profile.validate()?;
        let guid = profile.character_guid;
        if self.profiles.contains_key(&guid) {
            return Err(SocialError::InvalidIdentity);
        }
        self.profiles.insert(guid, profile);
        Ok(())
    }

    pub fn profile(&self, guid: u64) -> Option<&SocialLists> {
        self.profiles.get(&guid)
    }

    pub fn profile_mut(&mut self, guid: u64) -> Option<&mut SocialLists> {
        self.profiles.get_mut(&guid)
    }

    pub fn is_online(&self, guid: u64) -> bool {
        self.online.contains(&guid)
    }

    /// (R) Call after self-data (message id 12); ordering after that message is local policy.
    /// The 74/1 list includes online flags; 112/1 sends the ignore rows.
    pub fn login(&mut self, guid: u64) -> Result<Vec<SocialMessage>, SocialError> {
        let profile = self
            .profiles
            .get(&guid)
            .ok_or(SocialError::MissingProfile)?;
        if !self.online.insert(guid) {
            return Ok(Vec::new());
        }
        let friends = profile.friends.clone();
        let ignores = profile.ignored.clone();
        let mut out = vec![
            message(guid, encode_friend_list(&friends, &self.online)?),
            message(guid, encode_ignore_list(&ignores)?),
        ];
        for friend_guid in friends.keys().filter(|friend| self.online.contains(friend)) {
            let friend_name = self
                .profiles
                .get(friend_guid)
                .map(|p| &p.name)
                .unwrap_or(&friends[friend_guid]);
            out.push(message(
                guid,
                encode_friend_online(*friend_guid, friend_name)?,
            ));
        }
        let online_friends: Vec<u64> = self
            .profiles
            .iter()
            .filter(|(_, profile)| profile.friends.contains_key(&guid))
            .map(|(friend_guid, _)| *friend_guid)
            .filter(|friend_guid| self.online.contains(friend_guid))
            .collect();
        for friend_guid in online_friends {
            out.push(message(
                friend_guid,
                encode_friend_online(guid, &self.profiles[&guid].name)?,
            ));
        }
        Ok(out)
    }

    /// Family 74/3 is sent only to online friends. Repeated logout is harmless.
    pub fn logout(&mut self, guid: u64) -> Result<Vec<SocialMessage>, SocialError> {
        let profile = self
            .profiles
            .get(&guid)
            .ok_or(SocialError::MissingProfile)?;
        if !self.online.remove(&guid) {
            return Ok(Vec::new());
        }
        let recipients: Vec<u64> = profile
            .friends
            .keys()
            .copied()
            .filter(|friend| self.online.contains(friend))
            .collect();
        let packet = encode_friend_offline(guid);
        Ok(recipients
            .into_iter()
            .map(|recipient_guid| message(recipient_guid, packet.clone()))
            .collect())
    }

    /// The client sends 26/14; the server replies with 74/8 code 5 and opens a 26/16 GAQ invite.
    pub fn request_friend<L: CharacterLookup>(
        &mut self,
        requester_guid: u64,
        target_name: &SocialName,
        lookup: &L,
    ) -> Result<Vec<SocialMessage>, SocialError> {
        let requester = self
            .profiles
            .get(&requester_guid)
            .ok_or(SocialError::MissingProfile)?
            .clone();
        let target = lookup
            .find_by_name(target_name)
            .ok_or(SocialError::UnknownName)?;
        target.validate()?;
        let target_profile = self
            .profiles
            .get(&target.character_guid)
            .ok_or(SocialError::MissingProfile)?;
        if requester_guid == target.character_guid {
            return Err(SocialError::SelfFriend);
        }
        if requester.friends.contains_key(&target.character_guid)
            || target_profile.friends.contains_key(&requester_guid)
        {
            return Err(SocialError::AlreadyFriend);
        }
        if target_profile.ignored.contains_key(&requester_guid) {
            // (R) Suppress requests from an ignored character.
            return Err(SocialError::IgnoredByTarget);
        }
        let key = (requester_guid, target.character_guid);
        if self.friend_requests.contains_key(&key) {
            // (R) Map a repeated outstanding request to the client's no-friend-spam result.
            return Ok(vec![message(
                requester_guid,
                encode_friend_status(
                    FriendResultCode::NoFriendSpam,
                    target.character_guid,
                    &requester.name,
                    &target.name,
                )?,
            )]);
        }
        self.friend_requests.insert(
            key,
            PendingFriendRequest {
                requester_guid,
                target_guid: target.character_guid,
            },
        );
        let mut out = vec![message(
            requester_guid,
            encode_friend_status(
                FriendResultCode::RequestSent,
                target.character_guid,
                &requester.name,
                &target.name,
            )?,
        )];
        if self.online.contains(&target.character_guid) {
            out.push(message(
                target.character_guid,
                encode_friend_invite(&requester)?,
            ));
        }
        Ok(out)
    }

    /// AutoRefuse friend option 0 answers through the same 26/17 response path.
    pub fn respond_friend(
        &mut self,
        target_guid: u64,
        requester_guid: u64,
        accepted: bool,
    ) -> Result<Vec<SocialMessage>, SocialError> {
        let key = (requester_guid, target_guid);
        let request = self
            .friend_requests
            .remove(&key)
            .ok_or(SocialError::NoPendingFriendRequest)?;
        let requester = self
            .profiles
            .get(&request.requester_guid)
            .ok_or(SocialError::MissingProfile)?
            .clone();
        let target = self
            .profiles
            .get(&request.target_guid)
            .ok_or(SocialError::MissingProfile)?
            .clone();
        if !accepted {
            return Ok(vec![message(
                requester_guid,
                encode_friend_status(
                    FriendResultCode::Declined,
                    target_guid,
                    &requester.name,
                    &target.name,
                )?,
            )]);
        }
        self.profiles
            .get_mut(&requester_guid)
            .ok_or(SocialError::MissingProfile)?
            .friends
            .insert(target_guid, target.name.clone());
        self.profiles
            .get_mut(&target_guid)
            .ok_or(SocialError::MissingProfile)?
            .friends
            .insert(requester_guid, requester.name.clone());
        let mut out = vec![
            message(
                requester_guid,
                encode_friend_add(&target, self.online.contains(&target_guid))?,
            ),
            message(
                target_guid,
                encode_friend_add(&requester, self.online.contains(&requester_guid))?,
            ),
            message(
                requester_guid,
                encode_friend_status(
                    FriendResultCode::Accepted,
                    target_guid,
                    &requester.name,
                    &target.name,
                )?,
            ),
        ];
        if self.online.contains(&requester_guid) && self.online.contains(&target_guid) {
            out.push(message(
                requester_guid,
                encode_friend_online(target_guid, &target.name)?,
            ));
            out.push(message(
                target_guid,
                encode_friend_online(requester_guid, &requester.name)?,
            ));
        }
        Ok(out)
    }

    /// Reconstructed timeout policy (R); the host timer calls this after its chosen deadline.
    pub fn timeout_friend_request(
        &mut self,
        requester_guid: u64,
        target_guid: u64,
    ) -> Result<Vec<SocialMessage>, SocialError> {
        let request = self
            .friend_requests
            .remove(&(requester_guid, target_guid))
            .ok_or(SocialError::NoPendingFriendRequest)?;
        let requester = self
            .profiles
            .get(&request.requester_guid)
            .ok_or(SocialError::MissingProfile)?;
        let target = self
            .profiles
            .get(&request.target_guid)
            .ok_or(SocialError::MissingProfile)?;
        Ok(vec![message(
            requester_guid,
            encode_friend_status(
                FriendResultCode::TimedOut,
                target_guid,
                &requester.name,
                &target.name,
            )?,
        )])
    }

    /// 26/15 names the target; 74/7 removes a guid. Mutual removal is reconstructed as local-only (R).
    pub fn remove_friend<L: CharacterLookup>(
        &mut self,
        owner_guid: u64,
        name: &SocialName,
        lookup: &L,
    ) -> Result<Vec<SocialMessage>, SocialError> {
        let target = lookup.find_by_name(name).ok_or(SocialError::UnknownName)?;
        let owner = self
            .profiles
            .get_mut(&owner_guid)
            .ok_or(SocialError::MissingProfile)?;
        if owner.friends.remove(&target.character_guid).is_none() {
            return Err(SocialError::NotFriend);
        }
        Ok(vec![message(
            owner_guid,
            encode_friend_remove(target.character_guid),
        )])
    }

    pub fn add_ignore<L: CharacterLookup>(
        &mut self,
        owner_guid: u64,
        name: &SocialName,
        lookup: &L,
    ) -> Result<Vec<SocialMessage>, SocialError> {
        let target = lookup.find_by_name(name).ok_or(SocialError::UnknownName)?;
        let owner = self
            .profiles
            .get_mut(&owner_guid)
            .ok_or(SocialError::MissingProfile)?;
        if owner_guid == target.character_guid {
            return Err(SocialError::InvalidIdentity);
        }
        if owner.ignored.contains_key(&target.character_guid) {
            return Err(SocialError::AlreadyIgnored);
        }
        owner.ignored.insert(target.character_guid, target.name);
        Ok(vec![message(
            owner_guid,
            encode_ignore_add(target.character_guid, name)?,
        )])
    }

    pub fn remove_ignore<L: CharacterLookup>(
        &mut self,
        owner_guid: u64,
        name: &SocialName,
        lookup: &L,
    ) -> Result<Vec<SocialMessage>, SocialError> {
        let target = lookup.find_by_name(name).ok_or(SocialError::UnknownName)?;
        let owner = self
            .profiles
            .get_mut(&owner_guid)
            .ok_or(SocialError::MissingProfile)?;
        if owner.ignored.remove(&target.character_guid).is_none() {
            return Err(SocialError::NotIgnored);
        }
        Ok(vec![message(
            owner_guid,
            encode_ignore_remove(target.character_guid),
        )])
    }

    /// Reuse the same directional ignore model as `social_chat::IgnoreLists`.
    pub fn chat_ignore_lists(&self) -> IgnoreLists {
        let mut lists = IgnoreLists::default();
        for (recipient, profile) in &self.profiles {
            for sender in profile.ignored.keys() {
                lists.add(*recipient, *sender);
            }
        }
        lists
    }

    /// Family 40/1 invite and 40/2 result. Only group leaders invite (client result 5).
    pub fn invite_to_group<L: CharacterLookup>(
        &mut self,
        inviter_guid: u64,
        target_name: &SocialName,
        lookup: &L,
        now_ms: u64,
    ) -> Result<Vec<SocialMessage>, SocialError> {
        self.expire_group_invites(now_ms);
        let inviter = self
            .profiles
            .get(&inviter_guid)
            .ok_or(SocialError::MissingProfile)?
            .clone();
        let Some(target) = lookup.find_by_name(target_name) else {
            return Ok(vec![message(inviter_guid, encode_group_result(2, 1))]);
        };
        target.validate()?;
        let Some(target_profile) = self.profiles.get(&target.character_guid) else {
            return Ok(vec![message(inviter_guid, encode_group_result(2, 1))]);
        };
        if target.character_guid == inviter_guid {
            // (R) A self-invite would reserve the inviter's own slot; report the
            // client's already-grouped result before any group is created.
            return Ok(vec![message(inviter_guid, encode_group_result(2, 3))]);
        }
        if target_profile.ignored.contains_key(&inviter_guid) {
            // (R) Ignore blocks group invites. Result 2 has no client notice.
            return Ok(vec![message(inviter_guid, encode_group_result(2, 2))]);
        }
        if self.member_group.contains_key(&target.character_guid) {
            return Ok(vec![message(inviter_guid, encode_group_result(2, 3))]);
        }

        if !self.online.contains(&target.character_guid) {
            // (R) A group GAQ invite cannot be delivered while its target is offline.
            return Ok(vec![message(inviter_guid, encode_group_result(2, 1))]);
        }

        let current_group = self.member_group.get(&inviter_guid).copied();
        if let Some(group_id) = current_group {
            if self.groups[&group_id].leader_guid != inviter_guid {
                return Ok(vec![message(inviter_guid, encode_group_result(2, 5))]);
            }
        }
        if let Some(existing) = self.group_invites.get(&target.character_guid) {
            // (R) Distinguish a repeated invite from this group (6) from another group's invite (4).
            let code = if Some(existing.group_id) == current_group {
                6
            } else {
                4
            };
            return Ok(vec![message(inviter_guid, encode_group_result(2, code))]);
        }
        let members = current_group
            .map(|group_id| self.groups[&group_id].members.len())
            .unwrap_or(1);
        let open = current_group
            .map(|group_id| {
                self.group_invites
                    .values()
                    .filter(|invite| invite.group_id == group_id)
                    .count()
            })
            .unwrap_or(0);
        // (R) Pending invites reserve room so later accepts cannot exceed the configured cap.
        if members + open >= self.max_group_members {
            return Ok(vec![message(inviter_guid, encode_group_result(2, 7))]);
        }
        let (group_id, created) = match current_group {
            Some(group_id) => (group_id, false),
            None => (self.create_group(inviter_guid)?, true),
        };
        self.group_invites.insert(
            target.character_guid,
            PendingGroupInvite {
                group_id,
                inviter_guid,
                created_ms: now_ms,
            },
        );
        let invite = encode_group_invite(group_id, &inviter, &target)?;
        let mut out = Vec::new();
        if created {
            // (R) First invite creates a volatile group; the client treats its one-member row as Solo/leader.
            out.push(message(inviter_guid, self.encode_group_snapshot(group_id)?));
        }
        if self.online.contains(&target.character_guid) {
            out.push(message(target.character_guid, invite));
        }
        out.push(message(inviter_guid, encode_group_result(2, 0)));
        Ok(out)
    }

    /// AutoRefuse index 6 is client-side (`00917b60`); its false 40/4 answer follows this path.
    pub fn respond_group_invite(
        &mut self,
        target_guid: u64,
        group_id: u32,
        accepted: bool,
        now_ms: u64,
    ) -> Result<Vec<SocialMessage>, SocialError> {
        self.expire_group_invites(now_ms);
        // 40/5 body reader 00909090; dispatcher 0093daf0 case 5 maps 3 to
        // GroupAcceptFailNotInvited.
        let Some(invite) = self.group_invites.get(&target_guid).copied() else {
            return Ok(vec![message(target_guid, encode_group_result(5, 3))]);
        };
        if invite.group_id != group_id {
            // The client echoes the group id from the pop-up. Refuse a stale pop-up
            // rather than consuming a replacement invitation from another group.
            return Ok(vec![message(target_guid, encode_group_result(5, 3))]);
        }
        let Some(group) = self.groups.get(&invite.group_id) else {
            self.group_invites.remove(&target_guid);
            return Ok(vec![message(target_guid, encode_group_result(5, 1))]);
        };
        let target = self
            .profiles
            .get(&target_guid)
            .ok_or(SocialError::MissingProfile)?
            .clone();
        let inviter = self
            .profiles
            .get(&invite.inviter_guid)
            .ok_or(SocialError::MissingProfile)?
            .clone();
        if accepted && self.member_group.contains_key(&target_guid) {
            // (R) Joining another party after this invite arrived makes the old
            // invitation unactionable until that party dissolves. Do not consume
            // it or append a second roster entry. No recovered 40/5 result means
            // "already grouped", so use the existing no-group refusal.
            return Ok(vec![message(target_guid, encode_group_result(5, 1))]);
        }
        let group_data = encode_group_invite(invite.group_id, &inviter, &target.identity())?;
        self.group_invites.remove(&target_guid);
        if !accepted {
            let body = encode_group_accept_notice(&group_data, false)?;
            return Ok(group
                .members
                .iter()
                .copied()
                .map(|recipient| message(recipient, body.clone()))
                .collect());
        }
        if group.members.len() >= self.max_group_members {
            return Ok(vec![message(target_guid, encode_group_result(5, 2))]);
        }
        self.groups
            .get_mut(&invite.group_id)
            .expect("group checked above")
            .members
            .push(target_guid);
        self.member_group.insert(target_guid, invite.group_id);
        let members = self.groups[&invite.group_id].members.clone();
        let accept_packet = encode_group_accept_notice(&group_data, true)?;
        let member_record = self.group_member_record(target_guid)?;
        let member_packet = encode_group_member_update(&member_record)?;
        let update_packet = self.encode_group_snapshot(invite.group_id)?;
        let mut out = Vec::new();
        for recipient in members {
            out.push(message(recipient, accept_packet.clone()));
            out.push(message(recipient, member_packet.clone()));
            out.push(message(recipient, update_packet.clone()));
        }
        Ok(out)
    }

    /// (R) Drop expired invitations lazily before the next invite or response.
    /// Family 40 has no timeout notice, and idle cleanup has no observable effect.
    fn expire_group_invites(&mut self, now_ms: u64) {
        self.group_invites.retain(|_, invite| {
            now_ms.saturating_sub(invite.created_ms) < self.group_invite_ttl_ms
        });
    }

    /// Leave, kick and disconnect update state before emitting family-40 member-list messages.
    pub fn leave_group(&mut self, member_guid: u64) -> Result<Vec<SocialMessage>, SocialError> {
        self.remove_group_member(member_guid, false)
    }

    pub fn kick_from_group(
        &mut self,
        actor_guid: u64,
        target_guid: u64,
    ) -> Result<Vec<SocialMessage>, SocialError> {
        let Some(group_id) = self.member_group.get(&actor_guid).copied() else {
            return Ok(vec![message(actor_guid, encode_group_result(7, 1))]);
        };
        let group = &self.groups[&group_id];
        if group.leader_guid != actor_guid {
            return Ok(vec![message(actor_guid, encode_group_result(7, 2))]);
        }
        if !group.members.contains(&target_guid) || actor_guid == target_guid {
            return Ok(vec![message(actor_guid, encode_group_result(7, 3))]);
        }
        self.remove_group_member(target_guid, true)
    }

    pub fn disconnect(&mut self, guid: u64) -> Result<Vec<SocialMessage>, SocialError> {
        let mut out = self.logout(guid)?;
        // (R) Pending invites are volatile and cleared when either participant disconnects.
        self.group_invites
            .retain(|target, invite| *target != guid && invite.inviter_guid != guid);
        if self.member_group.contains_key(&guid) {
            out.extend(self.remove_group_member(guid, false)?);
        }
        self.changing_job.remove(&guid);
        // (R) Friend requests are volatile and cleared when either participant disconnects.
        self.friend_requests
            .retain(|(requester, target), _| *requester != guid && *target != guid);
        Ok(out)
    }

    pub fn group_of(&self, guid: u64) -> Option<u32> {
        self.member_group.get(&guid).copied()
    }

    pub fn group_leader(&self, group_id: u32) -> Option<u64> {
        self.groups.get(&group_id).map(|group| group.leader_guid)
    }

    pub fn group_members(&self, group_id: u32) -> Option<&[u64]> {
        self.groups
            .get(&group_id)
            .map(|group| group.members.as_slice())
    }

    pub fn group_invitee(&self, guid: u64) -> Option<(u32, u64)> {
        self.group_invites
            .get(&guid)
            .map(|invite| (invite.group_id, invite.inviter_guid))
    }

    /// 26/21 -> 74/5 for `requester` only (R scope): mutual, online, non-ignoring friends the
    /// host can place. Same zone and instance and not in battle gives the coordinates; any
    /// other online friend is sent hidden so the atlas drops a stale marker. `spot_of` must read
    /// the host's session registry, never client bytes. `None`: nothing to report.
    pub fn friend_positions(
        &self,
        requester: u64,
        spot_of: impl Fn(u64) -> Option<MapSpot>,
    ) -> Result<Option<SocialMessage>, SocialError> {
        let profile = self
            .profiles
            .get(&requester)
            .ok_or(SocialError::MissingProfile)?;
        let Some(here) = self
            .online
            .contains(&requester)
            .then(|| spot_of(requester))
            .flatten()
        else {
            return Ok(None);
        };
        let mut entries = Vec::new();
        for guid in profile.friends.keys().copied() {
            let mutual = self
                .profiles
                .get(&guid)
                .is_some_and(|friend| friend.friends.contains_key(&requester));
            if !mutual || !self.online.contains(&guid) || self.ignores_either(requester, guid) {
                continue;
            }
            let Some(spot) = spot_of(guid) else { continue };
            let visible = !spot.in_battle && spot.same_scope(&here) && spot.on_map();
            entries.push(FriendPosition {
                guid,
                location: visible.then_some([spot.x, spot.z]),
            });
            if entries.len() == MAX_POSITION_ENTRIES {
                break;
            }
        }
        if entries.is_empty() {
            return Ok(None);
        }
        Ok(Some(message(requester, encode_friend_positions(&entries)?)))
    }

    /// 40/14 -> 40/15 to the whole group, sender included (R). The flag is stored so later
    /// 40/8 and 40/9 snapshots keep it. Repeating the current state sends nothing.
    pub fn group_changing_job(
        &mut self,
        actor: u64,
        changing: bool,
    ) -> Result<Vec<SocialMessage>, SocialError> {
        let group_id = self.group_of(actor).ok_or(SocialError::NotInGroup)?;
        let members = self
            .groups
            .get(&group_id)
            .ok_or(SocialError::NotInGroup)?
            .members
            .clone();
        let changed = if changing {
            self.changing_job.insert(actor)
        } else {
            self.changing_job.remove(&actor)
        };
        if !changed {
            return Ok(Vec::new());
        }
        let packet = encode_group_changing_job(actor, changing);
        Ok(members
            .into_iter()
            .map(|member| message(member, packet.clone()))
            .collect())
    }

    /// 40/16 -> 40/16 to group members in the sender's zone and instance (R), sender included.
    /// The packet's guid is the authenticated `actor`, never the guid the client wrote.
    pub fn group_map_ping(
        &self,
        actor: u64,
        x: i32,
        z: i32,
        spot_of: impl Fn(u64) -> Option<MapSpot>,
    ) -> Result<Vec<SocialMessage>, SocialError> {
        let group_id = self.group_of(actor).ok_or(SocialError::NotInGroup)?;
        if !(coord_ok(x) && coord_ok(z)) {
            return Err(SocialError::InvalidPosition);
        }
        let Some(here) = spot_of(actor) else {
            return Ok(Vec::new());
        };
        let group = self.groups.get(&group_id).ok_or(SocialError::NotInGroup)?;
        let packet = encode_group_map_ping(actor, x, z);
        Ok(group
            .members
            .iter()
            .copied()
            .filter(|member| {
                !self.ignores_either(actor, *member)
                    && spot_of(*member).is_some_and(|spot| spot.same_scope(&here))
            })
            .map(|member| message(member, packet.clone()))
            .collect())
    }

    fn ignores_either(&self, a: u64, b: u64) -> bool {
        let ignores = |owner: u64, other: u64| {
            self.profiles
                .get(&owner)
                .is_some_and(|profile| profile.ignored.contains_key(&other))
        };
        ignores(a, b) || ignores(b, a)
    }

    fn create_group(&mut self, leader_guid: u64) -> Result<u32, SocialError> {
        let id = self.next_group_id;
        self.next_group_id = self
            .next_group_id
            .checked_add(1)
            .ok_or(SocialError::InvalidIdentity)?;
        self.groups.insert(
            id,
            Group {
                leader_guid,
                members: vec![leader_guid],
            },
        );
        self.member_group.insert(leader_guid, id);
        Ok(id)
    }

    fn group_member_record(&self, guid: u64) -> Result<GroupMemberRecord, SocialError> {
        let profile = self
            .profiles
            .get(&guid)
            .ok_or(SocialError::MissingProfile)?;
        Ok(GroupMemberRecord {
            guid,
            name: GroupRecordName {
                // (R) The three NameData metadata values are opaque; use neutral defaults.
                fields: [0; 3],
                first: profile.name.first.clone(),
                last: profile.name.last.clone(),
            },
            // Client member-record constructor `008dcd60` initializes +0x90..+0xa8 this way.
            opaque_fields: [0, 0, 0, u32::MAX, u32::MAX, 1, 0],
            // `008dcd60` clears both flags; +0xc8 starts at zero. +0xc4 follows 40/14.
            flags: [self.changing_job.contains(&guid), false],
            minigame_type: 0,
        })
    }

    fn encode_group_snapshot(&self, group_id: u32) -> Result<Vec<u8>, SocialError> {
        let group = self.groups.get(&group_id).ok_or(SocialError::NotInGroup)?;
        let mut members = Vec::with_capacity(group.members.len());
        for guid in &group.members {
            members.push(self.group_member_record(*guid)?);
        }
        encode_group_update(&members)
    }

    fn remove_group_member(
        &mut self,
        member_guid: u64,
        kicked: bool,
    ) -> Result<Vec<SocialMessage>, SocialError> {
        let group_id = self
            .member_group
            .remove(&member_guid)
            .ok_or(SocialError::NotInGroup)?;
        let mut group = self
            .groups
            .remove(&group_id)
            .ok_or(SocialError::NotInGroup)?;
        let original_members = group.members.clone();
        group.members.retain(|guid| *guid != member_guid);
        self.changing_job.remove(&member_guid);
        self.group_invites.retain(|target, invite| {
            *target != member_guid
                && invite.inviter_guid != member_guid
                && invite.group_id != group_id
        });
        let event_packet = if kicked {
            encode_group_kick(member_guid, &self.profiles[&member_guid].name)?
        } else {
            encode_group_leave(member_guid, &self.profiles[&member_guid].name)?
        };
        let mut out: Vec<SocialMessage> = original_members
            .iter()
            .copied()
            .map(|recipient| message(recipient, event_packet.clone()))
            .collect();

        if group.members.len() == 1 {
            // (R) The client guide says a group with one remaining member dissolves.
            let last = group.members[0];
            self.member_group.remove(&last);
            self.changing_job.remove(&last);
            out.push(message(
                last,
                encode_group_leave(last, &self.profiles[&last].name)?,
            ));
            return Ok(out);
        }
        if group.members.is_empty() {
            return Ok(out);
        }
        if group.leader_guid == member_guid {
            // (R) Preserve join order and transfer leadership to the oldest remaining member.
            group.leader_guid = group.members[0];
        }
        let snapshot = {
            let mut records = Vec::with_capacity(group.members.len());
            for guid in &group.members {
                records.push(self.group_member_record(*guid)?);
            }
            encode_group_update(&records)?
        };
        self.groups.insert(group_id, group.clone());
        for recipient in &group.members {
            out.push(message(*recipient, snapshot.clone()));
        }
        if group.leader_guid != member_guid {
            let leader = self.group_member_record(group.leader_guid)?;
            let update = encode_group_member_update(&leader)?;
            for recipient in group.members {
                out.push(message(recipient, update.clone()));
            }
        }
        Ok(out)
    }
}

fn invalid_data(error: impl std::fmt::Display) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, error.to_string())
}

fn message(recipient_guid: u64, bytes: Vec<u8>) -> SocialMessage {
    SocialMessage {
        recipient_guid,
        bytes,
    }
}

fn write_string(out: &mut Vec<u8>, value: &str) -> Result<(), SocialError> {
    if value.len() > i32::MAX as usize {
        return Err(SocialError::InvalidName);
    }
    out.extend_from_slice(&(value.len() as i32).to_le_bytes());
    out.extend_from_slice(value.as_bytes());
    Ok(())
}

fn write_social_name(out: &mut Vec<u8>, name: &SocialName) -> Result<(), SocialError> {
    name.validate()?;
    // (R) NameData's three metadata fields are opaque; emit zero until their values are recovered.
    out.extend_from_slice(
        &[0i32; 3]
            .into_iter()
            .flat_map(i32::to_le_bytes)
            .collect::<Vec<_>>(),
    );
    write_string(out, &name.first)?;
    write_string(out, &name.last)
}

fn friend_header(sub: u8) -> Vec<u8> {
    let mut out = FRIEND_FAMILY.to_le_bytes().to_vec();
    out.push(sub);
    out
}

fn ignore_header(sub: u8) -> Vec<u8> {
    let mut out = IGNORE_FAMILY.to_le_bytes().to_vec();
    out.push(sub);
    out
}

fn command_header(sub: u16) -> Vec<u8> {
    let mut out = COMMAND_FAMILY.to_le_bytes().to_vec();
    out.extend_from_slice(&sub.to_le_bytes());
    out
}

fn group_header(sub: u16) -> Vec<u8> {
    let mut out = GROUP_FAMILY.to_le_bytes().to_vec();
    out.extend_from_slice(&sub.to_le_bytes());
    out
}

/// 74/8 dispatcher `00c814d0` case 8 -> `00c80ab0` -> `00c80500`: byte, guid pair, two NameData blocks.
pub fn encode_friend_status(
    code: FriendResultCode,
    target_guid: u64,
    requester: &SocialName,
    target: &SocialName,
) -> Result<Vec<u8>, SocialError> {
    let mut out = friend_header(8);
    out.push(code.wire() as u8);
    out.extend_from_slice(&target_guid.to_le_bytes());
    write_social_name(&mut out, requester)?;
    write_social_name(&mut out, target)?;
    Ok(out)
}

fn encode_friend_row(guid: u64, name: &SocialName, online: bool) -> Result<Vec<u8>, SocialError> {
    let mut out = Vec::new();
    write_social_name(&mut out, name)?;
    out.extend_from_slice(&guid.to_le_bytes());
    if !online {
        out.push(0); // (R) Interpret FriendData's first status byte as offline.
        // (R) The offline row's guid-shaped field is opaque; emit zero.
        out.extend_from_slice(&0u64.to_le_bytes());
    } else {
        out.push(1); // (R) Interpret FriendData's first status byte as online.
        out.push(0); // (R) Assume not in an encounter.
        // (R) Unmapped status, position, popularity, local, and static-zone fields use neutral defaults.
        out.extend_from_slice(
            &[0u32; 7]
                .into_iter()
                .flat_map(u32::to_le_bytes)
                .collect::<Vec<_>>(),
        );
        out.extend_from_slice(&0f32.to_le_bytes());
        out.extend_from_slice(&0f32.to_le_bytes());
        out.extend_from_slice(&0u32.to_le_bytes());
        out.push(0);
    }
    // Final byte is row +0xa1 = `Is_In_Static_Zone` (`00cb44c0` case 0x11), the atlas gate.
    // 74/5 cannot set it, so online rows say true (R); 74/5 hides out-of-scope friends.
    out.push(u8::from(online));
    Ok(out)
}

/// 74/1 list reader `00c80c40`; friend rows follow `FriendData` reader `00c7fd50`.
pub fn encode_friend_list(
    friends: &BTreeMap<u64, SocialName>,
    online: &BTreeSet<u64>,
) -> Result<Vec<u8>, SocialError> {
    let mut out = friend_header(1);
    out.extend_from_slice(&(friends.len() as i32).to_le_bytes());
    for (guid, name) in friends {
        out.extend_from_slice(&encode_friend_row(*guid, name, online.contains(guid))?);
    }
    Ok(out)
}

/// 74/2 online reader `00c814d0`; body is string, guid, flag (`00c7f...` friend reader set).
fn encode_friend_online(guid: u64, name: &SocialName) -> Result<Vec<u8>, SocialError> {
    let mut out = friend_header(2);
    write_string(&mut out, &name.display())?;
    out.extend_from_slice(&guid.to_le_bytes());
    out.push(1);
    Ok(out)
}

/// 74/3 offline reader `00c814d0`: guid only.
fn encode_friend_offline(guid: u64) -> Vec<u8> {
    let mut out = friend_header(3);
    out.extend_from_slice(&guid.to_le_bytes());
    out
}

/// 74/6 add uses the same FriendData row as 74/1 (`00c80c40`, `00c7fd50`).
fn encode_friend_add(profile: &SocialLists, online: bool) -> Result<Vec<u8>, SocialError> {
    let mut out = friend_header(6);
    out.extend_from_slice(&encode_friend_row(
        profile.character_guid,
        &profile.name,
        online,
    )?);
    Ok(out)
}

/// 74/7 remove reader `00c814d0`: guid only.
fn encode_friend_remove(guid: u64) -> Vec<u8> {
    let mut out = friend_header(7);
    out.extend_from_slice(&guid.to_le_bytes());
    out
}

/// 26/16 body reader `00aa0210` reads two short fields, a guid pair and NameData via `008e7290`, then a byte.
/// The short fields and byte default to zero (R).
fn encode_friend_invite(requester: &SocialLists) -> Result<Vec<u8>, SocialError> {
    let mut out = command_header(16);
    out.extend_from_slice(&0u16.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes());
    out.extend_from_slice(&requester.character_guid.to_le_bytes());
    write_social_name(&mut out, &requester.name)?;
    out.push(0);
    Ok(out)
}

fn encode_ignore_record(guid: u64, name: &SocialName) -> Result<Vec<u8>, SocialError> {
    let mut out = guid.to_le_bytes().to_vec();
    write_string(&mut out, &name.display())?;
    Ok(out)
}

/// 112/1 list reader `00c81250` -> `00c81150`; records are guid pair plus string (`00c7f590`).
pub fn encode_ignore_list(ignored: &BTreeMap<u64, SocialName>) -> Result<Vec<u8>, SocialError> {
    let mut out = ignore_header(1);
    out.extend_from_slice(&(ignored.len() as i32).to_le_bytes());
    for (guid, name) in ignored {
        out.extend_from_slice(&encode_ignore_record(*guid, name)?);
    }
    Ok(out)
}

/// 112/2 add reader `00c81250` -> `00c80b20`.
fn encode_ignore_add(guid: u64, name: &SocialName) -> Result<Vec<u8>, SocialError> {
    let mut out = ignore_header(2);
    out.extend_from_slice(&encode_ignore_record(guid, name)?);
    Ok(out)
}

/// 112/3 remove reader `00c81250`: guid only.
fn encode_ignore_remove(guid: u64) -> Vec<u8> {
    let mut out = ignore_header(3);
    out.extend_from_slice(&guid.to_le_bytes());
    out
}

fn coord_ok(value: i32) -> bool {
    value.unsigned_abs() <= MAX_MAP_COORD.unsigned_abs()
}

/// Where the host's session registry says one online character stands (R). Built by the host
/// from authenticated sessions; nothing here is read from a client packet.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MapSpot {
    pub zone_id: u32,
    pub instance_id: u64,
    pub in_battle: bool,
    pub x: f32,
    pub z: f32,
}

impl MapSpot {
    fn same_scope(&self, other: &Self) -> bool {
        self.zone_id == other.zone_id && self.instance_id == other.instance_id
    }

    fn on_map(&self) -> bool {
        [self.x, self.z]
            .into_iter()
            .all(|v| v.is_finite() && v.abs() <= MAX_MAP_COORD as f32)
    }
}

/// One 74/5 entry. `None` hides the friend on the atlas (client `In_Encounter` flag set).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FriendPosition {
    pub guid: u64,
    pub location: Option<[f32; 2]>,
}

/// 74/5 reader `00c81450` -> `00c80cb0` -> `00c7f620`, applied by `00cb37d0`: `i32` count, then per
/// entry guid (`u64`), a length-prefixed string the client discards (sent empty), a flag byte
/// (row +0xa2 = `In_Encounter`) and, only when the flag is 0, X and Z floats (rows +0xa4, +0xa8).
pub fn encode_friend_positions(entries: &[FriendPosition]) -> Result<Vec<u8>, SocialError> {
    if entries.len() > MAX_POSITION_ENTRIES {
        return Err(SocialError::InvalidPosition);
    }
    let mut out = friend_header(5);
    out.extend_from_slice(&(entries.len() as i32).to_le_bytes());
    for entry in entries {
        out.extend_from_slice(&entry.guid.to_le_bytes());
        out.extend_from_slice(&0i32.to_le_bytes());
        match entry.location {
            Some(at) => {
                if at
                    .iter()
                    .any(|v| !v.is_finite() || v.abs() > MAX_MAP_COORD as f32)
                {
                    return Err(SocialError::InvalidPosition);
                }
                out.push(0);
                out.extend(at.into_iter().flat_map(f32::to_le_bytes));
            }
            None => out.push(1),
        }
    }
    Ok(out)
}

/// 40/15 reader `0093daf0` case 0xf (`008d8f40`): header, member guid (looked up in the group),
/// flag byte stored at member +0xc4.
fn encode_group_changing_job(guid: u64, changing: bool) -> Vec<u8> {
    let mut out = group_header(15);
    out.extend_from_slice(&guid.to_le_bytes());
    out.push(u8::from(changing));
    out
}

/// 40/16 reader `0093daf0` case 0x10 (`008d9020`): header, guid pair (unused by the client), then
/// two `i32` handed to Lua `MiniMap:PingReceived(x, z)`.
fn encode_group_map_ping(sender: u64, x: i32, z: i32) -> Vec<u8> {
    let mut out = group_header(16);
    out.extend_from_slice(&sender.to_le_bytes());
    out.extend_from_slice(&x.to_le_bytes());
    out.extend_from_slice(&z.to_le_bytes());
    out
}

/// The three location requests, as the client writes them. Only the shape is read: the guid the
/// ping body carries (client `+0x3a0`) is dropped because the sender comes from the session.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LocationRequest {
    /// 26/21 `00c23e50`: header only.
    FriendPositions,
    /// 40/14 `00c0b300` -> `00c0b150`: header and one boolean byte (Lua `SetChangingJob`).
    ChangingJob(bool),
    /// 40/16 `00938650` -> `009133c0`: header, guid pair, `i32` x, `i32` z (Lua `Ui.SendMapPing`).
    MapPing { x: i32, z: i32 },
}

/// `None` for any other packet and for any length, flag or coordinate that is not exact.
pub fn parse_location_request(bytes: &[u8]) -> Option<LocationRequest> {
    let word = |at: usize| Some(u16::from_le_bytes(bytes.get(at..at + 2)?.try_into().ok()?));
    let int = |at: usize| Some(i32::from_le_bytes(bytes.get(at..at + 4)?.try_into().ok()?));
    match (word(0)?, word(2)?, bytes.len()) {
        (COMMAND_FAMILY, 21, 4) => Some(LocationRequest::FriendPositions),
        (GROUP_FAMILY, 14, 5) => match bytes[4] {
            0 => Some(LocationRequest::ChangingJob(false)),
            1 => Some(LocationRequest::ChangingJob(true)),
            _ => None,
        },
        (GROUP_FAMILY, 16, 20) => {
            let (x, z) = (int(12)?, int(16)?);
            (coord_ok(x) && coord_ok(z)).then_some(LocationRequest::MapPing { x, z })
        }
        _ => None,
    }
}

/// Family-40 member row read by `0093daf0` case 9 -> `00921680` -> `008fcb20`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GroupMemberRecord {
    pub guid: u64,
    pub name: GroupRecordName,
    /// Seven client-read dwords at record offsets +0x90..+0xa8; semantics are not mapped.
    pub opaque_fields: [u32; 7],
    /// Bytes at +0xc4/+0xc5. The first is the changing-job flag; the second stays opaque.
    pub flags: [bool; 2],
    /// Client compares +0xc8 with its current minigame type (`00a7cd60`, column 8).
    pub minigame_type: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GroupRecordName {
    pub fields: [i32; 3],
    /// The shared reader `008e7290` reads two length-prefixed strings after these fields.
    pub first: String,
    pub last: String,
}

impl GroupMemberRecord {
    pub fn encode(&self) -> Result<Vec<u8>, SocialError> {
        if self.guid == 0 {
            return Err(SocialError::InvalidIdentity);
        }
        SocialName {
            first: self.name.first.clone(),
            last: self.name.last.clone(),
        }
        .validate()?;
        let mut out = self.guid.to_le_bytes().to_vec();
        for field in self.name.fields {
            out.extend_from_slice(&field.to_le_bytes());
        }
        write_string(&mut out, &self.name.first)?;
        write_string(&mut out, &self.name.last)?;
        for field in self.opaque_fields {
            out.extend_from_slice(&field.to_le_bytes());
        }
        out.push(u8::from(self.flags[0]));
        out.push(u8::from(self.flags[1]));
        out.extend_from_slice(&self.minigame_type.to_le_bytes());
        Ok(out)
    }
}

/// 40/8 reader `0093daf0` case 8 -> `0092aed0` / `009140f0`: two i16 fields, count, member rows.
pub fn encode_group_update(members: &[GroupMemberRecord]) -> Result<Vec<u8>, SocialError> {
    let mut out = group_header(8);
    out.extend_from_slice(&0i16.to_le_bytes()); // (R) group control field not semantically named
    out.extend_from_slice(&0i16.to_le_bytes()); // (R) group control field not semantically named
    out.extend_from_slice(&(members.len() as i32).to_le_bytes());
    for member in members {
        out.extend_from_slice(&member.encode()?);
    }
    Ok(out)
}

/// 40/9 reader `0093daf0` case 9 -> `00921680` reads two i16, a member row, then one byte.
pub fn encode_group_member_update(member: &GroupMemberRecord) -> Result<Vec<u8>, SocialError> {
    let mut out = group_header(9);
    out.extend_from_slice(&0i16.to_le_bytes()); // (R) group control fields are not recovered
    out.extend_from_slice(&0i16.to_le_bytes());
    out.extend_from_slice(&member.encode()?);
    out.push(0); // (R) `00921680` reads this final byte; its meaning is unresolved.
    Ok(out)
}

fn write_group_name_data(out: &mut Vec<u8>, name: &SocialName) -> Result<(), SocialError> {
    write_social_name(out, name)
}

/// 40/1 reader `0093daf0` case 1 / `00921640`; client writer `00c235c0` + `00a95c50`.
/// Two short fields, two guid/NameData pairs, group id, then five opaque dwords.
fn encode_group_invite(
    group_id: u32,
    inviter: &SocialLists,
    target: &CharacterIdentity,
) -> Result<Vec<u8>, SocialError> {
    let mut out = group_header(1);
    // (R) The two short control fields have no recovered meaning; emit zero.
    out.extend_from_slice(&0i16.to_le_bytes());
    out.extend_from_slice(&0i16.to_le_bytes());
    out.extend_from_slice(&inviter.character_guid.to_le_bytes());
    write_group_name_data(&mut out, &inviter.name)?;
    out.extend_from_slice(&target.character_guid.to_le_bytes());
    write_group_name_data(&mut out, &target.name)?;
    out.extend_from_slice(&group_id.to_le_bytes());
    out.extend_from_slice(
        &[0u32; 5]
            .into_iter()
            .flat_map(u32::to_le_bytes)
            .collect::<Vec<_>>(),
    );
    Ok(out)
}

/// 40/4 uses the invite record, an accept byte, and five dwords (`00a97250`; reader `00921700`).
fn encode_group_accept_notice(data: &[u8], accepted: bool) -> Result<Vec<u8>, SocialError> {
    // `data` is a 40/1 payload; replace its family/sub header and insert the byte before opaque tail.
    if data.len() < 4 + 5 * 4 {
        return Err(SocialError::InvalidIdentity);
    }
    let tail_start = data.len() - 5 * 4;
    let mut out = group_header(4);
    out.extend_from_slice(&data[4..tail_start]);
    out.push(u8::from(accepted));
    out.extend_from_slice(&data[tail_start..]);
    Ok(out)
}

/// 40/2, 40/5 and 40/7 readers accept one dword (`0093daf0`, cases 2, 5 and 7).
pub fn encode_group_result(sub: u16, code: u32) -> Vec<u8> {
    let mut out = group_header(sub);
    out.extend_from_slice(&code.to_le_bytes());
    out
}

/// 40/3 reader `0093daf0` case 3: guid pair and NameData.
fn encode_group_leave(guid: u64, name: &SocialName) -> Result<Vec<u8>, SocialError> {
    let mut out = group_header(3);
    out.extend_from_slice(&guid.to_le_bytes());
    write_social_name(&mut out, name)?;
    Ok(out)
}

/// 40/6 reader `0093daf0` case 6: target guid pair and member NameData.
fn encode_group_kick(guid: u64, name: &SocialName) -> Result<Vec<u8>, SocialError> {
    let mut out = group_header(6);
    out.extend_from_slice(&guid.to_le_bytes());
    write_social_name(&mut out, name)?;
    Ok(out)
}
