use guilds::{
    GuildError, GuildMemberName, GuildNoticeKind, GuildRank, GuildResultCode, GuildStore,
    decode_request, error_message,
};
use osfr_tools::{guilds, social_chat};
use social_chat::{
    ChatChannel, ChatRequest, ChatSession, IgnoreLists, NameData, resolve_recipients,
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
        let path = std::env::temp_dir().join(format!("osfr-guilds-{}-{id}", std::process::id()));
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

fn three_members() -> (GuildStore, u64) {
    let mut store = GuildStore::new();
    let guild = store.create_guild(1, "Guild-A", &member(1), true).unwrap();
    for guid in [2, 3] {
        store
            .invite(1, guild, guid, false, false, 0, 60_000)
            .unwrap();
        store
            .accept_invite(guid, guild, 1, false, &member(guid))
            .unwrap();
    }
    (store, guild)
}

fn member(guid: u64) -> GuildMemberName {
    GuildMemberName::new(format!("First{guid}"), "Last".to_string())
}

fn request_header(sub: u16) -> Vec<u8> {
    let mut bytes = vec![129, 0];
    bytes.extend_from_slice(&sub.to_le_bytes());
    bytes
}

fn push_string(bytes: &mut Vec<u8>, value: &str) {
    bytes.extend_from_slice(&(value.len() as i32).to_le_bytes());
    bytes.extend_from_slice(value.as_bytes());
}

#[test]
fn full_lifecycle_tracks_three_members_and_presence() {
    let (mut store, guild) = three_members();
    assert_eq!(store.roster(guild).unwrap().len(), 3);
    assert_eq!(
        store.guild(guild).unwrap().members[&1].rank,
        GuildRank::Leader
    );
    assert_eq!(
        store.guild(guild).unwrap().members[&2].rank,
        GuildRank::Recruit
    );

    store.promote(1, 2).unwrap();
    assert_eq!(
        store.guild(guild).unwrap().members[&2].rank,
        GuildRank::Member
    );
    store.promote(1, 2).unwrap();
    assert_eq!(
        store.guild(guild).unwrap().members[&2].rank,
        GuildRank::Officer
    );
    store.demote(1, 2).unwrap();
    assert_eq!(
        store.guild(guild).unwrap().members[&2].rank,
        GuildRank::Member
    );
    store.set_motd(1, "").unwrap();

    let login = store.login(1).unwrap();
    assert!(login.iter().any(|notice| {
        notice.recipient_guid == 2
            && notice.kind
                == GuildNoticeKind::PresenceChanged {
                    member_guid: 1,
                    online: true,
                }
    }));
    assert!(
        store
            .roster(guild)
            .unwrap()
            .iter()
            .find(|m| m.character_guid == 1)
            .unwrap()
            .online
    );
    let logout = store.logout(2).unwrap();
    assert!(logout.iter().any(|notice| {
        notice.recipient_guid == 1
            && notice.kind
                == GuildNoticeKind::PresenceChanged {
                    member_guid: 2,
                    online: false,
                }
    }));

    store.leave(3).unwrap();
    assert_eq!(store.guild_guid_for(3), None);
    assert_eq!(store.guild(guild).unwrap().members.len(), 2);
}

#[test]
fn kick_and_rank_actions_enforce_permissions() {
    let (mut store, guild) = three_members();
    store.promote(1, 2).unwrap();
    store.promote(1, 2).unwrap();

    assert_eq!(
        store.kick(3, 2),
        Err(GuildError::Client(GuildResultCode::NotHighRankToRemove))
    );
    assert_eq!(
        store.kick(2, 1),
        Err(GuildError::Client(GuildResultCode::RemoveCantRemoveAbove))
    );
    assert_eq!(
        store.promote(1, 2),
        Err(GuildError::Client(GuildResultCode::PromoteAtMaxRank))
    );
    store.promote(2, 3).unwrap();
    assert_eq!(
        store.guild(guild).unwrap().members[&3].rank,
        GuildRank::Member
    );
    store.kick(2, 3).unwrap();
    assert_eq!(store.guild_guid_for(3), None);
}

#[test]
fn leader_departure_transfers_to_highest_rank_then_lowest_guid() {
    let (mut store, guild) = three_members();
    store.promote(1, 2).unwrap();
    store.promote(1, 2).unwrap();
    store.promote(1, 3).unwrap();
    store.promote(1, 3).unwrap();

    store.leave(1).unwrap();
    assert_eq!(store.guild_guid_for(1), None);
    assert_eq!(
        store.guild(guild).unwrap().members[&2].rank,
        GuildRank::Leader
    );
    assert_eq!(
        store.guild(guild).unwrap().members[&3].rank,
        GuildRank::Officer
    );
    assert_eq!(
        store.transfer_leadership(2, 3).unwrap()[0].kind,
        GuildNoticeKind::LeadershipTransferred {
            from_guid: 2,
            to_guid: 3,
        }
    );
}

#[test]
fn leader_can_disband_and_all_character_references_are_removed() {
    let (mut store, guild) = three_members();
    store.login(1).unwrap();
    let notices = store.disband(1).unwrap();
    assert!(
        notices
            .iter()
            .any(|notice| notice.kind == GuildNoticeKind::Disbanded)
    );
    assert_eq!(store.guild(guild), None);
    for guid in [1, 2, 3] {
        assert_eq!(store.guild_guid_for(guid), None);
    }
}

#[test]
fn server_wide_store_roundtrips_and_reloads_members_offline() {
    let (mut store, guild) = three_members();
    store.login(1).unwrap();
    let temp = TempDir::new();
    let path = temp.path().join("guilds.json");
    store.save(&path).unwrap();

    let restored = GuildStore::load(&path).unwrap();
    assert_eq!(restored.guild(guild), store.guild(guild));
    assert_eq!(restored.character_guild_ref(2).unwrap().guild_guid, guild);
    assert!(!restored.is_online(1));
    assert_eq!(
        restored
            .roster(guild)
            .unwrap()
            .iter()
            .filter(|m| m.online)
            .count(),
        0
    );
}

#[test]
fn duplicate_guild_name_is_rejected_case_insensitively() {
    let mut store = GuildStore::new();
    assert_eq!(store.create_guild(1, "guild-a", &member(1), true), Ok(1));
    assert_eq!(
        store.create_guild(2, "GUILD-A", &member(2), true),
        Err(GuildError::Client(GuildResultCode::NameAlreadyExists))
    );
}

#[test]
fn invite_honors_ignore_and_autorefuse_and_expires() {
    let mut store = GuildStore::new();
    let guild = store.create_guild(1, "Guild-A", &member(1), true).unwrap();
    assert_eq!(
        store.invite(1, guild, 2, true, false, 0, 100),
        Err(GuildError::Client(GuildResultCode::InvitePlayerNotFound))
    );
    let refused = store.invite(1, guild, 2, false, true, 0, 100).unwrap();
    assert_eq!(
        refused[0].kind,
        GuildNoticeKind::InviteDeclined { target_guid: 2 }
    );
    store.invite(1, guild, 2, false, false, 0, 100).unwrap();
    let ignored = store.accept_invite(2, guild, 1, true, &member(2)).unwrap();
    assert_eq!(
        ignored[0].kind,
        GuildNoticeKind::InviteDeclined { target_guid: 2 }
    );
    store.invite(1, guild, 2, false, false, 0, 100).unwrap();
    let declined = store.decline_invite(2, guild).unwrap();
    assert_eq!(
        declined[0].kind,
        GuildNoticeKind::InviteDeclined { target_guid: 2 }
    );
    store.invite(1, guild, 2, false, false, 0, 100).unwrap();
    let timed_out = store
        .accept_invite(2, guild, 100, false, &member(2))
        .unwrap();
    assert_eq!(
        timed_out[0].kind,
        GuildNoticeKind::InviteTimedOut { target_guid: 2 }
    );
    assert_eq!(store.guild_guid_for(2), None);
}

#[test]
fn guild_membership_feeds_social_chat_recipient_resolution() {
    let (mut store, guild) = three_members();
    store.login(1).unwrap();
    store.login(2).unwrap();
    let sessions =
        [(10, 1), (20, 2), (30, 3), (40, 4)].map(|(session_id, character_guid)| ChatSession {
            session_id,
            character_guid,
            name: NameData::default(),
            zone_id: 1,
            position: [0.0; 3],
            chat_area_id: None,
            group_id: None,
            guild_id: store.guild_guid_for(character_guid),
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
    let recipients = resolve_recipients(&request, 10, &sessions, &IgnoreLists::default()).unwrap();
    assert_eq!(recipients, vec![10, 20, 30]);
}

#[test]
fn client_request_reader_uses_family_129_and_string_lengths() {
    // 129/11 body: guild GUID followed by an i32-length-prefixed string (`00c0f710`).
    let mut bytes = vec![129, 0, 11, 0];
    bytes.extend_from_slice(&7u64.to_le_bytes());
    bytes.extend_from_slice(&0i32.to_le_bytes());
    assert_eq!(
        decode_request(&bytes).unwrap(),
        guilds::GuildRequest::Motd {
            guild_guid: 7,
            text: String::new()
        }
    );
}

#[test]
fn all_supported_client_guild_request_bodies_decode() {
    let mut create = request_header(1);
    for value in ["Guild-A", "", ""] {
        push_string(&mut create, value);
    }
    assert!(matches!(
        decode_request(&create),
        Ok(guilds::GuildRequest::Create { .. })
    ));

    let mut invite = request_header(3);
    invite.extend_from_slice(&1u64.to_le_bytes());
    invite.extend_from_slice(&2u64.to_le_bytes());
    push_string(&mut invite, "");
    assert!(matches!(
        decode_request(&invite),
        Ok(guilds::GuildRequest::Invite { .. })
    ));

    for sub in [4, 5, 6] {
        let mut pair = request_header(sub);
        pair.extend_from_slice(&1u64.to_le_bytes());
        pair.extend_from_slice(&2u64.to_le_bytes());
        assert!(decode_request(&pair).is_ok());
    }
    for sub in [2, 7, 8, 9, 10, 14] {
        let mut single = request_header(sub);
        single.extend_from_slice(&7u64.to_le_bytes());
        assert!(decode_request(&single).is_ok());
    }

    let mut motd = request_header(11);
    motd.extend_from_slice(&7u64.to_le_bytes());
    push_string(&mut motd, "");
    assert!(decode_request(&motd).is_ok());

    let mut rename = request_header(13);
    rename.extend_from_slice(&7u64.to_le_bytes());
    push_string(&mut rename, "Guild-B");
    push_string(&mut rename, "");
    rename.push(0);
    assert!(decode_request(&rename).is_ok());

    let mut locations = request_header(15);
    locations.extend_from_slice(&7u64.to_le_bytes());
    locations.extend_from_slice(&2u32.to_le_bytes());
    locations.extend_from_slice(&2u64.to_le_bytes());
    locations.extend_from_slice(&3u64.to_le_bytes());
    assert!(decode_request(&locations).is_ok());

    let mut paid_name_check = request_header(16);
    paid_name_check.extend_from_slice(&7u64.to_le_bytes());
    push_string(&mut paid_name_check, "Guild-B");
    assert!(decode_request(&paid_name_check).is_ok());
}

#[test]
fn guild_error_reply_uses_family_129_sub_23_and_client_key() {
    let reply = error_message(7, GuildResultCode::NameNotValid);
    assert_eq!(&reply.bytes[..4], &[129, 0, 23, 0]);
    let length = i32::from_le_bytes(reply.bytes[4..8].try_into().unwrap()) as usize;
    assert_eq!(
        &reply.bytes[8..],
        GuildResultCode::NameNotValid.client_key().as_bytes()
    );
    assert_eq!(length, reply.bytes.len() - 8);
}
