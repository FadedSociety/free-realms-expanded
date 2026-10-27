//! T763: the Card Duelist (TCG) service wire, decoded from `tcg/FreeRealmsTCG.dll`
//! (lab/content/tcg/analysis/lead-login-decode-2026-10-26.md): the compact integer and string
//! codec, the per-level class header, client frames, the gateway (414) and the session login
//! (411 -> 81). T764 adds the one-time launch ticket store the login is checked against; no
//! socket lives here: the TCP listener is `bin/osfr_gateway/card_listener.rs` and the session's
//! own store rides in `Gameplay` (`gameplay/card_launch.rs`).

use std::{collections::BTreeMap, fmt, io};

/// `GatewayCommand_GetConnectionServer`: request and reply share the layout.
pub const GET_CONNECTION_SERVER: i32 = 414;
/// `LoginCommand_SendSessionID`, written by the client at version 8 (`1017b920`).
pub const SEND_SESSION_ID: i32 = 411;
/// `LoginCommand_Results`; the client's own copy is version 6, which adds a zlib blob.
pub const LOGIN_RESULTS: i32 = 81;
/// The version this server writes 81 at: every field the handler `10179530` uses, no blob.
const LOGIN_RESULTS_VERSION: i32 = 5;
/// T808: a refused 81's code, the client's message 63 "Your session is not valid." (message
/// factory `1038a440`). One code for every refusal: a spent ticket is gone, so it cannot be told
/// from an unknown one (70, "already been used", would need a spent list).
const REFUSED: i32 = 63;
/// `NetworkCommand_Ping` (creator `10190080`, id `1018fdf0`, version 1 `1018fe40`).
pub const PING: i32 = 112;
/// The header is written once per class level because `1019de90` is virtual: the command,
/// its family (Gateway/Login Command) and NetworkCommand.
const HEADER_LEVELS: usize = 3;
/// The DLL's frame sanity limit for a string (`10097b80`).
const MAX_STRING: usize = 0x140_0000;
/// (R) Largest client frame accepted; the client's own commands are far smaller.
pub const MAX_FRAME: usize = 64 * 1024;

/// Compact integer writer `10097a40` (the codec's big-endian mode byte `1080205c` is never
/// set): a value above `!value` is complemented and flagged; the first byte carries the low
/// nibble, the flag (0x10) and the count of following bytes (bits 5-7), which hold the rest
/// little-endian. The same writer as the trade flags' (F766: one implementation).
pub fn put_int(out: &mut Vec<u8>, value: i32) {
    use crate::game_rules::{IntegerEncoding, append_serial_integer};
    append_serial_integer(out, value as u32, IntegerEncoding::Compact);
}

/// String writer `10097b40`: the length, then the text and its NUL.
pub fn put_string(out: &mut Vec<u8>, value: &str) {
    put_int(out, value.len() as i32);
    out.extend(value.as_bytes());
    out.push(0);
}

/// The class header for every level of a command: `(id, version)` three times.
pub fn put_headers(out: &mut Vec<u8>, id: i32, version: i32) {
    for _ in 0..HEADER_LEVELS {
        put_int(out, id);
        put_int(out, version);
    }
}

/// A command body read field by field, as the DLL's readers do.
pub struct Reader<'a> {
    bytes: &'a [u8],
    at: usize,
}

impl<'a> Reader<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, at: 0 }
    }

    fn byte(&mut self) -> io::Result<u8> {
        let byte = *self
            .bytes
            .get(self.at)
            .ok_or_else(|| invalid("card command is truncated"))?;
        self.at += 1;
        Ok(byte)
    }

    /// Compact integer reader `100977a0`. More than four following bytes is refused: the
    /// writer never makes them, and the client's shift would wrap.
    pub fn int(&mut self) -> io::Result<i32> {
        let first = self.byte()?;
        let extra = u32::from(first >> 5);
        if extra > 4 {
            return Err(invalid("card integer is too long"));
        }
        let mut value = u32::from(first & 0xF);
        for index in 0..extra {
            value = value.wrapping_add(u32::from(self.byte()?) << (4 + 8 * index));
        }
        if first & 0x10 != 0 {
            value = !value;
        }
        Ok(value as i32)
    }

    /// String reader `10097b80`: the length, then that many bytes plus the NUL.
    pub fn string(&mut self) -> io::Result<String> {
        let length = usize::try_from(self.int()?)
            .ok()
            .filter(|length| *length <= MAX_STRING)
            .ok_or_else(|| invalid("card string length is invalid"))?;
        let end = self
            .at
            .checked_add(length + 1)
            .filter(|end| *end <= self.bytes.len())
            .ok_or_else(|| invalid("card string is truncated"))?;
        let text = std::str::from_utf8(&self.bytes[self.at..self.at + length])
            .map_err(|_| invalid("card string is not UTF-8"))?
            .to_owned();
        self.at = end;
        Ok(text)
    }

    /// The headers of every class level (`1019def0`); returns the first (most-derived)
    /// version, the one versioned readers branch on.
    pub fn headers(&mut self, id: i32) -> io::Result<i32> {
        let mut first = None;
        for _ in 0..HEADER_LEVELS {
            if self.int()? != id {
                return Err(invalid("card command has the wrong class id"));
            }
            let version = self.int()?;
            first.get_or_insert(version);
        }
        Ok(first.unwrap_or_default())
    }

    pub fn finished(&self) -> bool {
        self.at == self.bytes.len()
    }
}

/// The class id of a command body, without consuming it (`10097860`).
pub fn class_id(body: &[u8]) -> io::Result<i32> {
    Reader::new(body).int()
}

/// One client frame (`101921e0`): `htonl(length)`, then three routing words and the body; the
/// gateway request (414) is sent without routing words (`101935d0`).
#[derive(Debug, PartialEq, Eq)]
pub struct ClientFrame<'a> {
    pub routing: Option<[u32; 3]>,
    pub body: &'a [u8],
}

/// Split the next client frame off `buffer`: `Ok(None)` until the whole frame has arrived,
/// else the bytes consumed and the frame.
pub fn client_frame(buffer: &[u8]) -> io::Result<Option<(usize, ClientFrame<'_>)>> {
    let Some(length) = buffer.get(..4) else {
        return Ok(None);
    };
    let length = u32::from_be_bytes(length.try_into().unwrap()) as usize;
    if length > MAX_FRAME {
        return Err(invalid("card frame is too large"));
    }
    let Some(payload) = buffer.get(4..4 + length) else {
        return Ok(None);
    };
    // A routed frame starts with the big-endian routing word 1000000000, whose first byte never
    // reads as the compact 414 a gateway body starts with.
    let frame = if class_id(payload).ok() == Some(GET_CONNECTION_SERVER) {
        ClientFrame {
            routing: None,
            body: payload,
        }
    } else {
        let words = payload
            .get(..12)
            .ok_or_else(|| invalid("card frame has no routing words"))?;
        let word = |at: usize| u32::from_be_bytes(words[at..at + 4].try_into().unwrap());
        ClientFrame {
            routing: Some([word(0), word(4), word(8)]),
            body: &payload[12..],
        }
    };
    Ok(Some((4 + length, frame)))
}

/// A server frame as the client's receiver (`10192500`) reads it: `htonl(length)` and the body.
pub fn server_frame(body: &[u8]) -> Vec<u8> {
    let mut frame = (body.len() as u32).to_be_bytes().to_vec();
    frame.extend(body);
    frame
}

/// 414 `GatewayCommand_GetConnectionServer` (writer `10121d20`, reader `10121c60`). The client
/// fills the session id and user name (`101935d0`); the reply's `ok` sends it to `host:port`
/// (`10121980`, event 150), otherwise `error` is shown (event 149).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ConnectionServer {
    pub session_id: String,
    pub username: String,
    pub host: String,
    pub port: i32,
    pub ok: bool,
    pub error: i32,
}

impl ConnectionServer {
    pub fn decode(body: &[u8]) -> io::Result<Self> {
        let mut reader = Reader::new(body);
        reader.headers(GET_CONNECTION_SERVER)?;
        let command = Self {
            session_id: reader.string()?,
            username: reader.string()?,
            host: reader.string()?,
            port: reader.int()?,
            ok: reader.int()? != 0,
            error: reader.int()?,
        };
        if !reader.finished() {
            return Err(invalid("card gateway command has trailing bytes"));
        }
        Ok(command)
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut body = Vec::new();
        put_headers(&mut body, GET_CONNECTION_SERVER, 1);
        put_string(&mut body, &self.session_id);
        put_string(&mut body, &self.username);
        put_string(&mut body, &self.host);
        put_int(&mut body, self.port);
        put_int(&mut body, i32::from(self.ok));
        put_int(&mut body, self.error);
        body
    }
}

/// 411 `LoginCommand_SendSessionID` (writer `1017b700`, reader `1017b950`); field names from its
/// describe method `1017b4b0`. `session_id` is the game's own session ticket (`--sessionID`,
/// exe `00ad47f0`); `ticket` is the 64-bit launch ticket the server sent in 39/0x28
/// (`--ticket=%llu`), and `character_id` the character guid, both as decimal text. The retail
/// exe passes no `--characterID` (native 2026-10-27: its command line carries `--character=<name>`
/// and `--ticket` only), so `character_id` arrives empty. T805: the session dialog `103c9140`
/// builds the command with ctor `1017bac0`, which copies the DLL's launch arguments (stored by
/// `1037bba0` before it opens the dialog) into CharacterName, CharacterID and Ticket.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SessionLogin {
    /// The most-derived header version, the one the reader's gates use.
    pub version: i32,
    pub session_id: String,
    /// The client's local data versions ({1: 1314, 2: archetypes, 4: rules}).
    pub data_versions: Vec<(i32, i32)>,
    pub platform: String,
    pub match_challenge_key: String,
    pub is_parent_session: bool,
    pub login_client: String,
    pub locale: String,
    pub character_name: String,
    pub character_id: String,
    pub ticket: String,
    pub namespace: String,
}

impl SessionLogin {
    /// The reader's version gates (`1017b950`).
    pub fn decode(body: &[u8]) -> io::Result<Self> {
        let mut reader = Reader::new(body);
        let version = reader.headers(SEND_SESSION_ID)?;
        let mut login = Self {
            version,
            session_id: reader.string()?,
            ..Self::default()
        };
        if version > 1 {
            let count = usize::try_from(reader.int()?)
                .ok()
                .filter(|count| *count <= 64)
                .ok_or_else(|| invalid("card login version map is invalid"))?;
            for _ in 0..count {
                login.data_versions.push((reader.int()?, reader.int()?));
            }
            login.platform = reader.string()?;
        }
        if version > 2 {
            login.match_challenge_key = reader.string()?;
        }
        if version > 3 {
            login.is_parent_session = reader.int()? != 0;
        }
        if version > 4 {
            login.login_client = reader.string()?;
            login.locale = reader.string()?;
        }
        if version > 5 {
            login.character_name = reader.string()?;
            if version == 6 {
                reader.int()?;
            }
        }
        if version > 6 {
            login.character_id = reader.string()?;
            login.ticket = reader.string()?;
        }
        if version > 7 {
            login.namespace = reader.string()?;
        }
        if !reader.finished() {
            return Err(invalid("card login command has trailing bytes"));
        }
        Ok(login)
    }
}

/// 81 `LoginCommand_Results` at version 5 (reader `10179c70`): success, code, the account id
/// the client adopts as its player id (`10068ac0`), a flag and a word the handler passes on
/// (both 0, R), and the session id it keeps. A success raises client events 14 and 136.
pub fn login_results(success: bool, code: i32, account_id: i32, session_id: &str) -> Vec<u8> {
    let mut body = Vec::new();
    put_headers(&mut body, LOGIN_RESULTS, LOGIN_RESULTS_VERSION);
    put_int(&mut body, i32::from(success));
    put_int(&mut body, code);
    put_int(&mut body, account_id);
    put_int(&mut body, 0);
    put_int(&mut body, 0);
    put_string(&mut body, session_id);
    body
}

/// T808: the account id an accepted login's 81 carries for `character`. Never 0: the success
/// handler (`10179530` -> `10068ac0`) stores no player for 0 and `10007ed0` then writes through a
/// null player. Deterministic per character and positive; the account store's sequential guids
/// (from 1) map to themselves up to `i32::MAX`.
pub fn account_id(character: u64) -> i32 {
    (character.wrapping_sub(1) % i32::MAX as u64 + 1) as i32
}

/// 112 `NetworkCommand_Ping`: the client sends one at login and every 10 s after (`1037c490`), and
/// a server copy resets its "no response" counter (execute `1018fe80` -> `10068dd0`), so the
/// service echoes it. One header, since the class has no family level (writer `10190020` calls
/// the header writer `1019de90` once; reader `1018ffe0`), then the spoof value.
fn ping_echo(body: &[u8]) -> io::Result<Vec<u8>> {
    let mut reader = Reader::new(body);
    if reader.int()? != PING {
        return Err(invalid("card ping has the wrong class id"));
    }
    reader.int()?;
    let spoof = reader.int()?;
    if !reader.finished() {
        return Err(invalid("card ping has trailing bytes"));
    }
    let mut echo = Vec::new();
    for value in [PING, 1, spoof] {
        put_int(&mut echo, value);
    }
    Ok(echo)
}

/// T813: `LobbyCommand_AddGroups`, `LobbyCommand_Join` and `LobbyCommand_ChangeLobbyDisplay`.
pub const ADD_GROUPS: i32 = 94;
pub const JOIN: i32 = 92;
pub const CHANGE_LOBBY_DISPLAY: i32 = 291;
/// T825: `LobbyCommand_ChangeStatus` and the `SynchronizationCommand` family the collection sync
/// uses: Start, StartInstances, SendInstances, Complete, Update.
pub const CHANGE_STATUS: i32 = 137;
pub const SYNC_START: i32 = 83;
pub const SYNC_START_INSTANCES: i32 = 84;
pub const SYNC_SEND_INSTANCES: i32 = 85;
pub const SYNC_COMPLETE: i32 = 86;
pub const SYNC_UPDATE: i32 = 87;
/// T825: `AccountCommand_UploadAvatarImage` (creator `10016720`, reader `10016810`: the account
/// level, a byte count, the image bytes). The client sends its avatar picture when it learns the
/// root group (`102a9710`, from `10371960`); its execute is a no-op (`100165d0`) and no response
/// class exists, so the server answers by silence. The image is not kept until other players
/// can fetch it (`AccountCommand_RequestAvatarImages` -> `SendAvatarImage`).
pub const UPLOAD_AVATAR_IMAGE: i32 = 503;
/// (R) The one delivery a starter sync makes; nothing compares it (`101db550`).
const DELIVERY: i32 = 1;

/// T813: the bodies that take a logged-in card window from "Entering main lobby..." to the
/// lobby screen, in order. The screen dereferences its group without a null check
/// (`102b9930`, called from setup `102b9500`), so the group is defined (94) before the screen
/// opens (291). Every lobby command starts with the LobbyCommand level (reader `10156340`,
/// writer `10156390`): the three class headers, then the account id it is about, here the
/// player's own (the 81's), so Join takes the client's own-account path.
/// - 94 v1 (`101583f0`, execute `101578b0`): a list (`10158350`) of one polymorphic `Lobby`, class
///   103 v3 with one header (reader `10155710`): five optional property sets (1 = absent; the
///   lobby's property reads `1014ba30`/`1014fcb0` treat an absent set as empty), five role lists
///   (count 0), containing group 0, the group id, the lobby type, no deck (1), tournament round
///   0 and sub-state 0. Its execute files the group (`10171b60`).
/// - 92 v4 (`10163b20`, execute `10163220`): group, role, a string (R, empty), an int list (empty),
///   the player's name, its property set (0 = present: an empty `PropertySet`, class 23 v1, map
///   count 0, reader `101ba700`; required once the name is set), and an int (R, 0). For the
///   client's own account it makes the group current (`10068cd0`) and adds the member
///   (`10155590`, role must not be 0).
/// - 291 v1 (`1015e5f0`, execute `1015e400`): the group id; event 70 opens the lobby screen for it
///   (`10372d70` -> `1037e5a0`).
pub fn lobby_entry(account: i32, lobby: &crate::card_game::Lobby, name: &str) -> [Vec<u8>; 3] {
    let level = |id, version| sync_command(id, version, [account]);
    let mut groups = level(ADD_GROUPS, 1);
    // One group: class 103 v3, five absent property sets, five empty role lists.
    let lobby_object = [1, 103, 3, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0];
    // Containing group, group, type, no deck, tournament round, sub-state.
    let ids = [0, lobby.group_id, lobby.lobby_type, 1, 0, 0];
    for value in lobby_object.into_iter().chain(ids) {
        put_int(&mut groups, value);
    }
    let mut join = level(JOIN, 4);
    put_int(&mut join, lobby.group_id);
    put_int(&mut join, lobby.role);
    put_string(&mut join, "");
    put_int(&mut join, 0);
    put_string(&mut join, name);
    // The property set is present (0): `PropertySet` 23 v1, no entries; then the trailing int.
    for value in [0, 23, 1, 0, 0] {
        put_int(&mut join, value);
    }
    let mut display = level(CHANGE_LOBBY_DISPLAY, 1);
    put_int(&mut display, lobby.group_id);
    [groups, join, display]
}

/// T825: the client's lobby status, echoed. 137 `LobbyCommand_ChangeStatus` v1 (reader
/// `1015ea00`, writer `1015ea60`): the LobbyCommand level, then the status. The client sends one
/// on every screen change (`1037c540`: 3 hub, 2 lobby, 7 collection, ...); a server copy sets the
/// player's status and raises event 25 (`1015e810`). With one player the server's broadcast is
/// this echo, always for the connection's own `account`.
pub fn change_status(account: i32, body: &[u8]) -> io::Result<Vec<u8>> {
    let mut reader = Reader::new(body);
    reader.headers(CHANGE_STATUS)?;
    reader.int()?;
    let status = reader.int()?;
    if !reader.finished() {
        return Err(invalid("card status has trailing bytes"));
    }
    let mut echo = sync_command(CHANGE_STATUS, 1, [account]);
    put_int(&mut echo, status);
    Ok(echo)
}

/// T825: the group and role a client 92 Join request asks for (the Join reader `10163b20`: the
/// LobbyCommand level, then group and role; the rest stays empty in a request). The client sends
/// one for the root group 1 as soon as it learns it (`10371960`), and others from its lobby list
/// (`1036fff0`), as an observer (`103717e0`, role 3) and back from a game (`1032dde0`).
pub fn join_request(body: &[u8]) -> io::Result<(i32, i32)> {
    let mut reader = Reader::new(body);
    reader.headers(JOIN)?;
    reader.int()?;
    Ok((reader.int()?, reader.int()?))
}

/// T825: the collection sync, server-driven (`SynchronizationService`): every command has the
/// three class headers (family level `101d7090`/`101d70d0`) and no account level. The cards are
/// owned as `InstanceID`s (class 350 v1, one header, reader `101476a0`): archetype, series 0 and
/// instance numbers 0.. per archetype, as the client's own `default.cln` numbers them, so the
/// same starter always has the same ids and nothing needs saving. The LobbyCommand level is the
/// same shape (reader `10156340`, writer `10156390`): its one value is the account.
fn sync_command(id: i32, version: i32, values: impl IntoIterator<Item = i32>) -> Vec<u8> {
    let mut body = Vec::new();
    put_headers(&mut body, id, version);
    for value in values {
        put_int(&mut body, value);
    }
    body
}

fn instances(cards: &[crate::card_game::StarterCard]) -> i32 {
    cards.iter().map(|card| card.copies).sum()
}

/// The collection's ModID: a stable hash of the starter list, positive and never 0. The client
/// caches a synced collection with it (`1006f1c0`) and skips the next sync when it matches.
pub fn collection_mod_id(cards: &[crate::card_game::StarterCard]) -> i32 {
    let hash = cards.iter().fold(0, |acc, card| {
        crate::hash::mix64(acc ^ ((card.archetype_id as u64) << 8) ^ card.copies as u64)
    });
    (hash >> 33) as i32 | 1
}

/// 83 `SynchronizationCommand_Start` v3 (reader `101da040`, execute `101d9e70` -> `101dbbb0`):
/// the instance count, the ModID, the account, 0. The client answers with Complete when its
/// cached collection has that count and ModID, else with Update.
pub fn sync_start(account: i32, cards: &[crate::card_game::StarterCard]) -> Vec<u8> {
    let values = [instances(cards), collection_mod_id(cards), account, 0];
    sync_command(SYNC_START, 3, values)
}

/// 84 `SynchronizationCommand_StartInstances` v2 (reader `101dadf0`, the vtable `1067266c` +0x24
/// slot; writer/describe `101dae80`, execute `101dbc80`): account, collection type 1 (online), the
/// archetype count (the writer's `Archetype Count` label; the execute stores it and reports it in
/// event 27), the ModID. The client clears the collection and answers with Update.
pub fn sync_start_instances(account: i32, cards: &[crate::card_game::StarterCard]) -> Vec<u8> {
    let values = [account, 1, cards.len() as i32, collection_mod_id(cards)];
    sync_command(SYNC_START_INSTANCES, 2, values)
}

/// 85 `SynchronizationCommand_SendInstances` v2 (reader `101d94c0`, writer/describe `101d8d70`,
/// execute `101dc290`): base archetype 0, the archetype count (the writer's `Archetype Count`
/// label; `101dc290` adds it to the collection counter), the instances (`10077f00`: count, then
/// each `InstanceID`), no non-tradable ones, the delivery id. The client adds them and answers
/// with Update.
pub fn sync_send_instances(cards: &[crate::card_game::StarterCard]) -> Vec<u8> {
    let count = instances(cards);
    let ids = cards.iter().flat_map(|card| {
        (0..card.copies).flat_map(move |number| [350, 1, card.archetype_id, 0, number])
    });
    let values = [0, cards.len() as i32, count]
        .into_iter()
        .chain(ids)
        .chain([0, DELIVERY]);
    sync_command(SYNC_SEND_INSTANCES, 2, values)
}

/// 86 `SynchronizationCommand_Complete` v2 (reader `101d77c0`, execute -> `101db550`): the
/// delivery id. The client stamps the ModID and caches the collection.
pub fn sync_complete() -> Vec<u8> {
    sync_command(SYNC_COMPLETE, 2, [DELIVERY])
}

/// The refusal of a ticket no live session holds: never minted, already spent, or its session
/// ended.
pub const UNKNOWN_TICKET: &str = "unknown_ticket";

/// T764: one-time card launch tickets (lead security). The game server mints one per launch and
/// sends it in 39/0x28; the card login (411) must present it, and a character id the 411 carries
/// must name the owner, before it expires; it works once. (R) The store is per session (one
/// card character, one live launch), and `now_ms` is the caller's monotonic clock, the same one
/// for mint and redeem.
#[derive(Clone)]
pub struct LaunchTickets {
    ttl_ms: u64,
    /// ticket -> (character guid, expiry ms); at most one outstanding ticket per character.
    issued: BTreeMap<u64, (u64, u64)>,
}

/// (R) Ceiling on held tickets, expired ones included until the next mint sweeps them. A session
/// has one card character, so this only bounds a store shared by many, like
/// `login_auth::MAX_TOKENS`.
pub const MAX_TICKETS: usize = 64;
/// (R) Draws before a random source that keeps returning zero or a held ticket is called broken.
const TICKET_DRAWS: usize = 4;

impl fmt::Debug for LaunchTickets {
    /// Redacted: a ticket is a bearer credential, so it must never reach a trace.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LaunchTickets")
            .field("ttl_ms", &self.ttl_ms)
            .field("outstanding", &self.issued.len())
            .finish_non_exhaustive()
    }
}

impl LaunchTickets {
    /// `ttl_ms` is the card content's `launch.ticket_ttl_ms`, bounded 1 s to 10 min there
    /// (`card_game::Content::validate`).
    pub fn new(ttl_ms: u64) -> Self {
        Self {
            ttl_ms,
            issued: BTreeMap::new(),
        }
    }

    /// A fresh non-zero random ticket for `character`. It drops expired tickets and replaces any
    /// the character still holds, so the table stays bounded by live launches. A full table is
    /// refused, and a broken random source (zero or a repeat on every draw) is refused after four
    /// draws rather than retried forever.
    pub fn mint(&mut self, character: u64, now_ms: u64) -> io::Result<u64> {
        self.issued
            .retain(|_, (owner, expires)| *expires > now_ms && *owner != character);
        if self.outstanding() >= MAX_TICKETS {
            return Err(io::Error::other("card launch ticket table is full"));
        }
        let ticket = draw_ticket(&self.issued, random_ticket)?;
        let expires = now_ms.saturating_add(self.ttl_ms);
        self.issued.insert(ticket, (character, expires));
        Ok(ticket)
    }

    /// Redeem the ticket and character id a 411 carries (decimal text, the client's `%llu`):
    /// `Some` holds the character the ticket was issued to, while it has not expired. `None`
    /// means this store does not hold the ticket at all (never minted here, already spent, or
    /// its session ended), so a listener scanning the live sessions moves on to the next store;
    /// the card listener logs that case as [`UNKNOWN_TICKET`]. The random, single-use ticket is
    /// the credential; the retail client sends no character id (empty), and a non-empty one must
    /// name the owner. A refusal changes nothing: only a matching, unexpired ticket is spent, so
    /// a wrong id leaves the ticket as it was (and an expired one stays until the next mint
    /// sweeps it). A refusal is the fixed word the card listener logs (T805):
    /// `no_ticket_field`, `ticket_not_decimal`, `expired` or `character_mismatch`.
    pub fn redeem(
        &mut self,
        ticket: &str,
        character: &str,
        now_ms: u64,
    ) -> Option<Result<u64, &'static str>> {
        if ticket.is_empty() {
            return Some(Err("no_ticket_field"));
        }
        let Some(ticket) = decimal(ticket) else {
            return Some(Err("ticket_not_decimal"));
        };
        let (owner, expires) = self.issued.get(&ticket).copied()?;
        if now_ms >= expires {
            return Some(Err("expired"));
        }
        if !character.is_empty() && decimal(character) != Some(owner) {
            return Some(Err("character_mismatch"));
        }
        self.issued.remove(&ticket);
        Some(Ok(owner))
    }

    /// Tickets still held, expired ones included until the next [`Self::mint`] sweeps them.
    pub fn outstanding(&self) -> usize {
        self.issued.len()
    }
}

/// A non-zero ticket not already held, from at most [`TICKET_DRAWS`] draws. A source that keeps
/// returning zero or a held value is broken, not unlucky: refuse rather than loop.
fn draw_ticket(
    held: &BTreeMap<u64, (u64, u64)>,
    mut next: impl FnMut() -> io::Result<u64>,
) -> io::Result<u64> {
    for _ in 0..TICKET_DRAWS {
        let ticket = next()?;
        if ticket != 0 && !held.contains_key(&ticket) {
            return Ok(ticket);
        }
    }
    Err(invalid("no usable random card launch ticket"))
}

fn random_ticket() -> io::Result<u64> {
    let mut bytes = [0; 8];
    getrandom::getrandom(&mut bytes)
        .map_err(|_| invalid("no OS randomness for a card launch ticket"))?;
    Ok(u64::from_le_bytes(bytes))
}

/// A strict unsigned decimal: digits only, no sign, spaces or overflow.
fn decimal(text: &str) -> Option<u64> {
    if text.is_empty() || text.len() > 20 || !text.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    text.parse().ok()
}

/// The login stub: answer one client body. A 414 is sent to `host:port` (this service); a 411
/// is accepted when `character_for` maps the login (its launch ticket, and the character id it
/// names when present) to a character, whose [`account_id`] the 81 carries; a ping is echoed
/// once `logged_in`. Anything else is not handled yet (`None`). Replies are server frames.
pub fn reply(
    body: &[u8],
    host: &str,
    port: u16,
    logged_in: bool,
    character_for: impl FnOnce(&SessionLogin) -> Option<u64>,
) -> io::Result<Option<Vec<u8>>> {
    match class_id(body)? {
        GET_CONNECTION_SERVER => {
            let request = ConnectionServer::decode(body)?;
            let answer = ConnectionServer {
                host: host.to_owned(),
                port: i32::from(port),
                ok: true,
                error: 0,
                ..request
            };
            Ok(Some(server_frame(&answer.encode())))
        }
        SEND_SESSION_ID => {
            let login = SessionLogin::decode(body)?;
            let body = match character_for(&login) {
                // Code 0: on success, 76 would open username registration (`103c9750`).
                Some(character) => login_results(true, 0, account_id(character), &login.session_id),
                None => login_results(false, REFUSED, 0, ""),
            };
            Ok(Some(server_frame(&body)))
        }
        PING if logged_in => Ok(Some(server_frame(&ping_echo(body)?))),
        _ => Ok(None),
    }
}

fn invalid(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_broken_random_source_is_refused_after_four_draws() {
        let held = BTreeMap::from([(7, (1, u64::MAX))]);
        let mut draws = 0;
        let error = draw_ticket(&held, || {
            draws += 1;
            Ok(if draws == 1 { 7 } else { 0 })
        })
        .unwrap_err();
        assert_eq!(draws, TICKET_DRAWS, "no unbounded retry");
        assert_eq!(error.kind(), io::ErrorKind::InvalidData);

        // Zero and a held ticket are skipped; the next fresh value is used.
        let mut draws = 0;
        let ticket = draw_ticket(&held, || {
            draws += 1;
            Ok(if draws < 3 { 0 } else { 9 })
        })
        .unwrap();
        assert_eq!((ticket, draws), (9, 3));

        // A random-source error is the caller's, not a retry.
        assert!(draw_ticket(&held, || Err(io::Error::other("unavailable"))).is_err());
    }
}
