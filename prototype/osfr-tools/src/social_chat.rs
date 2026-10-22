//! Chat packet handling and server-side chat policy.
//!
//! Wire facts come from the native client readers and writers cited in
//! `notes/Research/Chat Core.md`. Recipient scopes, the local radius, account
//! enforcement, text filtering, and ignore behavior are reconstructed policy (R).

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque};
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::time::Instant;

use serde::{Deserialize, Serialize};

use crate::net_policy::KeyedBuckets;

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
#[derive(Debug)]
pub struct ActionBudgets {
    buckets: KeyedBuckets<u64>,
}

impl Default for ActionBudgets {
    fn default() -> Self {
        Self {
            buckets: KeyedBuckets::new(ACTION_RATE, ACTION_BURST, MAX_ACTION_ACCOUNTS),
        }
    }
}

impl ActionBudgets {
    /// Check each budgeted INNER application before cloning gameplay, parsing
    /// text, SpamGate, mutation or fanout, including every bundle child.
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
            (15, 1)
                | (67, 2 | 3)
                | (26, 10 | 14 | 15 | 17 | 18 | 19 | 21 | 30 | 32 | 33)
                | (40, 1 | 3 | 4 | 6 | 14 | 16)
                | (108, 1 | 3 | 5)
                | (152, 4 | 5)
                // T006: every family-129 guild request shares the account action budget.
                | (129, _)
        ) {
            return true;
        }
        self.buckets.allow(account_id, now)
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

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct NameData {
    /// (R) Three client metadata words (locale name ids); character identity supplies none.
    #[serde(default)]
    pub fields: [i32; 3],
    pub first: String,
    pub last: String,
}

impl NameData {
    pub fn new(first: impl Into<String>, last: impl Into<String>) -> Self {
        Self {
            fields: [0; 3],
            first: first.into(),
            last: last.into(),
        }
    }

    pub(crate) fn display_name(&self) -> String {
        [self.first.trim(), self.last.trim()]
            .into_iter()
            .filter(|part| !part.is_empty())
            .collect::<Vec<_>>()
            .join(" ")
    }

    fn matches(&self, other: &Self) -> bool {
        self.first.eq_ignore_ascii_case(&other.first) && self.last.eq_ignore_ascii_case(&other.last)
    }

    /// Whether the joined display name equals `raw` (case-insensitive), for name invites.
    pub fn matches_display(&self, raw: &str) -> bool {
        self.display_name().eq_ignore_ascii_case(raw.trim())
    }

    /// A bounded name that can be persisted and written as wire `NameData`.
    pub fn is_valid(&self) -> bool {
        self.first.len() <= MAX_NAME_BYTES
            && self.last.len() <= MAX_NAME_BYTES
            && !self
                .first
                .chars()
                .chain(self.last.chars())
                .any(char::is_control)
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

/// Write one client `NameData` (`008e7290`): three dwords, then first and last strings.
/// Callers that accept client text check the bounds and use `put_name` instead.
pub(crate) fn put_name_data(out: &mut Vec<u8>, name: &NameData) {
    for value in name.fields {
        out.extend_from_slice(&value.to_le_bytes());
    }
    for part in [&name.first, &name.last] {
        debug_assert!(part.len() <= MAX_NAME_BYTES, "callers bound NameData parts");
        out.extend_from_slice(&(part.len() as i32).to_le_bytes());
        out.extend_from_slice(part.as_bytes());
    }
}

fn put_name(out: &mut Vec<u8>, name: &NameData) -> Result<(), ChatError> {
    if name.first.len() > MAX_NAME_BYTES || name.last.len() > MAX_NAME_BYTES {
        return Err(ChatError::InvalidPacket);
    }
    put_name_data(out, name);
    Ok(())
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
    /// animation and the actor state must allow it. Refusals are silent at the
    /// gateway, like the client's disabled button.
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

// ---- Channel switches and bubble colour: 26/32, 26/33 and 26/18 (W173) ----
//
// Recovered from the client: the chat window can switch exactly four channels, named
// WorldArea (8), WorldTrade (6), WorldLfg (7) and WorldMembersOnly (10); the name compare
// ignores case (`00b978b0`, `StrICmpUpper`). 26/32 (`CommandPacketChatChannelOn`, ctor
// `00c1dcb0`) and 26/33 (`...Off`, `00c1dd60`) carry one string: i32 length, then the name.
// 26/18 (`CommandPacketSetChatBubbleColor`, ctor `008b6a20`, sent by `0092f380` for the Lua
// `Ui.SetChatColorFgBgSz(fg, bg, size)`) is three i32 then a u64 the client always sends as 0;
// the server never trusts that u64. Native trace bodies: 26/32 "WorldMembersOnly" is
// `1a00 2000 10000000 <16 bytes>`; 26/18 is `1a00 1200 66330000 ffffcc00 02000000 <8 zero>`.
// The client sends every switch at each zone load and keeps its own copy in UserOptions
// (ScriptsBase.lua `setChannelsShowing`: Area and MembersOnly on, Trade and Lfg off by default).
//
// Reconstructed policy (R): a switched-off channel is not delivered to that player, whose own
// echo is never dropped; the stored state starts all-on (today's behaviour) until the client
// reports; colours are 24-bit RGB and the size is 0..=8; the stored bubble is kept and
// exposed, and reaches other players only through `ChatSettings::bubble`.

/// The four client-switchable channels by the name the client sends.
const TOGGLE_CHANNELS: [(&str, ChatChannel); 4] = [
    ("WorldArea", ChatChannel::Area),
    ("WorldTrade", ChatChannel::Trade),
    ("WorldLfg", ChatChannel::LookingForGroup),
    ("WorldMembersOnly", ChatChannel::MembersOnly),
];
pub const MAX_CHANNEL_NAME_BYTES: usize = 32; // (R) The longest real name is 16.
pub const MAX_BUBBLE_COLOR: i32 = 0x00FF_FFFF; // (R) 24-bit RGB; the client's defaults fit.
pub const MAX_BUBBLE_SIZE: i32 = 8; // (R) Client default 2, server default 1.
const MAX_CHAT_SETTINGS_FILE_BYTES: usize = 4096;
const CHAT_SETTINGS_SCHEMA: u32 = 1;

fn toggle_channel(name: &str) -> Option<ChatChannel> {
    TOGGLE_CHANNELS
        .iter()
        .find(|(known, _)| known.eq_ignore_ascii_case(name))
        .map(|(_, channel)| *channel)
}

fn is_toggle_wire(wire: u16) -> bool {
    TOGGLE_CHANNELS
        .iter()
        .any(|(_, channel)| channel.wire() == wire)
}

/// Chat bubble text colour, bubble colour and text size, as the 26/18 fields.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BubbleStyle {
    pub foreground: i32,
    pub background: i32,
    pub size: i32,
}

impl Default for BubbleStyle {
    /// The captured NPC bubble defaults (`world.rs` `content_spawn_template`).
    fn default() -> Self {
        Self {
            foreground: 408_679,
            background: 13_951_728,
            size: 1,
        }
    }
}

impl BubbleStyle {
    fn in_range(self) -> bool {
        (0..=MAX_BUBBLE_COLOR).contains(&self.foreground)
            && (0..=MAX_BUBBLE_COLOR).contains(&self.background)
            && (0..=MAX_BUBBLE_SIZE).contains(&self.size)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SettingsRequest {
    Channel { channel: ChatChannel, on: bool },
    Bubble(BubbleStyle),
}

/// Decode 26/32, 26/33 or 26/18. A wrong header, truncation, trailing bytes, a name that is
/// not one of the four, or an out-of-range value is an error and changes nothing.
pub fn decode_settings_request(inner: &[u8]) -> Result<SettingsRequest, ChatError> {
    let mut r = Reader {
        bytes: inner,
        at: 0,
    };
    let family = r.u16()?;
    let sub = r.u16()?;
    if family != 26 {
        return Err(ChatError::InvalidPacket);
    }
    match sub {
        32 | 33 => {
            let name = r.string(MAX_CHANNEL_NAME_BYTES)?;
            r.finish()?;
            let channel = toggle_channel(&name).ok_or(ChatError::InvalidChannel)?;
            Ok(SettingsRequest::Channel {
                channel,
                on: sub == 32,
            })
        }
        18 => {
            let style = BubbleStyle {
                foreground: r.i32()?,
                background: r.i32()?,
                size: r.i32()?,
            };
            r.u64()?; // The client's guid slot: ignored, the server knows who sent this.
            r.finish()?;
            if style.in_range() {
                Ok(SettingsRequest::Bubble(style))
            } else {
                Err(ChatError::InvalidMessage)
            }
        }
        _ => Err(ChatError::InvalidPacket),
    }
}

/// One character's chat switches and bubble. Persisted per character, never shared.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChatSettings {
    /// Wire numbers of switched-off channels; only the four in `TOGGLE_CHANNELS`.
    muted: BTreeSet<u16>,
    bubble: BubbleStyle,
}

impl ChatSettings {
    pub fn is_muted(&self, channel: ChatChannel) -> bool {
        self.muted.contains(&channel.wire())
    }

    pub fn bubble(&self) -> BubbleStyle {
        self.bubble
    }

    /// True when the state changed. A request that is not one of the four channels or is out
    /// of range changes nothing, even when built by hand.
    pub fn apply(&mut self, request: SettingsRequest) -> bool {
        match request {
            SettingsRequest::Channel { channel, on } if is_toggle_wire(channel.wire()) => {
                if on {
                    self.muted.remove(&channel.wire())
                } else {
                    self.muted.insert(channel.wire())
                }
            }
            SettingsRequest::Bubble(style) if style.in_range() => {
                std::mem::replace(&mut self.bubble, style) != style
            }
            _ => false,
        }
    }

    fn valid(&self) -> bool {
        self.muted.iter().all(|wire| is_toggle_wire(*wire)) && self.bubble.in_range()
    }
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ChatSettingsDocument {
    schema_version: u32,
    character_guid: u64,
    settings: ChatSettings,
}

fn bad_settings(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.to_owned())
}

impl ChatSettings {
    pub fn save(&self, path: &Path, character_guid: u64) -> io::Result<()> {
        if !self.valid() {
            return Err(bad_settings("invalid chat settings"));
        }
        if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent)?;
        }
        let bytes = serde_json::to_vec_pretty(&ChatSettingsDocument {
            schema_version: CHAT_SETTINGS_SCHEMA,
            character_guid,
            settings: self.clone(),
        })
        .map_err(|error| bad_settings(&error.to_string()))?;
        crate::character_store::replace_bytes(path, &bytes)
    }

    pub fn load(path: &Path, character_guid: u64) -> io::Result<Self> {
        let mut bytes = Vec::new();
        std::fs::File::open(path)?
            .take((MAX_CHAT_SETTINGS_FILE_BYTES + 1) as u64)
            .read_to_end(&mut bytes)?;
        if bytes.len() > MAX_CHAT_SETTINGS_FILE_BYTES {
            return Err(bad_settings("chat settings file too large"));
        }
        let document: ChatSettingsDocument =
            serde_json::from_slice(&bytes).map_err(|error| bad_settings(&error.to_string()))?;
        if document.schema_version != CHAT_SETTINGS_SCHEMA
            || document.character_guid != character_guid
            || !document.settings.valid()
        {
            return Err(bad_settings("chat settings do not match this character"));
        }
        Ok(document.settings)
    }
}

/// Every joined character's settings, keyed by the server-bound character guid (never a wire
/// id). Keep one in the shared world; a staged clone is committed or dropped with its message.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ChatSettingsBook {
    settings: BTreeMap<u64, ChatSettings>,
    paths: BTreeMap<u64, PathBuf>,
}

impl ChatSettingsBook {
    pub fn get(&self, character_guid: u64) -> Option<&ChatSettings> {
        self.settings.get(&character_guid)
    }

    /// Load a character at join. A missing or unreadable file starts from the defaults (R): a
    /// cosmetic setting must never block a login, and the next change rewrites the file.
    pub fn join(&mut self, character_guid: u64, path: Option<&Path>) {
        if let Some(path) = path {
            self.paths.insert(character_guid, path.to_owned());
        }
        self.settings.entry(character_guid).or_insert_with(|| {
            path.and_then(|path| ChatSettings::load(path, character_guid).ok())
                .unwrap_or_default()
        });
    }

    /// Apply a client 26/32, 26/33 or 26/18 for the sending character. Ok(true) means the
    /// state changed; an error changed nothing and is a silent drop for the caller (R).
    pub fn handle(&mut self, character_guid: u64, inner: &[u8]) -> Result<bool, ChatError> {
        let request = decode_settings_request(inner)?;
        Ok(self
            .settings
            .entry(character_guid)
            .or_default()
            .apply(request))
    }

    /// Drop recipients that switched `channel` off. The sender's own session is never
    /// dropped, and a channel the client cannot switch (tells, group, say) is untouched.
    pub fn listeners(
        &self,
        channel: ChatChannel,
        sender_session: u64,
        sessions: &[ChatSession],
        mut recipients: Vec<u64>,
    ) -> Vec<u64> {
        if is_toggle_wire(channel.wire()) {
            let muted: HashSet<u64> = sessions
                .iter()
                .filter(|session| {
                    session.session_id != sender_session
                        && self
                            .get(session.character_guid)
                            .is_some_and(|settings| settings.is_muted(channel))
                })
                .map(|session| session.session_id)
                .collect();
            recipients.retain(|id| !muted.contains(id));
        }
        recipients
    }

    /// Write every character whose settings differ from `before` to its file.
    pub fn save_changed(&self, before: &Self) -> io::Result<()> {
        let default = ChatSettings::default();
        for (guid, settings) in &self.settings {
            if before.settings.get(guid).unwrap_or(&default) != settings {
                if let Some(path) = self.paths.get(guid) {
                    settings.save(path, *guid)?;
                }
            }
        }
        Ok(())
    }
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
            // T006: guild requests share the same account action budget.
            [129, 0, 1, 0],
            [129, 0, 0xff, 0xff],
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
