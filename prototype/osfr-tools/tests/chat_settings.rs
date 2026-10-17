//! W173: chat channel switches (26/32, 26/33) and bubble colour (26/18).
//!
//! Wire vectors are the bytes the native client sent in the 2026-10-14 baseline trace
//! (channel names and numbers only, no player text). Everything else is synthetic.

use osfr_tools::social_chat::{
    ActionBudgets, BubbleStyle, ChatChannel, ChatError, ChatSession, ChatSettings,
    ChatSettingsBook, NameData, SettingsRequest, decode_settings_request,
};
use std::path::PathBuf;
use std::time::Instant;

fn channel_packet(sub: u16, name: &str) -> Vec<u8> {
    let mut out = vec![26, 0];
    out.extend(sub.to_le_bytes());
    out.extend((name.len() as i32).to_le_bytes());
    out.extend(name.as_bytes());
    out
}

fn bubble_packet(fg: i32, bg: i32, size: i32, guid: u64) -> Vec<u8> {
    let mut out = vec![26, 0, 18, 0];
    for value in [fg, bg, size] {
        out.extend(value.to_le_bytes());
    }
    out.extend(guid.to_le_bytes());
    out
}

fn hex(text: &str) -> Vec<u8> {
    (0..text.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&text[i..i + 2], 16).unwrap())
        .collect()
}

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("osfr-w173-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn native_trace_bodies_decode() {
    // 26/32 "WorldMembersOnly" and 26/18 as the native client sent them.
    let on = hex("1a00200010000000576f726c644d656d626572734f6e6c79");
    assert_eq!(
        decode_settings_request(&on),
        Ok(SettingsRequest::Channel {
            channel: ChatChannel::MembersOnly,
            on: true
        })
    );
    let bubble = hex("1a00120066330000ffffcc00020000000000000000000000");
    assert_eq!(
        decode_settings_request(&bubble),
        Ok(SettingsRequest::Bubble(BubbleStyle {
            foreground: 13_158,
            background: 13_434_879,
            size: 2
        }))
    );
}

#[test]
fn the_four_client_names_map_to_the_pinned_channel_numbers() {
    for (name, channel, wire) in [
        ("WorldArea", ChatChannel::Area, 8),
        ("WorldTrade", ChatChannel::Trade, 6),
        ("WorldLfg", ChatChannel::LookingForGroup, 7),
        ("WorldMembersOnly", ChatChannel::MembersOnly, 10),
    ] {
        assert_eq!(channel.wire(), wire);
        for (sub, on) in [(32, true), (33, false)] {
            assert_eq!(
                decode_settings_request(&channel_packet(sub, name)),
                Ok(SettingsRequest::Channel { channel, on })
            );
        }
    }
    // The client compares ignoring case.
    assert_eq!(
        decode_settings_request(&channel_packet(33, "worldtrade")),
        Ok(SettingsRequest::Channel {
            channel: ChatChannel::Trade,
            on: false
        })
    );
}

#[test]
fn forged_malformed_and_out_of_range_requests_are_refused() {
    let big = "x".repeat(33);
    for name in [
        "WorldSay",
        "GroupSay",
        "WorldShout",
        "Tell",
        "",
        " WorldArea",
        "WorldArea\0",
    ] {
        assert_eq!(
            decode_settings_request(&channel_packet(32, name)),
            Err(ChatError::InvalidChannel),
            "{name:?}"
        );
    }
    assert_eq!(
        decode_settings_request(&channel_packet(32, &big)),
        Err(ChatError::InvalidPacket)
    );
    let good = channel_packet(32, "WorldArea");
    for cut in 0..good.len() {
        assert!(decode_settings_request(&good[..cut]).is_err(), "cut {cut}");
    }
    let mut trailing = good.clone();
    trailing.push(0);
    assert!(decode_settings_request(&trailing).is_err());
    // A negative length, a huge length and bad UTF-8.
    for length in [-1i32, i32::MAX, i32::MIN] {
        let mut bad = vec![26, 0, 32, 0];
        bad.extend(length.to_le_bytes());
        bad.extend(b"WorldArea");
        assert!(decode_settings_request(&bad).is_err(), "{length}");
    }
    let mut utf8 = vec![26, 0, 32, 0, 2, 0, 0, 0, 0xff, 0xfe];
    assert!(decode_settings_request(&utf8).is_err());
    utf8[2] = 34;
    assert!(decode_settings_request(&utf8).is_err());
    // Another family or another subtype under 26.
    assert!(decode_settings_request(&[27, 0, 32, 0]).is_err());
    assert!(decode_settings_request(&channel_packet(34, "WorldArea")).is_err());

    let ok = (0x3366, 0xccffff, 2);
    assert!(decode_settings_request(&bubble_packet(ok.0, ok.1, ok.2, 0)).is_ok());
    // The wire guid is ignored, whatever it says.
    assert_eq!(
        decode_settings_request(&bubble_packet(ok.0, ok.1, ok.2, u64::MAX)),
        decode_settings_request(&bubble_packet(ok.0, ok.1, ok.2, 0))
    );
    for (fg, bg, size) in [
        (-1, 0, 1),
        (0, -1, 1),
        (0x0100_0000, 0, 1),
        (0, i32::MAX, 1),
        (0, 0, -1),
        (0, 0, 9),
        (i32::MIN, i32::MIN, i32::MIN),
    ] {
        assert_eq!(
            decode_settings_request(&bubble_packet(fg, bg, size, 0)),
            Err(ChatError::InvalidMessage),
            "{fg} {bg} {size}"
        );
    }
    let bubble = bubble_packet(1, 2, 3, 0);
    for cut in 0..bubble.len() {
        assert!(
            decode_settings_request(&bubble[..cut]).is_err(),
            "cut {cut}"
        );
    }
    let mut longer = bubble.clone();
    longer.push(0);
    assert!(decode_settings_request(&longer).is_err());
}

#[test]
fn refusals_change_nothing_and_settings_are_per_character() {
    let mut book = ChatSettingsBook::default();
    book.join(1, None);
    book.join(2, None);
    assert_eq!(book.handle(1, &channel_packet(33, "WorldTrade")), Ok(true));
    assert_eq!(book.handle(1, &bubble_packet(7, 8, 3, 0)), Ok(true));
    let after = book.clone();
    for bad in [
        channel_packet(33, "WorldSay"),
        channel_packet(33, "WorldTrade\0"),
        bubble_packet(-5, 0, 1, 0),
        bubble_packet(0, 0, 100, 0),
        vec![26, 0, 18, 0],
        vec![],
    ] {
        assert!(book.handle(1, &bad).is_err());
        assert!(book.handle(9, &bad).is_err());
        assert_eq!(book, after, "a refusal must change nothing");
    }
    // Repeating the same setting is not a change.
    assert_eq!(book.handle(1, &channel_packet(33, "WorldTrade")), Ok(false));
    assert!(book.get(1).unwrap().is_muted(ChatChannel::Trade));
    assert_eq!(book.get(1).unwrap().bubble().foreground, 7);
    // Character 2 is untouched.
    assert!(!book.get(2).unwrap().is_muted(ChatChannel::Trade));
    assert_eq!(book.get(2).unwrap().bubble(), BubbleStyle::default());
    // Switching back on.
    assert_eq!(book.handle(1, &channel_packet(32, "WorldTrade")), Ok(true));
    assert!(!book.get(1).unwrap().is_muted(ChatChannel::Trade));
}

#[test]
fn a_hand_built_request_cannot_mute_say_or_set_a_wild_bubble() {
    let mut settings = ChatSettings::default();
    for channel in [
        ChatChannel::Say,
        ChatChannel::Tell,
        ChatChannel::Group,
        ChatChannel::Guild,
    ] {
        assert!(!settings.apply(SettingsRequest::Channel { channel, on: false }));
        assert!(!settings.is_muted(channel));
    }
    let wild = BubbleStyle {
        foreground: -1,
        background: 0,
        size: 1,
    };
    assert!(!settings.apply(SettingsRequest::Bubble(wild)));
    assert_eq!(settings, ChatSettings::default());
}

fn session(id: u64, guid: u64) -> ChatSession {
    ChatSession {
        session_id: id,
        character_guid: guid,
        name: NameData::default(),
        zone_id: 1,
        position: [0.0; 3],
        chat_area_id: None,
        group_id: None,
        guild_id: None,
        is_member: true,
    }
}

#[test]
fn a_muted_channel_is_not_delivered_but_tells_and_group_chat_still_are() {
    let sessions = [session(10, 1), session(11, 2), session(12, 3)];
    let mut book = ChatSettingsBook::default();
    for guid in [1, 2, 3] {
        book.join(guid, None);
    }
    book.handle(2, &channel_packet(33, "WorldTrade")).unwrap();
    book.handle(2, &channel_packet(33, "WorldLfg")).unwrap();
    book.handle(3, &channel_packet(33, "WorldMembersOnly"))
        .unwrap();
    let everyone = vec![10, 11, 12];
    // Player 2 muted Trade and LFG; player 3 muted MembersOnly.
    assert_eq!(
        book.listeners(ChatChannel::Trade, 10, &sessions, everyone.clone()),
        vec![10, 12]
    );
    assert_eq!(
        book.listeners(
            ChatChannel::LookingForGroup,
            10,
            &sessions,
            everyone.clone()
        ),
        vec![10, 12]
    );
    assert_eq!(
        book.listeners(ChatChannel::MembersOnly, 10, &sessions, everyone.clone()),
        vec![10, 11]
    );
    assert_eq!(
        book.listeners(ChatChannel::Area, 10, &sessions, everyone.clone()),
        everyone
    );
    // Channels the client cannot switch are never filtered.
    for channel in [
        ChatChannel::Say,
        ChatChannel::Yell,
        ChatChannel::Tell,
        ChatChannel::Group,
        ChatChannel::Guild,
    ] {
        assert_eq!(
            book.listeners(channel, 10, &sessions, everyone.clone()),
            everyone,
            "{channel:?}"
        );
    }
    // The speaker keeps their own echo even with the channel off.
    assert_eq!(
        book.listeners(ChatChannel::Trade, 11, &sessions, everyone),
        vec![10, 11, 12]
    );
}

#[test]
fn settings_survive_a_relog_and_a_second_character_is_independent() {
    let dir = scratch("relog");
    let (path_a, path_b) = (dir.join("1.chat.json"), dir.join("2.chat.json"));
    let before = ChatSettingsBook::default();
    let mut book = before.clone();
    book.join(1, Some(&path_a));
    book.join(2, Some(&path_b));
    // A fresh join writes nothing: the defaults are not a change.
    book.save_changed(&before).unwrap();
    assert!(!path_a.exists());
    let staged_before = book.clone();
    book.handle(1, &channel_packet(33, "WorldArea")).unwrap();
    book.handle(1, &bubble_packet(0x123456, 0xabcdef, 4, 99))
        .unwrap();
    book.save_changed(&staged_before).unwrap();
    assert!(path_a.exists());
    assert!(!path_b.exists(), "only the changed character is written");

    // Relog: a brand-new book reads the file back.
    let mut relogged = ChatSettingsBook::default();
    relogged.join(1, Some(&path_a));
    relogged.join(2, Some(&path_b));
    let settings = relogged.get(1).unwrap();
    assert!(settings.is_muted(ChatChannel::Area));
    assert!(!settings.is_muted(ChatChannel::Trade));
    assert_eq!(
        settings.bubble(),
        BubbleStyle {
            foreground: 0x123456,
            background: 0xabcdef,
            size: 4
        }
    );
    assert_eq!(relogged.get(2), Some(&ChatSettings::default()));
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn a_bad_file_never_blocks_the_join_and_never_applies() {
    let dir = scratch("badfile");
    let path = dir.join("1.chat.json");
    let mut good = ChatSettings::default();
    good.apply(SettingsRequest::Channel {
        channel: ChatChannel::Trade,
        on: false,
    });
    good.save(&path, 1).unwrap();
    assert!(ChatSettings::load(&path, 1).is_ok());
    // Another character's file is refused.
    assert!(ChatSettings::load(&path, 2).is_err());
    let text = std::fs::read_to_string(&path).unwrap();
    for broken in [
        text.replace("\"schema_version\": 1", "\"schema_version\": 2"),
        text.replace("6", "0"), // would mute Say
        text.replace("\"muted\"", "\"extra\": 1, \"muted\""),
        "{".to_owned(),
        "x".repeat(10_000),
        String::new(),
    ] {
        std::fs::write(&path, &broken).unwrap();
        assert!(ChatSettings::load(&path, 1).is_err(), "{broken:.40}");
        let mut book = ChatSettingsBook::default();
        book.join(1, Some(&path));
        assert_eq!(book.get(1), Some(&ChatSettings::default()));
    }
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn the_action_budget_covers_the_three_requests() {
    let now = Instant::now();
    for packet in [
        channel_packet(32, "WorldArea"),
        channel_packet(33, "WorldArea"),
        bubble_packet(1, 2, 1, 0),
    ] {
        let mut budgets = ActionBudgets::default();
        let allowed = (0..40).filter(|_| budgets.allow(5, &packet, now)).count();
        assert_eq!(allowed, 10, "burst of 10, then silent drops");
        assert!(
            budgets.allow(6, &packet, now),
            "another account is separate"
        );
    }
}
