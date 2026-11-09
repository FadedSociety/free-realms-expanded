//! Guilds piece: the name tag over a player's head.
//!
//! The client reader for 35/1 AddPc (`0091aa10`) drains a counted (dword key, guid pair) list
//! into +328; the viewer's name tag resolves each guid with 129/14 (`0096ab60` -> `00b1e960`)
//! and the server answers 129/18. A live change reaches current viewers through 129/31
//! (`GuildPlayerStatusUpdatePacket__ReadBody`, handler `00b1ce80`). These tests pin the three
//! layouts and the rule that any existing guild answers a name request.

use osfr_tools::{
    guilds::{GuildCaller, GuildRequest, GuildService, player_status_bytes},
    players::{InterestPolicy, MessageKind, PlayerRegistry, SessionId, ZoneInstance},
    social_chat::NameData,
};

const GUILD: u64 = 0x1122_3344_5566_7788;

fn registry() -> PlayerRegistry {
    PlayerRegistry::new(InterestPolicy::new(50.0, 8).unwrap())
}

fn zone() -> ZoneInstance {
    ZoneInstance {
        zone_id: 1,
        instance_id: 1,
    }
}

/// Player 1 joins (with or without a guild tag), then player 2 arrives next to them; the
/// bytes are player 1's AddPc as sent to player 2.
fn add_pc_seen_by_a_newcomer(guild_guid: u64) -> Vec<u8> {
    let mut registry = registry();
    registry
        .join(SessionId(1), 11, 101, zone(), [0.0, 0.0, 0.0], 0.0)
        .unwrap();
    registry.set_guild(SessionId(1), guild_guid);
    let messages = registry
        .join(SessionId(2), 12, 102, zone(), [1.0, 0.0, 0.0], 0.0)
        .unwrap();
    messages
        .into_iter()
        .find(|message| {
            message.recipient == SessionId(2) && matches!(message.kind, MessageKind::Spawn)
        })
        .expect("the newcomer is sent player 1's AddPc")
        .bytes
}

#[test]
fn add_pc_carries_one_guild_entry_under_key_zero() {
    let without = add_pc_seen_by_a_newcomer(0);
    let with = add_pc_seen_by_a_newcomer(GUILD);
    assert_eq!(
        with.len(),
        without.len() + 12,
        "one (key, pair) entry is added"
    );
    let mut entry = vec![1, 0, 0, 0, 0, 0, 0, 0];
    entry.extend_from_slice(&GUILD.to_le_bytes());
    let at = (0..with.len() - 16)
        .find(|index| with[*index..*index + 16] == entry[..])
        .expect("count 1, key 0, the guild guid");
    let tail = with.len() - (at + 16);
    let count_at = without.len() - tail - 4;
    assert_eq!(
        &without[count_at..count_at + 4],
        &[0, 0, 0, 0],
        "no guild: an empty list"
    );
    assert_eq!(
        &with[at + 16..],
        &without[count_at + 4..],
        "the rest of the frame is unchanged"
    );
    // Before the list only the frame's length prefixes move (by 12 bytes).
    let moved = with[..at]
        .iter()
        .zip(&without[..count_at])
        .filter(|(a, b)| a != b)
        .count();
    assert!(
        moved <= 4,
        "only length bytes differ before the list, {moved} did"
    );
}

#[test]
fn the_guild_tag_is_session_state_and_leaves_with_the_player() {
    let mut registry = registry();
    registry.set_guild(SessionId(1), GUILD); // unknown session: ignored, never panics
    assert_eq!(registry.guild_of(SessionId(1)), 0);
    registry
        .join(SessionId(1), 11, 101, zone(), [0.0, 0.0, 0.0], 0.0)
        .unwrap();
    registry.set_guild(SessionId(1), GUILD);
    assert_eq!(registry.guild_of(SessionId(1)), GUILD);
    registry.set_guild(SessionId(1), 0);
    assert_eq!(registry.guild_of(SessionId(1)), 0);
    registry.set_guild(SessionId(1), GUILD);
    registry.leave(SessionId(1));
    registry
        .join(SessionId(1), 11, 101, zone(), [0.0, 0.0, 0.0], 0.0)
        .unwrap();
    assert_eq!(
        registry.guild_of(SessionId(1)),
        0,
        "a relog starts without a mirrored tag"
    );
}

#[test]
fn player_status_update_is_player_then_guild_then_flag() {
    let bytes = player_status_bytes(601, GUILD, true);
    let mut expected = vec![129, 0, 31, 0];
    expected.extend_from_slice(&601u64.to_le_bytes());
    expected.extend_from_slice(&GUILD.to_le_bytes());
    expected.push(1);
    assert_eq!(bytes, expected);
    assert_eq!(player_status_bytes(601, GUILD, false).last(), Some(&0));
}

fn name_request(guild_guid: u64) -> Vec<u8> {
    let mut bytes = vec![129, 0, 14, 0];
    bytes.extend_from_slice(&guild_guid.to_le_bytes());
    bytes
}

#[test]
fn any_existing_guild_answers_a_name_request_from_a_stranger() {
    let mut service = GuildService::default();
    let founder_name = NameData::new("Lantern", "Keeper");
    let founder = GuildCaller {
        character_guid: 601,
        name: &founder_name,
        eligible: true,
        ignored_pair: false,
    };
    let mut create = vec![129, 0, 1, 0];
    for text in ["Lantern Guild", "en_US", ""] {
        create.extend_from_slice(&(text.len() as i32).to_le_bytes());
        create.extend_from_slice(text.as_bytes());
    }
    service.handle(&founder, &|_| None, &create, 1_000);
    let guild = service.store().guild_guid_for(601).expect("guild created");

    let stranger_name = NameData::new("Passer", "By");
    let stranger = GuildCaller {
        character_guid: 777,
        name: &stranger_name,
        eligible: false,
        ignored_pair: false,
    };
    assert!(service.responds_to(777, &GuildRequest::NameRequest { guild_guid: guild }));
    let outcome = service.handle(&stranger, &|_| None, &name_request(guild), 2_000);
    assert_eq!(outcome.messages.len(), 1, "one 129/18 for the stranger");
    let message = &outcome.messages[0];
    assert_eq!(message.recipient_guid, 777);
    let mut expected = vec![129, 0, 18, 0];
    expected.extend_from_slice(&guild.to_le_bytes());
    expected.extend_from_slice(&("Lantern Guild".len() as i32).to_le_bytes());
    expected.extend_from_slice(b"Lantern Guild");
    assert_eq!(message.bytes, expected);

    // A guid no guild owns stays silent and touches nothing (no enumeration by error).
    assert!(!service.responds_to(
        777,
        &GuildRequest::NameRequest {
            guild_guid: guild + 1
        }
    ));
    let silent = service.handle(&stranger, &|_| None, &name_request(guild + 1), 3_000);
    assert!(silent.messages.is_empty());
}
