//! T763: the Card Duelist (TCG) service wire, decoded from `tcg/FreeRealmsTCG.dll`
//! (lab/content/tcg/analysis/lead-login-decode-2026-10-26.md): the compact integer and string
//! codec, the per-level class header, client frames, the gateway (414) and the session login
//! (411 -> 81). Pure: the TCP listener and the one-time ticket check live with their callers.

use std::{collections::BTreeMap, io};

/// `GatewayCommand_GetConnectionServer`: request and reply share the layout.
pub const GET_CONNECTION_SERVER: i32 = 414;
/// `LoginCommand_SendSessionID`, written by the client at version 8 (`1017b920`).
pub const SEND_SESSION_ID: i32 = 411;
/// `LoginCommand_Results`; the client's own copy is version 6, which adds a zlib blob.
pub const LOGIN_RESULTS: i32 = 81;
/// The version this server writes 81 at: every field the handler `10179530` uses, no blob.
const LOGIN_RESULTS_VERSION: i32 = 5;
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

/// The refusal of a ticket no live session holds: never minted, already spent, or its session
/// ended.
pub const UNKNOWN_TICKET: &str = "unknown_ticket";

/// T764: one-time card launch tickets (lead security). The game server mints one per launch and
/// sends it in 39/0x28; the card login (411) must present it with the same character id before
/// it expires, and it works once.
#[derive(Clone, Debug)]
pub struct LaunchTickets {
    ttl_ms: u64,
    /// ticket -> (character guid, expiry ms); at most one outstanding ticket per character.
    issued: BTreeMap<u64, (u64, u64)>,
}

impl LaunchTickets {
    pub fn new(ttl_ms: u64) -> Self {
        Self {
            ttl_ms,
            issued: BTreeMap::new(),
        }
    }

    /// A fresh non-zero random ticket for `character`. It replaces any the character still
    /// holds, and expired tickets are dropped, so the table stays bounded by live launches.
    pub fn mint(&mut self, character: u64, now_ms: u64) -> io::Result<u64> {
        self.issued
            .retain(|_, (owner, expires)| *expires > now_ms && *owner != character);
        loop {
            let mut bytes = [0; 8];
            getrandom::getrandom(&mut bytes)
                .map_err(|_| invalid("no OS randomness for a card launch ticket"))?;
            let ticket = u64::from_le_bytes(bytes);
            if ticket != 0 && !self.issued.contains_key(&ticket) {
                let expires = now_ms.saturating_add(self.ttl_ms);
                self.issued.insert(ticket, (character, expires));
                return Ok(ticket);
            }
        }
    }

    /// Redeem the ticket and character id a 411 carries (decimal text, the client's `%llu`):
    /// the character the ticket was issued to, while it has not expired. The random, single-use
    /// ticket is the credential; the retail client sends no character id (empty), and a
    /// non-empty one must name the owner. A presented ticket is spent whether or not it
    /// matched, so one guess never gets a second try. A refusal is the fixed word the card
    /// listener logs (T805): `no_ticket_field`, `ticket_not_decimal`, [`UNKNOWN_TICKET`],
    /// `expired` or `character_mismatch`.
    pub fn redeem(
        &mut self,
        ticket: &str,
        character: &str,
        now_ms: u64,
    ) -> Result<u64, &'static str> {
        if ticket.is_empty() {
            return Err("no_ticket_field");
        }
        let ticket = decimal(ticket).ok_or("ticket_not_decimal")?;
        let (owner, expires) = self.issued.remove(&ticket).ok_or(UNKNOWN_TICKET)?;
        if now_ms >= expires {
            return Err("expired");
        }
        if !character.is_empty() && decimal(character) != Some(owner) {
            return Err("character_mismatch");
        }
        Ok(owner)
    }

    pub fn outstanding(&self) -> usize {
        self.issued.len()
    }
}

/// A strict unsigned decimal: digits only, no sign, spaces or overflow.
fn decimal(text: &str) -> Option<u64> {
    if text.is_empty() || text.len() > 20 || !text.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    text.parse().ok()
}

/// The login stub: answer one client body. A 414 is sent to `host:port` (this service); a 411
/// is accepted when `account_for` maps the login (its launch ticket and character id) to an
/// account. Anything else is not handled yet (`None`). Replies are server frames.
pub fn reply(
    body: &[u8],
    host: &str,
    port: u16,
    account_for: impl FnOnce(&SessionLogin) -> Option<i32>,
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
            let body = match account_for(&login) {
                Some(account) => login_results(true, 0, account, &login.session_id),
                // (R) The failure code's text is not recovered; 1 is "refused".
                None => login_results(false, 1, 0, ""),
            };
            Ok(Some(server_frame(&body)))
        }
        _ => Ok(None),
    }
}

fn invalid(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}
