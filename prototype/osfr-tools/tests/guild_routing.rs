//! D07: family-129 guild routing gate. Drives the library service with two bot identities,
//! checks the recovered client layouts, refusals that change nothing, restart persistence and
//! the shared routing hooks the binary patch adds.
//!
//! Recovered layouts (client readers): 129/22 `00b1e730`/`00b1e160`/`00b1d2d0`/`00b1bf30`,
//! 129/17 `00b1c840`, 129/24 `00b1be30`+`00b1d210`, 129/23 `00b1ad70`, 129/32 `00b1bd30`.
//! Policy marks are (R): the module's reconstructed rules, not original server behaviour.

use osfr_tools::{
    guilds::{GuildCaller, GuildRank, GuildService, GuildStore},
    social_chat::{self, ChatChannel, ChatRequest, ChatSession, IgnoreLists, NameData},
};
use std::{
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

static TEMP_ID: AtomicU64 = AtomicU64::new(0);

struct TempDir(PathBuf);

impl TempDir {
    fn new() -> Self {
        let id = TEMP_ID.fetch_add(1, Ordering::Relaxed);
        let path =
            std::env::temp_dir().join(format!("osfr-guild-route-{}-{id}", std::process::id()));
        std::fs::create_dir_all(&path).unwrap();
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn name(guid: u64) -> NameData {
    NameData::new(format!("Player{guid}"), "Tester")
}

fn caller(character_guid: u64, name: &NameData) -> GuildCaller<'_> {
    GuildCaller {
        character_guid,
        name,
        eligible: true,
        ignored_pair: false,
    }
}

fn resolve(raw: &str) -> Option<u64> {
    // Host-owned identity lookup: the client may invite by name with a zero guid.
    match raw {
        "Player202 Tester" => Some(202),
        "Player101 Tester" => Some(101),
        _ => None,
    }
}

fn header(sub: u16) -> Vec<u8> {
    let mut bytes = vec![129, 0];
    bytes.extend_from_slice(&sub.to_le_bytes());
    bytes
}

fn push_string(bytes: &mut Vec<u8>, value: &str) {
    bytes.extend_from_slice(&(value.len() as i32).to_le_bytes());
    bytes.extend_from_slice(value.as_bytes());
}

fn create(name: &str) -> Vec<u8> {
    let mut bytes = header(1);
    push_string(&mut bytes, name);
    push_string(&mut bytes, "en_US");
    push_string(&mut bytes, "");
    bytes
}

fn invite(guild: u64, target: u64, player: &str) -> Vec<u8> {
    let mut bytes = header(3);
    bytes.extend_from_slice(&guild.to_le_bytes());
    bytes.extend_from_slice(&target.to_le_bytes());
    push_string(&mut bytes, player);
    bytes
}

fn one_guid(sub: u16, guild: u64) -> Vec<u8> {
    let mut bytes = header(sub);
    bytes.extend_from_slice(&guild.to_le_bytes());
    bytes
}

fn pair(sub: u16, first: u64, second: u64) -> Vec<u8> {
    let mut bytes = header(sub);
    bytes.extend_from_slice(&first.to_le_bytes());
    bytes.extend_from_slice(&second.to_le_bytes());
    bytes
}

fn motd(guild: u64, text: &str) -> Vec<u8> {
    let mut bytes = one_guid(11, guild);
    push_string(&mut bytes, text);
    bytes
}

fn rename(guild: u64, requested: &str) -> Vec<u8> {
    let mut bytes = one_guid(13, guild);
    push_string(&mut bytes, requested);
    push_string(&mut bytes, "en_US");
    bytes.push(1);
    bytes
}

struct Reader<'a> {
    bytes: &'a [u8],
    at: usize,
}

impl<'a> Reader<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, at: 0 }
    }

    fn take(&mut self, len: usize) -> &'a [u8] {
        let end = self.at + len;
        assert!(end <= self.bytes.len(), "layout overrun at {}", self.at);
        let slice = &self.bytes[self.at..end];
        self.at = end;
        slice
    }

    fn u8(&mut self) -> u8 {
        self.take(1)[0]
    }

    fn u16(&mut self) -> u16 {
        u16::from_le_bytes(self.take(2).try_into().unwrap())
    }

    fn u32(&mut self) -> u32 {
        u32::from_le_bytes(self.take(4).try_into().unwrap())
    }

    fn u64(&mut self) -> u64 {
        u64::from_le_bytes(self.take(8).try_into().unwrap())
    }

    fn string(&mut self) -> String {
        let len = i32::from_le_bytes(self.take(4).try_into().unwrap());
        assert!(len >= 0, "negative string length");
        String::from_utf8(self.take(len as usize).to_vec()).unwrap()
    }

    fn name_data(&mut self) -> String {
        for _ in 0..3 {
            self.take(4);
        }
        let first = self.string();
        let last = self.string();
        [first.trim(), last.trim()]
            .into_iter()
            .filter(|part| !part.is_empty())
            .collect::<Vec<_>>()
            .join(" ")
    }
}

struct FullData {
    guild_guid: u64,
    second_identifier: u64,
    name: String,
    motd: String,
    members: Vec<(u64, u32, String, bool)>,
    roles: Vec<(u32, u32)>,
}

/// Reads 129/22 in the client's recovered field order (`00b1e730`, `00b1e160`, `00b1d2d0`,
/// `00b1bf30`, `00b1bfc0`, `00b19350`; role default id is entry `+8` per `00b1a510`). The
/// member bool after `NameData` is the online flag: `00b1bfc0` stores it at member `+180`
/// and `00b1b340` copies the 129/17 `+172` online flag into it.
fn parse_full(bytes: &[u8]) -> FullData {
    let mut r = Reader::new(bytes);
    assert_eq!(r.u16(), 129);
    assert_eq!(r.u16(), 22);
    let guild_guid = r.u64();
    let second_identifier = r.u64();
    let name = r.string();
    let motd = r.string();
    r.u8();
    r.u8();
    let member_count = r.u32();
    assert!(member_count <= 100, "member count is bounded");
    let mut members = Vec::new();
    for _ in 0..member_count {
        let key = r.u32();
        let guid = r.u64();
        assert_eq!(key, (guid as u32) ^ ((guid >> 32) as u32), "member key");
        let rank = r.u32();
        let display = r.name_data();
        let online = r.u8() != 0;
        for _ in 0..4 {
            r.u32();
        }
        members.push((guid, rank, display, online));
    }
    let role_count = r.u32();
    assert_eq!(role_count, 4, "four default ranks");
    let mut roles = Vec::new();
    for _ in 0..role_count {
        let key = r.u32();
        let role_low = r.u32();
        let role_high = r.u32();
        let default_id = r.u32();
        let name_id = r.u32();
        let trailing = r.u32();
        assert_eq!(
            (role_low, role_high, name_id, trailing),
            (0, 0, u32::MAX, 0),
            "unused role fields are zero and the custom name id is unset (R)"
        );
        roles.push((key, default_id));
    }
    assert_eq!(r.at, bytes.len(), "no trailing bytes");
    FullData {
        guild_guid,
        second_identifier,
        name,
        motd,
        members,
        roles,
    }
}

fn status(bytes: &[u8]) -> (u64, u64, String, u32, bool, u32) {
    let mut r = Reader::new(bytes);
    assert_eq!(r.u16(), 129);
    assert_eq!(r.u16(), 17);
    let guild = r.u64();
    let member = r.u64();
    let display = r.name_data();
    // `00b1c840` reads role (+168), online (+172), status (+176); `00b1d7d0` switches on
    // the status, so a status written first would be read as a rank.
    let rank = r.u32();
    let online = r.u8() != 0;
    let code = r.u32();
    for _ in 0..4 {
        r.u32();
    }
    assert_eq!(r.at, bytes.len());
    (guild, member, display, code, online, rank)
}

fn error_key(bytes: &[u8]) -> String {
    let mut r = Reader::new(bytes);
    assert_eq!(r.u16(), 129);
    assert_eq!(r.u16(), 23);
    let key = r.string();
    assert_eq!(r.at, bytes.len());
    key
}

/// Reads 129/24 in the client's order (`00b1be30` then `00b1d210`): inviter guid, unused
/// pair, guild guid, unused dword, unused pair, inviter NameData, invitee NameData, guild name.
/// `00b1e7e0` checks the first pair against the client ignore list.
fn invite_notification(bytes: &[u8]) -> (u64, u64, String, String) {
    let mut r = Reader::new(bytes);
    assert_eq!(r.u16(), 129);
    assert_eq!(r.u16(), 24);
    let inviter = r.u64();
    assert_eq!(r.u64(), 0, "unused pair");
    let guild = r.u64();
    assert_eq!(r.u32(), 0, "unused dword");
    assert_eq!(r.u64(), 0, "unused pair");
    let inviter_name = r.name_data();
    assert_eq!(r.name_data(), "", "invitee NameData is unused (R)");
    let guild_name = r.string();
    assert_eq!(r.at, bytes.len());
    (inviter, guild, inviter_name, guild_name)
}

/// 129/27 and 129/28 carry one NameData first (`00b1d5b0`, `00b1d1b0`).
fn notification_name(bytes: &[u8]) -> (u16, String, Option<bool>) {
    let mut r = Reader::new(bytes);
    assert_eq!(r.u16(), 129);
    let sub = r.u16();
    let name = r.name_data();
    let timed_out = (sub == 28).then(|| r.u8() != 0);
    assert_eq!(r.at, bytes.len());
    (sub, name, timed_out)
}

fn sub(message: &[u8]) -> u16 {
    u16::from_le_bytes([message[2], message[3]])
}

fn store_bytes(service: &GuildService, path: &Path) -> Vec<u8> {
    service.store().save(path).unwrap();
    std::fs::read(path).unwrap()
}

#[test]
fn two_bots_create_invite_accept_and_chat() {
    let mut service = GuildService::new(60_000);
    let a = name(101);
    let b = name(202);
    let caller_a = GuildCaller {
        character_guid: 101,
        name: &a,
        eligible: true,
        ignored_pair: false,
    };

    let created = service.handle(&caller_a, &resolve, &create("Lantern Guild"), 1_000);
    let guild = service.store().guild_guid_for(101).expect("founder joined");
    let full = created
        .messages
        .iter()
        .find(|message| sub(&message.bytes) == 22)
        .expect("founder sees the guild record");
    let parsed = parse_full(&full.bytes);
    assert_eq!(parsed.guild_guid, guild);
    assert_eq!(parsed.second_identifier, guild);
    assert_eq!(parsed.name, "Lantern Guild");
    assert_eq!(parsed.motd, "");
    assert_eq!(parsed.members.len(), 1);
    assert_eq!(parsed.members[0].0, 101);
    assert_eq!(parsed.members[0].1, GuildRank::Leader as u32);
    assert_eq!(parsed.members[0].2, "Player101 Tester");
    assert_eq!(parsed.roles, vec![(1, 1), (2, 2), (3, 3), (4, 4)]);
    // The creator is a connected session, so its 129/22 row is online (member bool +180).
    assert!(parsed.members[0].3, "the founder row is online");
    assert!(created.messages.iter().any(|message| {
        sub(&message.bytes) == 32 && message.bytes[4] == 0 && message.recipient_guid == 101
    }));
    assert_eq!(
        created.messages.first().map(|m| m.recipient_guid),
        Some(101)
    );

    // Invite by name with a zero target guid, as `Guild.Invite(guild, 0, name)` sends it.
    let invited = service.handle(
        &caller_a,
        &resolve,
        &invite(guild, 0, "Player202 Tester"),
        1_100,
    );
    assert!(
        invited
            .messages
            .iter()
            .all(|message| message.recipient_guid == 202)
    );
    let notification = invited
        .messages
        .iter()
        .find(|message| sub(&message.bytes) == 24)
        .expect("invitee pop-up");
    let (inviter, invited_guild, inviter_name, invited_name) =
        invite_notification(&notification.bytes);
    assert_eq!(
        inviter, 101,
        "the inviter guid feeds the client ignore check"
    );
    assert_eq!(invited_guild, guild);
    assert_eq!(inviter_name, "Player101 Tester");
    assert_eq!(invited_name, "Lantern Guild");
    assert_eq!(
        service.store().guild_guid_for(202),
        None,
        "not a member yet"
    );

    let caller_b = GuildCaller {
        character_guid: 202,
        name: &b,
        eligible: true,
        ignored_pair: false,
    };
    let accepted = service.handle(&caller_b, &resolve, &one_guid(8, guild), 1_200);
    let joined = accepted
        .messages
        .iter()
        .find(|message| sub(&message.bytes) == 17)
        .expect("join status");
    let (status_guild, member, display, code, online, rank) = status(&joined.bytes);
    assert_eq!((status_guild, member, code, online), (guild, 202, 1, true));
    assert_eq!(display, "Player202 Tester");
    assert_eq!(rank, GuildRank::Recruit as u32);
    let roster = accepted
        .messages
        .iter()
        .find(|message| sub(&message.bytes) == 22)
        .expect("joiner roster");
    let parsed = parse_full(&roster.bytes);
    assert_eq!(parsed.members.len(), 2);
    // Both members are connected in this flow: the founder created the guild and the
    // joiner accepted the invite.
    assert!(
        parsed.members.iter().all(|member| member.3),
        "both roster rows are online: {:?}",
        parsed.members
    );
    // 129/27 tells the inviter that the invitee joined (`00b1d5b0`).
    let accepted_notice = accepted
        .messages
        .iter()
        .find(|message| sub(&message.bytes) == 27 && message.recipient_guid == 101)
        .expect("inviter sees the accept notice");
    assert_eq!(
        notification_name(&accepted_notice.bytes),
        (27, "Player202 Tester".into(), None)
    );

    // Guild chat: `chat_session` reads `guild_guid_for`; the existing channel router then
    // reaches exactly the members of that guild.
    let sessions = [(1, 101), (2, 202), (3, 303)].map(|(session_id, character_guid)| ChatSession {
        session_id,
        character_guid,
        name: NameData::default(),
        zone_id: 1,
        position: [0.0; 3],
        chat_area_id: None,
        group_id: None,
        guild_id: service.store().guild_guid_for(character_guid),
        is_member: false,
    });
    let request = ChatRequest {
        channel: ChatChannel::Guild,
        message: String::new(),
        tell_target: None,
        claimed_guild_guid: guild,
        area_id: None,
        language: 0,
    };
    let recipients =
        social_chat::resolve_recipients(&request, 1, &sessions, &IgnoreLists::default()).unwrap();
    assert_eq!(recipients, vec![1, 2]);
}

#[test]
fn relog_roster_survives_a_gateway_restart() {
    let temp = TempDir::new();
    let path = temp.path().join("guilds.json");
    let mut service = GuildService::new(60_000);
    let a = name(101);
    let b = name(202);
    let caller_a = GuildCaller {
        character_guid: 101,
        name: &a,
        eligible: true,
        ignored_pair: false,
    };
    service.handle(&caller_a, &resolve, &create("Lantern Guild"), 1_000);
    let guild = service.store().guild_guid_for(101).unwrap();
    service.handle(&caller_a, &resolve, &invite(guild, 202, ""), 1_100);
    let caller_b = GuildCaller {
        character_guid: 202,
        name: &b,
        eligible: true,
        ignored_pair: false,
    };
    service.handle(&caller_b, &resolve, &one_guid(8, guild), 1_200);
    assert!(service.is_dirty());
    service.save(&path).unwrap();
    assert!(!service.is_dirty());

    // Restart: a new service reads the same file; membership and names survive.
    let mut restored = GuildService::load(&path, 60_000).unwrap();
    assert_eq!(restored.store().guild_guid_for(202), Some(guild));
    assert!(!restored.store().is_online(101));
    let login = restored.login(202, &b, true);
    let roster = login
        .messages
        .iter()
        .find(|message| sub(&message.bytes) == 22)
        .expect("relog roster");
    let parsed = parse_full(&roster.bytes);
    assert_eq!(parsed.name, "Lantern Guild");
    assert_eq!(parsed.members.len(), 2);
    assert!(
        parsed
            .members
            .contains(&(101, 1, "Player101 Tester".into(), false))
    );
    // The relogging member is marked online before the 129/22 roster is built, so its own
    // member row is online (the pre-fix order wrote every row offline).
    assert!(
        parsed
            .members
            .contains(&(202, 4, "Player202 Tester".into(), true))
    );
    assert!(login.messages.iter().any(|message| {
        sub(&message.bytes) == 32 && message.bytes[4] == 0 && message.recipient_guid == 202
    }));
    // Only the relogging member is online; no stale presence survives the restart.
    assert!(restored.store().is_online(202));
    assert!(!restored.store().is_online(101));
}

#[test]
fn non_members_forged_guids_and_bad_names_change_nothing() {
    let temp = TempDir::new();
    let path = temp.path().join("guilds.json");
    let mut service = GuildService::new(60_000);
    let a = name(101);
    let caller_a = GuildCaller {
        character_guid: 101,
        name: &a,
        eligible: true,
        ignored_pair: false,
    };
    service.handle(&caller_a, &resolve, &create("Lantern Guild"), 1_000);
    let guild = service.store().guild_guid_for(101).unwrap();
    service.handle(&caller_a, &resolve, &invite(guild, 202, ""), 1_100);
    let before = store_bytes(&service, &path);

    // A non-member cannot invite or kick.
    let outsider = name(999);
    let caller_c = GuildCaller {
        character_guid: 999,
        name: &outsider,
        eligible: true,
        ignored_pair: false,
    };
    let refused_invite = service.handle(&caller_c, &resolve, &invite(guild, 202, ""), 2_000);
    assert!(
        refused_invite.messages.is_empty(),
        "non-member invite is silent"
    );
    let refused_kick = service.handle(&caller_c, &resolve, &pair(6, guild, 101), 2_100);
    assert!(
        refused_kick.messages.is_empty(),
        "non-member kick is silent"
    );

    // Members cannot act on a guild that is not theirs.
    let forged_quit = service.handle(&caller_a, &resolve, &one_guid(7, guild + 1), 2_200);
    assert!(forged_quit.messages.is_empty());
    let forged_delete = service.handle(&caller_a, &resolve, &one_guid(2, guild + 1), 2_300);
    assert!(forged_delete.messages.is_empty());
    let forged_motd = service.handle(&caller_a, &resolve, &motd(guild + 1, "x"), 2_400);
    assert!(forged_motd.messages.is_empty());
    // A member's own rename attempt is refused with a client key and no state change.
    let refused_rename = service.handle(&caller_a, &resolve, &rename(guild, "Other Guild"), 2_500);
    assert_eq!(refused_rename.messages.len(), 1);
    assert_eq!(
        error_key(&refused_rename.messages[0].bytes),
        "GuildCantRenameAtThisTime"
    );

    assert_eq!(
        store_bytes(&service, &path),
        before,
        "refusals must not change the guild store"
    );
    assert_eq!(service.store().guild_guid_for(101), Some(guild));
    assert_eq!(service.store().guild_guid_for(999), None);
    assert_eq!(service.store().guild(guild).unwrap().name, "Lantern Guild");
    assert_eq!(service.store().guild(guild).unwrap().motd, "");
}

#[test]
fn oversize_and_invalid_names_are_refused() {
    let temp = TempDir::new();
    let path = temp.path().join("guilds.json");
    let mut service = GuildService::new(60_000);
    let a = name(101);
    let caller_a = GuildCaller {
        character_guid: 101,
        name: &a,
        eligible: true,
        ignored_pair: false,
    };
    let before = store_bytes(&service, &path);
    for bad in ["x".repeat(64), "bad\u{0}name".into(), "  ".into()] {
        let out = service.handle(&caller_a, &resolve, &create(&bad), 1_000);
        assert_eq!(out.messages.len(), 1);
        assert_eq!(error_key(&out.messages[0].bytes), "GuildNameNotValid");
    }
    assert_eq!(store_bytes(&service, &path), before);
    assert_eq!(service.store().guild_guid_for(101), None);

    // A guildless eligible player is told they may create one (129/32).
    let login = service.login(101, &a, true);
    assert_eq!(login.messages.len(), 1);
    assert_eq!(sub(&login.messages[0].bytes), 32);
    assert_eq!(login.messages[0].bytes[4], 1);
    // A guildless ineligible player is told they may not.
    let login = service.login(101, &a, false);
    assert_eq!(login.messages.len(), 1);
    assert_eq!(sub(&login.messages[0].bytes), 32);
    assert_eq!(login.messages[0].bytes[4], 0);
}

#[test]
fn control_characters_in_the_motd_are_refused_without_change() {
    let temp = TempDir::new();
    let path = temp.path().join("guilds.json");
    let mut service = GuildService::new(60_000);
    let a = name(101);
    let caller_a = caller(101, &a);
    service.handle(&caller_a, &resolve, &create("Lantern Guild"), 1_000);
    let guild = service.store().guild_guid_for(101).unwrap();
    let before = store_bytes(&service, &path);
    for text in ["line\nbreak".to_owned(), "null\u{0}byte".to_owned()] {
        let refused = service.handle(&caller_a, &resolve, &motd(guild, &text), 1_100);
        assert_eq!(refused.messages.len(), 1);
        assert_eq!(error_key(&refused.messages[0].bytes), "GuildMotdTooLong");
    }
    assert_eq!(store_bytes(&service, &path), before);
    assert_eq!(service.store().guild(guild).unwrap().motd, "");
}

#[test]
fn malformed_and_member_location_requests_are_answered_with_silence() {
    let mut service = GuildService::new(60_000);
    let a = name(101);
    let caller_a = GuildCaller {
        character_guid: 101,
        name: &a,
        eligible: true,
        ignored_pair: false,
    };
    service.handle(&caller_a, &resolve, &create("Lantern Guild"), 1_000);
    let guild = service.store().guild_guid_for(101).unwrap();
    let mut location = one_guid(15, guild);
    location.extend_from_slice(&0u32.to_le_bytes());
    assert!(
        service
            .handle(&caller_a, &resolve, &location, 1_100)
            .messages
            .is_empty()
    );
    assert!(
        service
            .handle(&caller_a, &resolve, &[129, 0, 99, 0], 1_200)
            .messages
            .is_empty()
    );
    assert!(
        service
            .handle(&caller_a, &resolve, &[129, 0, 11, 0, 1], 1_300)
            .messages
            .is_empty()
    );
    assert_eq!(service.store().guild_guid_for(101), Some(guild));
}

#[test]
fn declined_and_timed_out_invites_carry_the_actor_name() {
    let mut service = GuildService::new(60_000);
    let a = name(101);
    let b = name(202);
    let caller_a = GuildCaller {
        character_guid: 101,
        name: &a,
        eligible: true,
        ignored_pair: false,
    };
    let caller_b = GuildCaller {
        character_guid: 202,
        name: &b,
        eligible: true,
        ignored_pair: false,
    };
    service.handle(&caller_a, &resolve, &create("Lantern Guild"), 1_000);
    let guild = service.store().guild_guid_for(101).unwrap();

    service.handle(&caller_a, &resolve, &invite(guild, 202, ""), 1_100);
    let declined = service.handle(&caller_b, &resolve, &one_guid(9, guild), 1_200);
    let notice = declined
        .messages
        .iter()
        .find(|message| sub(&message.bytes) == 28 && message.recipient_guid == 101)
        .expect("inviter sees the decline notice");
    assert_eq!(
        notification_name(&notice.bytes),
        (28, "Player202 Tester".into(), Some(false))
    );

    service.handle(&caller_a, &resolve, &invite(guild, 202, ""), 1_300);
    let timed_out = service.handle(&caller_b, &resolve, &one_guid(10, guild), 1_400);
    let notice = timed_out
        .messages
        .iter()
        .find(|message| sub(&message.bytes) == 28 && message.recipient_guid == 101)
        .expect("inviter sees the timeout notice");
    assert_eq!(
        notification_name(&notice.bytes),
        (28, "Player202 Tester".into(), Some(true))
    );
}

#[test]
fn forged_accept_is_silent_and_keeps_the_pending_invite() {
    let mut service = GuildService::new(60_000);
    let a = name(101);
    let b = name(202);
    service.handle(&caller(101, &a), &resolve, &create("Lantern Guild"), 1_000);
    let guild = service.store().guild_guid_for(101).unwrap();
    service.handle(&caller(101, &a), &resolve, &invite(guild, 202, ""), 1_100);

    let refused = service.handle(&caller(202, &b), &resolve, &one_guid(8, guild + 1), 1_200);
    assert!(refused.messages.is_empty(), "a forged accept is silent");
    assert_eq!(service.store().guild_guid_for(202), None);

    // The pending row survived the refusal: the correct accept still joins.
    let accepted = service.handle(&caller(202, &b), &resolve, &one_guid(8, guild), 1_300);
    assert!(
        accepted
            .messages
            .iter()
            .any(|message| { sub(&message.bytes) == 22 && message.recipient_guid == 202 })
    );
    assert_eq!(service.store().guild_guid_for(202), Some(guild));
}

#[test]
fn abandoned_invites_are_reclaimed_after_the_grace_window() {
    let mut service = GuildService::new(60_000);
    let a = name(101);
    let b = name(202);
    service.handle(&caller(101, &a), &resolve, &create("Lantern Guild"), 1_000);
    let guild = service.store().guild_guid_for(101).unwrap();
    service.handle(&caller(101, &a), &resolve, &invite(guild, 202, ""), 1_000);

    // One window of grace after expiry still answers a late accept with the timeout notice.
    let late = service.handle(&caller(202, &b), &resolve, &one_guid(8, guild), 100_000);
    assert!(
        late.messages
            .iter()
            .any(|message| sub(&message.bytes) == 28)
    );

    // A row whose target never answered is reclaimed by a later guild request.
    service.handle(&caller(101, &a), &resolve, &invite(guild, 202, ""), 100_001);
    service.handle(&caller(101, &a), &resolve, &one_guid(14, guild), 300_000);
    let stale = service.handle(&caller(202, &b), &resolve, &one_guid(8, guild), 300_001);
    assert!(
        stale.messages.is_empty(),
        "a reclaimed invite is not pending"
    );
    assert_eq!(service.store().guild_guid_for(202), None);
}

#[test]
fn an_expired_invite_does_not_block_a_new_one() {
    let mut store = GuildStore::new();
    let guild = store.create_guild(1, "Guild-A", &name(1), true).unwrap();
    store.invite(1, guild, 202, false, false, 0, 100).unwrap();
    store.invite(1, guild, 202, false, false, 200, 100).unwrap();
    assert_eq!(store.guild_guid_for(202), None);
}

/// A leader who quits is removed (status 3) and the successor is promoted (status 4);
/// the removed member is never encoded as demoted (status 5) to the client.
#[test]
fn leader_quit_encodes_leave_and_succession_never_a_demotion() {
    let mut service = GuildService::new(60_000);
    let a = name(101);
    let b = name(202);
    service.handle(&caller(101, &a), &resolve, &create("Lantern Guild"), 1_000);
    let guild = service.store().guild_guid_for(101).unwrap();
    service.handle(&caller(101, &a), &resolve, &invite(guild, 202, ""), 1_100);
    service.handle(&caller(202, &b), &resolve, &one_guid(8, guild), 1_200);

    let quit = service.handle(&caller(101, &a), &resolve, &one_guid(7, guild), 1_300);
    let statuses: Vec<(u64, u32, bool, u32)> = quit
        .messages
        .iter()
        .filter(|message| sub(&message.bytes) == 17)
        .map(|message| {
            let (_, _, _, code, online, rank) = status(&message.bytes);
            (message.recipient_guid, code, online, rank)
        })
        .collect();
    let leader = GuildRank::Leader as u32;
    assert!(
        statuses.contains(&(101, 3, false, leader)),
        "the departing leader is told it left: {statuses:?}"
    );
    assert!(
        statuses.contains(&(202, 4, true, leader)),
        "the successor is promoted to Leader: {statuses:?}"
    );
    assert!(
        statuses.contains(&(202, 3, false, leader)),
        "the remaining member sees the leader leave: {statuses:?}"
    );
    assert!(
        statuses.iter().all(|row| row.1 != 5),
        "a removed member is never encoded as demoted: {statuses:?}"
    );
    assert_eq!(service.store().guild_guid_for(101), None);
    assert_eq!(
        service.store().guild(guild).unwrap().members[&202].rank,
        GuildRank::Leader
    );
}

/// A timed-out accept only changed runtime pending state: it writes no store and sends no
/// roster (`handle_decoded` must distinguish "joined" from "timed out").
#[test]
fn a_timed_out_accept_changes_no_persisted_state() {
    let temp = TempDir::new();
    let path = temp.path().join("guilds.json");
    let mut service = GuildService::new(100);
    let a = name(101);
    let b = name(202);
    service.handle(&caller(101, &a), &resolve, &create("Lantern Guild"), 0);
    let guild = service.store().guild_guid_for(101).unwrap();
    service.handle(&caller(101, &a), &resolve, &invite(guild, 202, ""), 0);
    service.save(&path).unwrap();
    assert!(!service.is_dirty());
    let before = std::fs::read(&path).unwrap();

    let late = service.handle(&caller(202, &b), &resolve, &one_guid(8, guild), 1_000);
    assert!(
        late.messages
            .iter()
            .any(|message| sub(&message.bytes) == 28)
    );
    assert!(
        late.messages
            .iter()
            .all(|message| sub(&message.bytes) != 22),
        "a timed-out accept must not receive a guild roster"
    );
    assert!(!service.is_dirty(), "only runtime pending state changed");
    assert_eq!(std::fs::read(&path).unwrap(), before);
    assert_eq!(service.store().guild_guid_for(202), None);
}

/// The expired-invite sweep is documented runtime cleanup (review 19): a refused request is
/// answered against the rows it found and changes no persisted state, even when this handled
/// request also reclaims the stale runtime row.
#[test]
fn a_refused_request_changes_no_persisted_state_even_when_the_sweep_runs() {
    let temp = TempDir::new();
    let path = temp.path().join("guilds.json");
    let mut service = GuildService::new(1_000);
    let a = name(101);
    let b = name(202);
    service.handle(&caller(101, &a), &resolve, &create("Lantern Guild"), 0);
    let guild = service.store().guild_guid_for(101).unwrap();
    service.handle(&caller(101, &a), &resolve, &invite(guild, 202, ""), 0);
    service.save(&path).unwrap();
    assert!(!service.is_dirty());
    let before = std::fs::read(&path).unwrap();

    // A forged accept is refused and silent; the invite is far past its one-window grace,
    // so the handled request also reclaims the stale runtime row (runtime-only cleanup).
    let refused = service.handle(&caller(202, &b), &resolve, &one_guid(8, guild + 1), 10_000);
    assert!(refused.messages.is_empty(), "a forged accept is silent");
    assert!(
        !service.is_dirty(),
        "the refusal changed no persisted state"
    );
    assert_eq!(std::fs::read(&path).unwrap(), before);
    assert_eq!(service.store().guild_guid_for(202), None);

    // The handled request also reclaimed the stale runtime row: a later valid accept is
    // silent instead of the one-window-grace timed-out notice.
    let after_sweep = service.handle(&caller(202, &b), &resolve, &one_guid(8, guild), 10_001);
    assert!(
        after_sweep.messages.is_empty(),
        "the stale runtime invite was reclaimed"
    );
}
