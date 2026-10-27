//! T770: the Card Duelist card service, a TCP listener beside the UDP gateway. The card window
//! (`tcg/FreeRealmsTCG.dll`) asks 414 for its connection server (answered with this listener),
//! then logs in with 411; the login's launch ticket and character id are redeemed once in the
//! session that minted the ticket (T764/T765). Commands after login are logged, not answered
//! yet. Off unless the card content's launch block names a port.
//!
//! T782: every step prints one JSON event line (`events`, drained by `Server::poll_card`):
//! `card_connected`, `card_gateway_request` (a 414 answered), `card_login` (a 411: `ok`, and the
//! character on success), `card_command_unhandled` and `card_closed` with its reason. A ticket or
//! session id is never in an event.
use super::*;
use osfr_tools::card_service;
use serde_json::Value;
use std::io::{ErrorKind, Read, Write};
use std::net::{IpAddr, SocketAddr, TcpStream};

/// (R) The gateway's own player ceilings: a stranger cannot hold more sockets than players.
const MAX_CONNECTIONS: usize = 16;
const MAX_PER_ADDRESS: usize = 4;
/// (R) A connection that has not logged in this long after opening is closed.
const LOGIN_DEADLINE_MS: u64 = 30_000;
/// (R) Unhandled commands, and answered 414/411 requests, logged per connection (each count
/// apart), so a chatty client cannot fill the log.
const LOGGED_PER_KIND: u32 = 16;

pub(super) struct CardListener {
    listener: TcpListener,
    host: String,
    port: u16,
    connections: Vec<Connection>,
    /// The event lines of the polls since the last drain.
    events: Vec<Value>,
}

struct Connection {
    stream: TcpStream,
    peer: SocketAddr,
    inbox: Vec<u8>,
    opened_ms: u64,
    character: Option<u64>,
    unhandled: u32,
    answered: u32,
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
            events: Vec::new(),
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
                .filter(|c| c.peer.ip() == peer.ip())
                .count();
            // Dropping a refused stream closes it.
            if self.connections.len() < MAX_CONNECTIONS
                && from_peer < MAX_PER_ADDRESS
                && stream.set_nonblocking(true).is_ok()
            {
                self.connections.push(Connection {
                    stream,
                    peer,
                    inbox: Vec::new(),
                    opened_ms: now_ms,
                    character: None,
                    unhandled: 0,
                    answered: 0,
                });
                self.events
                    .push(json!({"event":"card_connected","peer":peer.to_string()}));
            }
        }
        let (host, port) = (self.host.clone(), self.port);
        let events = &mut self.events;
        self.connections.retain_mut(|c| {
            let Some(reason) = c.serve(&host, port, now_ms, &mut redeem, events) else {
                return true;
            };
            events.push(json!({
                "event":"card_closed","peer":c.peer.to_string(),"reason":reason,"character":c.character
            }));
            false
        });
    }
}

impl Connection {
    /// Read what arrived and answer each whole frame; `Some(reason)` closes the connection.
    fn serve(
        &mut self,
        host: &str,
        port: u16,
        now_ms: u64,
        redeem: &mut impl FnMut(&str, &str) -> Option<u64>,
        events: &mut Vec<Value>,
    ) -> Option<&'static str> {
        if self.character.is_none() && now_ms.saturating_sub(self.opened_ms) > LOGIN_DEADLINE_MS {
            return Some("deadline");
        }
        // At most 64 KiB per poll, so the inbox stays near one frame (`client_frame` refuses
        // longer ones).
        let mut chunk = [0; 4096];
        for _ in 0..16 {
            match self.stream.read(&mut chunk) {
                Ok(0) => return Some("eof"),
                Ok(n) => self.inbox.extend_from_slice(&chunk[..n]),
                Err(error) if error.kind() == ErrorKind::WouldBlock => break,
                Err(_) => return Some("read_failed"),
            }
        }
        loop {
            let body = match card_service::client_frame(&self.inbox) {
                Ok(Some((used, frame))) => {
                    let body = frame.body.to_vec();
                    self.inbox.drain(..used);
                    body
                }
                Ok(None) => return None,
                Err(_) => return Some("bad_frame"),
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
                    self.log_answer(&body, login, host, port, events);
                    if self.stream.write_all(&frame).is_err() {
                        return Some("write_failed");
                    }
                    self.character = self.character.or(login);
                }
                Ok(None) => {
                    self.unhandled += 1;
                    if self.unhandled <= LOGGED_PER_KIND {
                        let class = card_service::class_id(&body).ok();
                        events.push(json!({
                            "event":"card_command_unhandled","class":class,"character":self.character
                        }));
                    }
                }
                Err(_) => return Some("bad_message"),
            }
        }
    }

    /// The event for an answered 414 (the address it sent the client to) or 411 (`login`: the
    /// character its ticket was redeemed for). Neither the ticket nor the session id is named.
    fn log_answer(
        &mut self,
        body: &[u8],
        login: Option<u64>,
        host: &str,
        port: u16,
        events: &mut Vec<Value>,
    ) {
        let peer = self.peer.to_string();
        let event = match card_service::class_id(body) {
            Ok(card_service::GET_CONNECTION_SERVER) => {
                json!({"event":"card_gateway_request","peer":peer,"host":host,"port":port})
            }
            Ok(card_service::SEND_SESSION_ID) => {
                json!({"event":"card_login","peer":peer,"ok":login.is_some(),"character":login})
            }
            _ => return,
        };
        self.answered += 1;
        if self.answered <= LOGGED_PER_KIND {
            events.push(event);
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
        for event in card.events.drain(..) {
            println!("{event}");
        }
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

    /// The reasons of the `card_closed` events so far.
    fn closed(card: &CardListener) -> Vec<String> {
        let events = card.events.iter();
        let closed = events.filter(|e| e["event"] == "card_closed");
        closed
            .map(|e| e["reason"].as_str().unwrap().to_owned())
            .collect()
    }

    /// Poll until an event named `name` is logged.
    fn wait_for(card: &mut CardListener, name: &str) {
        for _ in 0..200 {
            card.poll(1, |_, _| None);
            if card.events.iter().any(|e| e["event"] == name) {
                return;
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        panic!("no {name} event");
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
        assert_eq!(closed(&card), ["deadline"]);
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
                assert_eq!(closed(&card), ["bad_frame"]);
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

    #[test]
    fn card_events_follow_a_connection_from_connect_to_close() {
        if !test_support::bin_test_target("card") {
            return;
        }
        let mut card = listener();
        let mut client = client(&card);
        let peer = client.local_addr().unwrap().to_string();
        client.write_all(&frame(false, &gateway_request())).unwrap();
        answer(&mut card, &mut client, 1);
        // A half-close is the client's orderly end; the listener sees eof.
        client.shutdown(std::net::Shutdown::Write).unwrap();
        wait_for(&mut card, "card_closed");
        assert_eq!(
            card.events,
            [
                json!({"event":"card_connected","peer":peer}),
                json!({"event":"card_gateway_request","peer":peer,"host":"127.0.0.1","port":card.port}),
                json!({"event":"card_closed","peer":peer,"reason":"eof","character":null}),
            ]
        );
    }

    #[test]
    fn card_login_events_name_the_character_never_the_ticket() {
        if !test_support::bin_test_target("card") {
            return;
        }
        let mut card = listener();
        let mut client = client(&card);
        let guid = (1u64 << 40) | 5041;
        log_in(&mut card, &mut client, "5041", "271828182845", None);
        log_in(
            &mut card,
            &mut client,
            &guid.to_string(),
            "314159265358",
            Some(guid),
        );
        let logins: Vec<_> = card
            .events
            .iter()
            .filter(|e| e["event"] == "card_login")
            .collect();
        assert_eq!(logins.len(), 2);
        assert_eq!(
            (&logins[0]["ok"], &logins[0]["character"]),
            (&json!(false), &Value::Null)
        );
        assert_eq!(
            (&logins[1]["ok"], &logins[1]["character"]),
            (&json!(true), &json!(guid))
        );
        // Neither ticket nor the session id the login carries is logged.
        let text = serde_json::to_string(&card.events).unwrap();
        for secret in ["271828182845", "314159265358", "w-session"] {
            assert!(!text.contains(secret), "{secret} in {text}");
        }
    }

    #[test]
    fn card_close_reasons_name_a_bad_message_and_logged_events_are_capped() {
        if !test_support::bin_test_target("card") {
            return;
        }
        // A routed frame with no body has no class id.
        let mut card = listener();
        let mut silent = client(&card);
        silent.write_all(&frame(true, &[])).unwrap();
        wait_for(&mut card, "card_closed");
        assert_eq!(closed(&card), ["bad_message"]);

        // One connection repeating 414s and unknown commands logs 16 of each, then stays quiet.
        let mut card = listener();
        let mut chatty = client(&card);
        for _ in 0..(LOGGED_PER_KIND + 4) {
            chatty.write_all(&frame(false, &gateway_request())).unwrap();
            chatty.write_all(&frame(true, &int(1234))).unwrap();
        }
        for _ in 0..40 {
            card.poll(1, |_, _| None);
            std::thread::sleep(Duration::from_millis(5));
        }
        let count = |name: &str| card.events.iter().filter(|e| e["event"] == name).count();
        assert_eq!(count("card_gateway_request"), LOGGED_PER_KIND as usize);
        assert_eq!(count("card_command_unhandled"), LOGGED_PER_KIND as usize);
        assert_eq!(
            card.connections.len(),
            1,
            "the chatty client stays connected"
        );
    }
}
