//! Work item 088: original chat channel identity and recipient routing.
//!
//! ORIGINAL facts recovered for this module
//!
//! * Channel names, from the decompiled original client UI
//!   (`lab/decompiled/client-ui/ScriptsBase.lua`): `ChatBubbles.WORLDSAY_CHANNEL = "WorldSay"`,
//!   `GROUPSAY_CHANNEL = "GroupSay"`, `GUILDSAY_CHANNEL = "GuildSay"`,
//!   `QUICKCHAT_WORLDSAY_CHANNEL = "QuickChatWorldSay"`, `AREASAY_CHANNEL = "WorldArea"`,
//!   `WORLDTRADE_CHANNEL = "WorldTrade"`, `LFGSAY_CHANNEL = "WorldLfg"`,
//!   `MOSAY_CHANNEL = "WorldMembersOnly"`, `WORLDSHOUT_CHANNEL = "WorldShout"`; the chat log
//!   filters `ALL_CHANNEL`/`GROUP_CHANNEL`/`GUILD_CHANNEL`/`AREA_CHANNEL`/`TRADE_CHANNEL`/
//!   `LFG_CHANNEL`/`MO_CHANNEL`; and the four `Ui.ChannelOn`/`Ui.ChannelOff` toggles.
//! * Channel numbers, from the original client binary (`lab/decompiled/client/FreeRealms/`):
//!   `FUN_00b978b0` maps `WorldArea` -> 8, `WorldTrade` -> 6, `WorldLfg` -> 7 and
//!   `WorldMembersOnly` -> 10. Numbers 0/1/2/3/4/5/9 are NOT confirmed by the client; they come
//!   from the pinned community enum and are marked `client_number_confirmed = false`.
//! * Command surface, from the client binary's `ClientChatProcessor` table plus the string pool of
//!   `FreeRealms.exe`: `say` 0, `groupsay` 1, `tell`/`whisper` 2, `yell` 3, `lfg`/`3` 4,
//!   `area`/`1` 5, `tradechat`/`2` 6, `guildsay`/`gu` 7, `mo`/`4` 8. The tab labels the client
//!   maps through `CodeStringMappings.txt` are `1Area`, `2Trade`, `3LFG`, `4MembersOnly`.
//! * Presentation, from `OnReceiveSay` in the client UI: only Say, Yell and Group show a bubble;
//!   Area, Trade, LFG, Guild and Members-Only are chat-log only; Tell is the whisper path.
//! * `/tell "Name" ` and `/groupsay "msg"` compose strings, both in the client UI.
//!
//! RECONSTRUCTED facts (community-backed, not client-confirmed)
//!
//! * The full 0..10 wire numbering: `WorldSay` 0, `Tell` 1, `System` 2, `Whisper` 3, `GroupSay` 4,
//!   `WorldShout` 5, `WorldTrade` 6, `WorldLfg` 7, `WorldArea` 8, `GuildSay` 9,
//!   `WorldMembersOnly` 10. Four of these agree with the client binary; the rest are the pinned
//!   community `Sanctuary` enum. The supplied server independently agrees that only channel 8
//!   carries the trailing field (`ChatSystem.SendPacketChat`).
//! * The `15/1` field order and the `WorldArea`-only trailing area id: supplied server
//!   `Gateway.MapChat.ChatSystem` and pinned `Sanctuary.Packet.BaseChatPacket.PacketChat`.
//! * Every recipient rule in [`route`], the ignore handling and the ambiguity refusal. These are
//!   reconstruction policy built on the community handlers (informed by captures), not on a
//!   recovered server binary.
//!
//! BOUNDARIES: this module grants nothing and speaks no packet to anyone. It parses one client
//! request and decides who should receive it; the gateway owns framing, delivery and persistence.
//! Group membership is supplied by the caller because the party system (work item 089) is not
//! recovered here. No native client run backs this module.
//!
//! GAPS: the per-recipient membership gate the client applies to Members-Only
//! (`Ui.IsMember()` before `ChannelOn`), the server-side mute and spam limits, and the recovered
//! `ChatSpeed` suppression defaults (Any 330 ms, Dup 1000 ms, Dup3 2000 ms, Max 30000 ms - read
//! from the client config table, not implemented here).

use std::io;

/// Matches the production `crate::chat` limit: the client truncates long input before sending.
pub const MAX_MESSAGE_BYTES: usize = 2048;
/// Both name records are bounded; the client sends empty names for non-tell channels.
pub const MAX_NAME_BYTES: usize = 128;
/// `BaseChatPacket` opcode (15) and the `PacketChat` sub-opcode (1).
pub const CHAT_OPCODE: u16 = 15;
pub const PACKET_CHAT_SUBOPCODE: u16 = 1;
const REQUEST_PREFIX: [u8; 4] = [15, 0, 1, 0];
const NAME_HEADER_BYTES: usize = 12;

fn bad() -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, "invalid chat packet")
}

/// The recovered channel set. Discriminants are the reconstructed wire numbers.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ChatChannel {
    WorldSay,
    Tell,
    System,
    Whisper,
    GroupSay,
    WorldShout,
    WorldTrade,
    WorldLfg,
    WorldArea,
    GuildSay,
    WorldMembersOnly,
}

/// Every channel in wire-number order.
pub const CHANNELS: [ChatChannel; 11] = [
    ChatChannel::WorldSay,
    ChatChannel::Tell,
    ChatChannel::System,
    ChatChannel::Whisper,
    ChatChannel::GroupSay,
    ChatChannel::WorldShout,
    ChatChannel::WorldTrade,
    ChatChannel::WorldLfg,
    ChatChannel::WorldArea,
    ChatChannel::GuildSay,
    ChatChannel::WorldMembersOnly,
];

/// What is recovered about one channel.
#[derive(Clone, Copy, Debug)]
pub struct ChannelFacts {
    pub channel: ChatChannel,
    /// The name the original client uses. `None` when no client-rendered name was recovered.
    pub client_name: Option<&'static str>,
    /// True when the client binary itself pins this wire number (`FUN_00b978b0`).
    pub client_number_confirmed: bool,
    /// The client's chat command without its dot, when the client registers one.
    pub client_command: Option<&'static str>,
    /// Every name the client registers for this channel, including the numeric shortcuts.
    pub client_aliases: &'static [&'static str],
    /// True when a chat request from the client can carry this channel.
    pub client_requestable: bool,
    /// True when the client renders the message as a chat bubble instead of log-only.
    pub shows_bubble: bool,
    /// True when the channel has a `ChannelOn`/`ChannelOff` toggle on the client.
    pub has_client_toggle: bool,
}

/// The recovered channel table. See the module header for per-entry provenance.
pub const CHANNEL_FACTS: [ChannelFacts; 11] = [
    ChannelFacts {
        channel: ChatChannel::WorldSay,
        client_name: Some("WorldSay"),
        client_number_confirmed: false,
        client_command: Some("say"),
        client_aliases: &["say"],
        client_requestable: true,
        shows_bubble: true,
        has_client_toggle: false,
    },
    ChannelFacts {
        channel: ChatChannel::Tell,
        client_name: Some("Tell"),
        client_number_confirmed: false,
        client_command: Some("tell"),
        client_aliases: &["tell", "whisper"],
        client_requestable: true,
        shows_bubble: false,
        has_client_toggle: false,
    },
    ChannelFacts {
        channel: ChatChannel::System,
        client_name: Some("System"),
        client_number_confirmed: false,
        client_command: None,
        client_aliases: &[],
        client_requestable: false,
        shows_bubble: false,
        has_client_toggle: false,
    },
    ChannelFacts {
        channel: ChatChannel::Whisper,
        client_name: None,
        client_number_confirmed: false,
        client_command: None,
        client_aliases: &[],
        client_requestable: false,
        shows_bubble: false,
        has_client_toggle: false,
    },
    ChannelFacts {
        channel: ChatChannel::GroupSay,
        client_name: Some("GroupSay"),
        client_number_confirmed: false,
        client_command: Some("groupsay"),
        client_aliases: &["groupsay"],
        client_requestable: true,
        shows_bubble: true,
        has_client_toggle: false,
    },
    ChannelFacts {
        channel: ChatChannel::WorldShout,
        client_name: Some("WorldShout"),
        client_number_confirmed: false,
        client_command: Some("yell"),
        client_aliases: &["yell"],
        client_requestable: true,
        shows_bubble: true,
        has_client_toggle: false,
    },
    ChannelFacts {
        channel: ChatChannel::WorldTrade,
        client_name: Some("WorldTrade"),
        client_number_confirmed: true,
        client_command: Some("tradechat"),
        client_aliases: &["tradechat", "2"],
        client_requestable: true,
        shows_bubble: false,
        has_client_toggle: true,
    },
    ChannelFacts {
        channel: ChatChannel::WorldLfg,
        client_name: Some("WorldLfg"),
        client_number_confirmed: true,
        client_command: Some("lfg"),
        client_aliases: &["lfg", "3"],
        client_requestable: true,
        shows_bubble: false,
        has_client_toggle: true,
    },
    ChannelFacts {
        channel: ChatChannel::WorldArea,
        client_name: Some("WorldArea"),
        client_number_confirmed: true,
        client_command: Some("area"),
        client_aliases: &["area", "1"],
        client_requestable: true,
        shows_bubble: false,
        has_client_toggle: true,
    },
    ChannelFacts {
        channel: ChatChannel::GuildSay,
        client_name: Some("GuildSay"),
        client_number_confirmed: false,
        client_command: Some("guildsay"),
        client_aliases: &["guildsay", "gu"],
        client_requestable: true,
        shows_bubble: false,
        has_client_toggle: false,
    },
    ChannelFacts {
        channel: ChatChannel::WorldMembersOnly,
        client_name: Some("WorldMembersOnly"),
        client_number_confirmed: true,
        client_command: Some("mo"),
        client_aliases: &["mo", "4"],
        client_requestable: true,
        shows_bubble: false,
        has_client_toggle: true,
    },
];

impl ChatChannel {
    /// The reconstructed wire number used by the `15/1` request and the client's reply.
    pub const fn wire(self) -> u16 {
        match self {
            ChatChannel::WorldSay => 0,
            ChatChannel::Tell => 1,
            ChatChannel::System => 2,
            ChatChannel::Whisper => 3,
            ChatChannel::GroupSay => 4,
            ChatChannel::WorldShout => 5,
            ChatChannel::WorldTrade => 6,
            ChatChannel::WorldLfg => 7,
            ChatChannel::WorldArea => 8,
            ChatChannel::GuildSay => 9,
            ChatChannel::WorldMembersOnly => 10,
        }
    }

    /// `None` for a number the recovered client never sends. Channel 8 is the only one that
    /// carries the trailing area field, which is how the supplied server reads it too.
    pub fn from_wire(value: u16) -> Option<Self> {
        CHANNELS.into_iter().find(|channel| channel.wire() == value)
    }

    pub fn client_name(self) -> Option<&'static str> {
        facts(self).client_name
    }

    pub fn client_requestable(self) -> bool {
        facts(self).client_requestable
    }

    pub fn shows_bubble(self) -> bool {
        facts(self).shows_bubble
    }

    pub fn has_client_toggle(self) -> bool {
        facts(self).has_client_toggle
    }
}

pub fn facts(channel: ChatChannel) -> &'static ChannelFacts {
    CHANNEL_FACTS
        .iter()
        .find(|facts| facts.channel == channel)
        .expect("every channel has facts")
}

/// One parsed client chat request. Client-supplied GUIDs are carried for logging only: the
/// selected session owns sender identity, exactly like `crate::chat::request`.
#[derive(Clone, Debug, PartialEq)]
pub struct ParsedChat {
    pub channel: ChatChannel,
    pub from_guid: u64,
    pub to_guid: u64,
    pub from_first: String,
    pub from_last: String,
    pub to_first: String,
    pub to_last: String,
    pub message: String,
    pub guild_guid: u64,
    pub language_id: i32,
    /// Only present on [`ChatChannel::WorldArea`].
    pub area_name_id: Option<i32>,
}

impl ParsedChat {
    /// `First Last`, falling back to the first name when the client sent no last name.
    pub fn from_full_name(&self) -> String {
        full_name(&self.from_first, &self.from_last)
    }

    /// The `/tell` target the client composed, in the same form.
    pub fn target_full_name(&self) -> String {
        full_name(&self.to_first, &self.to_last)
    }
}

fn full_name(first: &str, last: &str) -> String {
    if last.trim().is_empty() {
        first.to_string()
    } else {
        format!("{first} {last}")
    }
}

struct Reader<'a> {
    bytes: &'a [u8],
    at: usize,
}

impl<'a> Reader<'a> {
    fn take(&mut self, count: usize) -> io::Result<&'a [u8]> {
        let end = self.at.checked_add(count).ok_or_else(bad)?;
        let part = self.bytes.get(self.at..end).ok_or_else(bad)?;
        self.at = end;
        Ok(part)
    }

    fn string(&mut self, limit: usize) -> io::Result<&'a str> {
        let count = u32::from_le_bytes(self.take(4)?.try_into().unwrap()) as usize;
        if count > limit {
            return Err(bad());
        }
        std::str::from_utf8(self.take(count)?).map_err(|_| bad())
    }

    fn name(&mut self) -> io::Result<(String, String)> {
        self.take(NAME_HEADER_BYTES)?;
        let first = self.string(MAX_NAME_BYTES)?.to_owned();
        let last = self.string(MAX_NAME_BYTES)?.to_owned();
        Ok((first, last))
    }
}

/// Parse one application payload in the recovered `15/1` `PacketChat` shape.
///
/// `Ok(None)` means the payload is not a `PacketChat` request at all, so the caller can keep
/// dispatching other applications. `Err` means it claims to be one and is malformed: a truncated
/// record, an unknown channel number, a name or message beyond its bound, or trailing bytes.
pub fn parse_request(inner: &[u8]) -> io::Result<Option<ParsedChat>> {
    if !inner.starts_with(&REQUEST_PREFIX) {
        return Ok(None);
    }
    let mut reader = Reader {
        bytes: inner,
        at: REQUEST_PREFIX.len(),
    };
    let channel_wire = u16::from_le_bytes(reader.take(2)?.try_into().unwrap());
    let channel = ChatChannel::from_wire(channel_wire).ok_or_else(bad)?;
    let from_guid = u64::from_le_bytes(reader.take(8)?.try_into().unwrap());
    let to_guid = u64::from_le_bytes(reader.take(8)?.try_into().unwrap());
    let (from_first, from_last) = reader.name()?;
    let (to_first, to_last) = reader.name()?;
    let message = reader.string(MAX_MESSAGE_BYTES)?.to_owned();
    reader.take(16)?; // position: four floats the client sends and the server ignores
    let guild_guid = u64::from_le_bytes(reader.take(8)?.try_into().unwrap());
    let language_id = i32::from_le_bytes(reader.take(4)?.try_into().unwrap());
    let area_name_id = if channel == ChatChannel::WorldArea {
        Some(i32::from_le_bytes(reader.take(4)?.try_into().unwrap()))
    } else {
        None
    };
    if reader.at != inner.len() {
        return Err(bad());
    }
    Ok(Some(ParsedChat {
        channel,
        from_guid,
        to_guid,
        from_first,
        from_last,
        to_first,
        to_last,
        message,
        guild_guid,
        language_id,
        area_name_id,
    }))
}

/// A candidate recipient. The caller fills in the relations it actually knows; every relation
/// defaults to "no" in the caller's own vocabulary.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Peer {
    pub guid: u64,
    pub name: String,
    /// In the sender's visible/interest set (the client draws these characters).
    pub visible: bool,
    /// In the sender's zone.
    pub same_zone: bool,
    /// In the sender's group/party. Supplied by the party subsystem (work item 089).
    pub group: bool,
    /// In the sender's guild.
    pub guild: bool,
    /// This player ignores the sender.
    pub ignores_sender: bool,
    /// This player has the world channels (Area/Trade/LFG/Members-Only) switched on. The client
    /// sends `CommandPacketChatChannelOn/Off` per channel; one flag covers the bounded case.
    pub all_world_channels_enabled: bool,
}

/// The delivery plan for one accepted request. `recipients` never contains the sender.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Delivery {
    pub channel: ChatChannel,
    pub recipients: Vec<u64>,
    /// The sender's own client also renders the message (`WorldSay`, the world channels, guild).
    pub echo_to_sender: bool,
    /// The sender gets the client's separate `/tell` confirmation instead.
    pub tell_echo: bool,
    /// Peers that matched but ignore the sender. `recipients` excludes them.
    pub ignored_by: Vec<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Refusal {
    /// A channel the client never composes (System, Whisper) - only the server originates these.
    NotClientChannel(ChatChannel),
    /// No candidate carries the `/tell` target name.
    UnknownTarget(String),
    /// More than one candidate carries the `/tell` target name, so targeting would be a guess.
    AmbiguousTarget(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Route {
    Deliver(Delivery),
    Refused(Refusal),
}

/// Decide who receives one parsed request.
///
/// Reconstructed policy, in the same shape as the pinned community handlers:
/// `WorldSay` reaches the sender's visible set; `WorldShout` the zone; `GroupSay` the group;
/// `GuildSay` the guild; `WorldTrade`/`WorldLfg`/`WorldArea`/`WorldMembersOnly` the visible
/// players that left the channel on; `Tell` exactly one named player. Ignoring players are
/// reported instead of receiving the message.
pub fn route(request: &ParsedChat, sender_guid: u64, peers: &[Peer]) -> Route {
    if !request.channel.client_requestable() {
        return Route::Refused(Refusal::NotClientChannel(request.channel));
    }
    if request.channel == ChatChannel::Tell {
        return route_tell(request, sender_guid, peers);
    }

    let mut recipients: Vec<u64> = Vec::new();
    let mut ignored_by: Vec<u64> = Vec::new();
    for peer in peers {
        if peer.guid == sender_guid {
            continue;
        }
        let reaches = match request.channel {
            ChatChannel::WorldSay => peer.visible,
            ChatChannel::WorldShout => peer.same_zone,
            ChatChannel::GroupSay => peer.group,
            ChatChannel::GuildSay => peer.guild,
            ChatChannel::WorldTrade
            | ChatChannel::WorldLfg
            | ChatChannel::WorldArea
            | ChatChannel::WorldMembersOnly => peer.visible && peer.all_world_channels_enabled,
            ChatChannel::Tell | ChatChannel::System | ChatChannel::Whisper => false,
        };
        if !reaches {
            continue;
        }
        if peer.ignores_sender {
            if !ignored_by.contains(&peer.guid) {
                ignored_by.push(peer.guid);
            }
            continue;
        }
        if !recipients.contains(&peer.guid) {
            recipients.push(peer.guid);
        }
    }
    Route::Deliver(Delivery {
        channel: request.channel,
        recipients,
        echo_to_sender: true,
        tell_echo: false,
        ignored_by,
    })
}

fn route_tell(request: &ParsedChat, sender_guid: u64, peers: &[Peer]) -> Route {
    let target = request.target_full_name();
    let mut matched: Vec<&Peer> = Vec::new();
    for peer in peers {
        if peer.guid == sender_guid || peer.name.trim().is_empty() {
            continue;
        }
        if peer.name.eq_ignore_ascii_case(&target) {
            matched.push(peer);
        }
    }
    let mut guids: Vec<u64> = matched.iter().map(|peer| peer.guid).collect();
    guids.dedup();
    match guids.len() {
        0 => Route::Refused(Refusal::UnknownTarget(target)),
        1 => {
            let peer = matched[0];
            let mut recipients = Vec::new();
            let mut ignored_by = Vec::new();
            if peer.ignores_sender {
                ignored_by.push(peer.guid);
            } else {
                recipients.push(peer.guid);
            }
            Route::Deliver(Delivery {
                channel: ChatChannel::Tell,
                recipients,
                echo_to_sender: false,
                // The client echoes `/tell` only after a real delivery.
                tell_echo: ignored_by.is_empty(),
                ignored_by,
            })
        }
        _ => Route::Refused(Refusal::AmbiguousTarget(target)),
    }
}
