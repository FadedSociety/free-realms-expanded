//! Work item 088 acceptance: original chat channel identity and recipient routing.
//!
//! ORIGINAL evidence used by these vectors (all outside this test):
//! * `lab/decompiled/client-ui/ScriptsBase.lua` - the decompiled original client UI. It names
//!   the channels (`ChatBubbles.WORLDSAY_CHANNEL = "WorldSay"` and the seven siblings), the chat
//!   log filters (`ChatLog.ALL_CHANNEL/GROUP_CHANNEL/GUILD_CHANNEL/AREA_CHANNEL/TRADE_CHANNEL/
//!   LFG_CHANNEL/MO_CHANNEL`), the bubble suppression list in `OnReceiveSay`, the four channel
//!   toggles driven by `Ui.ChannelOn`/`Ui.ChannelOff`, the `/tell "` compose strings and the
//!   minigame `/groupsay "` compose.
//! * `lab/decompiled/client/FreeRealms/pseudocode.c` - the original client binary. `FUN_00b978b0`
//!   maps `WorldArea` -> 8, `WorldTrade` -> 6, `WorldLfg` -> 7 and `WorldMembersOnly` -> 10 when
//!   the chat window toggles a channel. The chat command table in `ClientChatProcessor` registers
//!   `say`/`groupsay`/`tell`+`whisper`/`yell`/`lfg`+`3`/`area`+`1`/`tradechat`+`2`/`guildsay`/
//!   `mo`+`4`.
//! * `lab/working/OSFR.Client/OSFR Client/FreeRealms.exe` - the string pool behind that table
//!   (`WorldSay`..`GuildSay`, the tab labels, `mo`, and the `1`/`2`/`3`/`4` aliases).
//! * `lab/decompiled/server/Gateway/Gateway.MapChat/ChatSystem.cs` (the supplied server) and the
//!   pinned community `Sanctuary` sources - the `15/1` packet field order, the `WorldArea`-only
//!   trailing field and the per-channel recipient policy used as the reconstruction baseline.
//!
//! BOUNDARIES: the client proves channel names, four numbers and the command surface. The full
//! 0..10 numbering, every recipient rule below, and the ambiguity refusal are reconstructed; they
//! are labelled as such in the module. Nothing here is native acceptance.
//!
//! This file is a `#[path]` include so it compiles and exercises the real module before the lead
//! registers `pub mod piece_088;` under `src/restoration/mod.rs`/`src/lib.rs`.
#[path = "../src/restoration/piece_088.rs"]
#[allow(dead_code)]
mod piece_088;

use osfr_tools::chat;
use piece_088::{
    CHANNEL_FACTS, CHANNELS, ChatChannel, MAX_MESSAGE_BYTES, MAX_NAME_BYTES, ParsedChat, Peer,
    Refusal, Route, parse_request, route,
};

fn string(value: &str) -> Vec<u8> {
    let mut out = (value.len() as u32).to_le_bytes().to_vec();
    out.extend_from_slice(value.as_bytes());
    out
}

fn name_data(first: &str, last: &str) -> Vec<u8> {
    let mut out = vec![0u8; 12];
    out.extend(string(first));
    out.extend(string(last));
    out
}

/// Synthetic `PacketChat` request in the recovered `15/1` shape. No captured bytes are embedded.
fn request(channel: u16, from: (&str, &str), to: (&str, &str), message: &str) -> Vec<u8> {
    let mut packet = vec![15, 0, 1, 0];
    packet.extend(channel.to_le_bytes());
    packet.extend(0x0000_0000_dead_beefu64.to_le_bytes());
    packet.extend(0x0000_0000_1234_5678u64.to_le_bytes());
    packet.extend(name_data(from.0, from.1));
    packet.extend(name_data(to.0, to.1));
    packet.extend(string(message));
    for _ in 0..4 {
        packet.extend(1.0f32.to_le_bytes());
    }
    packet.extend(0u64.to_le_bytes());
    packet.extend(2i32.to_le_bytes());
    if channel == 8 {
        packet.extend(7i32.to_le_bytes());
    }
    packet
}

fn peer(guid: u64, name: &str) -> Peer {
    Peer {
        guid,
        name: name.to_string(),
        visible: true,
        same_zone: true,
        group: false,
        guild: false,
        ignores_sender: false,
        all_world_channels_enabled: true,
    }
}

fn parsed(channel: ChatChannel, message: &str) -> ParsedChat {
    let packet = request(channel.wire(), ("Mira", "Vale"), ("Bram", "Stone"), message);
    parse_request(&packet).unwrap().unwrap()
}

fn delivered(route: Route) -> piece_088::Delivery {
    match route {
        Route::Deliver(delivery) => delivery,
        other => panic!("expected delivery, got {other:?}"),
    }
}

#[test]
fn channel_table_matches_the_recovered_client_surface() {
    assert_eq!(CHANNELS.len(), 11);
    assert_eq!(CHANNEL_FACTS.len(), 11);
    assert_eq!(ChatChannel::WorldSay.wire(), 0);
    assert_eq!(ChatChannel::Tell.wire(), 1);
    assert_eq!(ChatChannel::System.wire(), 2);
    assert_eq!(ChatChannel::Whisper.wire(), 3);
    assert_eq!(ChatChannel::GroupSay.wire(), 4);
    assert_eq!(ChatChannel::WorldShout.wire(), 5);
    assert_eq!(ChatChannel::WorldTrade.wire(), 6);
    assert_eq!(ChatChannel::WorldLfg.wire(), 7);
    assert_eq!(ChatChannel::WorldArea.wire(), 8);
    assert_eq!(ChatChannel::GuildSay.wire(), 9);
    assert_eq!(ChatChannel::WorldMembersOnly.wire(), 10);
    for channel in CHANNELS {
        assert_eq!(ChatChannel::from_wire(channel.wire()), Some(channel));
    }
    assert_eq!(ChatChannel::from_wire(11), None);

    // Names, commands and aliases come from the client string pool and the command table.
    let expected: [(ChatChannel, &str, &str, &[&str]); 9] = [
        (ChatChannel::WorldSay, "WorldSay", "say", &["say"]),
        (ChatChannel::GroupSay, "GroupSay", "groupsay", &["groupsay"]),
        (ChatChannel::Tell, "Tell", "tell", &["tell", "whisper"]),
        (ChatChannel::WorldShout, "WorldShout", "yell", &["yell"]),
        (ChatChannel::WorldLfg, "WorldLfg", "lfg", &["lfg", "3"]),
        (ChatChannel::WorldArea, "WorldArea", "area", &["area", "1"]),
        (
            ChatChannel::WorldTrade,
            "WorldTrade",
            "tradechat",
            &["tradechat", "2"],
        ),
        (
            ChatChannel::GuildSay,
            "GuildSay",
            "guildsay",
            &["guildsay", "gu"],
        ),
        (
            ChatChannel::WorldMembersOnly,
            "WorldMembersOnly",
            "mo",
            &["mo", "4"],
        ),
    ];
    for (channel, name, command, aliases) in expected {
        let facts = piece_088::facts(channel);
        assert_eq!(facts.client_name, Some(name), "{channel:?}");
        assert_eq!(facts.client_command, Some(command), "{channel:?}");
        assert_eq!(facts.client_aliases, aliases, "{channel:?}");
        assert!(facts.client_requestable, "{channel:?}");
    }
    assert!(!ChatChannel::System.client_requestable());
    assert!(!ChatChannel::Whisper.client_requestable());

    // `FUN_00b978b0` confirms these four numbers in the original client binary.
    assert!(piece_088::facts(ChatChannel::WorldTrade).client_number_confirmed);
    assert!(piece_088::facts(ChatChannel::WorldLfg).client_number_confirmed);
    assert!(piece_088::facts(ChatChannel::WorldArea).client_number_confirmed);
    assert!(piece_088::facts(ChatChannel::WorldMembersOnly).client_number_confirmed);
    assert!(!piece_088::facts(ChatChannel::GroupSay).client_number_confirmed);

    // `OnReceiveSay` only bubbles Say/Yell/Group; the world channels are log-only.
    assert!(ChatChannel::WorldSay.shows_bubble());
    assert!(ChatChannel::WorldShout.shows_bubble());
    assert!(ChatChannel::GroupSay.shows_bubble());
    for channel in [
        ChatChannel::Tell,
        ChatChannel::System,
        ChatChannel::WorldTrade,
        ChatChannel::WorldLfg,
        ChatChannel::WorldArea,
        ChatChannel::GuildSay,
        ChatChannel::WorldMembersOnly,
    ] {
        assert!(!channel.shows_bubble(), "{channel:?}");
    }

    // Only the four world channels carry the client's ChannelOn/ChannelOff toggle.
    for channel in CHANNELS {
        let toggled = matches!(
            channel,
            ChatChannel::WorldTrade
                | ChatChannel::WorldLfg
                | ChatChannel::WorldArea
                | ChatChannel::WorldMembersOnly
        );
        assert_eq!(channel.has_client_toggle(), toggled, "{channel:?}");
    }
}

#[test]
fn parses_the_original_packet_chat_shape() {
    let say = parse_request(&request(0, ("Mira", "Vale"), ("Bram", "Stone"), "hello"))
        .unwrap()
        .unwrap();
    assert_eq!(say.channel, ChatChannel::WorldSay);
    assert_eq!(say.message, "hello");
    assert_eq!(say.from_full_name(), "Mira Vale");
    assert_eq!(say.target_full_name(), "Bram Stone");
    assert_eq!(say.from_guid, 0xdead_beef);
    assert_eq!(say.to_guid, 0x1234_5678);
    assert_eq!(say.language_id, 2);
    assert_eq!(say.area_name_id, None);

    // WorldArea is the only channel with the trailing area field.
    let area = parse_request(&request(8, ("Mira", "Vale"), ("", ""), "in the area"))
        .unwrap()
        .unwrap();
    assert_eq!(area.channel, ChatChannel::WorldArea);
    assert_eq!(area.area_name_id, Some(7));
    assert_eq!(area.target_full_name(), "");

    // Somebody else's application packet is not a chat request.
    assert_eq!(parse_request(&[9, 0, 5, 6]).unwrap(), None);
    assert_eq!(parse_request(&[]).unwrap(), None);
}

#[test]
fn parse_agrees_with_the_production_chat_parser() {
    for channel in CHANNELS {
        let packet = request(
            channel.wire(),
            ("Mira", "Vale"),
            ("Bram", "Stone"),
            "realm help",
        );
        let mine = parse_request(&packet)
            .unwrap()
            .map(|parsed| parsed.message.to_owned());
        let existing = chat::request(&packet, 1).unwrap();
        assert_eq!(mine, existing, "{channel:?}");
    }
    for broken in [
        vec![15, 0, 1, 0],
        request(11, ("Mira", "Vale"), ("Bram", "Stone"), "x"),
    ] {
        assert!(parse_request(&broken).is_err());
        assert!(chat::request(&broken, 1).is_err());
    }
}

#[test]
fn bounded_parser_rejects_truncation_and_trailing_bytes() {
    let packet = request(0, ("Mira", "Vale"), ("Bram", "Stone"), "hello");
    for length in 4..packet.len() {
        assert!(parse_request(&packet[..length]).is_err(), "length {length}");
    }
    let mut extra = packet.clone();
    extra.push(0);
    assert!(parse_request(&extra).is_err());

    let long_message = "m".repeat(MAX_MESSAGE_BYTES + 1);
    assert!(
        parse_request(&request(
            0,
            ("Mira", "Vale"),
            ("Bram", "Stone"),
            &long_message
        ))
        .is_err()
    );

    let long_name = "n".repeat(MAX_NAME_BYTES + 1);
    assert!(parse_request(&request(0, (&long_name, "Vale"), ("Bram", "Stone"), "x")).is_err());
}

#[test]
fn local_say_goes_to_visible_players_and_echoes() {
    let request = parsed(ChatChannel::WorldSay, "hello");
    let mut far = peer(2, "Far Away");
    far.visible = false;
    let nearby = peer(3, "Near By");

    let delivery = delivered(route(&request, 1, &[far, nearby]));
    assert_eq!(delivery.recipients, vec![3]);
    assert!(delivery.echo_to_sender);
    assert!(!delivery.tell_echo);
    assert!(delivery.ignored_by.is_empty());
}

#[test]
fn yell_reaches_the_zone_and_group_chat_reaches_the_group() {
    let yell = parsed(ChatChannel::WorldShout, "yell");
    let mut other_zone = peer(2, "Other Zone");
    other_zone.same_zone = false;
    let same_zone = peer(3, "Same Zone");
    let delivery = delivered(route(&yell, 1, &[other_zone, same_zone]));
    assert_eq!(delivery.recipients, vec![3]);
    assert!(delivery.echo_to_sender);

    let group_say = parsed(ChatChannel::GroupSay, "group");
    let mut outsider = peer(4, "Outsider");
    outsider.group = false;
    let mut member = peer(5, "Member");
    member.group = true;
    member.visible = false;
    let delivery = delivered(route(&group_say, 1, &[outsider, member]));
    assert_eq!(delivery.recipients, vec![5]);
    assert!(delivery.echo_to_sender);
}

#[test]
fn guild_chat_reaches_guild_members() {
    let guild = parsed(ChatChannel::GuildSay, "guild");
    let mut member = peer(7, "Guild Mate");
    member.guild = true;
    member.visible = false;
    member.same_zone = false;
    let mut stranger = peer(8, "Stranger");
    stranger.guild = false;

    let delivery = delivered(route(&guild, 1, &[member, stranger]));
    assert_eq!(delivery.recipients, vec![7]);
    // The community guild handler delivers to the author as well; the client shows no bubble.
    assert!(delivery.echo_to_sender);
}

#[test]
fn world_channels_respect_the_client_channel_toggle_and_ignores() {
    for channel in [
        ChatChannel::WorldTrade,
        ChatChannel::WorldLfg,
        ChatChannel::WorldArea,
        ChatChannel::WorldMembersOnly,
    ] {
        let request = parsed(channel, "world");
        let listener = peer(9, "Listener");
        let mut off = peer(10, "Toggled Off");
        off.all_world_channels_enabled = false;
        let mut ignoring = peer(11, "Ignoring");
        ignoring.ignores_sender = true;

        let delivery = delivered(route(&request, 1, &[listener, off, ignoring]));
        assert_eq!(delivery.recipients, vec![9], "{channel:?}");
        assert_eq!(delivery.ignored_by, vec![11], "{channel:?}");
        assert!(delivery.echo_to_sender, "{channel:?}");
    }
}

#[test]
fn tell_goes_to_exactly_one_named_player() {
    let request = parsed(ChatChannel::Tell, "psst");
    // `parsed` composes `/tell` to Bram Stone.
    let mut other = peer(12, "Mira Vale");
    other.visible = false;
    let target = peer(13, "bram stone");
    let delivery = delivered(route(&request, 1, &[other, target]));
    assert_eq!(delivery.recipients, vec![13]);
    assert!(!delivery.echo_to_sender);
    assert!(delivery.tell_echo);
}

#[test]
fn tell_refuses_unknown_and_ambiguous_targets_and_reports_ignoring_targets() {
    let request = parsed(ChatChannel::Tell, "psst");
    match route(&request, 1, &[peer(2, "Someone Else")]) {
        Route::Refused(Refusal::UnknownTarget(name)) => assert_eq!(name, "Bram Stone"),
        other => panic!("expected unknown target, got {other:?}"),
    }

    let stale = peer(3, "Bram Stone");
    let live = peer(4, "Bram Stone");
    match route(&request, 1, &[stale, live]) {
        Route::Refused(Refusal::AmbiguousTarget(name)) => assert_eq!(name, "Bram Stone"),
        other => panic!("expected ambiguous target, got {other:?}"),
    }

    let mut ignoring = peer(5, "Bram Stone");
    ignoring.ignores_sender = true;
    let delivery = delivered(route(&request, 1, &[ignoring]));
    assert!(delivery.recipients.is_empty());
    assert_eq!(delivery.ignored_by, vec![5]);
    assert!(!delivery.tell_echo);
}

#[test]
fn server_channels_are_not_client_requestable() {
    for channel in [ChatChannel::System, ChatChannel::Whisper] {
        let request = parsed(channel, "not from a client");
        match route(&request, 1, &[peer(2, "Someone")]) {
            Route::Refused(Refusal::NotClientChannel(refused)) => assert_eq!(refused, channel),
            other => panic!("expected refusal for {channel:?}, got {other:?}"),
        }
    }
}

#[test]
fn recipients_are_deduplicated_and_never_include_the_sender() {
    let request = parsed(ChatChannel::WorldSay, "hello");
    let sender = peer(1, "Mira Vale");
    let first = peer(2, "Near By");
    let duplicate = peer(2, "Near By");
    let delivery = delivered(route(&request, 1, &[sender, first, duplicate]));
    assert_eq!(delivery.recipients, vec![2]);
}
