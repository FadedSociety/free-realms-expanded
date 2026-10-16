use osfr_tools::{social_chat, startup_world};

use social_chat::{
    AccountChatPermissions, ChatChannel, ChatHistory, ChatLine, ChatRequest, ChatSession,
    IgnoreLists, NameData, PacketChat, QuickChatCatalog, QuickChatRequest, ReportInbox, SpamGate,
    TextFilter, WhitelistFilter, decode_packet_chat, decode_quick_chat_request, encode_packet_chat,
    encode_quick_chat_channel, encode_quick_chat_tell, intake_report, parse_client_chat,
    resolve_quick_chat_recipients, resolve_recipients, server_chat_message,
};

fn fixture_name(first: &str, last: &str) -> NameData {
    NameData {
        fields: [0; 3],
        first: first.to_owned(),
        last: last.to_owned(),
    }
}

fn chat_packet(channel: ChatChannel) -> PacketChat {
    PacketChat {
        channel,
        sender_guid: 0x1122_3344_5566_7788,
        target_guid: 0x8877_6655_4433_2211,
        sender_name: NameData::default(),
        target_name: NameData::default(),
        message: "synthetic line".to_owned(),
        position: [1.0, 2.0, 3.0, 1.0],
        guild_guid: 0x1234_5678,
        language: 2,
        area_id: (channel == ChatChannel::Area).then_some(7),
    }
}

fn put_string(out: &mut Vec<u8>, value: &str) {
    out.extend_from_slice(&(value.len() as i32).to_le_bytes());
    out.extend_from_slice(value.as_bytes());
}

fn put_name(out: &mut Vec<u8>, value: &NameData) {
    for field in value.fields {
        out.extend_from_slice(&field.to_le_bytes());
    }
    put_string(out, &value.first);
    put_string(out, &value.last);
}

fn golden_quick_chat_base(sub: u16, phrase: i32, sender_guid: u64, name: &NameData) -> Vec<u8> {
    let mut out = vec![67, 0];
    out.extend_from_slice(&sub.to_le_bytes());
    out.extend_from_slice(&phrase.to_le_bytes());
    out.extend_from_slice(&sender_guid.to_le_bytes());
    put_name(&mut out, name);
    out
}

fn golden_chat_packet(packet: &PacketChat) -> Vec<u8> {
    let mut out = vec![15, 0, 1, 0];
    out.extend_from_slice(&packet.channel.wire().to_le_bytes());
    out.extend_from_slice(&packet.sender_guid.to_le_bytes());
    out.extend_from_slice(&packet.target_guid.to_le_bytes());
    put_name(&mut out, &packet.sender_name);
    put_name(&mut out, &packet.target_name);
    put_string(&mut out, &packet.message);
    for value in packet.position {
        out.extend_from_slice(&value.to_le_bytes());
    }
    out.extend_from_slice(&packet.guild_guid.to_le_bytes());
    out.extend_from_slice(&packet.language.to_le_bytes());
    if let Some(area_id) = packet.area_id {
        out.extend_from_slice(&area_id.to_le_bytes());
    }
    out
}

fn session(
    id: u64,
    guid: u64,
    name: (&str, &str),
    zone: u32,
    x: f32,
    group: Option<u64>,
    guild: Option<u64>,
) -> ChatSession {
    ChatSession {
        session_id: id,
        character_guid: guid,
        name: fixture_name(name.0, name.1),
        zone_id: zone,
        position: [x, 0.0, 0.0],
        chat_area_id: Some(7),
        group_id: group,
        guild_id: guild,
        is_member: true,
    }
}

fn request(channel: ChatChannel, target: Option<NameData>) -> ChatRequest {
    ChatRequest {
        channel,
        message: "synthetic line".to_owned(),
        tell_target: target,
        claimed_guild_guid: 0,
        area_id: Some(7),
        language: 2,
    }
}

#[test]
fn packet_chat_bytes_match_client_writer_and_reader_for_every_channel() {
    // Client writer 00b14fa0 and mirrored reader 00b11600; channel-specific area tail is 00b11600.
    for wire in 0..=10 {
        let channel = ChatChannel::from_wire(wire).unwrap();
        let packet = chat_packet(channel);
        let expected = golden_chat_packet(&packet);
        let encoded = encode_packet_chat(&packet).unwrap();
        assert_eq!(encoded, expected, "channel {wire}");
        assert_eq!(
            decode_packet_chat(&expected).unwrap(),
            packet,
            "channel {wire}"
        );
    }
}

#[test]
fn client_chat_parser_rejects_server_only_modes() {
    for channel in [ChatChannel::System, ChatChannel::Whisper] {
        let bytes = golden_chat_packet(&chat_packet(channel));
        assert!(parse_client_chat(&bytes).is_err());
    }
}

#[test]
fn spam_gate_enforces_any_dup3_max_and_similarity_threshold() {
    let mut gate = SpamGate::default();
    assert!(gate.allow(0, "abcdef"));
    assert!(!gate.allow(329, "different"));
    assert!(!gate.allow(330, "abcdeg"));
    assert!(!gate.allow(660, "abcdeh"));

    let mut repeats = SpamGate::default();
    assert!(repeats.allow(0, "synthetic phrase"));
    assert!(repeats.allow(1_000, "synthetic phrase"));
    assert!(repeats.allow(2_000, "synthetic phrase"));
    assert!(!repeats.allow(3_000, "synthetic phrase"));
    assert!(repeats.allow(4_000, "synthetic phrase"));
    assert!(!repeats.allow(5_000, "synthetic phrase"));
}

#[test]
fn account_flags_enforce_quick_chat_only_and_filter_is_pluggable() {
    let restricted = AccountChatPermissions::new(true, false);
    assert!(!restricted.allows_free_text());
    assert!(restricted.allows_quick_chat());
    assert_eq!(
        restricted.authorize_free_text("synthetic line"),
        Err(social_chat::ChatError::QuickChatOnly)
    );
    assert!(AccountChatPermissions::new(true, true).allows_free_text());
    assert!(AccountChatPermissions::new(false, false).allows_free_text());

    let filter = WhitelistFilter::new(["hello", "there"]);
    assert!(filter.allows("hello there!"));
    assert!(!filter.allows("hello 123"));
    assert!(!filter.allows("unlisted token"));
    assert_eq!(
        AccountChatPermissions::new(false, false).authorize_free_text("synthetic line"),
        Ok(())
    );
    assert_eq!(
        AccountChatPermissions::new(false, false)
            .authorize_free_text_with_filter("hello there!", &filter),
        Ok(())
    );
}

#[test]
fn quick_chat_ids_are_checked_against_startup_rows_and_packets_roundtrip() {
    let rows = [
        startup_world::QuickChatRow {
            key: 41,
            id: 4101,
            display_string_id: 0,
            say_string_id: 0,
            animation_id: 0,
            parent_id: 0,
            admin_only: 0,
            menu_icon_id: 0,
            item_id: 0,
            item_category: 0,
        },
        startup_world::QuickChatRow {
            key: 42,
            id: 4202,
            display_string_id: 0,
            say_string_id: 0,
            animation_id: 0,
            parent_id: 0,
            admin_only: 0,
            menu_icon_id: 0,
            item_id: 0,
            item_category: 0,
        },
    ];
    let catalog = QuickChatCatalog::from_ids(rows.iter().map(|row| row.id));
    let sender = fixture_name("fixture-sender", "");
    let channel =
        encode_quick_chat_channel(&catalog, 4101, ChatChannel::Group, 7, &sender, 0).unwrap();
    let mut expected_channel = golden_quick_chat_base(3, 4101, 7, &sender);
    expected_channel.extend_from_slice(&ChatChannel::Group.wire().to_le_bytes());
    expected_channel.extend_from_slice(&0u32.to_le_bytes());
    expected_channel.extend_from_slice(&0u64.to_le_bytes());
    assert_eq!(channel, expected_channel);
    assert_eq!(
        decode_quick_chat_request(&channel, &catalog).unwrap(),
        QuickChatRequest::Channel {
            phrase_id: 4101,
            channel: ChatChannel::Group,
            guild_guid: 0,
        }
    );
    let tell = encode_quick_chat_tell(&catalog, 4202, 7, &sender, "fixture-target").unwrap();
    let mut expected_tell = golden_quick_chat_base(2, 4202, 7, &sender);
    expected_tell.extend_from_slice(&0u64.to_le_bytes());
    put_string(&mut expected_tell, "fixture-target");
    assert_eq!(tell, expected_tell);
    assert_eq!(
        decode_quick_chat_request(&tell, &catalog).unwrap(),
        QuickChatRequest::Tell {
            phrase_id: 4202,
            target_name: "fixture-target".to_owned(),
        }
    );
    let mut unknown = channel;
    unknown[4..8].copy_from_slice(&999i32.to_le_bytes());
    assert!(decode_quick_chat_request(&unknown, &catalog).is_err());
    assert!(encode_quick_chat_channel(&catalog, 999, ChatChannel::Say, 7, &sender, 0).is_err());
}

#[test]
fn ignored_sender_is_removed_from_recipient_fanout() {
    let sender = session(1, 101, ("fixture-sender", ""), 1, 0.0, None, None);
    let target = session(2, 202, ("fixture-target", ""), 1, 1.0, None, None);
    let sessions = [sender, target];
    let request = request(ChatChannel::Say, None);
    let mut ignores = IgnoreLists::default();
    ignores.add(202, 101);
    assert_eq!(
        resolve_recipients(&request, 1, &sessions, &ignores).unwrap(),
        [1]
    );
}

#[test]
fn offline_tell_returns_the_clients_family_22_failure_packet() {
    let sessions = [session(1, 101, ("fixture-sender", ""), 1, 0.0, None, None)];
    let request = request(ChatChannel::Tell, Some(fixture_name("fixture-target", "")));
    let error = resolve_recipients(&request, 1, &sessions, &IgnoreLists::default()).unwrap_err();
    let expected = {
        let mut bytes = 22u16.to_le_bytes().to_vec();
        put_string(&mut bytes, "fixture-target");
        bytes
    };
    assert_eq!(error.client_reply().as_deref(), Some(expected.as_slice()));
}

#[test]
fn local_say_includes_exact_radius_and_excludes_outside_and_other_zone() {
    let sender = session(1, 101, ("fixture-sender", ""), 1, 0.0, None, None);
    let boundary = session(2, 202, ("fixture-boundary", ""), 1, 30.0, None, None);
    let outside = session(3, 303, ("fixture-outside", ""), 1, 30.001, None, None);
    let other_zone = session(4, 404, ("fixture-zone", ""), 2, 1.0, None, None);
    let sessions = [sender, boundary, outside, other_zone];
    assert_eq!(
        resolve_recipients(
            &request(ChatChannel::Say, None),
            1,
            &sessions,
            &IgnoreLists::default()
        )
        .unwrap(),
        [1, 2]
    );
}

#[test]
fn server_echo_rebuilds_sender_identity_from_the_session() {
    let sender = session(1, 101, ("fixture-sender", ""), 1, 5.0, None, None);
    let request = request(ChatChannel::Say, None);
    let packet = server_chat_message(&request, &sender, None).unwrap();
    assert_eq!(packet.sender_guid, 101);
    assert_eq!(packet.sender_name, sender.name);
    assert_eq!(packet.position, [5.0, 0.0, 0.0, 1.0]);
}

#[test]
fn quick_chat_fanout_uses_session_scope_and_ignore_lists() {
    let sender = session(1, 101, ("fixture-sender", ""), 1, 0.0, Some(7), None);
    let member = session(2, 202, ("fixture-member", ""), 1, 1.0, Some(7), None);
    let other = session(3, 303, ("fixture-other", ""), 1, 1.0, None, None);
    let sessions = [sender, member, other];
    let request = QuickChatRequest::Channel {
        phrase_id: 4101,
        channel: ChatChannel::Group,
        guild_guid: 0,
    };
    assert_eq!(
        resolve_quick_chat_recipients(&request, 1, &sessions, &IgnoreLists::default()).unwrap(),
        [1, 2]
    );
}

#[test]
fn report_intake_uses_server_history_and_keeps_only_last_ten_lines_in_memory() {
    let target = session(2, 202, ("fixture-target", ""), 1, 0.0, None, None);
    let mut history = ChatHistory::default();
    for i in 0..12 {
        history.record(&target, ChatLine::new([i, 2], format!("synthetic-{i}")));
    }
    let mut request = vec![68, 0, 1, 0];
    put_string(&mut request, "fixture-target");
    request.extend_from_slice(&1i32.to_le_bytes());
    request.extend_from_slice(&99i32.to_le_bytes());
    request.extend_from_slice(&99i32.to_le_bytes());
    put_string(&mut request, "forged synthetic line");
    let mut inbox = ReportInbox::default();
    let report = intake_report(&request, 101, &[], &history, &mut inbox).unwrap();
    assert_eq!(report.lines.len(), 10);
    assert_eq!(report.lines.first().unwrap().metadata, [2, 2]);
    assert_eq!(report.lines.last().unwrap().metadata, [11, 2]);
    assert_eq!(report.lines.first().unwrap().text, "synthetic-2");
    assert_eq!(report.lines.last().unwrap().text, "synthetic-11");
    assert_eq!(inbox.reports(), [report]);
}

#[test]
fn typed_client_request_uses_session_identity_and_decodes_target_name() {
    let mut packet = chat_packet(ChatChannel::Tell);
    packet.sender_name = fixture_name("untrusted", "sender");
    packet.target_name = fixture_name("fixture-target", "");
    let decoded = parse_client_chat(&encode_packet_chat(&packet).unwrap()).unwrap();
    assert_eq!(
        decoded.tell_target,
        Some(fixture_name("fixture-target", ""))
    );
    let sender = session(1, 101, ("fixture-authoritative", ""), 1, 5.0, None, None);
    let target = session(2, 202, ("fixture-target", ""), 1, 6.0, None, None);
    let rebuilt = server_chat_message(&decoded, &sender, Some(&target)).unwrap();
    assert_eq!(rebuilt.sender_guid, sender.character_guid);
    assert_eq!(rebuilt.sender_name, sender.name);
}

#[test]
fn catalog_from_rows_keeps_membership_and_plain_rows_stay_unrestricted() {
    let catalog = QuickChatCatalog::from_rows([(10, 0), (11, 3306)]);
    assert!(catalog.contains(10) && catalog.contains(11) && !catalog.contains(12));
    let mounted = social_chat::piece_093::EmoteContext {
        mounted: true,
        moving: false,
    };
    assert!(catalog.authorize_emote(10, mounted).is_ok());
    assert!(catalog.authorize_emote(11, mounted).is_err());
    assert_eq!(
        QuickChatCatalog::from_ids([10]).authorize_emote(10, mounted),
        Ok(())
    );
}
