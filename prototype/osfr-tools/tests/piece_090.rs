//! Work item 090 acceptance: guild protocol recovery plus a bounded create/member/role model.
//!
//! ORIGINAL evidence used by these vectors (all outside this test):
//! * `lab/decompiled/server/LibSOE/SOE/BasePackets.cs` - `BaseGuildPacket = 129` and
//!   `BaseAdminGuildPacket = 131` (SHA-256 `d4b863f4...`).
//! * `lab/decompiled/server/LibSOE/SOE/BaseGuildPackets.cs` - the 28 numbered guild sub-packets
//!   (SHA-256 `9f723688...`); `BaseAdminGuildPackets.cs` names `AdminGuildServerVersionPacket = 41`.
//! * `lab/decompiled/client/FreeRealms/pseudocode.c` (SHA-256 `f84a5867...`) - the native
//!   `BaseGuildPacket` constructor writes `param_1[1] = 0x81` and `BaseAdminGuildPacket` writes
//!   `0x83`; every one of the 28 guild classes owns a constructor calling `FUN_00a09b70(<sub>)`
//!   with the same id as the enum, so the ids are confirmed by the shipped client binary.
//! * `lab/decompiled/client-ui/ScriptsBase.lua` (SHA-256 `c7c71c7c...`) - the `BaseClient.Guilds`
//!   columns (GuildGuid, Name, MessageOfTheDay, CanCreateGuild) and the 15 `BaseClient.GuildMembers`
//!   columns, which match the client's own column-name tables (`Guild Guid`/`MOTD`/`CanCreateGuild?`/
//!   `CanRenameGuild?` and `Guid`..`Is Friend`).
//!
//! COMMUNITY evidence (recorded, not original): the wire field order, the role ids, the `Recruit`
//! join role, the invite/promote permission split, the leader-succession ordering and the 100-member
//! default come from the pinned AGPL-3.0 `lab/references/Sanctuary` server. The client-bound
//! sub-packets it does not implement (2, 11, 26) have an enum id and a native class but no recovered
//! body, so this module refuses them instead of guessing.
//!
//! RECONSTRUCTED policy (labelled, local): the store's monotonic guild ids, the caller-supplied
//! invite deadline, the "no second Leader through promote" rule and the event list. No historical
//! balance number, reward or MOTD body is invented; the `GuildDataFullPacket` body keeps unrecovered
//! `Unknown`/`Unknown2` fields and is therefore not encoded. Nothing here sends a packet, grants a
//! rank, or touches currency, items or experience.
//!
//! This file is a `#[path]` include so it compiles and exercises the real module before the lead
//! registers `pub mod piece_090;` under `src/restoration/mod.rs`/`src/lib.rs`.

#[path = "../src/restoration/piece_090.rs"]
#[allow(dead_code)]
mod piece_090;

use piece_090::{
    ADMIN_GUILD_MAIN, ClientGuildRequest, GUILD_MAIN, GuildError, GuildEvent, GuildRole,
    GuildStore, MAX_WIRE_STRING_LEN, MEMBER_UPDATE_DEMOTED, MEMBER_UPDATE_JOINED,
    MEMBER_UPDATE_PROMOTED, MEMBER_UPDATE_QUIT, MEMBER_UPDATE_REMOVED, MemberStatus, NAME_MAX_LEN,
    NameData, PAID_RENAME_INVALID, PAID_RENAME_NOT_LEADER, PAID_RENAME_TAKEN, can_create_guild,
    guild_error, member_status, name_update, paid_rename_reply, parse_client_request,
    player_status, sub_opcode_name,
};

fn request(sub: u16, body: &[u8]) -> Vec<u8> {
    let mut packet = GUILD_MAIN.to_le_bytes().to_vec();
    packet.extend_from_slice(&sub.to_le_bytes());
    packet.extend_from_slice(body);
    packet
}

fn push_guid(packet: &mut Vec<u8>, value: u64) {
    packet.extend_from_slice(&value.to_le_bytes());
}

fn push_string(packet: &mut Vec<u8>, value: &str) {
    packet.extend_from_slice(&(value.len() as u32).to_le_bytes());
    packet.extend_from_slice(value.as_bytes());
}

fn name_data() -> NameData {
    NameData {
        first_name_id: 7,
        last_name_prefix_id: 0,
        last_name_suffix_id: 0,
        first_name: "Alice".into(),
        last_name: String::new(),
    }
}

#[test]
fn sub_opcode_table_matches_the_original_enum() {
    assert_eq!(129, GUILD_MAIN);
    assert_eq!(131, ADMIN_GUILD_MAIN);
    assert_eq!(Some("GuildCreatePacket"), sub_opcode_name(1));
    assert_eq!(Some("GuildDeletePacket"), sub_opcode_name(2));
    assert_eq!(Some("GuildMemberLocationRequest"), sub_opcode_name(15));
    assert_eq!(Some("GuildDataFullPacket"), sub_opcode_name(22));
    assert_eq!(Some("GuildCanCreateGuildPacket"), sub_opcode_name(32));
    assert_eq!(None, sub_opcode_name(12));
    assert_eq!(None, sub_opcode_name(23));
    assert_eq!(None, sub_opcode_name(33));
}

#[test]
fn create_invite_and_role_requests_parse_with_the_recovered_layout() {
    let mut create = Vec::new();
    push_string(&mut create, "Old Name");
    push_string(&mut create, " Moonlit Pact ");
    push_string(&mut create, "en_US");
    assert_eq!(
        ClientGuildRequest::Create {
            name: "Old Name".into(),
            temporary_name: " Moonlit Pact ".into(),
            locale: "en_US".into(),
        },
        parse_client_request(&request(1, &create)).unwrap()
    );

    let mut invite = Vec::new();
    push_guid(&mut invite, 0x11);
    push_guid(&mut invite, 0x22);
    push_string(&mut invite, "Alice");
    assert_eq!(
        ClientGuildRequest::Invite {
            player_guid: 0x11,
            guild_guid: 0x22,
            player_name: "Alice".into(),
        },
        parse_client_request(&request(3, &invite)).unwrap()
    );

    for sub in [4u16, 5, 6] {
        let mut body = Vec::new();
        push_guid(&mut body, 0x33);
        push_guid(&mut body, 0x44);
        let expected = match sub {
            4 => ClientGuildRequest::Promote {
                player_guid: 0x33,
                guild_guid: 0x44,
            },
            5 => ClientGuildRequest::Demote {
                player_guid: 0x33,
                guild_guid: 0x44,
            },
            _ => ClientGuildRequest::Remove {
                player_guid: 0x33,
                guild_guid: 0x44,
            },
        };
        assert_eq!(
            expected,
            parse_client_request(&request(sub, &body)).unwrap(),
            "sub {sub}"
        );
    }
}

#[test]
fn quit_accept_decline_timeout_rename_and_paid_rename_parse() {
    let mut quit = Vec::new();
    push_guid(&mut quit, 0x55);
    assert_eq!(
        ClientGuildRequest::Quit { guid: 0x55 },
        parse_client_request(&request(7, &quit)).unwrap()
    );

    for sub in [8u16, 9, 10, 14] {
        let mut body = Vec::new();
        push_guid(&mut body, 0x66);
        let expected = match sub {
            8 => ClientGuildRequest::InviteAccept { player_guid: 0x66 },
            9 => ClientGuildRequest::InviteDecline { player_guid: 0x66 },
            10 => ClientGuildRequest::InviteTimeout { player_guid: 0x66 },
            _ => ClientGuildRequest::NameRequest { guid: 0x66 },
        };
        assert_eq!(
            expected,
            parse_client_request(&request(sub, &body)).unwrap(),
            "sub {sub}"
        );
    }

    let mut rename = Vec::new();
    push_guid(&mut rename, 0x77);
    push_string(&mut rename, "Moonlit Pact");
    push_string(&mut rename, "en_US");
    rename.push(1);
    assert_eq!(
        ClientGuildRequest::Rename {
            guid: 0x77,
            name: "Moonlit Pact".into(),
            locale: "en_US".into(),
            is_non_china_environment: true,
        },
        parse_client_request(&request(13, &rename)).unwrap()
    );

    let mut paid = Vec::new();
    push_guid(&mut paid, 0x88);
    push_string(&mut paid, "New Pact");
    assert_eq!(
        ClientGuildRequest::PaidRenameCheck {
            guid: 0x88,
            name: "New Pact".into(),
        },
        parse_client_request(&request(16, &paid)).unwrap()
    );
}

#[test]
fn malformed_requests_are_refused() {
    let mut invite = Vec::new();
    push_guid(&mut invite, 0x11);
    push_guid(&mut invite, 0x22);
    push_string(&mut invite, "Alice");
    let good = request(3, &invite);
    assert!(matches!(
        parse_client_request(&good).unwrap(),
        ClientGuildRequest::Invite { .. }
    ));
    for length in 0..good.len() {
        assert!(
            parse_client_request(&good[..length]).is_err(),
            "truncated at {length}"
        );
    }
    let mut extra = good.clone();
    extra.push(0);
    assert_eq!(Err(GuildError::TrailingBytes), parse_client_request(&extra));
    assert_eq!(
        Err(GuildError::BadMainOpcode),
        parse_client_request(&[15, 0, 1, 0])
    );
    assert_eq!(
        Err(GuildError::UnknownSubtype),
        parse_client_request(&request(12, &[]))
    );
    // 32 is a server-to-client reply; 2 is client-bound but its body was never recovered.
    assert_eq!(
        Err(GuildError::ServerBoundSubtype { sub: 32 }),
        parse_client_request(&request(32, &[0]))
    );
    assert_eq!(
        Err(GuildError::UnrecoveredBody { sub: 2 }),
        parse_client_request(&request(2, &[0; 8]))
    );
    assert_eq!(
        Err(GuildError::UnrecoveredBody { sub: 26 }),
        parse_client_request(&request(26, &[0; 8]))
    );

    let mut oversize = Vec::new();
    push_guid(&mut oversize, 0x11);
    push_guid(&mut oversize, 0x22);
    push_string(&mut oversize, &"x".repeat(MAX_WIRE_STRING_LEN + 1));
    assert_eq!(
        Err(GuildError::OversizedString),
        parse_client_request(&request(3, &oversize))
    );
}

#[test]
fn location_request_keeps_a_body_the_community_class_does_not_model() {
    // The native constructor allocates a character-GUID array beside the 8-byte pair, so a non-empty
    // body is preserved rather than interpreted.
    assert_eq!(
        ClientGuildRequest::MemberLocationRequest {
            trailing: Vec::new()
        },
        parse_client_request(&request(15, &[])).unwrap()
    );
    assert_eq!(
        ClientGuildRequest::MemberLocationRequest {
            trailing: vec![1, 2, 3, 4],
        },
        parse_client_request(&request(15, &[1, 2, 3, 4])).unwrap()
    );
}

#[test]
fn replies_encode_the_recovered_field_order() {
    let mut expected = vec![129, 0, 23, 0];
    push_string(&mut expected, "GuildNameAlreadyExists");
    assert_eq!(expected, guild_error("GuildNameAlreadyExists"));

    assert_eq!(vec![129, 0, 32, 0, 1], can_create_guild(true));
    assert_eq!(vec![129, 0, 32, 0, 0], can_create_guild(false));

    let mut expected = vec![129, 0, 31, 0];
    push_guid(&mut expected, 0x11);
    push_guid(&mut expected, 0x22);
    expected.push(1);
    assert_eq!(expected, player_status(0x11, 0x22, true));

    let mut expected = vec![129, 0, 18, 0];
    push_guid(&mut expected, 0x22);
    push_string(&mut expected, "Moonlit Pact");
    assert_eq!(expected, name_update(0x22, "Moonlit Pact"));

    let mut expected = vec![129, 0, 30, 0];
    push_guid(&mut expected, 0x22);
    push_string(&mut expected, "New Pact");
    expected.extend_from_slice(&PAID_RENAME_TAKEN.to_le_bytes());
    assert_eq!(
        expected,
        paid_rename_reply(0x22, "New Pact", PAID_RENAME_TAKEN)
    );
    assert_eq!(1, PAID_RENAME_NOT_LEADER);
    assert_eq!(2, PAID_RENAME_INVALID);

    let mut expected = vec![129, 0, 17, 0];
    push_guid(&mut expected, 0x22);
    push_guid(&mut expected, 0x11);
    for value in [7i32, 0, 0] {
        expected.extend_from_slice(&value.to_le_bytes());
    }
    push_string(&mut expected, "Alice");
    push_string(&mut expected, "");
    for value in [GuildRole::Officer.id(), MEMBER_UPDATE_PROMOTED, 3, 0, 48, 9] {
        expected.extend_from_slice(&value.to_le_bytes());
    }
    // Online sits between Role and Type: 4 header + 8 + 8 + 12 name ids + 9 + 4 strings + 4 role.
    expected.insert(49, 1);
    let status = MemberStatus {
        guild_guid: 0x22,
        member_guid: 0x11,
        name: name_data(),
        role: GuildRole::Officer.id(),
        online: true,
        update_type: MEMBER_UPDATE_PROMOTED,
        world_id: 3,
        unknown: 0,
        profile_id: 48,
        profile_rank: 9,
    };
    assert_eq!(expected, member_status(&status));
}

#[test]
fn guild_lifecycle_creates_invites_and_changes_roles() {
    let mut store = GuildStore::new(10);
    let leader = 0x100;
    let officer = 0x200;
    let recruit = 0x300;

    let guild = match store.create_guild(leader, "  Moonlit   Pact ").unwrap() {
        GuildEvent::Created { guild, creator } => {
            assert_eq!(leader, creator);
            guild
        }
        other => panic!("unexpected {other:?}"),
    };
    assert_eq!(Some("Moonlit Pact"), store.guild_name(guild));
    assert_eq!(Some(GuildRole::Leader), store.role_of(leader));
    assert_eq!(
        Err(GuildError::NameTaken),
        store.create_guild(0x999, "moonlit pact")
    );
    assert_eq!(
        Err(GuildError::AlreadyInGuild),
        store.create_guild(leader, "Second Pact")
    );
    assert_eq!(
        Err(GuildError::NameTooShort),
        store.create_guild(0x998, "ab")
    );
    assert_eq!(
        Err(GuildError::IllegalNameCharacter),
        store.create_guild(0x998, "Bad<Name>")
    );

    store.invite(leader, officer, 10).unwrap();
    store.invite(leader, recruit, 10).unwrap();
    assert_eq!(
        Err(GuildError::NoInviteForPlayer),
        store.accept_invite(0x999, guild, 11, 1_000)
    );
    assert_eq!(
        GuildEvent::MemberJoined {
            guild,
            member: officer,
            role: GuildRole::Recruit,
        },
        store.accept_invite(officer, guild, 11, 1_000).unwrap()
    );
    assert_eq!(Some(GuildRole::Recruit), store.role_of(officer));
    assert_eq!(
        Err(GuildError::InviteExpired),
        store.accept_invite(recruit, guild, 99, 0)
    );
    store.invite(leader, recruit, 100).unwrap();
    store.accept_invite(recruit, guild, 101, 1_000).unwrap();
    assert_eq!(3, store.member_count(guild));
    // A recruit cannot invite anyone.
    assert_eq!(
        Err(GuildError::NotPermitted),
        store.invite(recruit, 0x400, 30)
    );
    assert_eq!(
        Err(GuildError::AlreadyInGuild),
        store.accept_invite(recruit, guild, 102, 1_000)
    );

    // Recruit -> Member -> Officer through the leader, then an officer handles a lower rank.
    assert_eq!(
        GuildEvent::RoleChanged {
            guild,
            member: officer,
            role: GuildRole::Member,
            promoted: true,
        },
        store.promote(leader, officer).unwrap()
    );
    store.promote(leader, officer).unwrap();
    assert_eq!(Some(GuildRole::Officer), store.role_of(officer));
    assert_eq!(
        GuildEvent::RoleChanged {
            guild,
            member: recruit,
            role: GuildRole::Member,
            promoted: true,
        },
        store.promote(officer, recruit).unwrap()
    );
    store.promote(leader, recruit).unwrap();
    assert_eq!(Some(GuildRole::Officer), store.role_of(recruit));

    // Refusals: an officer cannot touch another officer, nobody promotes to Leader, no self targets.
    assert_eq!(
        Err(GuildError::CantPromoteAbove),
        store.promote(officer, recruit)
    );
    assert_eq!(
        Err(GuildError::CannotPromoteToLeader),
        store.promote(leader, recruit)
    );
    assert_eq!(Err(GuildError::SelfTarget), store.promote(leader, leader));

    assert_eq!(
        GuildEvent::RoleChanged {
            guild,
            member: recruit,
            role: GuildRole::Member,
            promoted: false,
        },
        store.demote(leader, recruit).unwrap()
    );
    store.demote(leader, recruit).unwrap();
    assert_eq!(Some(GuildRole::Recruit), store.role_of(recruit));
    assert_eq!(Err(GuildError::AtMinRank), store.demote(leader, recruit));
    assert_eq!(
        Err(GuildError::CantPromoteAbove),
        store.demote(officer, officer)
    );
}

#[test]
fn capacity_removal_and_leader_succession_are_enforced() {
    let mut store = GuildStore::new(3);
    let (leader, second, third) = (1, 2, 3);
    let guild = match store.create_guild(leader, "Small Pact").unwrap() {
        GuildEvent::Created { guild, .. } => guild,
        other => panic!("unexpected {other:?}"),
    };
    for (member, tick) in [(second, 1u64), (third, 2)] {
        store.invite(leader, member, tick).unwrap();
        store.accept_invite(member, guild, tick, 100).unwrap();
    }
    assert_eq!(3, store.member_count(guild));
    assert_eq!(Err(GuildError::GuildFull), store.invite(leader, 4, 3));
    assert_eq!(
        Err(GuildError::NoInviteForPlayer),
        store.accept_invite(4, guild, 3, 100)
    );
    let mut tiny = GuildStore::new(1);
    tiny.create_guild(1, "Tiny Pact").unwrap();
    assert_eq!(Err(GuildError::GuildFull), tiny.invite(1, 2, 0));

    // The leader leaves: the senior remaining member takes over and the guild survives.
    assert_eq!(
        GuildEvent::MemberLeft {
            guild,
            member: leader
        },
        store.quit(leader).unwrap()
    );
    assert_eq!(Some(GuildRole::Leader), store.role_of(second));
    assert_eq!(2, store.member_count(guild));
    assert_eq!(Err(GuildError::NotInGuild), store.quit(leader));

    assert_eq!(
        GuildEvent::MemberRemoved {
            guild,
            member: third,
            actor: second,
        },
        store.remove(second, third).unwrap()
    );
    assert_eq!(
        Err(GuildError::TargetNotMember),
        store.remove(second, third)
    );
    assert_eq!(
        Err(GuildError::TargetNotMember),
        store.remove(second, leader)
    );

    // The last member leaves: the guild is deleted and its guid is never reused.
    assert_eq!(
        GuildEvent::GuildDisbanded { guild },
        store.quit(second).unwrap()
    );
    assert_eq!(None, store.guild_name(guild));
    let next = match store.create_guild(9, "Later Pact").unwrap() {
        GuildEvent::Created { guild, .. } => guild,
        other => panic!("unexpected {other:?}"),
    };
    assert_ne!(guild, next);
}

#[test]
fn rename_is_leader_only_and_state_survives_a_round_trip() {
    let mut store = GuildStore::new(5);
    let (leader, member) = (0x10, 0x20);
    let guild = match store.create_guild(leader, "First Pact").unwrap() {
        GuildEvent::Created { guild, .. } => guild,
        other => panic!("unexpected {other:?}"),
    };
    store.create_guild(0x30, "Taken Pact").unwrap();
    store.invite(leader, member, 1).unwrap();
    store.accept_invite(member, guild, 2, 100).unwrap();

    assert_eq!(
        Err(GuildError::NotPermitted),
        store.rename(member, "Other Pact")
    );
    assert_eq!(
        Err(GuildError::NameTaken),
        store.rename(leader, "taken pact")
    );
    assert_eq!(
        GuildEvent::GuildRenamed {
            guild,
            name: "Second Pact".into(),
        },
        store.rename(leader, "Second Pact").unwrap()
    );

    let json = store.to_json().unwrap();
    let restored = GuildStore::from_json(&json).unwrap();
    assert_eq!(store, restored);
    assert_eq!(Some("Second Pact"), restored.guild_name(guild));
    assert_eq!(Some(GuildRole::Recruit), restored.role_of(member));
    assert_eq!(3, restored.next_guild_id());
    assert_eq!(32, NAME_MAX_LEN);
    assert!(store.rename(leader, &"n".repeat(32)).is_ok());
    assert_eq!(
        Err(GuildError::NameTooLong),
        store.rename(leader, &"n".repeat(33))
    );

    assert_eq!(MEMBER_UPDATE_JOINED, 1);
    assert_eq!(MEMBER_UPDATE_REMOVED, 2);
    assert_eq!(MEMBER_UPDATE_QUIT, 3);
    assert_eq!(MEMBER_UPDATE_DEMOTED, 5);
}
