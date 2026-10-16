//! Chat packet handling and server-side chat policy.
//!
//! Wire facts come from the native client readers and writers cited in
//! `notes/Research/Chat Core.md`. Recipient scopes, the local radius, account
//! enforcement, text filtering, and ignore behavior are reconstructed policy (R).

use std::collections::{HashMap, HashSet, VecDeque};
use std::time::Instant;

use crate::net_policy::Bucket;

// Piece 093 is not registered in lib.rs (shared); include the one file here.
#[path = "restoration/piece_093.rs"]
pub mod piece_093;
use piece_093::{EmoteContext, EmoteRejection};

pub const MAX_MESSAGE_BYTES: usize = 2048; // (R) Bounded input limit; the client's maximum is unknown.
pub const MAX_NAME_BYTES: usize = 128; // (R) Bounded decoded names; no typed-chat maximum is recovered.
pub const LOCAL_CHAT_RADIUS: f32 = 30.0; // (R) Provisional say radius; no server rule is recovered.
pub const CHAT_SPEED_ANY_MS: u64 = 330;
pub const CHAT_SPEED_DUP_MS: u64 = 1_000;
pub const CHAT_SPEED_DUP3_MS: u64 = 2_000;
pub const CHAT_SPEED_MAX_MS: u64 = 30_000;
pub const CHAT_SPEED_SIMILARITY: f32 = 0.2;
const MAX_SPAM_ENTRIES: usize = 256;

// (R) Security Model L6/L9 requires an action budget but specifies no chat rate.
// Start with L6's 5/s, burst 10; 1024 retained accounts is a playtest memory cap.
const ACTION_RATE: f64 = 5.0;
const ACTION_BURST: f64 = 10.0;
const MAX_ACTION_ACCOUNTS: usize = 1024;

/// One shared budget per authenticated account, never per connection/character.
/// Keep one instance for the gateway's lifetime, outside staged/cloned gameplay.
/// Do not remove keys on logout. Evict only buckets that could already pay a full
/// burst: reconnect/key churn then grants no extra allowance. Process restart
/// resets this in-memory policy (R).
#[derive(Debug, Default)]
pub struct ActionBudgets {
    buckets: HashMap<u64, Bucket>,
}

impl ActionBudgets {
    /// Check each validated INNER application before cloning, parsing text,
    /// SpamGate, mutation or fanout, including every application-bundle child.
    /// `account_id` must come from the authenticated server binding, never wire ids.
    /// Typed tells and realm commands use 15/1, so share typed chat's allowance.
    /// False means silent drop (R): no disconnect and no refusal amplification.
    /// This guard supplements all existing parsers, permissions and SpamGate.
    pub fn allow(&mut self, account_id: u64, inner: &[u8], now: Instant) -> bool {
        let Some(header) = inner.get(..4) else {
            return true; // Existing parsers reject malformed messages.
        };
        let family = u16::from_le_bytes([header[0], header[1]]);
        let sub = u16::from_le_bytes([header[2], header[3]]);
        if !matches!(
            (family, sub),
            (15, 1) | (67, 2 | 3) | (26, 14 | 15 | 17 | 30) | (40, 1 | 3 | 4 | 6)
        ) {
            return true;
        }
        if !self.buckets.contains_key(&account_id) && self.buckets.len() >= MAX_ACTION_ACCOUNTS {
            // ponytail: at most 1024 probes on new keys at capacity; no LRU/cache dependency.
            self.buckets.retain(|_, bucket| {
                let mut probe = *bucket;
                !probe.take(ACTION_BURST, ACTION_RATE, ACTION_BURST, now)
            });
            if self.buckets.len() >= MAX_ACTION_ACCOUNTS {
                return false;
            }
        }
        self.buckets
            .entry(account_id)
            .or_insert_with(|| Bucket::full(ACTION_BURST, now))
            .take(1.0, ACTION_RATE, ACTION_BURST, now)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ChatChannel {
    Say,
    Tell,
    System,
    Whisper,
    Group,
    Yell,
    Trade,
    LookingForGroup,
    Area,
    Guild,
    MembersOnly,
}

impl ChatChannel {
    pub const fn wire(self) -> u16 {
        match self {
            Self::Say => 0,
            Self::Tell => 1,
            Self::System => 2,
            Self::Whisper => 3,
            Self::Group => 4,
            Self::Yell => 5,
            Self::Trade => 6,
            Self::LookingForGroup => 7,
            Self::Area => 8,
            Self::Guild => 9,
            Self::MembersOnly => 10,
        }
    }

    pub const fn from_wire(value: u16) -> Option<Self> {
        Some(match value {
            0 => Self::Say,
            1 => Self::Tell,
            2 => Self::System,
            3 => Self::Whisper,
            4 => Self::Group,
            5 => Self::Yell,
            6 => Self::Trade,
            7 => Self::LookingForGroup,
            8 => Self::Area,
            9 => Self::Guild,
            10 => Self::MembersOnly,
            _ => return None,
        })
    }

    const fn client_sendable(self) -> bool {
        !matches!(self, Self::System | Self::Whisper)
    }

    const fn quick_chat_sendable(self) -> bool {
        matches!(
            self,
            Self::Say
                | Self::Group
                | Self::Trade
                | Self::LookingForGroup
                | Self::Area
                | Self::Guild
                | Self::MembersOnly
        )
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct NameData {
    pub fields: [i32; 3],
    pub first: String,
    pub last: String,
}

impl NameData {
    fn display_name(&self) -> String {
        [self.first.trim(), self.last.trim()]
            .into_iter()
            .filter(|part| !part.is_empty())
            .collect::<Vec<_>>()
            .join(" ")
    }

    fn matches(&self, other: &Self) -> bool {
        self.first.eq_ignore_ascii_case(&other.first) && self.last.eq_ignore_ascii_case(&other.last)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PacketChat {
    pub channel: ChatChannel,
    pub sender_guid: u64,
    pub target_guid: u64,
    pub sender_name: NameData,
    pub target_name: NameData,
    pub message: String,
    pub position: [f32; 4],
    pub guild_guid: u64,
    pub language: i32,
    pub area_id: Option<i32>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChatRequest {
    pub channel: ChatChannel,
    pub message: String,
    pub tell_target: Option<NameData>,
    pub claimed_guild_guid: u64,
    pub area_id: Option<i32>,
    pub language: i32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ChatError {
    InvalidPacket,
    InvalidChannel,
    ServerOnlyChannel,
    InvalidMessage,
    QuickChatOnly,
    TextFiltered,
    MissingTellTarget,
    PlayerNotOnline(String),
    NotInGroup,
    GuildMismatch,
    NotGuildMember,
    NotMember,
    AreaMismatch,
    UnknownQuickChatId(i32),
    /// (R) An animated phrase refused by the recovered emote restrictions.
    EmoteRefused(EmoteRejection),
    UnknownReportTarget,
    NoReportHistory,
}

impl ChatError {
    /// Family 22 `PacketTargetClientNotOnline`; the client renders its offline notice.
    pub fn client_reply(&self) -> Option<Vec<u8>> {
        match self {
            Self::PlayerNotOnline(name) => {
                let mut out = 22u16.to_le_bytes().to_vec();
                put_string(&mut out, name).ok()?;
                Some(out)
            }
            _ => None,
        }
    }
}

struct Reader<'a> {
    bytes: &'a [u8],
    at: usize,
}

impl<'a> Reader<'a> {
    fn take(&mut self, count: usize) -> Result<&'a [u8], ChatError> {
        let end = self.at.checked_add(count).ok_or(ChatError::InvalidPacket)?;
        let bytes = self
            .bytes
            .get(self.at..end)
            .ok_or(ChatError::InvalidPacket)?;
        self.at = end;
        Ok(bytes)
    }

    fn u16(&mut self) -> Result<u16, ChatError> {
        Ok(u16::from_le_bytes(self.take(2)?.try_into().unwrap()))
    }

    fn i32(&mut self) -> Result<i32, ChatError> {
        Ok(i32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }

    fn u32(&mut self) -> Result<u32, ChatError> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }

    fn u64(&mut self) -> Result<u64, ChatError> {
        Ok(u64::from_le_bytes(self.take(8)?.try_into().unwrap()))
    }

    fn f32(&mut self) -> Result<f32, ChatError> {
        Ok(f32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }

    fn string(&mut self, max: usize) -> Result<String, ChatError> {
        let len = usize::try_from(self.i32()?).map_err(|_| ChatError::InvalidPacket)?;
        if len > max {
            return Err(ChatError::InvalidPacket);
        }
        String::from_utf8(self.take(len)?.to_vec()).map_err(|_| ChatError::InvalidPacket)
    }

    fn name(&mut self) -> Result<NameData, ChatError> {
        Ok(NameData {
            fields: [self.i32()?, self.i32()?, self.i32()?],
            first: self.string(MAX_NAME_BYTES)?,
            last: self.string(MAX_NAME_BYTES)?,
        })
    }

    fn finish(self) -> Result<(), ChatError> {
        if self.at == self.bytes.len() {
            Ok(())
        } else {
            Err(ChatError::InvalidPacket)
        }
    }
}

fn put_string(out: &mut Vec<u8>, value: &str) -> Result<(), ChatError> {
    let len = i32::try_from(value.len()).map_err(|_| ChatError::InvalidMessage)?;
    out.extend_from_slice(&len.to_le_bytes());
    out.extend_from_slice(value.as_bytes());
    Ok(())
}

fn put_name(out: &mut Vec<u8>, name: &NameData) -> Result<(), ChatError> {
    for value in name.fields {
        out.extend_from_slice(&value.to_le_bytes());
    }
    if name.first.len() > MAX_NAME_BYTES || name.last.len() > MAX_NAME_BYTES {
        return Err(ChatError::InvalidPacket);
    }
    put_string(out, &name.first)?;
    put_string(out, &name.last)
}

fn read_header(reader: &mut Reader<'_>, family: u16, sub: u16) -> Result<(), ChatError> {
    if reader.u16()? != family || reader.u16()? != sub {
        return Err(ChatError::InvalidPacket);
    }
    Ok(())
}

/// Decode the family 15, mode 1 packet using client reader `00b11600`.
pub fn decode_packet_chat(bytes: &[u8]) -> Result<PacketChat, ChatError> {
    let mut r = Reader { bytes, at: 0 };
    read_header(&mut r, 15, 1)?;
    let channel = ChatChannel::from_wire(r.u16()?).ok_or(ChatError::InvalidChannel)?;
    let sender_guid = r.u64()?;
    let target_guid = r.u64()?;
    let sender_name = r.name()?;
    let target_name = r.name()?;
    let message = r.string(MAX_MESSAGE_BYTES)?;
    let position = [r.f32()?, r.f32()?, r.f32()?, r.f32()?];
    let guild_guid = r.u64()?;
    let language = r.i32()?;
    let area_id = if channel == ChatChannel::Area {
        Some(r.i32()?)
    } else {
        None
    };
    r.finish()?;
    Ok(PacketChat {
        channel,
        sender_guid,
        target_guid,
        sender_name,
        target_name,
        message,
        position,
        guild_guid,
        language,
        area_id,
    })
}

/// Encode the family 15, mode 1 packet in client writer order `00b14fa0`.
pub fn encode_packet_chat(packet: &PacketChat) -> Result<Vec<u8>, ChatError> {
    if packet.message.len() > MAX_MESSAGE_BYTES
        || (packet.channel == ChatChannel::Area) != packet.area_id.is_some()
    {
        return Err(ChatError::InvalidMessage);
    }
    let mut out = vec![15, 0, 1, 0];
    out.extend_from_slice(&packet.channel.wire().to_le_bytes());
    out.extend_from_slice(&packet.sender_guid.to_le_bytes());
    out.extend_from_slice(&packet.target_guid.to_le_bytes());
    put_name(&mut out, &packet.sender_name)?;
    put_name(&mut out, &packet.target_name)?;
    put_string(&mut out, &packet.message)?;
    for value in packet.position {
        out.extend_from_slice(&value.to_le_bytes());
    }
    out.extend_from_slice(&packet.guild_guid.to_le_bytes());
    out.extend_from_slice(&packet.language.to_le_bytes());
    if let Some(area_id) = packet.area_id {
        out.extend_from_slice(&area_id.to_le_bytes());
    }
    Ok(out)
}

/// Parse a typed client request. Sender identity fields remain untrusted packet data.
pub fn parse_client_chat(bytes: &[u8]) -> Result<ChatRequest, ChatError> {
    let packet = decode_packet_chat(bytes)?;
    if !packet.channel.client_sendable() {
        return Err(ChatError::ServerOnlyChannel);
    }
    let tell_target = (packet.channel == ChatChannel::Tell).then_some(packet.target_name);
    if tell_target
        .as_ref()
        .is_some_and(|name| name.first.trim().is_empty())
    {
        return Err(ChatError::MissingTellTarget);
    }
    Ok(ChatRequest {
        channel: packet.channel,
        message: packet.message,
        tell_target,
        claimed_guild_guid: packet.guild_guid,
        area_id: packet.area_id,
        language: packet.language,
    })
}

/// Startup-table phrase ids with the animation id of each row (0 = plain phrase).
/// The client packet carries only the phrase id, so the animation always comes
/// from this server-side table, never from the sender.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct QuickChatCatalog(HashMap<i32, i32>);

impl QuickChatCatalog {
    /// Plain phrases only: no row is treated as an emote.
    pub fn from_ids(ids: impl IntoIterator<Item = i32>) -> Self {
        Self::from_rows(ids.into_iter().map(|id| (id, 0)))
    }

    /// `(row id, animation id)` pairs from the startup quick-chat table.
    pub fn from_rows(rows: impl IntoIterator<Item = (i32, i32)>) -> Self {
        Self(rows.into_iter().collect())
    }

    pub fn contains(&self, phrase_id: i32) -> bool {
        self.0.contains_key(&phrase_id)
    }

    /// (R) Apply the recovered emote restrictions (`piece_093`) to one phrase.
    /// Plain phrases always pass; an animated row must be a recovered emote
    /// animation (3200 `Dance` stays unavailable) and the actor state must allow
    /// it. Refusals are silent at the gateway, like the client's disabled button.
    pub fn authorize_emote(&self, phrase_id: i32, context: EmoteContext) -> Result<(), ChatError> {
        let animation = *self
            .0
            .get(&phrase_id)
            .ok_or(ChatError::UnknownQuickChatId(phrase_id))?;
        if animation == 0 {
            return Ok(());
        }
        // A negative id cannot be a recovered animation: map it outside the block.
        let animation = u32::try_from(animation).unwrap_or(u32::MAX);
        let phrase = u32::try_from(phrase_id).unwrap_or(u32::MAX);
        piece_093::validate_emote_phrase(phrase, animation, context)
            .map_err(ChatError::EmoteRefused)
    }
}

/// (R) A `freezeMovement` emote cannot start within this window of the last
/// accepted position update (opcode 125 carries no velocity, so movement is
/// inferred from recent updates).
pub const EMOTE_MOVING_WINDOW_MS: u64 = 500;

/// (R) Actor state for emote validation. `mounted` comes from the mount service
/// (not routed yet, so callers pass false); `moving` from the last accepted
/// movement time on the same clock as `now_ms`.
pub fn emote_context(mounted: bool, last_movement_ms: Option<u64>, now_ms: u64) -> EmoteContext {
    EmoteContext {
        mounted,
        moving: last_movement_ms
            .is_some_and(|last| now_ms.saturating_sub(last) < EMOTE_MOVING_WINDOW_MS),
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum QuickChatRequest {
    Tell {
        phrase_id: i32,
        target_name: String,
    },
    Channel {
        phrase_id: i32,
        channel: ChatChannel,
        guild_guid: u64,
    },
}

/// Decode family 67 sub 2/3 and reject phrase IDs absent from the startup table.
pub fn decode_quick_chat_request(
    bytes: &[u8],
    catalog: &QuickChatCatalog,
) -> Result<QuickChatRequest, ChatError> {
    let mut r = Reader { bytes, at: 0 };
    let family = r.u16()?;
    let sub = r.u16()?;
    if family != 67 || !matches!(sub, 2 | 3) {
        return Err(ChatError::InvalidPacket);
    }
    let phrase_id = r.i32()?;
    if !catalog.contains(phrase_id) {
        return Err(ChatError::UnknownQuickChatId(phrase_id));
    }
    let _untrusted_sender_guid = r.u64()?;
    let _untrusted_sender_name = r.name()?;
    let request = if sub == 2 {
        let _untrusted_target_guid = r.u64()?;
        QuickChatRequest::Tell {
            phrase_id,
            target_name: r.string(MAX_NAME_BYTES)?,
        }
    } else {
        let channel = ChatChannel::from_wire(r.u16()?).ok_or(ChatError::InvalidChannel)?;
        let _reserved = r.u32()?;
        let guild_guid = r.u64()?;
        if !channel.quick_chat_sendable() {
            return Err(ChatError::ServerOnlyChannel);
        }
        QuickChatRequest::Channel {
            phrase_id,
            channel,
            guild_guid,
        }
    };
    r.finish()?;
    Ok(request)
}

/// Build a family 67 sub 3 message. Sender data must come from the session registry.
pub fn encode_quick_chat_channel(
    catalog: &QuickChatCatalog,
    phrase_id: i32,
    channel: ChatChannel,
    sender_guid: u64,
    sender_name: &NameData,
    guild_guid: u64,
) -> Result<Vec<u8>, ChatError> {
    if !catalog.contains(phrase_id) {
        return Err(ChatError::UnknownQuickChatId(phrase_id));
    }
    if !channel.quick_chat_sendable() || (channel != ChatChannel::Guild && guild_guid != 0) {
        return Err(ChatError::InvalidChannel);
    }
    let mut out = vec![67, 0, 3, 0];
    out.extend_from_slice(&phrase_id.to_le_bytes());
    out.extend_from_slice(&sender_guid.to_le_bytes());
    put_name(&mut out, sender_name)?;
    out.extend_from_slice(&channel.wire().to_le_bytes());
    out.extend_from_slice(&0u32.to_le_bytes());
    out.extend_from_slice(&guild_guid.to_le_bytes());
    Ok(out)
}

/// Build a family 67 sub 2 message; the unused target GUID remains zero.
pub fn encode_quick_chat_tell(
    catalog: &QuickChatCatalog,
    phrase_id: i32,
    sender_guid: u64,
    sender_name: &NameData,
    target_name: &str,
) -> Result<Vec<u8>, ChatError> {
    if !catalog.contains(phrase_id) {
        return Err(ChatError::UnknownQuickChatId(phrase_id));
    }
    if target_name.is_empty() || target_name.len() > MAX_NAME_BYTES {
        return Err(ChatError::MissingTellTarget);
    }
    let mut out = vec![67, 0, 2, 0];
    out.extend_from_slice(&phrase_id.to_le_bytes());
    out.extend_from_slice(&sender_guid.to_le_bytes());
    put_name(&mut out, sender_name)?;
    out.extend_from_slice(&0u64.to_le_bytes());
    put_string(&mut out, target_name)?;
    Ok(out)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AccountChatPermissions {
    underage: bool,
    open_chat: bool,
}

impl AccountChatPermissions {
    pub const fn new(underage: bool, open_chat: bool) -> Self {
        Self {
            underage,
            open_chat,
        }
    }

    /// (R) The native UI blocks typed input for underage accounts unless open chat is enabled.
    pub const fn allows_free_text(self) -> bool {
        !self.underage || self.open_chat
    }

    pub const fn allows_quick_chat(self) -> bool {
        true
    }

    pub fn authorize_free_text(self, message: &str) -> Result<(), ChatError> {
        self.authorize_free_text_with_filter(message, &PermissiveTextFilter)
    }

    pub fn authorize_free_text_with_filter(
        self,
        message: &str,
        filter: &impl TextFilter,
    ) -> Result<(), ChatError> {
        if !self.allows_free_text() {
            return Err(ChatError::QuickChatOnly);
        }
        if message.is_empty() || message.len() > MAX_MESSAGE_BYTES {
            return Err(ChatError::InvalidMessage);
        }
        if !filter.allows(message) {
            return Err(ChatError::TextFiltered);
        }
        Ok(())
    }
}

pub trait TextFilter {
    fn allows(&self, message: &str) -> bool;
}

#[derive(Clone, Copy, Debug, Default)]
pub struct PermissiveTextFilter;

impl TextFilter for PermissiveTextFilter {
    fn allows(&self, _: &str) -> bool {
        true // (R) Default until a verified moderation list exists.
    }
}

/// (R) Configurable safe-chat stub: allowed words are supplied by deployment policy.
#[derive(Clone, Debug, Default)]
pub struct WhitelistFilter {
    words: HashSet<String>,
}

impl WhitelistFilter {
    pub fn new(words: impl IntoIterator<Item = impl AsRef<str>>) -> Self {
        Self {
            words: words
                .into_iter()
                .map(|word| word.as_ref().to_ascii_lowercase())
                .collect(),
        }
    }
}

impl TextFilter for WhitelistFilter {
    fn allows(&self, message: &str) -> bool {
        if message.is_empty()
            || !message.chars().all(|c| {
                c.is_ascii_alphabetic() || c.is_ascii_whitespace() || c.is_ascii_punctuation()
            })
        {
            return false;
        }
        let words: Vec<_> = message
            .split(|c: char| !c.is_ascii_alphabetic())
            .filter(|word| !word.is_empty())
            .collect();
        !words.is_empty()
            && words
                .iter()
                .all(|word| self.words.contains(&word.to_ascii_lowercase()))
    }
}

#[derive(Clone, Debug)]
struct SentLine {
    at_ms: u64,
    text: String,
}

#[derive(Clone, Debug, Default)]
pub struct SpamGate {
    lines: VecDeque<SentLine>,
    last_any_ms: Option<u64>,
}

impl SpamGate {
    /// Mirror `ChatSpeed` timings and normalized character edit distance (R: server enforcement).
    pub fn allow(&mut self, now_ms: u64, text: &str) -> bool {
        self.lines
            .retain(|line| now_ms.saturating_sub(line.at_ms) <= CHAT_SPEED_MAX_MS);
        if self
            .last_any_ms
            .is_some_and(|last| now_ms.saturating_sub(last) < CHAT_SPEED_ANY_MS)
        {
            return false;
        }

        let similar: Vec<_> = self
            .lines
            .iter()
            .filter(|line| similar_text(&line.text, text))
            .collect();
        if let Some(last) = similar.last() {
            let threshold = if similar.len() >= 4 {
                CHAT_SPEED_MAX_MS
            } else if similar.len() >= 3 {
                CHAT_SPEED_DUP3_MS
            } else {
                CHAT_SPEED_DUP_MS
            };
            if now_ms.saturating_sub(last.at_ms) < threshold {
                return false;
            }
        }

        self.lines.push_back(SentLine {
            at_ms: now_ms,
            text: text.to_owned(),
        });
        while self.lines.len() > MAX_SPAM_ENTRIES {
            self.lines.pop_front();
        }
        self.last_any_ms = Some(now_ms);
        true
    }
}

fn similar_text(left: &str, right: &str) -> bool {
    let a: Vec<char> = left.chars().collect();
    let b: Vec<char> = right.chars().collect();
    let scale = a.len().max(b.len());
    if scale == 0 {
        return true;
    }
    edit_distance(&a, &b) as f32 / scale as f32 <= CHAT_SPEED_SIMILARITY
}

fn edit_distance(a: &[char], b: &[char]) -> usize {
    let mut previous: Vec<usize> = (0..=b.len()).collect();
    let mut current = vec![0; b.len() + 1];
    for (i, left) in a.iter().enumerate() {
        current[0] = i + 1;
        for (j, right) in b.iter().enumerate() {
            current[j + 1] = (previous[j + 1] + 1)
                .min(current[j] + 1)
                .min(previous[j] + usize::from(left != right));
        }
        std::mem::swap(&mut previous, &mut current);
    }
    previous[b.len()]
}

#[derive(Clone, Debug, PartialEq)]
pub struct ChatSession {
    pub session_id: u64,
    pub character_guid: u64,
    pub name: NameData,
    pub zone_id: u32,
    pub position: [f32; 3],
    pub chat_area_id: Option<i32>,
    pub group_id: Option<u64>,
    pub guild_id: Option<u64>,
    pub is_member: bool,
}

#[derive(Clone, Debug, Default)]
pub struct IgnoreLists(HashMap<u64, HashSet<u64>>);

impl IgnoreLists {
    pub fn add(&mut self, recipient_guid: u64, ignored_sender_guid: u64) {
        self.0
            .entry(recipient_guid)
            .or_default()
            .insert(ignored_sender_guid);
    }

    fn ignores(&self, recipient_guid: u64, sender_guid: u64) -> bool {
        self.0
            .get(&recipient_guid)
            .is_some_and(|list| list.contains(&sender_guid))
    }
}

/// Resolve session IDs only. Every channel scope here is reconstructed policy (R).
pub fn resolve_recipients(
    request: &ChatRequest,
    sender_session_id: u64,
    sessions: &[ChatSession],
    ignores: &IgnoreLists,
) -> Result<Vec<u64>, ChatError> {
    let sender = sessions
        .iter()
        .find(|session| session.session_id == sender_session_id)
        .ok_or(ChatError::InvalidPacket)?;
    let target = if request.channel == ChatChannel::Tell {
        let name = request
            .tell_target
            .as_ref()
            .ok_or(ChatError::MissingTellTarget)?;
        Some(
            sessions
                .iter()
                .find(|session| session.name.matches(name))
                .ok_or_else(|| ChatError::PlayerNotOnline(name.display_name()))?,
        )
    } else {
        None
    };

    let mut recipients: Vec<&ChatSession> = match request.channel {
        ChatChannel::Say => sessions
            .iter()
            .filter(|session| {
                session.zone_id == sender.zone_id
                    && distance_squared(session.position, sender.position)
                        <= LOCAL_CHAT_RADIUS * LOCAL_CHAT_RADIUS
            })
            .collect(),
        ChatChannel::Tell => vec![target.unwrap()],
        ChatChannel::System | ChatChannel::Whisper => return Err(ChatError::ServerOnlyChannel),
        ChatChannel::Group => {
            let group = sender.group_id.ok_or(ChatError::NotInGroup)?;
            sessions
                .iter()
                .filter(|session| session.group_id == Some(group))
                .collect()
        }
        ChatChannel::Yell => sessions
            .iter()
            .filter(|session| session.zone_id == sender.zone_id)
            .collect(),
        ChatChannel::Trade | ChatChannel::LookingForGroup => sessions.iter().collect(),
        ChatChannel::Area => {
            let area = sender.chat_area_id.ok_or(ChatError::AreaMismatch)?;
            if request.area_id != Some(area) {
                return Err(ChatError::AreaMismatch);
            }
            sessions
                .iter()
                .filter(|session| {
                    session.zone_id == sender.zone_id && session.chat_area_id == Some(area)
                })
                .collect()
        }
        ChatChannel::Guild => {
            let guild = sender.guild_id.ok_or(ChatError::NotGuildMember)?;
            if request.claimed_guild_guid != guild {
                return Err(ChatError::GuildMismatch);
            }
            sessions
                .iter()
                .filter(|session| session.guild_id == Some(guild))
                .collect()
        }
        ChatChannel::MembersOnly => {
            if !sender.is_member {
                return Err(ChatError::NotMember);
            }
            sessions
                .iter()
                .filter(|session| session.is_member)
                .collect()
        }
    };
    recipients.retain(|session| !ignores.ignores(session.character_guid, sender.character_guid));
    Ok(recipients
        .into_iter()
        .map(|session| session.session_id)
        .collect())
}

/// Resolve a quick-chat request with the same session and ignore policy as typed chat.
pub fn resolve_quick_chat_recipients(
    request: &QuickChatRequest,
    sender_session_id: u64,
    sessions: &[ChatSession],
    ignores: &IgnoreLists,
) -> Result<Vec<u64>, ChatError> {
    let sender = sessions
        .iter()
        .find(|session| session.session_id == sender_session_id)
        .ok_or(ChatError::InvalidPacket)?;
    match request {
        QuickChatRequest::Channel {
            channel,
            guild_guid,
            ..
        } => resolve_recipients(
            &ChatRequest {
                channel: *channel,
                message: String::new(),
                tell_target: None,
                claimed_guild_guid: *guild_guid,
                area_id: sender.chat_area_id,
                language: 0,
            },
            sender_session_id,
            sessions,
            ignores,
        ),
        QuickChatRequest::Tell { target_name, .. } => {
            let target = sessions
                .iter()
                .find(|session| {
                    session
                        .name
                        .display_name()
                        .eq_ignore_ascii_case(target_name)
                })
                .ok_or_else(|| ChatError::PlayerNotOnline(target_name.clone()))?;
            if ignores.ignores(target.character_guid, sender.character_guid) {
                Ok(Vec::new())
            } else {
                Ok(vec![target.session_id])
            }
        }
    }
}

fn distance_squared(a: [f32; 3], b: [f32; 3]) -> f32 {
    (a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)
}

/// (R) Replace client identity fields with registry values before server fanout.
pub fn server_chat_message(
    request: &ChatRequest,
    sender: &ChatSession,
    target: Option<&ChatSession>,
) -> Result<PacketChat, ChatError> {
    let (target_guid, target_name) = if request.channel == ChatChannel::Tell {
        let target = target.ok_or(ChatError::MissingTellTarget)?;
        (target.character_guid, target.name.clone())
    } else {
        (0, NameData::default())
    };
    Ok(PacketChat {
        channel: request.channel,
        sender_guid: sender.character_guid,
        target_guid,
        sender_name: sender.name.clone(),
        target_name,
        message: request.message.clone(),
        // The W component follows the client fixture shape; session coordinates are authoritative.
        position: [
            sender.position[0],
            sender.position[1],
            sender.position[2],
            1.0,
        ],
        guild_guid: if request.channel == ChatChannel::Guild {
            sender.guild_id.ok_or(ChatError::NotGuildMember)?
        } else {
            0
        },
        language: request.language,
        area_id: if request.channel == ChatChannel::Area {
            sender.chat_area_id
        } else {
            None
        },
    })
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChatLine {
    pub metadata: [i32; 2],
    pub text: String,
}

impl ChatLine {
    pub fn new(metadata: [i32; 2], text: String) -> Self {
        Self { metadata, text }
    }
}

#[derive(Clone, Debug, Default)]
pub struct ChatHistory {
    by_guid: HashMap<u64, VecDeque<ChatLine>>,
    name_to_guid: HashMap<String, u64>,
}

impl ChatHistory {
    pub fn record(&mut self, speaker: &ChatSession, line: ChatLine) {
        let name_key = speaker.name.display_name().to_ascii_lowercase();
        self.name_to_guid.insert(name_key, speaker.character_guid);
        let lines = self.by_guid.entry(speaker.character_guid).or_default();
        lines.push_back(line);
        while lines.len() > 10 {
            lines.pop_front();
        }
    }

    fn lines(&self, character_guid: u64) -> Option<&VecDeque<ChatLine>> {
        self.by_guid.get(&character_guid)
    }

    fn guid_for_name(&self, name: &str) -> Option<u64> {
        self.name_to_guid
            .get(&name.trim().to_ascii_lowercase())
            .copied()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ClientReportRequest {
    target_name: String,
    _client_lines: Vec<ChatLine>,
}

fn decode_report_request(bytes: &[u8]) -> Result<ClientReportRequest, ChatError> {
    // (R) Count and record envelope follows the client report constructor's ten-record vector;
    // the server-side report reader was not recovered.
    let mut r = Reader { bytes, at: 0 };
    read_header(&mut r, 68, 1)?;
    let target_name = r.string(MAX_NAME_BYTES)?;
    let count = usize::try_from(r.i32()?).map_err(|_| ChatError::InvalidPacket)?;
    if count > 10 {
        return Err(ChatError::InvalidPacket);
    }
    let mut lines = Vec::with_capacity(count);
    for _ in 0..count {
        let metadata = [r.i32()?, r.i32()?];
        let text = r.string(MAX_MESSAGE_BYTES)?;
        lines.push(ChatLine { metadata, text });
    }
    r.finish()?;
    Ok(ClientReportRequest {
        target_name,
        _client_lines: lines,
    })
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModerationReport {
    pub reporter_guid: u64,
    pub reported_guid: u64,
    pub reported_name: String,
    pub lines: Vec<ChatLine>,
}

#[derive(Clone, Debug, Default)]
pub struct ReportInbox(Vec<ModerationReport>);

impl ReportInbox {
    pub fn reports(&self) -> &[ModerationReport] {
        &self.0
    }
}

/// Use server-owned history; client-supplied line text is parsed but never trusted or retained.
pub fn intake_report(
    bytes: &[u8],
    reporter_guid: u64,
    sessions: &[ChatSession],
    history: &ChatHistory,
    inbox: &mut ReportInbox,
) -> Result<ModerationReport, ChatError> {
    let request = decode_report_request(bytes)?;
    let online_target = sessions.iter().find(|session| {
        session
            .name
            .display_name()
            .eq_ignore_ascii_case(&request.target_name)
    });
    let reported_guid = history
        .guid_for_name(&request.target_name)
        .or_else(|| online_target.map(|target| target.character_guid))
        .ok_or(ChatError::UnknownReportTarget)?;
    let lines = history
        .lines(reported_guid)
        .filter(|lines| !lines.is_empty())
        .ok_or(ChatError::NoReportHistory)?
        .iter()
        .cloned()
        .collect();
    let report = ModerationReport {
        reporter_guid,
        reported_guid,
        reported_name: online_target
            .map(|target| target.name.display_name())
            .unwrap_or(request.target_name),
        lines,
    };
    inbox.0.push(report.clone());
    Ok(report)
}

#[cfg(test)]
mod action_budget_tests {
    use super::*;
    use std::time::{Duration, Instant};

    const TYPED: &[u8] = &[15, 0, 1, 0];

    #[test]
    fn one_burst_is_shared_by_chat_quick_tells_friends_and_groups() {
        let now = Instant::now();
        let mut budgets = ActionBudgets::default();
        let actions: [&[u8]; 5] = [
            TYPED,
            &[67, 0, 3, 0],
            &[67, 0, 2, 0],
            &[26, 0, 14, 0],
            &[40, 0, 1, 0],
        ];
        for action in actions.into_iter().cycle().take(ACTION_BURST as usize) {
            assert!(budgets.allow(7, action, now));
        }
        for action in actions {
            assert!(!budgets.allow(7, action, now));
        }
        assert!(budgets.allow(8, TYPED, now));
        assert!(budgets.allow(7, &[125, 0], now)); // Movement has its own policy.
    }

    #[test]
    fn refill_is_monotonic_and_reconnect_does_not_reset_account_budget() {
        let now = Instant::now();
        let mut budgets = ActionBudgets::default();
        assert!((0..ACTION_BURST as usize).all(|_| budgets.allow(7, TYPED, now)));
        // Reconnecting or changing character supplies the same authenticated account id;
        // no session/character id participates in the limiter and disconnect removes nothing.
        assert!(!budgets.allow(7, &[40, 0, 1, 0], now));
        let later = now + Duration::from_millis(400);
        assert!(budgets.allow(7, TYPED, later));
        assert!(budgets.allow(7, TYPED, later));
        assert!(!budgets.allow(7, TYPED, later));
        assert!(!budgets.allow(7, TYPED, later)); // Refusal does not grant tokens.
    }

    #[test]
    fn full_key_map_refuses_new_accounts_until_existing_buckets_refill() {
        let now = Instant::now();
        let mut budgets = ActionBudgets::default();
        for account in 0..MAX_ACTION_ACCOUNTS as u64 {
            assert!(budgets.allow(account, TYPED, now));
        }
        assert!(!budgets.allow(MAX_ACTION_ACCOUNTS as u64, TYPED, now));
        assert_eq!(budgets.buckets.len(), MAX_ACTION_ACCOUNTS);
        // A busy account cannot be evicted to gain a new burst on reconnect.
        assert!((1..ACTION_BURST as usize).all(|_| budgets.allow(0, TYPED, now)));
        let later = now + Duration::from_millis(200);
        assert!(budgets.allow(MAX_ACTION_ACCOUNTS as u64, TYPED, later));
        assert!(budgets.buckets.len() <= MAX_ACTION_ACCOUNTS);
        assert!(budgets.allow(0, TYPED, later));
        assert!(!budgets.allow(0, TYPED, later));
    }

    #[test]
    fn header_guard_covers_social_mutations_and_never_reads_payload_text() {
        let now = Instant::now();
        for header in [
            [15, 0, 1, 0],
            [67, 0, 2, 0],
            [67, 0, 3, 0],
            [26, 0, 14, 0],
            [26, 0, 15, 0],
            [26, 0, 17, 0],
            [26, 0, 30, 0],
            [40, 0, 1, 0],
            [40, 0, 3, 0],
            [40, 0, 4, 0],
            [40, 0, 6, 0],
        ] {
            let mut budgets = ActionBudgets::default();
            assert!((0..ACTION_BURST as usize).all(|_| budgets.allow(9, &header, now)));
            assert!(!budgets.allow(9, &header, now), "{header:?}");
        }
        let mut budgets = ActionBudgets::default();
        for len in 0..4 {
            assert!(budgets.allow(9, &TYPED[..len], now));
        }
        assert!(budgets.buckets.is_empty()); // Full parser remains responsible for malformed input.
    }
}
