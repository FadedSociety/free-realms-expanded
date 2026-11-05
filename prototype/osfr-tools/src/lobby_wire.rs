//! Family 101: the game lobby's packets, as the client reads them (`HandleLobbyPacket` `00923210`,
//! header `008d90b0`: `i16 101, i8 sub`). No capture holds this family, so every layout here rests
//! on the client's readers; the fields they do not name keep their offsets as names.
use std::io;

const FAMILY: [u8; 2] = 101_u16.to_le_bytes();
const JOIN: u8 = 1;
const LEAVE: u8 = 2;
const START: u8 = 3;
const UPDATE: u8 = 4;
const CREATE: u8 = 5;
const SHOW: u8 = 6;
const CLOSE: u8 = 7;
const REMOVE: u8 = 8;
const LEAVE_LOBBY: u8 = 9;
const ERROR: u8 = 11;
const SHOW_UI: u8 = 12;

/// `LobbyErrorMessage` codes (`00bb0c30`); any other reads "General Lobby Error Occurred".
pub const ERROR_START: u32 = 4;
pub const ERROR_JOIN: u32 = 5;
pub const ERROR_CREATE: u32 = 9;

/// What a client asks of the lobby.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Request {
    Join { game: u32, lobby_type: u32 },
    Leave { game: u32, lobby_type: u32 },
    Start { game: u32, lobby_type: u32 },
    /// The writer's bytes (`00bb1820`), kept whole until a trace of it reads them.
    Create(Vec<u8>),
    LeaveLobby { lobby_type: u32 },
}

/// The client's name record (`008e7290`) behind the Host column.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct HostName {
    pub unknown: [u32; 3],
    pub first: String,
    pub last: String,
}

/// One row of the game list (`00911130`).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LobbyGame {
    pub id: u32,
    pub unknown_3c: u32,
    pub unknown_60: u32,
    /// The game's chosen options, two u32 each (`008cdce0`).
    pub options: Vec<(u32, u32)>,
    pub host_name: HostName,
    pub flag_46: bool,
    pub unknown_48: u32,
    /// (R) The host: sub 4 compares it with the player's own guid.
    pub host: u64,
    pub tail: [u32; 3],
    pub flag_108: bool,
}

/// `None` when `inner` is not this family; a refusal when it is, and its body is wrong.
pub fn decode_request(inner: &[u8]) -> io::Result<Option<Request>> {
    let Some((&[low, high, sub], body)) = inner.split_first_chunk::<3>() else {
        return Ok(None);
    };
    if [low, high] != FAMILY {
        return Ok(None);
    }
    if sub == CREATE {
        return Ok(Some(Request::Create(body.to_vec())));
    }
    let mut read = Reader(body);
    let request = match sub {
        JOIN | LEAVE | START => {
            let (game, lobby_type) = (read.u32()?, read.u32()?);
            match sub {
                JOIN => Request::Join { game, lobby_type },
                LEAVE => Request::Leave { game, lobby_type },
                _ => Request::Start { game, lobby_type },
            }
        }
        LEAVE_LOBBY => Request::LeaveLobby { lobby_type: read.u32()? },
        _ => return Err(invalid("not a lobby request")),
    };
    read.end()?;
    Ok(Some(request))
}

fn header(sub: u8) -> Vec<u8> {
    let mut packet = FAMILY.to_vec();
    packet.push(sub);
    packet
}

fn pair(sub: u8, first: u32, second: u32) -> Vec<u8> {
    let mut packet = header(sub);
    packet.extend(first.to_le_bytes());
    packet.extend(second.to_le_bytes());
    packet
}

fn one(sub: u8, value: u32) -> Vec<u8> {
    let mut packet = header(sub);
    packet.extend(value.to_le_bytes());
    packet
}

fn blob(sub: u8, body: &[u8]) -> io::Result<Vec<u8>> {
    let mut packet = header(sub);
    packet.extend(length(body.len())?.to_le_bytes());
    packet.extend(body);
    Ok(packet)
}

fn length(len: usize) -> io::Result<u32> {
    u32::try_from(len).map_err(|_| invalid("lobby field too long"))
}

/// Sub 1: the player is in `game` (`00baff30`).
pub fn joined(game: u32, lobby_type: u32) -> Vec<u8> {
    pair(JOIN, game, lobby_type)
}

/// Sub 2: the player is out of their game.
pub fn left(game: u32, lobby_type: u32) -> Vec<u8> {
    pair(LEAVE, game, lobby_type)
}

/// Sub 3: `Lobby:GameIsStarting`, the client's own 5-second countdown.
pub fn starting(game: u32, lobby_type: u32) -> Vec<u8> {
    pair(START, game, lobby_type)
}

/// Sub 4: add or replace one row.
pub fn update(game: &LobbyGame) -> io::Result<Vec<u8>> {
    blob(UPDATE, &encode_game(game)?)
}

/// Sub 6: open the lobby of `lobby_type` with its list (`00921740`), each row under its id.
pub fn show(games: &[LobbyGame], lobby_type: i32) -> io::Result<Vec<u8>> {
    let mut body = length(games.len())?.to_le_bytes().to_vec();
    for game in games {
        body.extend(game.id.to_le_bytes());
        body.extend(encode_game(game)?);
    }
    body.extend(lobby_type.to_le_bytes());
    blob(SHOW, &body)
}

/// Sub 7: close the lobby.
pub fn close() -> Vec<u8> {
    header(CLOSE)
}

/// Sub 8: drop one row.
pub fn removed(game: u32) -> Vec<u8> {
    one(REMOVE, game)
}

/// Sub 11: one of the `ERROR_` codes.
pub fn error(code: u32) -> Vec<u8> {
    one(ERROR, code)
}

/// Sub 12: `Lobby:Show`.
pub fn show_ui(first: u32, second: u32) -> Vec<u8> {
    pair(SHOW_UI, first, second)
}

fn encode_game(game: &LobbyGame) -> io::Result<Vec<u8>> {
    let mut out = Vec::new();
    for value in [game.id, game.unknown_3c, game.unknown_60, length(game.options.len())?] {
        out.extend(value.to_le_bytes());
    }
    for (key, value) in &game.options {
        out.extend(key.to_le_bytes());
        out.extend(value.to_le_bytes());
    }
    for value in game.host_name.unknown {
        out.extend(value.to_le_bytes());
    }
    for text in [&game.host_name.first, &game.host_name.last] {
        out.extend(length(text.len())?.to_le_bytes());
        out.extend(text.as_bytes());
    }
    out.push(u8::from(game.flag_46));
    out.extend(game.unknown_48.to_le_bytes());
    out.extend(game.host.to_le_bytes());
    for value in game.tail {
        out.extend(value.to_le_bytes());
    }
    out.push(u8::from(game.flag_108));
    Ok(out)
}

/// One row as the client reads it, for the two-player test client.
pub fn decode_game(bytes: &[u8]) -> io::Result<LobbyGame> {
    let mut read = Reader(bytes);
    let game = read.game()?;
    read.end()?;
    Ok(game)
}

struct Reader<'a>(&'a [u8]);

impl Reader<'_> {
    fn bytes(&mut self, count: usize) -> io::Result<&[u8]> {
        let (taken, rest) = self.0.split_at_checked(count).ok_or_else(|| invalid("short lobby packet"))?;
        self.0 = rest;
        Ok(taken)
    }

    fn u32(&mut self) -> io::Result<u32> {
        let bytes = self.bytes(4)?;
        Ok(u32::from_le_bytes(bytes.try_into().map_err(invalid)?))
    }

    fn u64(&mut self) -> io::Result<u64> {
        let bytes = self.bytes(8)?;
        Ok(u64::from_le_bytes(bytes.try_into().map_err(invalid)?))
    }

    fn flag(&mut self) -> io::Result<bool> {
        Ok(self.bytes(1)?[0] != 0)
    }

    fn text(&mut self) -> io::Result<String> {
        let len = usize::try_from(self.u32()?).map_err(invalid)?;
        String::from_utf8(self.bytes(len)?.to_vec()).map_err(invalid)
    }

    fn game(&mut self) -> io::Result<LobbyGame> {
        let (id, unknown_3c, unknown_60) = (self.u32()?, self.u32()?, self.u32()?);
        let count = self.u32()?;
        let mut options = Vec::new();
        for _ in 0..count {
            options.push((self.u32()?, self.u32()?));
        }
        let unknown = [self.u32()?, self.u32()?, self.u32()?];
        let host_name = HostName { unknown, first: self.text()?, last: self.text()? };
        Ok(LobbyGame {
            id,
            unknown_3c,
            unknown_60,
            options,
            host_name,
            flag_46: self.flag()?,
            unknown_48: self.u32()?,
            host: self.u64()?,
            tail: [self.u32()?, self.u32()?, self.u32()?],
            flag_108: self.flag()?,
        })
    }

    fn end(&self) -> io::Result<()> {
        if self.0.is_empty() {
            Ok(())
        } else {
            Err(invalid("trailing lobby bytes"))
        }
    }
}

fn invalid(error: impl Into<Box<dyn std::error::Error + Send + Sync>>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, error)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn game() -> LobbyGame {
        LobbyGame {
            id: 7,
            unknown_3c: 1,
            unknown_60: 2,
            options: vec![(3, 4), (5, 6)],
            host_name: HostName {
                unknown: [8, 9, 10],
                first: "Pat".into(),
                last: "Lee".into(),
            },
            flag_46: true,
            unknown_48: 11,
            host: 0x0102_0304_0506_0708,
            tail: [12, 13, 14],
            flag_108: true,
        }
    }

    #[test]
    fn a_row_reads_back_as_written() {
        assert_eq!(decode_game(&encode_game(&game()).unwrap()).unwrap(), game());
    }

    #[test]
    fn a_row_keeps_the_readers_field_order() {
        let bytes = encode_game(&game()).unwrap();
        assert_eq!(&bytes[..4], &7_u32.to_le_bytes());
        // Three u32, the option count, two options, three name u32, then the first text's length.
        assert_eq!(&bytes[12..16], &2_u32.to_le_bytes());
        assert_eq!(&bytes[44..48], &3_u32.to_le_bytes());
        assert_eq!(&bytes[48..51], b"Pat");
    }

    #[test]
    fn the_list_is_a_counted_blob_of_keyed_rows_then_the_type() {
        let packet = show(&[game()], 4).unwrap();
        assert_eq!(&packet[..3], &[101, 0, 6]);
        let body = &packet[7..];
        assert_eq!(u32::from_le_bytes(packet[3..7].try_into().unwrap()) as usize, body.len());
        assert_eq!(&body[..8], &[1, 0, 0, 0, 7, 0, 0, 0]);
        assert_eq!(&body[body.len() - 4..], &4_i32.to_le_bytes());
        assert_eq!(decode_game(&body[8..body.len() - 4]).unwrap(), game());
    }

    #[test]
    fn requests_decode_and_bad_ones_are_refused() {
        let join = joined(7, 4);
        assert_eq!(
            decode_request(&join).unwrap(),
            Some(Request::Join { game: 7, lobby_type: 4 })
        );
        assert_eq!(
            decode_request(&[101, 0, 9, 4, 0, 0, 0]).unwrap(),
            Some(Request::LeaveLobby { lobby_type: 4 })
        );
        assert_eq!(
            decode_request(&[101, 0, 5, 1, 2]).unwrap(),
            Some(Request::Create(vec![1, 2]))
        );
        assert_eq!(decode_request(&[102, 0, 1, 0]).unwrap(), None);
        assert!(decode_request(&join[..7]).is_err(), "short");
        assert!(decode_request(&[&join[..], &[0]].concat()).is_err(), "trailing");
        assert!(decode_request(&[101, 0, 4]).is_err(), "a server packet");
    }
}
