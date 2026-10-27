//! T770: the Card Duelist card service, a TCP listener beside the UDP gateway. The card window
//! (`tcg/FreeRealmsTCG.dll`) asks 414 for its connection server (answered with this listener),
//! then logs in with 411; the login's launch ticket and character id are redeemed once in the
//! session that minted the ticket (T764/T765). Commands after login are logged, not answered
//! yet. Off unless the card content's launch block names a port.
use super::*;
use osfr_tools::card_service;
use std::io::{ErrorKind, Read, Write};
use std::net::{IpAddr, TcpStream};

/// (R) The gateway's own player ceilings: a stranger cannot hold more sockets than players.
const MAX_CONNECTIONS: usize = 16;
const MAX_PER_ADDRESS: usize = 4;
/// (R) A connection that has not logged in this long after opening is closed.
const LOGIN_DEADLINE_MS: u64 = 30_000;
/// (R) Unhandled commands logged per connection, so a chatty client cannot fill the log.
const LOGGED_UNHANDLED: u32 = 16;

pub(super) struct CardListener {
    listener: TcpListener,
    host: String,
    port: u16,
    connections: Vec<Connection>,
}

struct Connection {
    stream: TcpStream,
    peer: IpAddr,
    inbox: Vec<u8>,
    opened_ms: u64,
    character: Option<u64>,
    unhandled: u32,
}

/// The listener the card content asks for, on the gateway's own address (loopback, or the
/// private address `--allow-lan` admits). A port that cannot be bound is logged, not fatal.
pub(super) fn open(options: &Options) -> io::Result<Option<CardListener>> {
    let Some(dir) = options.content.as_deref() else {
        return Ok(None);
    };
    let dir = osfr_tools::content::npc_content(dir);
    let port = osfr_tools::card_game::Content::load_optional(&dir)?
        .and_then(|cards| cards.launch().map(|launch| launch.port))
        .unwrap_or(0);
    if port == 0 {
        return Ok(None);
    }
    Ok(match CardListener::bind(options.bind.ip(), port) {
        Ok(card) => {
            println!("{}", json!({"event":"card_listening","port":card.port}));
            Some(card)
        }
        Err(error) => {
            println!(
                "{}",
                json!({"event":"card_unavailable","reason":error.to_string()})
            );
            None
        }
    })
}

impl CardListener {
    pub(super) fn bind(ip: IpAddr, port: u16) -> io::Result<Self> {
        let listener = TcpListener::bind((ip, port))?;
        listener.set_nonblocking(true)?;
        let local = listener.local_addr()?;
        Ok(Self {
            listener,
            host: local.ip().to_string(),
            port: local.port(),
            connections: Vec::new(),
        })
    }

    /// Accept and answer what has arrived. `redeem(ticket, character_id)` names the character a
    /// login's launch ticket was minted for, once.
    fn poll(&mut self, now_ms: u64, mut redeem: impl FnMut(&str, &str) -> Option<u64>) {
        for _ in 0..4 {
            let Ok((stream, peer)) = self.listener.accept() else {
                break;
            };
            let from_peer = self
                .connections
                .iter()
                .filter(|c| c.peer == peer.ip())
                .count();
            // Dropping a refused stream closes it.
            if self.connections.len() < MAX_CONNECTIONS
                && from_peer < MAX_PER_ADDRESS
                && stream.set_nonblocking(true).is_ok()
            {
                self.connections.push(Connection {
                    stream,
                    peer: peer.ip(),
                    inbox: Vec::new(),
                    opened_ms: now_ms,
                    character: None,
                    unhandled: 0,
                });
            }
        }
        let (host, port) = (self.host.clone(), self.port);
        self.connections
            .retain_mut(|c| c.serve(&host, port, now_ms, &mut redeem));
    }
}

impl Connection {
    /// Read what arrived and answer each whole frame; `false` closes the connection.
    fn serve(
        &mut self,
        host: &str,
        port: u16,
        now_ms: u64,
        redeem: &mut impl FnMut(&str, &str) -> Option<u64>,
    ) -> bool {
        if self.character.is_none() && now_ms.saturating_sub(self.opened_ms) > LOGIN_DEADLINE_MS {
            return false;
        }
        // At most 64 KiB per poll, so the inbox stays near one frame (`client_frame` refuses
        // longer ones).
        let mut chunk = [0; 4096];
        for _ in 0..16 {
            match self.stream.read(&mut chunk) {
                Ok(0) => return false,
                Ok(n) => self.inbox.extend_from_slice(&chunk[..n]),
                Err(error) if error.kind() == ErrorKind::WouldBlock => break,
                Err(_) => return false,
            }
        }
        loop {
            let body = match card_service::client_frame(&self.inbox) {
                Ok(Some((used, frame))) => {
                    let body = frame.body.to_vec();
                    self.inbox.drain(..used);
                    body
                }
                Ok(None) => return true,
                Err(_) => return false,
            };
            let mut login = None;
            let reply = card_service::reply(&body, host, port, |l| {
                login = redeem(&l.ticket, &l.character_id);
                // (R) The account id the 81 carries: the character's 31-bit slot.
                login.map(|guid| (guid & 0x7fff_ffff) as i32)
            });
            match reply {
                // ponytail: replies are a few dozen bytes, so a full send buffer means a dead
                // peer; a write queue comes with the lobby's larger messages.
                Ok(Some(frame)) => {
                    if self.stream.write_all(&frame).is_err() {
                        return false;
                    }
                    self.character = self.character.or(login);
                }
                Ok(None) => {
                    self.unhandled += 1;
                    if self.unhandled <= LOGGED_UNHANDLED {
                        let class = card_service::class_id(&body).ok();
                        println!(
                            "{}",
                            json!({"event":"card_command_unhandled","class":class,"character":self.character})
                        );
                    }
                }
                Err(_) => return false,
            }
        }
    }
}

impl Server {
    /// T770: serve the card service once per step.
    pub(super) fn poll_card(&mut self, now_ms: u64) {
        let Some(mut card) = self.card.take() else {
            return;
        };
        card.poll(now_ms, |ticket, character| {
            self.redeem_card_ticket(ticket, character, now_ms)
        });
        self.card = Some(card);
    }

    /// The live session of the character a card login names redeems its launch ticket. No
    /// session, a logged-out character or a ticket that session never minted refuses.
    fn redeem_card_ticket(&mut self, ticket: &str, character: &str, now_ms: u64) -> Option<u64> {
        let guid: u64 = character.parse().ok()?;
        let id = self.world.active_sessions.get(&guid)?;
        let key = self.world.session_keys.get(id)?;
        self.sessions
            .get_mut(key)?
            .gateway
            .gameplay_mut()?
            .redeem_card_ticket(ticket, character, now_ms)
    }
}

#[cfg(test)]
pub(super) mod tests {
    use super::super::tests::test_support;
    use super::*;
    use std::time::Duration;

    fn int(value: i32) -> Vec<u8> {
        let mut out = Vec::new();
        card_service::put_int(&mut out, value);
        out
    }

    fn text(value: &str) -> Vec<u8> {
        let mut out = Vec::new();
        card_service::put_string(&mut out, value);
        out
    }

    /// A 414 body as the DLL writes it (`10121d20`): session id, user name, empty host, port 0,
    /// false, error 87.
    fn gateway_request() -> Vec<u8> {
        let mut body = Vec::new();
        card_service::put_headers(&mut body, 414, 1);
        body.extend(text("session"));
        body.extend(text("player"));
        body.extend(text(""));
        for value in [0, 0, 87] {
            body.extend(int(value));
        }
        body
    }

    fn frame(routed: bool, body: &[u8]) -> Vec<u8> {
        let mut payload = Vec::new();
        if routed {
            for word in [1_000_000_000u32, 0, 0] {
                payload.extend(word.to_be_bytes());
            }
        }
        payload.extend(body);
        let mut out = (payload.len() as u32).to_be_bytes().to_vec();
        out.extend(payload);
        out
    }

    fn listener() -> CardListener {
        CardListener::bind("127.0.0.1".parse().unwrap(), 0).unwrap()
    }

    fn client(card: &CardListener) -> TcpStream {
        let stream = TcpStream::connect(("127.0.0.1", card.port)).unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        stream
    }

    /// Poll until the client has a whole server frame, then return its body.
    fn answer(card: &mut CardListener, client: &mut TcpStream, now: u64) -> Vec<u8> {
        for _ in 0..200 {
            card.poll(now, |_, _| None);
            client.set_nonblocking(true).unwrap();
            let mut length = [0; 4];
            let ready = client.peek(&mut length).map(|n| n == 4).unwrap_or(false);
            client.set_nonblocking(false).unwrap();
            if ready {
                client.read_exact(&mut length).unwrap();
                let mut body = vec![0; u32::from_be_bytes(length) as usize];
                client.read_exact(&mut body).unwrap();
                return body;
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        panic!("no answer from the card service");
    }

    #[test]
    fn the_gateway_request_is_answered_with_this_listener() {
        if !test_support::bin_test_target("card") {
            return;
        }
        let mut card = listener();
        let mut client = client(&card);
        client.write_all(&frame(false, &gateway_request())).unwrap();
        let reply =
            card_service::ConnectionServer::decode(&answer(&mut card, &mut client, 1)).unwrap();
        assert!(reply.ok);
        assert_eq!(
            (reply.host.as_str(), reply.port),
            ("127.0.0.1", i32::from(card.port))
        );
        assert_eq!(
            reply.session_id, "session",
            "the request's own fields come back"
        );
    }

    /// The client's 411 at version 8 in the writer's field order (`103c9140` -> `1017b700`), as
    /// tests/card_service.rs builds it.
    fn session_login(character_id: &str, ticket: &str) -> Vec<u8> {
        let mut body = Vec::new();
        card_service::put_headers(&mut body, 411, 8);
        body.extend(text("w-session"));
        body.extend([0x03, 0x01, 0x22, 0x52, 0x02, 0x07, 0x04, 0x09]);
        for field in ["winXP", ""] {
            body.extend(text(field));
        }
        body.push(0x01);
        for field in ["FR", "en_US", "Tester", character_id, ticket, "STATION"] {
            body.extend(text(field));
        }
        body
    }

    /// Send one login and return the 81's success flag and account id, plus what redeem saw.
    fn log_in(
        card: &mut CardListener,
        client: &mut TcpStream,
        character_id: &str,
        ticket: &str,
        accept: Option<u64>,
    ) -> (bool, i32, Vec<(String, String)>) {
        client
            .write_all(&frame(true, &session_login(character_id, ticket)))
            .unwrap();
        let mut seen = Vec::new();
        let mut body = None;
        for _ in 0..200 {
            card.poll(2, |t, c| {
                seen.push((t.to_owned(), c.to_owned()));
                accept
            });
            if !seen.is_empty() {
                body = Some(answer(card, client, 2));
                break;
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        let body = body.expect("the login was read");
        let mut results = card_service::Reader::new(&body);
        results.headers(81).unwrap();
        let success = results.int().unwrap() != 0;
        let _code = results.int().unwrap();
        let account = results.int().unwrap();
        (success, account, seen)
    }

    #[test]
    fn a_login_is_accepted_only_for_the_ticket_redeem_names() {
        if !test_support::bin_test_target("card") {
            return;
        }
        let mut card = listener();
        let mut client = client(&card);
        let (ok, account, seen) = log_in(&mut card, &mut client, "5041", "77", None);
        assert!(!ok, "a ticket no session minted is refused");
        assert_eq!(account, 0);
        assert_eq!(seen, [("77".to_owned(), "5041".to_owned())]);
        assert_eq!(card.connections[0].character, None);

        let guid = (1u64 << 40) | 5041;
        let (ok, account, _) = log_in(&mut card, &mut client, &guid.to_string(), "78", Some(guid));
        assert!(ok);
        assert_eq!(
            account, 5041,
            "the account id is the character's low 31 bits"
        );
        assert_eq!(card.connections[0].character, Some(guid));
    }

    #[test]
    fn a_silent_connection_is_closed_after_the_login_deadline() {
        if !test_support::bin_test_target("card") {
            return;
        }
        let mut card = listener();
        let _client = client(&card);
        for _ in 0..200 {
            card.poll(1, |_, _| None);
            if !card.connections.is_empty() {
                break;
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        assert_eq!(card.connections.len(), 1);
        card.poll(1 + LOGIN_DEADLINE_MS, |_, _| None);
        assert_eq!(card.connections.len(), 1, "the deadline is inclusive");
        card.poll(2 + LOGIN_DEADLINE_MS, |_, _| None);
        assert!(card.connections.is_empty());
    }

    #[test]
    fn one_address_holds_at_most_four_connections() {
        if !test_support::bin_test_target("card") {
            return;
        }
        let mut card = listener();
        let clients: Vec<_> = (0..6).map(|_| client(&card)).collect();
        for _ in 0..50 {
            card.poll(1, |_, _| None);
            std::thread::sleep(Duration::from_millis(5));
        }
        assert_eq!(card.connections.len(), MAX_PER_ADDRESS);
        drop(clients);
    }

    #[test]
    fn an_oversized_frame_closes_the_connection() {
        if !test_support::bin_test_target("card") {
            return;
        }
        let mut card = listener();
        let mut client = client(&card);
        client
            .write_all(&((card_service::MAX_FRAME as u32 + 1).to_be_bytes()))
            .unwrap();
        for _ in 0..200 {
            card.poll(1, |_, _| None);
            std::thread::sleep(Duration::from_millis(5));
            if card.connections.is_empty() {
                return;
            }
        }
        panic!("the oversized frame left the connection open");
    }

    #[test]
    fn a_login_for_a_character_with_no_live_session_is_refused() {
        if !test_support::bin_test_target("card") {
            return;
        }
        let scratch = test_support::Scratch::new("t770-card-login");
        let options = test_support::options(scratch.state());
        if !test_support::fixtures_present(&options) {
            return;
        }
        let mut server = Server::new(options).unwrap();
        assert_eq!(server.redeem_card_ticket("77", "5041", 1), None);
        assert_eq!(server.redeem_card_ticket("77", "not-a-guid", 1), None);
        assert!(server.card.is_none(), "no launch block, no card service");
    }
}
