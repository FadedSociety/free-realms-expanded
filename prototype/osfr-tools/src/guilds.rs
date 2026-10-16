//! World-server guild state and family-129 request decoding.
//!
//! Packet layouts below come from the shipped client readers/writers cited at each arm.
//! Guild names, rank permissions, size limits, succession, invite expiry and storage are (R).

use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::File,
    io::{self, Read},
    path::Path,
};

pub const GUILD_FAMILY: u16 = 129;
pub const MAX_GUILD_MEMBERS: usize = 100; // (R) Wiki-era cap; no cap is encoded by the client.
const SCHEMA_VERSION: u32 = 1; // (R) Local server-wide JSON format.
const MAX_STORE_BYTES: usize = 1 << 20;
const MAX_PACKET_BYTES: usize = 64 << 10; // (R) Bound message memory before parsing.
const MAX_WIRE_STRING_BYTES: usize = 4096; // (R) The client length-prefix has no recovered cap.
const MAX_GUILD_NAME_CHARS: usize = 32; // (R) Client name-validation bounds were not recovered.
const MAX_MOTD_BYTES: usize = 1024; // (R) Client only exposes a "too long" result key.

/// Rank ids and permissions are reconstructed policy (R); the client exposes four display names.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[repr(u8)]
pub enum GuildRank {
    Leader = 1,
    Officer = 2,
    Member = 3,
    Recruit = 4,
}

impl GuildRank {
    fn promote(self) -> Option<Self> {
        match self {
            Self::Leader => None,
            Self::Officer => Some(Self::Leader),
            Self::Member => Some(Self::Officer),
            Self::Recruit => Some(Self::Member),
        }
    }

    fn demote(self) -> Option<Self> {
        match self {
            Self::Leader => Some(Self::Officer),
            Self::Officer => Some(Self::Member),
            Self::Member => Some(Self::Recruit),
            Self::Recruit => None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GuildMember {
    pub character_guid: u64,
    pub rank: GuildRank,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Guild {
    pub guid: u64,
    pub name: String,
    pub motd: String,
    pub members: BTreeMap<u64, GuildMember>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CharacterGuildRef {
    pub character_guid: u64,
    pub guild_guid: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GuildRosterEntry {
    pub character_guid: u64,
    pub rank: GuildRank,
    pub online: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GuildResultCode {
    NameNotValid,
    NameAlreadyExists,
    NameRequestPending,
    CantRenameAtThisTime,
    MotdTooLong,
    MemberCountExceeded,
    InMaxGuilds,
    InviteeInMaxGuilds,
    InvitePlayerNotFound,
    CreateRequirementsNotMet,
    PendingInviteTimedOut,
    PromoteAtMaxRank,
    PromoteAtMinRank,
    PromoteCantPromoteAbove,
    PromoteCantDemoteAbove,
    RemoveCantRemoveAbove,
    NotHighRankToDelete,
    NotHighRankToUpdateMotd,
    NotHighRankToInvite,
    NotHighRankToPromote,
    NotHighRankToDemote,
    NotHighRankToRemove,
}

impl GuildResultCode {
    /// Locale keys read from the client for family 129/23 (`00b1ee10`, `00b1cd30`).
    pub const fn client_key(self) -> &'static str {
        match self {
            Self::NameNotValid => "GuildNameNotValid",
            Self::NameAlreadyExists => "GuildNameAlreadyExists",
            Self::NameRequestPending => "GuildNameRequestPending",
            Self::CantRenameAtThisTime => "GuildCantRenameAtThisTime",
            Self::MotdTooLong => "GuildMotdTooLong",
            Self::MemberCountExceeded => "GuildMemberCountExceeded",
            Self::InMaxGuilds => "GuildInMaxGuilds",
            Self::InviteeInMaxGuilds => "GuildInviteeInMaxGuilds",
            Self::InvitePlayerNotFound => "GuildInvitePlayerNotFound",
            Self::CreateRequirementsNotMet => "GuildCreateRequirementsNotMet",
            Self::PendingInviteTimedOut => "GuildPendingInviteTimedOut",
            Self::PromoteAtMaxRank => "GuildPromoteAtMaxRank",
            Self::PromoteAtMinRank => "GuildPromoteAtMinRank",
            Self::PromoteCantPromoteAbove => "GuildPromoteCantPromoteAbove",
            Self::PromoteCantDemoteAbove => "GuildPromoteCantDemoteAbove",
            Self::RemoveCantRemoveAbove => "GuildRemoveCantRemoveAbove",
            Self::NotHighRankToDelete => "GuildNotHighRankToDelete",
            Self::NotHighRankToUpdateMotd => "GuildNotHighRankToUpdateMotd",
            Self::NotHighRankToInvite => "GuildNotHighRankToInvite",
            Self::NotHighRankToPromote => "GuildNotHighRankToPromote",
            Self::NotHighRankToDemote => "GuildNotHighRankToDemote",
            Self::NotHighRankToRemove => "GuildNotHighRankToRemove",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GuildError {
    Client(GuildResultCode),
    GuildNotFound,
    CharacterNotMember,
    DifferentGuild,
    InviteNotPending,
    InviteAlreadyPending,
    SelfAction,
    InvalidCharacter,
    InvalidInviteDuration,
    NotGuildLeader,
}

impl GuildError {
    pub const fn client_code(self) -> Option<GuildResultCode> {
        match self {
            Self::Client(code) => Some(code),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GuildNoticeKind {
    InviteReceived { inviter_guid: u64 },
    InviteSent { target_guid: u64 },
    InviteDeclined { target_guid: u64 },
    InviteTimedOut { target_guid: u64 },
    MemberJoined { member_guid: u64 },
    MemberLeft { member_guid: u64 },
    MemberKicked { member_guid: u64 },
    RankChanged { member_guid: u64, rank: GuildRank },
    LeadershipTransferred { from_guid: u64, to_guid: u64 },
    PresenceChanged { member_guid: u64, online: bool },
    MotdChanged { actor_guid: u64 },
    Disbanded,
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// Typed state changes for the host to route; member-status payload fields remain partly opaque.
pub struct GuildNotice {
    pub recipient_guid: u64,
    pub guild_guid: u64,
    pub kind: GuildNoticeKind,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PersistedGuildStore {
    next_guild_guid: u64,
    guilds: BTreeMap<u64, Guild>,
    character_refs: BTreeMap<u64, CharacterGuildRef>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct GuildDocument {
    schema_version: u32,
    store: PersistedGuildStore,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct PendingInvite {
    guild_guid: u64,
    inviter_guid: u64,
    expires_at_ms: u64,
}

/// Persisted guilds and character-to-guild references; presence and invites are runtime-only.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GuildStore {
    next_guild_guid: u64,
    guilds: BTreeMap<u64, Guild>,
    character_refs: BTreeMap<u64, CharacterGuildRef>,
    online: BTreeSet<u64>,
    pending_invites: BTreeMap<u64, PendingInvite>,
}

impl Default for GuildStore {
    fn default() -> Self {
        Self::new()
    }
}

impl GuildStore {
    pub fn new() -> Self {
        Self {
            next_guild_guid: 1,
            guilds: BTreeMap::new(),
            character_refs: BTreeMap::new(),
            online: BTreeSet::new(),
            pending_invites: BTreeMap::new(),
        }
    }

    pub fn guild(&self, guild_guid: u64) -> Option<&Guild> {
        self.guilds.get(&guild_guid)
    }

    pub fn guild_guid_for(&self, character_guid: u64) -> Option<u64> {
        self.character_refs
            .get(&character_guid)
            .map(|reference| reference.guild_guid)
    }

    pub fn character_guild_ref(&self, character_guid: u64) -> Option<CharacterGuildRef> {
        self.character_refs.get(&character_guid).copied()
    }

    pub fn is_online(&self, character_guid: u64) -> bool {
        self.online.contains(&character_guid)
    }

    /// (R) Enforce locale-described eligibility and one-guild membership through caller state.
    pub fn create_guild(
        &mut self,
        founder_guid: u64,
        name: &str,
        eligible: bool,
    ) -> Result<u64, GuildError> {
        if founder_guid == 0 {
            return Err(GuildError::InvalidCharacter);
        }
        if self.character_refs.contains_key(&founder_guid) {
            return Err(GuildError::Client(GuildResultCode::InMaxGuilds));
        }
        if !eligible {
            return Err(GuildError::Client(
                GuildResultCode::CreateRequirementsNotMet,
            ));
        }
        let name =
            normalize_guild_name(name).ok_or(GuildError::Client(GuildResultCode::NameNotValid))?;
        // (R) Guild-name uniqueness uses Unicode lowercase comparison after local normalization.
        if self
            .guilds
            .values()
            .any(|guild| guild.name.to_lowercase() == name.to_lowercase())
        {
            return Err(GuildError::Client(GuildResultCode::NameAlreadyExists));
        }
        let guild_guid = self.next_guild_guid;
        self.next_guild_guid = guild_guid
            .checked_add(1)
            .ok_or(GuildError::InvalidCharacter)?;
        let mut members = BTreeMap::new();
        // (R) The creator starts as Leader; the client exposes the role but not its wire id.
        members.insert(
            founder_guid,
            GuildMember {
                character_guid: founder_guid,
                rank: GuildRank::Leader,
            },
        );
        self.guilds.insert(
            guild_guid,
            Guild {
                guid: guild_guid,
                name,
                motd: String::new(),
                members,
            },
        );
        self.character_refs.insert(
            founder_guid,
            CharacterGuildRef {
                character_guid: founder_guid,
                guild_guid,
            },
        );
        Ok(guild_guid)
    }

    pub fn invite(
        &mut self,
        actor_guid: u64,
        claimed_guild_guid: u64,
        target_guid: u64,
        target_ignores_actor: bool,
        auto_refuse: bool,
        now_ms: u64,
        invite_window_ms: u64,
    ) -> Result<Vec<GuildNotice>, GuildError> {
        if actor_guid == 0 || target_guid == 0 || actor_guid == target_guid {
            return Err(GuildError::Client(GuildResultCode::InvitePlayerNotFound));
        }
        if invite_window_ms == 0 {
            return Err(GuildError::InvalidInviteDuration);
        }
        let expires_at_ms = now_ms
            .checked_add(invite_window_ms)
            .ok_or(GuildError::InvalidInviteDuration)?;
        let guild_guid = self
            .guild_guid_for(actor_guid)
            .ok_or(GuildError::CharacterNotMember)?;
        if guild_guid != claimed_guild_guid {
            return Err(GuildError::DifferentGuild);
        }
        let guild = self
            .guilds
            .get(&guild_guid)
            .ok_or(GuildError::GuildNotFound)?;
        let actor_rank = guild
            .members
            .get(&actor_guid)
            .ok_or(GuildError::CharacterNotMember)?
            .rank;
        // (R) Leader and Officer may invite; client error keys show an invite permission gate.
        if !matches!(actor_rank, GuildRank::Leader | GuildRank::Officer) {
            return Err(GuildError::Client(GuildResultCode::NotHighRankToInvite));
        }
        // (R) Enforce the client's one-guild-per-character rule on the server.
        if self.character_refs.contains_key(&target_guid) {
            return Err(GuildError::Client(GuildResultCode::InviteeInMaxGuilds));
        }
        if target_ignores_actor {
            // (R) Conceal an ignore relationship with the client's generic invite lookup error.
            return Err(GuildError::Client(GuildResultCode::InvitePlayerNotFound));
        }
        if guild.members.len() >= MAX_GUILD_MEMBERS {
            return Err(GuildError::Client(GuildResultCode::MemberCountExceeded));
        }
        // (R) Keep one outstanding guild invitation per target.
        if self.pending_invites.contains_key(&target_guid) {
            return Err(GuildError::InviteAlreadyPending);
        }
        if auto_refuse {
            // The client's AutoRefuse option sends the ordinary family-129/9 decline (`00a0f130`).
            return Ok(vec![self.notice(
                actor_guid,
                guild_guid,
                GuildNoticeKind::InviteDeclined { target_guid },
            )]);
        }
        // (R) The caller supplies the invitation window because the client exposes no server TTL.
        self.pending_invites.insert(
            target_guid,
            PendingInvite {
                guild_guid,
                inviter_guid: actor_guid,
                expires_at_ms,
            },
        );
        Ok(vec![
            self.notice(
                actor_guid,
                guild_guid,
                GuildNoticeKind::InviteSent { target_guid },
            ),
            self.notice(
                target_guid,
                guild_guid,
                GuildNoticeKind::InviteReceived {
                    inviter_guid: actor_guid,
                },
            ),
        ])
    }

    pub fn accept_invite(
        &mut self,
        target_guid: u64,
        guild_guid: u64,
        now_ms: u64,
        target_ignores_inviter: bool,
    ) -> Result<Vec<GuildNotice>, GuildError> {
        let pending = *self
            .pending_invites
            .get(&target_guid)
            .ok_or(GuildError::InviteNotPending)?;
        if pending.guild_guid != guild_guid {
            return Err(GuildError::InviteNotPending);
        }
        self.pending_invites.remove(&target_guid);
        // (R) Expire using the caller's monotonic clock and supplied invite window.
        if now_ms >= pending.expires_at_ms {
            return Ok(vec![self.notice(
                pending.inviter_guid,
                guild_guid,
                GuildNoticeKind::InviteTimedOut { target_guid },
            )]);
        }
        if target_ignores_inviter {
            // (R) Re-check at acceptance in case the ignore list changed after the invitation.
            return Ok(vec![self.notice(
                pending.inviter_guid,
                guild_guid,
                GuildNoticeKind::InviteDeclined { target_guid },
            )]);
        }
        // (R) Membership and capacity are rechecked at acceptance to avoid stale invites.
        if self.character_refs.contains_key(&target_guid) {
            return Err(GuildError::Client(GuildResultCode::InviteeInMaxGuilds));
        }
        let guild = self
            .guilds
            .get_mut(&guild_guid)
            .ok_or(GuildError::GuildNotFound)?;
        if guild.members.len() >= MAX_GUILD_MEMBERS {
            return Err(GuildError::Client(GuildResultCode::MemberCountExceeded));
        }
        // (R) New members start at the least privileged rank.
        guild.members.insert(
            target_guid,
            GuildMember {
                character_guid: target_guid,
                rank: GuildRank::Recruit,
            },
        );
        self.character_refs.insert(
            target_guid,
            CharacterGuildRef {
                character_guid: target_guid,
                guild_guid,
            },
        );
        self.online.insert(target_guid);
        Ok(self.broadcast(
            guild_guid,
            GuildNoticeKind::MemberJoined {
                member_guid: target_guid,
            },
            None,
        ))
    }

    pub fn decline_invite(
        &mut self,
        target_guid: u64,
        guild_guid: u64,
    ) -> Result<Vec<GuildNotice>, GuildError> {
        self.finish_invite(target_guid, guild_guid, false)
    }

    pub fn timeout_invite(
        &mut self,
        target_guid: u64,
        guild_guid: u64,
    ) -> Result<Vec<GuildNotice>, GuildError> {
        self.finish_invite(target_guid, guild_guid, true)
    }

    fn finish_invite(
        &mut self,
        target_guid: u64,
        guild_guid: u64,
        timed_out: bool,
    ) -> Result<Vec<GuildNotice>, GuildError> {
        let pending = *self
            .pending_invites
            .get(&target_guid)
            .ok_or(GuildError::InviteNotPending)?;
        if pending.guild_guid != guild_guid {
            return Err(GuildError::InviteNotPending);
        }
        self.pending_invites.remove(&target_guid);
        let kind = if timed_out {
            GuildNoticeKind::InviteTimedOut { target_guid }
        } else {
            GuildNoticeKind::InviteDeclined { target_guid }
        };
        Ok(vec![self.notice(pending.inviter_guid, guild_guid, kind)])
    }

    pub fn leave(&mut self, character_guid: u64) -> Result<Vec<GuildNotice>, GuildError> {
        let guild_guid = self
            .guild_guid_for(character_guid)
            .ok_or(GuildError::CharacterNotMember)?;
        let is_leader = self.guilds[&guild_guid].members[&character_guid].rank == GuildRank::Leader;
        if !is_leader {
            // (R) A normal leave removes membership immediately.
            let was_online = self.online.remove(&character_guid);
            self.remove_member(guild_guid, character_guid);
            let mut notices = self.broadcast(
                guild_guid,
                GuildNoticeKind::MemberLeft {
                    member_guid: character_guid,
                },
                None,
            );
            if was_online {
                notices.push(self.notice(
                    character_guid,
                    guild_guid,
                    GuildNoticeKind::MemberLeft {
                        member_guid: character_guid,
                    },
                ));
            }
            return Ok(notices);
        }

        // (R) A final leader leaving dissolves the one-member guild.
        let successor = self.guilds[&guild_guid]
            .members
            .values()
            .filter(|member| member.character_guid != character_guid)
            .min_by_key(|member| (member.rank, member.character_guid))
            .map(|member| member.character_guid);
        let Some(successor_guid) = successor else {
            return Ok(self.disband_guild(guild_guid));
        };

        // (R) On leader departure, promote the highest existing rank; GUID breaks rank ties.
        self.guilds
            .get_mut(&guild_guid)
            .unwrap()
            .members
            .get_mut(&successor_guid)
            .unwrap()
            .rank = GuildRank::Leader;
        let was_online = self.online.remove(&character_guid);
        self.remove_member(guild_guid, character_guid);
        let mut notices = self.broadcast(
            guild_guid,
            GuildNoticeKind::LeadershipTransferred {
                from_guid: character_guid,
                to_guid: successor_guid,
            },
            None,
        );
        notices.extend(self.broadcast(
            guild_guid,
            GuildNoticeKind::MemberLeft {
                member_guid: character_guid,
            },
            None,
        ));
        if was_online {
            notices.push(self.notice(
                character_guid,
                guild_guid,
                GuildNoticeKind::LeadershipTransferred {
                    from_guid: character_guid,
                    to_guid: successor_guid,
                },
            ));
        }
        Ok(notices)
    }

    pub fn kick(
        &mut self,
        actor_guid: u64,
        target_guid: u64,
    ) -> Result<Vec<GuildNotice>, GuildError> {
        let (guild_guid, actor_rank, target_rank) =
            self.same_guild_ranks(actor_guid, target_guid)?;
        if actor_guid == target_guid {
            return Err(GuildError::SelfAction);
        }
        self.require_manage_target(
            actor_rank,
            target_rank,
            GuildResultCode::NotHighRankToRemove,
            GuildResultCode::RemoveCantRemoveAbove,
        )?;
        let target_was_online = self.online.remove(&target_guid);
        self.remove_member(guild_guid, target_guid);
        let mut notices = self.broadcast(
            guild_guid,
            GuildNoticeKind::MemberKicked {
                member_guid: target_guid,
            },
            None,
        );
        if target_was_online {
            notices.push(self.notice(
                target_guid,
                guild_guid,
                GuildNoticeKind::MemberKicked {
                    member_guid: target_guid,
                },
            ));
        }
        Ok(notices)
    }

    pub fn promote(
        &mut self,
        actor_guid: u64,
        target_guid: u64,
    ) -> Result<Vec<GuildNotice>, GuildError> {
        let (guild_guid, actor_rank, target_rank) =
            self.same_guild_ranks(actor_guid, target_guid)?;
        if actor_guid == target_guid {
            return Err(GuildError::SelfAction);
        }
        self.require_manage_target(
            actor_rank,
            target_rank,
            GuildResultCode::NotHighRankToPromote,
            GuildResultCode::PromoteCantPromoteAbove,
        )?;
        if target_rank == GuildRank::Officer {
            // (R) A promote request cannot mint a second Leader; use explicit succession instead.
            return Err(GuildError::Client(GuildResultCode::PromoteAtMaxRank));
        }
        let new_rank = target_rank
            .promote()
            .ok_or(GuildError::Client(GuildResultCode::PromoteAtMaxRank))?;
        self.guilds
            .get_mut(&guild_guid)
            .unwrap()
            .members
            .get_mut(&target_guid)
            .unwrap()
            .rank = new_rank;
        Ok(self.broadcast(
            guild_guid,
            GuildNoticeKind::RankChanged {
                member_guid: target_guid,
                rank: new_rank,
            },
            None,
        ))
    }

    pub fn demote(
        &mut self,
        actor_guid: u64,
        target_guid: u64,
    ) -> Result<Vec<GuildNotice>, GuildError> {
        let (guild_guid, actor_rank, target_rank) =
            self.same_guild_ranks(actor_guid, target_guid)?;
        if actor_guid == target_guid {
            return Err(GuildError::SelfAction);
        }
        self.require_manage_target(
            actor_rank,
            target_rank,
            GuildResultCode::NotHighRankToDemote,
            GuildResultCode::PromoteCantDemoteAbove,
        )?;
        let new_rank = target_rank
            .demote()
            .ok_or(GuildError::Client(GuildResultCode::PromoteAtMinRank))?;
        self.guilds
            .get_mut(&guild_guid)
            .unwrap()
            .members
            .get_mut(&target_guid)
            .unwrap()
            .rank = new_rank;
        Ok(self.broadcast(
            guild_guid,
            GuildNoticeKind::RankChanged {
                member_guid: target_guid,
                rank: new_rank,
            },
            None,
        ))
    }

    pub fn transfer_leadership(
        &mut self,
        actor_guid: u64,
        target_guid: u64,
    ) -> Result<Vec<GuildNotice>, GuildError> {
        let (guild_guid, actor_rank, _) = self.same_guild_ranks(actor_guid, target_guid)?;
        if actor_guid == target_guid {
            return Err(GuildError::SelfAction);
        }
        // (R) Only the current leader may transfer leadership.
        if actor_rank != GuildRank::Leader {
            return Err(GuildError::NotGuildLeader);
        }
        let members = &mut self.guilds.get_mut(&guild_guid).unwrap().members;
        members.get_mut(&actor_guid).unwrap().rank = GuildRank::Officer;
        members.get_mut(&target_guid).unwrap().rank = GuildRank::Leader;
        Ok(self.broadcast(
            guild_guid,
            GuildNoticeKind::LeadershipTransferred {
                from_guid: actor_guid,
                to_guid: target_guid,
            },
            None,
        ))
    }

    pub fn disband(&mut self, actor_guid: u64) -> Result<Vec<GuildNotice>, GuildError> {
        let guild_guid = self
            .guild_guid_for(actor_guid)
            .ok_or(GuildError::CharacterNotMember)?;
        let rank = self.guilds[&guild_guid].members[&actor_guid].rank;
        // (R) Only the current Leader may disband; the client exposes a delete gate, not the matrix.
        if rank != GuildRank::Leader {
            return Err(GuildError::Client(GuildResultCode::NotHighRankToDelete));
        }
        Ok(self.disband_guild(guild_guid))
    }

    pub fn set_motd(
        &mut self,
        actor_guid: u64,
        motd: &str,
    ) -> Result<Vec<GuildNotice>, GuildError> {
        let guild_guid = self
            .guild_guid_for(actor_guid)
            .ok_or(GuildError::CharacterNotMember)?;
        let rank = self.guilds[&guild_guid].members[&actor_guid].rank;
        // (R) Leader and Officer may update the MOTD; cap follows the client's localized error.
        if !matches!(rank, GuildRank::Leader | GuildRank::Officer) {
            return Err(GuildError::Client(GuildResultCode::NotHighRankToUpdateMotd));
        }
        if motd.len() > MAX_MOTD_BYTES {
            return Err(GuildError::Client(GuildResultCode::MotdTooLong));
        }
        self.guilds.get_mut(&guild_guid).unwrap().motd = motd.to_owned();
        Ok(self.broadcast(
            guild_guid,
            GuildNoticeKind::MotdChanged { actor_guid },
            None,
        ))
    }

    pub fn login(&mut self, character_guid: u64) -> Result<Vec<GuildNotice>, GuildError> {
        let guild_guid = self
            .guild_guid_for(character_guid)
            .ok_or(GuildError::CharacterNotMember)?;
        // (R) Presence notices fan out to connected guild members; repeat login is idempotent.
        if !self.online.insert(character_guid) {
            return Ok(Vec::new());
        }
        Ok(self.broadcast(
            guild_guid,
            GuildNoticeKind::PresenceChanged {
                member_guid: character_guid,
                online: true,
            },
            Some(character_guid),
        ))
    }

    pub fn logout(&mut self, character_guid: u64) -> Result<Vec<GuildNotice>, GuildError> {
        let guild_guid = self
            .guild_guid_for(character_guid)
            .ok_or(GuildError::CharacterNotMember)?;
        // (R) Logout notices fan out to members still online afterward.
        if !self.online.remove(&character_guid) {
            return Ok(Vec::new());
        }
        Ok(self.broadcast(
            guild_guid,
            GuildNoticeKind::PresenceChanged {
                member_guid: character_guid,
                online: false,
            },
            None,
        ))
    }

    /// Character names are resolved by the host from character records using each GUID.
    pub fn roster(&self, guild_guid: u64) -> Option<Vec<GuildRosterEntry>> {
        self.guilds.get(&guild_guid).map(|guild| {
            guild
                .members
                .values()
                .map(|member| GuildRosterEntry {
                    character_guid: member.character_guid,
                    rank: member.rank,
                    online: self.online.contains(&member.character_guid),
                })
                .collect()
        })
    }

    pub fn save(&self, path: &Path) -> io::Result<()> {
        self.validate()?;
        if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent)?;
        }
        let bytes = serde_json::to_vec_pretty(&GuildDocument {
            schema_version: SCHEMA_VERSION,
            store: PersistedGuildStore {
                next_guild_guid: self.next_guild_guid,
                guilds: self.guilds.clone(),
                character_refs: self.character_refs.clone(),
            },
        })
        .map_err(invalid_data)?;
        if bytes.len() > MAX_STORE_BYTES {
            return Err(invalid_data("guild store exceeds 1 MiB"));
        }
        crate::character_store::replace_bytes(path, &bytes)
    }

    pub fn load(path: &Path) -> io::Result<Self> {
        let file = File::open(path)?;
        let mut bytes = Vec::new();
        file.take((MAX_STORE_BYTES + 1) as u64)
            .read_to_end(&mut bytes)?;
        if bytes.len() > MAX_STORE_BYTES {
            return Err(invalid_data("guild store exceeds 1 MiB"));
        }
        let document: GuildDocument = serde_json::from_slice(&bytes).map_err(invalid_data)?;
        if document.schema_version != SCHEMA_VERSION {
            return Err(invalid_data("unsupported guild store schema version"));
        }
        let store = Self {
            next_guild_guid: document.store.next_guild_guid,
            guilds: document.store.guilds,
            character_refs: document.store.character_refs,
            online: BTreeSet::new(),
            pending_invites: BTreeMap::new(),
        };
        store.validate()?;
        Ok(store)
    }

    fn same_guild_ranks(
        &self,
        actor_guid: u64,
        target_guid: u64,
    ) -> Result<(u64, GuildRank, GuildRank), GuildError> {
        let actor_guild = self
            .guild_guid_for(actor_guid)
            .ok_or(GuildError::CharacterNotMember)?;
        let target_guild = self
            .guild_guid_for(target_guid)
            .ok_or(GuildError::CharacterNotMember)?;
        if actor_guild != target_guild {
            return Err(GuildError::DifferentGuild);
        }
        let guild = self
            .guilds
            .get(&actor_guild)
            .ok_or(GuildError::GuildNotFound)?;
        let actor_rank = guild
            .members
            .get(&actor_guid)
            .ok_or(GuildError::CharacterNotMember)?
            .rank;
        let target_rank = guild
            .members
            .get(&target_guid)
            .ok_or(GuildError::CharacterNotMember)?
            .rank;
        Ok((actor_guild, actor_rank, target_rank))
    }

    fn require_manage_target(
        &self,
        actor_rank: GuildRank,
        target_rank: GuildRank,
        permission_error: GuildResultCode,
        above_error: GuildResultCode,
    ) -> Result<(), GuildError> {
        // (R) Leader manages all lower ranks; Officer manages Member and Recruit only.
        if !matches!(actor_rank, GuildRank::Leader | GuildRank::Officer) {
            return Err(GuildError::Client(permission_error));
        }
        if actor_rank != GuildRank::Leader && target_rank <= actor_rank {
            return Err(GuildError::Client(above_error));
        }
        Ok(())
    }

    fn remove_member(&mut self, guild_guid: u64, character_guid: u64) {
        if let Some(guild) = self.guilds.get_mut(&guild_guid) {
            guild.members.remove(&character_guid);
        }
        self.character_refs.remove(&character_guid);
        self.pending_invites.remove(&character_guid);
    }

    fn disband_guild(&mut self, guild_guid: u64) -> Vec<GuildNotice> {
        let Some(guild) = self.guilds.get(&guild_guid) else {
            return Vec::new();
        };
        let members: Vec<u64> = guild.members.keys().copied().collect();
        let notices = self.broadcast(guild_guid, GuildNoticeKind::Disbanded, None);
        self.guilds.remove(&guild_guid);
        for member in members {
            self.character_refs.remove(&member);
            self.online.remove(&member);
        }
        self.pending_invites
            .retain(|_, invite| invite.guild_guid != guild_guid);
        notices
    }

    fn broadcast(
        &self,
        guild_guid: u64,
        kind: GuildNoticeKind,
        include: Option<u64>,
    ) -> Vec<GuildNotice> {
        let Some(guild) = self.guilds.get(&guild_guid) else {
            return Vec::new();
        };
        guild
            .members
            .keys()
            .copied()
            .filter(|guid| self.online.contains(guid) || Some(*guid) == include)
            .map(|recipient_guid| self.notice(recipient_guid, guild_guid, kind.clone()))
            .collect()
    }

    fn notice(&self, recipient_guid: u64, guild_guid: u64, kind: GuildNoticeKind) -> GuildNotice {
        GuildNotice {
            recipient_guid,
            guild_guid,
            kind,
        }
    }

    fn validate(&self) -> io::Result<()> {
        if self.next_guild_guid == 0
            || self
                .guilds
                .last_key_value()
                .is_some_and(|(max_guid, _)| self.next_guild_guid <= *max_guid)
        {
            return Err(invalid_data("guild id allocator is invalid"));
        }
        let mut expected_refs = BTreeMap::new();
        let mut names = BTreeSet::new();
        for (guild_guid, guild) in &self.guilds {
            if *guild_guid == 0
                || guild.guid != *guild_guid
                || normalize_guild_name(&guild.name).as_deref() != Some(guild.name.as_str())
                || guild.motd.len() > MAX_MOTD_BYTES
                || guild.members.is_empty()
                || guild.members.len() > MAX_GUILD_MEMBERS
                || !names.insert(guild.name.to_lowercase())
            {
                return Err(invalid_data("guild record is invalid"));
            }
            let mut leaders = 0;
            for (character_guid, member) in &guild.members {
                if *character_guid == 0
                    || member.character_guid != *character_guid
                    || expected_refs
                        .insert(
                            *character_guid,
                            CharacterGuildRef {
                                character_guid: *character_guid,
                                guild_guid: *guild_guid,
                            },
                        )
                        .is_some()
                {
                    return Err(invalid_data("guild member reference is invalid"));
                }
                leaders += usize::from(member.rank == GuildRank::Leader);
            }
            if leaders != 1 {
                return Err(invalid_data("guild must have exactly one leader"));
            }
        }
        if expected_refs != self.character_refs
            || self
                .character_refs
                .iter()
                .any(|(guid, reference)| *guid == 0 || reference.character_guid != *guid)
        {
            return Err(invalid_data(
                "character guild references do not match guild rosters",
            ));
        }
        Ok(())
    }
}

/// Decode client-to-server family-129 bodies; strings use signed 32-bit byte lengths.
pub fn decode_request(bytes: &[u8]) -> Result<GuildRequest, GuildProtocolError> {
    if bytes.len() > MAX_PACKET_BYTES {
        return Err(GuildProtocolError::InvalidLength);
    }
    let mut reader = Reader { bytes, at: 0 };
    if reader.u16()? != GUILD_FAMILY {
        return Err(GuildProtocolError::InvalidHeader);
    }
    let sub = reader.u16()?;
    let request = match sub {
        // Client writer `00c0f3b0`: three length-prefixed IStrings.
        1 => GuildRequest::Create {
            name: reader.string()?,
            locale: reader.string()?,
            default_name: reader.string()?,
        },
        // Client writer `00c0f4e0`: one 8-byte guild GUID.
        2 => GuildRequest::Delete {
            guild_guid: reader.u64()?,
        },
        // Client writer `00c0f830`: two 8-byte GUIDs then an IString.
        3 => GuildRequest::Invite {
            first_guid: reader.u64()?,
            second_guid: reader.u64()?,
            player_name: reader.string()?,
        },
        // Writers `00c0fba0`, `00c0fcc0`, `00c0f960`: two 8-byte GUIDs.
        4 => GuildRequest::Promote {
            first_guid: reader.u64()?,
            second_guid: reader.u64()?,
        },
        5 => GuildRequest::Demote {
            first_guid: reader.u64()?,
            second_guid: reader.u64()?,
        },
        6 => GuildRequest::Remove {
            first_guid: reader.u64()?,
            second_guid: reader.u64()?,
        },
        // Client writer `00c0fa80`: one 8-byte guild GUID.
        7 => GuildRequest::Quit {
            guild_guid: reader.u64()?,
        },
        // Client responders `00a0f1d0`, `00a0f130`, `00a0f290`: one 8-byte GUID.
        8 => GuildRequest::InviteAccept {
            guild_guid: reader.u64()?,
        },
        9 => GuildRequest::InviteDecline {
            guild_guid: reader.u64()?,
        },
        10 => GuildRequest::InviteTimeout {
            guild_guid: reader.u64()?,
        },
        // Client writer `00c0f710`: guild GUID followed by an IString.
        11 => GuildRequest::Motd {
            guild_guid: reader.u64()?,
            text: reader.string()?,
        },
        // Client writer `00c0f600` delegates to `00c0eef0`: GUID, two IStrings, flag byte.
        13 => GuildRequest::Rename {
            guild_guid: reader.u64()?,
            name: reader.string()?,
            locale: reader.string()?,
            non_china: reader.u8()?,
        },
        // Client reader/writer path `00b1e960` constructs the request from one GUID pair.
        14 => GuildRequest::NameRequest {
            guild_guid: reader.u64()?,
        },
        // Client sender `00b1e600`; helper `00b1d280` writes a u32 count and u64 GUID array.
        15 => {
            let guild_guid = reader.u64()?;
            let count = reader.u32()? as usize;
            if count > 4096 {
                return Err(GuildProtocolError::InvalidLength);
            }
            let mut character_guids = Vec::with_capacity(count);
            for _ in 0..count {
                character_guids.push(reader.u64()?);
            }
            GuildRequest::MemberLocationRequest {
                guild_guid,
                character_guids,
            }
        }
        // Client constructor `00a16c50` stores GUID plus IString for sub-opcode 16.
        16 => GuildRequest::PaidRenameCheck {
            guild_guid: reader.u64()?,
            name: reader.string()?,
        },
        other => return Err(GuildProtocolError::UnsupportedSubOpcode(other)),
    };
    reader.finish()?;
    Ok(request)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GuildRequest {
    Create {
        name: String,
        locale: String,
        default_name: String,
    },
    Delete {
        guild_guid: u64,
    },
    Invite {
        first_guid: u64,
        second_guid: u64,
        player_name: String,
    },
    Promote {
        first_guid: u64,
        second_guid: u64,
    },
    Demote {
        first_guid: u64,
        second_guid: u64,
    },
    Remove {
        first_guid: u64,
        second_guid: u64,
    },
    Quit {
        guild_guid: u64,
    },
    InviteAccept {
        guild_guid: u64,
    },
    InviteDecline {
        guild_guid: u64,
    },
    InviteTimeout {
        guild_guid: u64,
    },
    Motd {
        guild_guid: u64,
        text: String,
    },
    Rename {
        guild_guid: u64,
        name: String,
        locale: String,
        non_china: u8,
    },
    NameRequest {
        guild_guid: u64,
    },
    MemberLocationRequest {
        guild_guid: u64,
        character_guids: Vec<u64>,
    },
    PaidRenameCheck {
        guild_guid: u64,
        name: String,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GuildProtocolError {
    InvalidHeader,
    InvalidLength,
    InvalidUtf8,
    Truncated,
    TrailingBytes,
    UnsupportedSubOpcode(u16),
}

/// Family-129/23 carries one localized error key (`GuildErrorPacket`, `00b1cd30`).
pub fn error_message(recipient_guid: u64, code: GuildResultCode) -> GuildMessage {
    let mut bytes = vec![GUILD_FAMILY as u8, (GUILD_FAMILY >> 8) as u8, 23, 0];
    put_string(&mut bytes, code.client_key());
    GuildMessage {
        recipient_guid,
        bytes,
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GuildMessage {
    pub recipient_guid: u64,
    /// Family payload before the gateway tunnel envelope.
    pub bytes: Vec<u8>,
}

fn normalize_guild_name(name: &str) -> Option<String> {
    // (R) Local rule: 3-32 letters/digits/spaces/hyphens/apostrophes, trimmed, no repeated spaces.
    let name = name.trim();
    let count = name.chars().count();
    if !(3..=MAX_GUILD_NAME_CHARS).contains(&count) || name.len() > 96 {
        return None;
    }
    let mut previous_space = false;
    for (index, ch) in name.chars().enumerate() {
        let valid = ch.is_alphanumeric() || matches!(ch, ' ' | '-' | '\'');
        if !valid || ch.is_control() || (ch == ' ' && (index == 0 || previous_space)) {
            return None;
        }
        previous_space = ch == ' ';
    }
    if previous_space {
        return None;
    }
    Some(name.to_owned())
}

fn invalid_data(error: impl std::fmt::Display) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, error.to_string())
}

fn put_string(out: &mut Vec<u8>, value: &str) {
    out.extend_from_slice(&(value.len() as i32).to_le_bytes());
    out.extend_from_slice(value.as_bytes());
}

struct Reader<'a> {
    bytes: &'a [u8],
    at: usize,
}

impl Reader<'_> {
    fn take(&mut self, len: usize) -> Result<&[u8], GuildProtocolError> {
        let end = self
            .at
            .checked_add(len)
            .ok_or(GuildProtocolError::InvalidLength)?;
        if end > self.bytes.len() {
            return Err(GuildProtocolError::Truncated);
        }
        let value = &self.bytes[self.at..end];
        self.at = end;
        Ok(value)
    }

    fn u8(&mut self) -> Result<u8, GuildProtocolError> {
        Ok(self.take(1)?[0])
    }

    fn u16(&mut self) -> Result<u16, GuildProtocolError> {
        Ok(u16::from_le_bytes(self.take(2)?.try_into().unwrap()))
    }

    fn u32(&mut self) -> Result<u32, GuildProtocolError> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }

    fn u64(&mut self) -> Result<u64, GuildProtocolError> {
        Ok(u64::from_le_bytes(self.take(8)?.try_into().unwrap()))
    }

    fn string(&mut self) -> Result<String, GuildProtocolError> {
        let len = i32::from_le_bytes(self.take(4)?.try_into().unwrap());
        if !(0..=MAX_WIRE_STRING_BYTES as i32).contains(&len) {
            return Err(GuildProtocolError::InvalidLength);
        }
        String::from_utf8(self.take(len as usize)?.to_vec())
            .map_err(|_| GuildProtocolError::InvalidUtf8)
    }

    fn finish(self) -> Result<(), GuildProtocolError> {
        if self.at == self.bytes.len() {
            Ok(())
        } else {
            Err(GuildProtocolError::TrailingBytes)
        }
    }
}
