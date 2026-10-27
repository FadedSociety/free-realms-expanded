//! T770: the Card Duelist card service, a TCP listener beside the UDP gateway. The card window
//! (`tcg/FreeRealmsTCG.dll`) asks 414 for its connection server (answered with this listener),
//! then logs in with 411; the login's launch ticket and character id are redeemed once in the
//! session that minted the ticket (T764/T765). T808: a refused login is closed once its 81 is
//! written; after login the client's ping (112) is echoed. T813: a login pushes the launch
//! block's lobby (94 -> 92 -> 291), which opens the lobby screen. T825: then the starter
//! collection sync (83, and per client Update 84, 85, 86); the client's status (137) is echoed and
//! its own Join requests (92) are logged, not answered. Other commands are logged, not answered
//! yet. Off unless the card content's launch block names a port.
//!
//! T782: every step prints one JSON event line (`events`, drained by `Server::poll_card`):
//! `card_connected`, `card_gateway_request` (a 414 answered), `card_login` (a 411: `ok`, the
//! character on success, else the refusal `reason`; T805: also the 411's `version` and field
//! `lengths`), `card_lobby` (T813: the lobby pushed, once per connection: ids only),
//! `card_join_request` (T825: the group and role a client Join asks for), `card_sync` (T825: the
//! collection synced, or already cached by the client), `card_ping`
//! (T808: a ping echoed), `card_command_unhandled` and `card_closed` with its reason. A ticket or
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
/// (R) Unhandled commands, and answered 414/411/ping requests, logged per connection (each count
/// apart), so a chatty client cannot fill the log.
const LOGGED_PER_KIND: u32 = 16;

pub(super) struct CardListener {
    listener: TcpListener,
    host: String,
    port: u16,
    connections: Vec<Connection>,
    /// The event lines of the polls since the last drain.
    events: Vec<Value>,
    /// T813: the lobby a login enters, from the launch block (none: the login stops there).
    lobby: Option<osfr_tools::card_game::Lobby>,
}

/// T825: where a connection's collection sync stands; each client Update advances it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SyncStep {
    Off,
    Started,
    InstancesStarted,
    InstancesSent,
    Done,
}

struct Connection {
    stream: TcpStream,
    peer: SocketAddr,
    inbox: Vec<u8>,
    opened_ms: u64,
    character: Option<u64>,
    sync: SyncStep,
    unhandled: u32,
    answered: Answered,
}

/// The answered-event counters, one budget each (T782 review): a client retrying 414 or pinging
/// cannot spend the single `card_login` line the login diagnosis needs. T825: the client's Join
/// requests have their own.
#[derive(Default)]
struct Answered {
    gateway_requests: u32,
    logins: u32,
    pings: u32,
    join_requests: u32,
}

impl Answered {
    fn count(&mut self, kind: AnsweredKind) -> &mut u32 {
        match kind {
            AnsweredKind::GatewayRequest => &mut self.gateway_requests,
            AnsweredKind::Login => &mut self.logins,
            AnsweredKind::Ping => &mut self.pings,
            AnsweredKind::JoinRequest => &mut self.join_requests,
        }
    }
}

#[derive(Clone, Copy)]
enum AnsweredKind {
    GatewayRequest,
    Login,
    Ping,
    JoinRequest,
}

/// The listener the card content asks for, on the gateway's own address (loopback, or the
/// private address `--allow-lan` admits). A port that cannot be bound is logged, not fatal.
pub(super) fn open(options: &Options) -> io::Result<Option<CardListener>> {
    let Some(dir) = options.content.as_deref() else {
        return Ok(None);
    };
    let dir = osfr_tools::content::npc_content(dir);
    let launch = osfr_tools::card_game::Content::load_optional(&dir)?.and_then(|cards| {
        cards
            .launch()
            .map(|launch| (launch.port, launch.lobby.clone()))
    });
    let Some((port @ 1.., lobby)) = launch else {
        return Ok(None);
    };
    Ok(match CardListener::bind(options.bind.ip(), port) {
        Ok(mut card) => {
            println!("{}", json!({"event":"card_listening","port":card.port}));
            card.lobby = lobby;
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
            lobby: None,
        })
    }

    /// Accept and answer what has arrived. `redeem(ticket, character_id)` names the character a
    /// login's launch ticket was minted for, once, else the refusal word.
    fn poll(
        &mut self,
        now_ms: u64,
        mut redeem: impl FnMut(&str, &str) -> Result<u64, &'static str>,
    ) {
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
                    sync: SyncStep::Off,
                    unhandled: 0,
                    answered: Answered::default(),
                });
                self.events
                    .push(json!({"event":"card_connected","peer":peer.to_string()}));
            }
        }
        let (host, port) = (self.host.clone(), self.port);
        let (events, lobby) = (&mut self.events, self.lobby.as_ref());
        self.connections.retain_mut(|c| {
            let Some(reason) = c.serve(&host, port, now_ms, lobby, &mut redeem, events) else {
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
        lobby: Option<&osfr_tools::card_game::Lobby>,
        redeem: &mut impl FnMut(&str, &str) -> Result<u64, &'static str>,
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
            match self.lobby_command(&body, lobby, events) {
                Some(Ok(reply)) if reply.is_empty() || self.stream.write_all(&reply).is_ok() => {
                    continue;
                }
                Some(Ok(_)) => return Some("write_failed"),
                Some(Err(_)) => return Some("bad_message"),
                None => {}
            }
            let (mut login, mut name) = (None, String::new());
            let logged_in = self.character.is_some();
            let reply = card_service::reply(&body, host, port, logged_in, |l| {
                let redeemed = redeem(&l.ticket, &l.character_id);
                // T805: the 411's shape for its event: lengths only, never a value.
                let lengths = json!({
                    "session_id":l.session_id.len(),"character_name":l.character_name.len(),
                    "character_id":l.character_id.len(),"ticket":l.ticket.len()
                });
                login = Some((redeemed, l.version, lengths));
                name = l.character_name.clone();
                redeemed.ok()
            });
            match reply {
                // ponytail: replies are a few dozen bytes, so a full send buffer means a dead
                // peer; a write queue comes with the lobby's larger messages.
                Ok(Some(frame)) => {
                    let redeemed = login.as_ref().map(|(redeemed, ..)| *redeemed);
                    self.log_answer(&body, login, host, port, events);
                    if self.stream.write_all(&frame).is_err() {
                        return Some("write_failed");
                    }
                    match redeemed {
                        // T808: the refused 81 is written; the client shows it, nothing follows.
                        Some(Err(_)) => return Some("login_refused"),
                        Some(Ok(character)) if self.character.is_none() => {
                            self.character = Some(character);
                            if let Some(lobby) = lobby {
                                if !self.enter_lobby(character, lobby, &name, events) {
                                    return Some("write_failed");
                                }
                            }
                        }
                        _ => {}
                    }
                }
                Ok(None) => {
                    self.unhandled = self.unhandled.saturating_add(1);
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

    /// T813: push the lobby after the first successful login (the 81 is already written): the
    /// three frames of `card_service::lobby_entry`, then one `card_lobby` event. False when the
    /// peer cannot take them.
    fn enter_lobby(
        &mut self,
        character: u64,
        lobby: &osfr_tools::card_game::Lobby,
        name: &str,
        events: &mut Vec<Value>,
    ) -> bool {
        let account = card_service::account_id(character);
        let mut bodies = card_service::lobby_entry(account, lobby, name).to_vec();
        // T825: the collection sync starts right after the lobby.
        if !lobby.collection.is_empty() {
            bodies.push(card_service::sync_start(account, &lobby.collection));
            self.sync = SyncStep::Started;
        }
        for body in bodies {
            if self
                .stream
                .write_all(&card_service::server_frame(&body))
                .is_err()
            {
                return false;
            }
        }
        events.push(json!({
            "event":"card_lobby","peer":self.peer.to_string(),"character":character,
            "account":account,"group":lobby.group_id
        }));
        true
    }

    /// T825: the client's lobby and collection commands once logged in: the server frames to
    /// write (none for a command answered by silence), or `None` for anything else.
    /// - 137 status: echoed for this connection's account (`card_service::change_status`).
    /// - 92 Join request: logged, not answered. The server knows one group, the root group 1,
    ///   whose membership it pushed at login; a server Join for it sends the client to its hub
    ///   screen 2 (event 5, `10371cd0`), away from the lobby it is in.
    /// - 503 avatar picture: answered by silence (`card_service::UPLOAD_AVATAR_IMAGE`).
    /// - 87 Update: the next sync step (StartInstances, SendInstances, Complete); 86 Complete
    ///   from the client: its cached collection matched the Start, nothing to send.
    fn lobby_command(
        &mut self,
        body: &[u8],
        lobby: Option<&osfr_tools::card_game::Lobby>,
        events: &mut Vec<Value>,
    ) -> Option<io::Result<Vec<u8>>> {
        let account = card_service::account_id(self.character?);
        let cards = lobby.map_or(&[][..], |lobby| &lobby.collection[..]);
        let reply = match card_service::class_id(body).ok()? {
            card_service::CHANGE_STATUS => card_service::change_status(account, body),
            card_service::JOIN => card_service::join_request(body).map(|_| {
                self.log_answer(body, None, "", 0, events);
                Vec::new()
            }),
            card_service::SYNC_UPDATE => {
                let (next, step) = match self.sync {
                    SyncStep::Started => (
                        card_service::sync_start_instances(account, cards),
                        SyncStep::InstancesStarted,
                    ),
                    SyncStep::InstancesStarted => (
                        card_service::sync_send_instances(cards),
                        SyncStep::InstancesSent,
                    ),
                    SyncStep::InstancesSent => (card_service::sync_complete(), SyncStep::Done),
                    SyncStep::Off | SyncStep::Done => return Some(Ok(Vec::new())),
                };
                if step == SyncStep::Done {
                    self.synced("sent", cards, events);
                }
                self.sync = step;
                Ok(next)
            }
            card_service::SYNC_COMPLETE if self.sync == SyncStep::Started => {
                self.sync = SyncStep::Done;
                self.synced("cached", cards, events);
                Ok(Vec::new())
            }
            // The client's own Complete after the sync, and its avatar picture: no reply.
            card_service::SYNC_COMPLETE | card_service::UPLOAD_AVATAR_IMAGE => Ok(Vec::new()),
            _ => return None,
        };
        Some(reply.map(|body| {
            if body.is_empty() {
                Vec::new()
            } else {
                card_service::server_frame(&body)
            }
        }))
    }

    /// One `card_sync` event per connection: the collection sent, or already cached.
    fn synced(
        &mut self,
        how: &str,
        cards: &[osfr_tools::card_game::StarterCard],
        events: &mut Vec<Value>,
    ) {
        events.push(json!({
            "event":"card_sync","peer":self.peer.to_string(),"character":self.character,
            "collection":how,"archetypes":cards.len(),
            "instances":cards.iter().map(|card| card.copies).sum::<i32>()
        }));
    }

    /// The event for an answered 414 (the address it sent the client to), 411 (`login`: the
    /// character its ticket was redeemed for, or the refusal word, plus the 411's version and
    /// field lengths), ping or (T825) client Join request (the group and role it asks for).
    /// Neither the ticket nor the session id is named. The kind and its budget are decided before
    /// any event is built, so a capped frame allocates nothing.
    fn log_answer(
        &mut self,
        body: &[u8],
        login: Option<(Result<u64, &'static str>, i32, Value)>,
        host: &str,
        port: u16,
        events: &mut Vec<Value>,
    ) {
        let kind = match (card_service::class_id(body), login.as_ref()) {
            (Ok(card_service::GET_CONNECTION_SERVER), _) => AnsweredKind::GatewayRequest,
            (Ok(card_service::SEND_SESSION_ID), Some(_)) => AnsweredKind::Login,
            (Ok(card_service::PING), _) => AnsweredKind::Ping,
            (Ok(card_service::JOIN), _) => AnsweredKind::JoinRequest,
            _ => return,
        };
        let answered = self.answered.count(kind);
        *answered = answered.saturating_add(1);
        if *answered > LOGGED_PER_KIND {
            return;
        }
        let peer = self.peer.to_string();
        let event = match kind {
            AnsweredKind::GatewayRequest => {
                json!({"event":"card_gateway_request","peer":peer,"host":host,"port":port})
            }
            AnsweredKind::Login => {
                let Some((redeemed, version, lengths)) = login else {
                    return;
                };
                json!({
                    "event":"card_login","peer":peer,"ok":redeemed.is_ok(),"character":redeemed.ok(),
                    "reason":redeemed.err(),"version":version,"lengths":lengths
                })
            }
            AnsweredKind::Ping => {
                json!({"event":"card_ping","peer":peer,"character":self.character})
            }
            AnsweredKind::JoinRequest => {
                let Ok((group, role)) = card_service::join_request(body) else {
                    return;
                };
                json!({"event":"card_join_request","peer":peer,"group":group,"role":role})
            }
        };
        events.push(event);
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

    /// The live session that minted a card login's launch ticket redeems it. T805: the retail
    /// 411 carries no character id, so the ticket alone finds its session; a ticket no live
    /// session holds is `unknown_ticket`.
    // ponytail: one scan of the live sessions per card login (one login per launch); a
    // server-wide ticket index if the player count ever makes it show.
    fn redeem_card_ticket(
        &mut self,
        ticket: &str,
        character: &str,
        now_ms: u64,
    ) -> Result<u64, &'static str> {
        for session in self.sessions.values_mut() {
            let Some(game) = session.gateway.gameplay_mut() else {
                continue;
            };
            match game.redeem_card_ticket(ticket, character, now_ms) {
                Err(card_service::UNKNOWN_TICKET) => {}
                redeemed => return redeemed,
            }
        }
        Err(card_service::UNKNOWN_TICKET)
    }
}

/// F785: the client's request bytes, shared with `tests/card_service.rs`.
#[cfg(test)]
#[path = "../../../tests/support/card_packets.rs"]
mod card_packets;

#[cfg(test)]
pub(super) mod tests {
    use super::super::tests::test_support;
    use super::card_packets::{ROUTING, frame, gateway_request, int, ping, session_login, text};
    use super::*;
    use std::time::Duration;

    fn listener() -> CardListener {
        CardListener::bind("127.0.0.1".parse().unwrap(), 0).unwrap()
    }

    /// A redeem that knows no ticket.
    fn refuse(_: &str, _: &str) -> Result<u64, &'static str> {
        Err(card_service::UNKNOWN_TICKET)
    }

    /// The reasons of the `card_closed` events so far.
    fn closed(card: &CardListener) -> Vec<String> {
        let events = card.events.iter();
        let closed = events.filter(|e| e["event"] == "card_closed");
        closed
            .map(|e| e["reason"].as_str().unwrap().to_owned())
            .collect()
    }

    /// How many events named `name` were logged so far.
    fn count(card: &CardListener, name: &str) -> usize {
        card.events.iter().filter(|e| e["event"] == name).count()
    }

    /// Poll until an event named `name` is logged.
    fn wait_for(card: &mut CardListener, name: &str) {
        for _ in 0..200 {
            card.poll(1, refuse);
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
            card.poll(now, refuse);
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
        client.write_all(&frame(None, &gateway_request())).unwrap();
        let reply =
            card_service::ConnectionServer::decode(&answer(&mut card, &mut client, 1)).unwrap();
        assert!(reply.ok);
        assert_eq!(
            (reply.host.as_str(), reply.port),
            ("127.0.0.1", i32::from(card.port))
        );
        assert_eq!(
            reply.session_id, "w-ticket",
            "the request's own fields come back"
        );
    }

    /// Send one login and return the 81's success flag, code and account id, plus what redeem
    /// saw.
    fn log_in(
        card: &mut CardListener,
        client: &mut TcpStream,
        character_id: &str,
        ticket: &str,
        accept: Result<u64, &'static str>,
    ) -> (bool, i32, i32, Vec<(String, String)>) {
        client
            .write_all(&frame(Some(ROUTING), &session_login(character_id, ticket)))
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
        let code = results.int().unwrap();
        let account = results.int().unwrap();
        (success, code, account, seen)
    }

    #[test]
    fn a_login_is_accepted_only_for_the_ticket_redeem_names() {
        if !test_support::bin_test_target("card") {
            return;
        }
        let mut card = listener();
        let mut refused = client(&card);
        let (ok, code, account, seen) =
            log_in(&mut card, &mut refused, "5041", "77", refuse("", ""));
        assert_eq!(
            (ok, code, account),
            (false, 63, 0),
            "refused with 63, \"Your session is not valid.\""
        );
        assert_eq!(seen, [("77".to_owned(), "5041".to_owned())]);
        // T808: the refused connection is closed once its 81 is written, not at the deadline.
        assert_eq!(closed(&card), ["login_refused"]);
        assert_eq!(
            refused.read(&mut [0; 1]).unwrap(),
            0,
            "the client sees the close"
        );

        // T808: a guid whose low 31 bits are 0 still gets a non-zero account id.
        let guid = 1u64 << 31;
        let mut accepted = client(&card);
        let (ok, code, account, _) =
            log_in(&mut card, &mut accepted, &guid.to_string(), "78", Ok(guid));
        assert_eq!((ok, code), (true, 0));
        assert_ne!(account, 0, "account id 0 crashes the client");
        assert_eq!(account, card_service::account_id(guid));
        assert_eq!(card.connections[0].character, Some(guid));
    }

    #[test]
    fn a_card_ping_is_echoed_once_logged_in() {
        if !test_support::bin_test_target("card") {
            return;
        }
        let mut card = listener();
        let mut stranger = client(&card);
        stranger.write_all(&frame(Some(ROUTING), &ping(7))).unwrap();
        wait_for(&mut card, "card_command_unhandled");
        let unhandled = card
            .events
            .iter()
            .find(|e| e["event"] == "card_command_unhandled");
        assert_eq!(unhandled.unwrap()["class"], 112, "no echo before login");

        let mut player = client(&card);
        log_in(&mut card, &mut player, "", "78", Ok(5041));
        player
            .write_all(&frame(Some(ROUTING), &ping(1234)))
            .unwrap();
        // Compact 112 = `20 07`, version `01`, spoof 1234 = `22 4D`.
        assert_eq!(
            answer(&mut card, &mut player, 3),
            [0x20, 0x07, 0x01, 0x22, 0x4D]
        );
        let pings = card.events.iter().filter(|e| e["event"] == "card_ping");
        assert_eq!(
            pings.map(|e| &e["character"]).collect::<Vec<_>>(),
            [&json!(5041)]
        );
    }

    /// T813: login -> lobby over the real socket: after the 81 come 94, 92 and 291, exactly the
    /// bodies `lobby_entry` builds for this account, group and the 411's name, then one
    /// `card_lobby` event with ids only. A refused login gets no lobby.
    #[test]
    fn a_card_login_enters_the_lobby_in_order() {
        if !test_support::bin_test_target("card") {
            return;
        }
        let mut card = listener();
        let lobby = osfr_tools::card_game::Lobby {
            group_id: 1,
            lobby_type: 1,
            role: 1,
            collection: Vec::new(),
        };
        card.lobby = Some(lobby.clone());
        let mut refused = client(&card);
        log_in(&mut card, &mut refused, "", "77", refuse("", ""));
        let guid = 5041u64;
        let mut player = client(&card);
        let (ok, ..) = log_in(&mut card, &mut player, "", "78", Ok(guid));
        assert!(ok);
        let account = card_service::account_id(guid);
        let pushed: Vec<_> = (0..3).map(|_| answer(&mut card, &mut player, 3)).collect();
        assert_eq!(
            pushed,
            card_service::lobby_entry(account, &lobby, "Tester"),
            "AddGroups, Join, ChangeLobbyDisplay, in that order"
        );
        let classes: Vec<_> = pushed
            .iter()
            .map(|b| card_service::class_id(b).unwrap())
            .collect();
        assert_eq!(classes, [94, 92, 291]);
        let lobbies: Vec<_> = card
            .events
            .iter()
            .filter(|e| e["event"] == "card_lobby")
            .collect();
        assert_eq!(
            lobbies,
            [&json!({
                "event":"card_lobby","peer":player.local_addr().unwrap().to_string(),
                "character":guid,"account":account,"group":1
            })]
        );
    }

    /// T825: the client side of one command, routed like the client's (`101921e0`).
    fn send(card: &mut CardListener, client: &mut TcpStream, body: &[u8]) {
        client.write_all(&frame(Some(ROUTING), body)).unwrap();
        for _ in 0..20 {
            card.poll(3, refuse);
            std::thread::sleep(Duration::from_millis(5));
        }
    }

    /// Whether the client has a frame waiting, without blocking.
    fn waiting(client: &TcpStream) -> bool {
        client.set_nonblocking(true).unwrap();
        let ready = client.peek(&mut [0; 4]).map(|n| n > 0).unwrap_or(false);
        client.set_nonblocking(false).unwrap();
        ready
    }

    /// T825: login -> lobby -> collection sync over the real socket. After the lobby comes the
    /// Start; each client Update gets the next step (StartInstances, SendInstances, Complete),
    /// byte-equal to the codec, then one `card_sync`. The client's status (137) comes back for
    /// its own account; its Join request (92) gets no answer, only a `card_join_request` event.
    #[test]
    fn a_card_login_syncs_the_starter_collection() {
        if !test_support::bin_test_target("card") {
            return;
        }
        let mut card = listener();
        let cards = vec![
            osfr_tools::card_game::StarterCard {
                archetype_id: 100009946,
                copies: 2,
            },
            osfr_tools::card_game::StarterCard {
                archetype_id: 100010026,
                copies: 1,
            },
        ];
        let lobby = osfr_tools::card_game::Lobby {
            group_id: 1,
            lobby_type: 1,
            role: 1,
            collection: cards.clone(),
        };
        card.lobby = Some(lobby);
        let guid = 5041u64;
        let account = card_service::account_id(guid);
        let mut player = client(&card);
        let (ok, ..) = log_in(&mut card, &mut player, "", "78", Ok(guid));
        assert!(ok);
        let pushed: Vec<_> = (0..4).map(|_| answer(&mut card, &mut player, 3)).collect();
        assert_eq!(
            pushed[3],
            card_service::sync_start(account, &cards),
            "the Start follows the lobby"
        );
        let update = {
            let mut body = Vec::new();
            card_service::put_headers(&mut body, card_service::SYNC_UPDATE, 1);
            body
        };
        let steps = [
            card_service::sync_start_instances(account, &cards),
            card_service::sync_send_instances(&cards),
            card_service::sync_complete(),
        ];
        for step in steps {
            player.write_all(&frame(Some(ROUTING), &update)).unwrap();
            assert_eq!(answer(&mut card, &mut player, 3), step);
        }
        // Done: a further Update is answered by silence.
        send(&mut card, &mut player, &update);
        assert!(!waiting(&player));
        let synced: Vec<_> = card
            .events
            .iter()
            .filter(|e| e["event"] == "card_sync")
            .collect();
        assert_eq!(synced.len(), 1);
        assert_eq!(
            (&synced[0]["collection"], &synced[0]["instances"]),
            (&json!("sent"), &json!(3))
        );

        // The status the client reports for itself comes back for this connection's account.
        let mut status = Vec::new();
        card_service::put_headers(&mut status, card_service::CHANGE_STATUS, 1);
        status.extend(int(99));
        status.extend(int(7));
        player.write_all(&frame(Some(ROUTING), &status)).unwrap();
        assert_eq!(
            answer(&mut card, &mut player, 3),
            card_service::change_status(account, &status).unwrap()
        );
        // Its own Join request: no answer, one event with the group and role.
        let mut join = Vec::new();
        card_service::put_headers(&mut join, card_service::JOIN, 4);
        for value in [
            int(account),
            int(1),
            int(1),
            text(""),
            int(0),
            text(""),
            int(1),
            int(0),
        ] {
            join.extend(value);
        }
        send(&mut card, &mut player, &join);
        // Its avatar picture (account level, byte count, bytes): no answer either.
        let mut avatar = Vec::new();
        card_service::put_headers(&mut avatar, card_service::UPLOAD_AVATAR_IMAGE, 1);
        avatar.extend([int(account), int(3), vec![1, 2, 3]].concat());
        send(&mut card, &mut player, &avatar);
        assert!(!waiting(&player));
        let requests: Vec<_> = card
            .events
            .iter()
            .filter(|e| e["event"] == "card_join_request")
            .collect();
        assert_eq!(
            (&requests[0]["group"], &requests[0]["role"]),
            (&json!(1), &json!(1))
        );
        assert!(
            !card
                .events
                .iter()
                .any(|e| e["event"] == "card_command_unhandled"),
            "{:?}",
            card.events
        );
    }

    /// T825: a client whose cached collection matches the Start answers with Complete; the
    /// server sends nothing more and logs the collection as cached.
    #[test]
    fn a_cached_card_collection_is_not_sent_again() {
        if !test_support::bin_test_target("card") {
            return;
        }
        let mut card = listener();
        let cards = vec![osfr_tools::card_game::StarterCard {
            archetype_id: 100009946,
            copies: 2,
        }];
        card.lobby = Some(osfr_tools::card_game::Lobby {
            group_id: 1,
            lobby_type: 1,
            role: 1,
            collection: cards,
        });
        let mut player = client(&card);
        log_in(&mut card, &mut player, "", "78", Ok(5041));
        for _ in 0..4 {
            answer(&mut card, &mut player, 3);
        }
        let mut complete = Vec::new();
        card_service::put_headers(&mut complete, card_service::SYNC_COMPLETE, 2);
        complete.extend(int(0));
        send(&mut card, &mut player, &complete);
        let mut update = Vec::new();
        card_service::put_headers(&mut update, card_service::SYNC_UPDATE, 1);
        send(&mut card, &mut player, &update);
        assert!(!waiting(&player), "nothing after the client's Complete");
        let synced: Vec<_> = card
            .events
            .iter()
            .filter(|e| e["event"] == "card_sync")
            .collect();
        assert_eq!(synced.len(), 1);
        assert_eq!(synced[0]["collection"], "cached");
    }

    #[test]
    fn a_silent_connection_is_closed_after_the_login_deadline() {
        if !test_support::bin_test_target("card") {
            return;
        }
        let mut card = listener();
        let _client = client(&card);
        for _ in 0..200 {
            card.poll(1, refuse);
            if !card.connections.is_empty() {
                break;
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        assert_eq!(card.connections.len(), 1);
        card.poll(1 + LOGIN_DEADLINE_MS, refuse);
        assert_eq!(card.connections.len(), 1, "the deadline is inclusive");
        card.poll(2 + LOGIN_DEADLINE_MS, refuse);
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
            card.poll(1, refuse);
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
            card.poll(1, refuse);
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
        for character in ["5041", "", "not-a-guid"] {
            assert_eq!(
                server.redeem_card_ticket("77", character, 1),
                Err(card_service::UNKNOWN_TICKET)
            );
        }
        assert!(server.card.is_none(), "no launch block, no card service");
    }

    /// The launch ticket of the 39/0x28 among `replies`: two i32 halves after the 15-byte run
    /// header, low then high.
    fn ticket_in(replies: &[Vec<u8>]) -> u64 {
        let mut head = vec![39, 0, 0x28];
        head.extend(osfr_tools::card_game::FIRST_INSTANCE_ID.to_le_bytes());
        let found = replies.iter().find_map(|reply| {
            let at = reply.windows(head.len()).position(|w| w == head)?;
            let half = |o: usize| reply[at + o..at + o + 4].try_into().map(u32::from_le_bytes);
            Some(u64::from(half(15).ok()?) | u64::from(half(19).ok()?) << 32)
        });
        found.expect("a 39/0x28 answers the 39/37")
    }

    /// T805 (native 2026-10-27): the retail 411 names no character id, so the server finds the
    /// session that minted a launch ticket by the ticket alone. Two in-world sessions each mint
    /// one through the real Play -> 39/37 path; each redeems once with an empty id.
    #[test]
    fn a_card_login_without_a_character_id_redeems_in_the_minting_session() {
        if !test_support::bin_test_target("card") {
            return;
        }
        let scratch = test_support::Scratch::new("t805-card-redeem");
        let mut options = test_support::options(scratch.state());
        if !test_support::fixtures_present(&options) {
            return;
        }
        let content = test_support::vendor_content(&scratch);
        let shipped = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content");
        let mut cards: Value =
            serde_json::from_slice(&fs::read(shipped.join("card_game.json")).unwrap()).unwrap();
        cards["launch"] = json!({"ticket_ttl_ms":60_000,
            "source":[{"kind":"reconstructed","evidence":"T805 test"}]});
        fs::write(content.join("card_game.json"), cards.to_string()).unwrap();
        options.content = Some(content);
        let mut server = Server::new(options).unwrap();
        let mut minted = Vec::new();
        for id in [805, 806] {
            let options = &server.options;
            let mut scene = test_support::Scene::new(options, id);
            scene.send(options, &[13, 0], 1_000).unwrap();
            scene.send(options, &[10, 0], 1_001).unwrap();
            // Play on the Trading Card Game row (167/1/2, row 490), then the card manager's 39/37.
            let mut play = vec![167, 0, 1, 2];
            play.extend(490u32.to_le_bytes());
            play.extend(0u32.to_le_bytes());
            scene.exchange(options, &play, 1_100);
            let mut ask = vec![39, 0, 37];
            for value in [osfr_tools::card_game::FIRST_INSTANCE_ID, -1, -1] {
                ask.extend(value.to_le_bytes());
            }
            let ticket = ticket_in(&scene.exchange(options, &ask, 1_200));
            minted.push((ticket.to_string(), scene.session.gateway.state().guid));
            let key = SessionKey {
                peer: scene.session.peer,
                connection_id: scene.session.request.id,
            };
            server.sessions.insert(key, scene.session);
        }
        for (ticket, guid) in &minted {
            assert_eq!(server.redeem_card_ticket(ticket, "", 2_000), Ok(*guid));
            assert_eq!(
                server.redeem_card_ticket(ticket, "", 2_001),
                Err(card_service::UNKNOWN_TICKET),
                "single use"
            );
        }
    }

    #[test]
    fn card_events_follow_a_connection_from_connect_to_close() {
        if !test_support::bin_test_target("card") {
            return;
        }
        let mut card = listener();
        let mut client = client(&card);
        let peer = client.local_addr().unwrap().to_string();
        client.write_all(&frame(None, &gateway_request())).unwrap();
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
        let guid = (1u64 << 40) | 5041;
        // T808: a refusal closes its connection, so the second login opens another.
        let mut refused = client(&card);
        log_in(&mut card, &mut refused, "", "271828182845", Err("expired"));
        let mut accepted = client(&card);
        log_in(
            &mut card,
            &mut accepted,
            &guid.to_string(),
            "314159265358",
            Ok(guid),
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
        // T805: a refusal names its reason, with the 411's version and field lengths.
        assert_eq!(
            (&logins[0]["reason"], &logins[1]["reason"]),
            (&json!("expired"), &Value::Null)
        );
        assert_eq!(logins[0]["version"], 8);
        assert_eq!(
            logins[0]["lengths"],
            json!({"session_id":9,"character_name":6,"character_id":0,"ticket":12})
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
        silent.write_all(&frame(Some(ROUTING), &[])).unwrap();
        wait_for(&mut card, "card_closed");
        assert_eq!(closed(&card), ["bad_message"]);

        // One connection repeating 414s and unknown commands logs 16 of each, then stays quiet.
        let mut card = listener();
        let mut chatty = client(&card);
        for _ in 0..(LOGGED_PER_KIND + 4) {
            chatty.write_all(&frame(None, &gateway_request())).unwrap();
            chatty.write_all(&frame(Some(ROUTING), &int(1234))).unwrap();
        }
        for _ in 0..40 {
            card.poll(1, refuse);
            std::thread::sleep(Duration::from_millis(5));
        }
        let unhandled = count(&card, "card_command_unhandled");
        assert_eq!(
            count(&card, "card_gateway_request"),
            LOGGED_PER_KIND as usize
        );
        assert_eq!(unhandled, LOGGED_PER_KIND as usize);
        assert_eq!(
            card.connections.len(),
            1,
            "the chatty client stays connected"
        );
    }

    /// F785 (T770 review): a logged-in or silent-but-chatty connection can send frames for as long
    /// as it likes, so its counters stop at `u32::MAX` instead of overflowing (a debug-build
    /// panic outside `contain` would stop the whole gateway).
    #[test]
    fn a_connections_counters_saturate_instead_of_overflowing() {
        if !test_support::bin_test_target("card") {
            return;
        }
        let mut card = listener();
        let mut chatty = client(&card);
        wait_for(&mut card, "card_connected");
        card.connections[0].unhandled = u32::MAX - 1;
        card.connections[0].answered.gateway_requests = u32::MAX - 1;
        for _ in 0..3 {
            chatty.write_all(&frame(Some(ROUTING), &int(1234))).unwrap();
            chatty.write_all(&frame(None, &gateway_request())).unwrap();
        }
        for _ in 0..40 {
            card.poll(1, refuse);
            std::thread::sleep(Duration::from_millis(5));
        }
        assert_eq!(card.connections.len(), 1, "the connection survives");
        assert!(closed(&card).is_empty());
        assert_eq!(card.connections[0].unhandled, u32::MAX);
        assert_eq!(card.connections[0].answered.gateway_requests, u32::MAX);
    }

    /// T782 review: every answered kind has its own budget, so a client that retries 414 sixteen
    /// times on one connection still gets its `card_login` line, the event the native diagnosis
    /// reads.
    #[test]
    fn the_answered_event_caps_are_per_kind() {
        if !test_support::bin_test_target("card") {
            return;
        }
        let mut card = listener();
        let mut chatty = client(&card);
        for _ in 0..(LOGGED_PER_KIND + 1) {
            chatty.write_all(&frame(None, &gateway_request())).unwrap();
        }
        // The 411 on the same connection: a shared counter would spend its budget on the 414s.
        chatty
            .write_all(&frame(Some(ROUTING), &session_login("", "79")))
            .unwrap();
        for _ in 0..40 {
            card.poll(1, refuse);
            std::thread::sleep(Duration::from_millis(5));
        }
        assert_eq!(
            count(&card, "card_gateway_request"),
            LOGGED_PER_KIND as usize
        );
        assert_eq!(
            count(&card, "card_login"),
            1,
            "the login has its own budget"
        );
        assert_eq!(closed(&card), ["login_refused"]);
    }
}
