//! T770: the Card Duelist card service, a TCP listener beside the UDP gateway. The card window
//! (`tcg/FreeRealmsTCG.dll`) asks 414 for its connection server (answered with this listener),
//! then logs in with 411; the login's launch ticket is redeemed once in the session that minted
//! the ticket (T764/T765; a character id the 411 carries, if any, must name its owner, and the
//! retail client sends none). T808: a refused login is closed once its 81 is
//! written; after login the client's ping (112) is echoed. T813: a login pushes the launch
//! block's lobby (94 -> 92 -> 291), which opens the lobby screen. T825: then the starter
//! collection sync (83, and per client Update 84, 85, 86); the client's status (137) is echoed and
//! its own Join requests (92) are logged, and one for the casual group is answered with it. L935: the single-player records of the
//! card window (487; it plays Tutorials, Scenarios and Skirmishes itself) are queued by their
//! connection's character and settled by `Server::poll_card` in that character's session
//! (`Gameplay::card_single_player`): a paying win credits the launched row and pays the duel
//! purse. Other commands are logged, not answered yet. Off unless the card content's launch
//! block names a port.
//!
//! T782: every step prints one JSON event line (`events`, drained by `Server::poll_card`):
//! `card_connected`, `card_gateway_request` (a 414 answered), `card_login` (a 411: `ok`, the
//! character on success, else the refusal `reason`; T805: also the 411's `version` and field
//! `lengths`), `card_lobby` (T813: the lobby pushed, once per connection: ids only),
//! `card_join_request` (T825: the group and role a client Join asks for), `card_sync` (T825: the
//! collection synced, or already cached by the client), `card_ping`
//! (T808: a ping echoed), `card_single_player` (L935: phase, game type and result of a record),
//! `card_duel_settled` (L935: a win paid, with the row and the server-measured seconds),
//! `card_command_unhandled` and `card_closed` with its reason. A ticket or session id is never in
//! an event.
use super::*;
use osfr_tools::card_service;
use serde_json::Value;
use std::io::{ErrorKind, Read, Write};
use std::net::{IpAddr, SocketAddr, TcpStream};

/// What a card login's ticket names: its character, that row's client scenario, the cards the
/// character has earned, the decks it has built and its photo booth headshot (empty for none);
/// or a refusal.
type Redeemed = Result<
    (
        u64,
        Option<i32>,
        Vec<osfr_tools::card_game::StarterCard>,
        Vec<osfr_tools::card_game::BuiltDeck>,
        Vec<u8>,
    ),
    &'static str,
>;

/// What a connection hands back to `Server::poll_card`: single-player records and deck changes.
type Outbox<'a> = (
    &'a mut Vec<(u64, card_service::SinglePlayerRecord)>,
    &'a mut Vec<(u64, osfr_tools::card_game::DeckChange)>,
);
/// How much of a deck save that could not be read is logged.
const DECK_LOG_BYTES: usize = 160;
/// (R) Deck saves and deletes one card window may make: more than a player needs, few enough
/// that a client looping them cannot keep the realm writing its save.
const MAX_DECK_COMMANDS: u32 = 128;

/// (R) The gateway's own player ceilings: a stranger cannot hold more sockets than players.
const MAX_CONNECTIONS: usize = 16;
const MAX_PER_ADDRESS: usize = 4;
/// (R) A connection that has not logged in this long after opening is closed.
const LOGIN_DEADLINE_MS: u64 = 30_000;
/// (R) A logged-in window pings every 10 s: this long silent, its PC is gone. Without it a dead
/// connection holds one of the slots above until the realm restarts.
const IDLE_DEADLINE_MS: u64 = 120_000;
/// (R) A game sends two records, and each costs a staged session, so one poll takes this many.
const MAX_RECORDS_PER_POLL: usize = 8;
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
    /// L935: the single-player records of logged-in connections since the last drain, with the
    /// character each connection logged in as (`Server::poll_card` settles them).
    records: Vec<(u64, card_service::SinglePlayerRecord)>,
    /// The decks logged-in connections saved or removed since the last drain.
    decks: Vec<(u64, osfr_tools::card_game::DeckChange)>,
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
    /// When the client last sent anything.
    heard_ms: u64,
    /// The 411's name, for the lobbies joined later.
    name: String,
    sync: SyncStep,
    /// The collection this login synced: the starter set plus what the character has earned.
    cards: Vec<osfr_tools::card_game::StarterCard>,
    headshot: Vec<u8>,
    deck_commands: u32,
    avatar_requests: u32,
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
    single_player: u32,
}

impl Answered {
    fn count(&mut self, kind: AnsweredKind) -> &mut u32 {
        match kind {
            AnsweredKind::GatewayRequest => &mut self.gateway_requests,
            AnsweredKind::Login => &mut self.logins,
            AnsweredKind::Ping => &mut self.pings,
            AnsweredKind::JoinRequest => &mut self.join_requests,
            AnsweredKind::SinglePlayer => &mut self.single_player,
        }
    }
}

#[derive(Clone, Copy)]
enum AnsweredKind {
    GatewayRequest,
    Login,
    Ping,
    JoinRequest,
    SinglePlayer,
}

/// The card service settings the content asks for: the launch block's port and lobby, read from
/// the same content directory the game server binds (`npc_content`). `None` when no content
/// directory is named, the file has no launch block, or its port is 0 (the off switch). A launch
/// block without a lobby still opens the service, with no lobby push after login.
fn launch_settings(
    options: &Options,
) -> io::Result<Option<(u16, Option<osfr_tools::card_game::Lobby>)>> {
    let Some(dir) = options.content.as_deref() else {
        return Ok(None);
    };
    let dir = osfr_tools::content::npc_content(dir);
    let launch = osfr_tools::card_game::Content::load_optional(&dir)?.and_then(|cards| {
        cards
            .launch()
            .map(|launch| (launch.port, launch.lobby.clone()))
    });
    Ok(launch.filter(|(port, _)| *port != 0))
}

/// The listener the card content asks for, on the gateway's own address (loopback, or the
/// private address `--allow-lan` admits). A port that cannot be bound is logged, not fatal.
pub(super) fn open(options: &Options) -> io::Result<Option<CardListener>> {
    let Some((port, lobby)) = launch_settings(options)? else {
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
            records: Vec::new(),
            decks: Vec::new(),
            lobby: None,
        })
    }

    /// Accept and answer what has arrived. `redeem(ticket, character_id)` names the character a
    /// login's launch ticket was minted for, once, else the refusal word.
    fn poll(
        &mut self,
        now_ms: u64,
        mut redeem: impl FnMut(&str, &str) -> Redeemed,
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
                    heard_ms: 0,
                    name: String::new(),
                    sync: SyncStep::Off,
                    cards: Vec::new(),
                    headshot: Vec::new(),
                    deck_commands: 0,
                    avatar_requests: 0,
                    unhandled: 0,
                    answered: Answered::default(),
                });
                self.events
                    .push(json!({"event":"card_connected","peer":peer.to_string()}));
            }
        }
        let (host, port) = (self.host.clone(), self.port);
        let (events, records, decks) = (&mut self.events, &mut self.records, &mut self.decks);
        let lobby = self.lobby.as_ref();
        self.connections.retain_mut(|c| {
            let outbox = (&mut *records, &mut *decks);
            let Some(reason) = c.serve(&host, port, now_ms, lobby, &mut redeem, events, outbox)
            else {
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
        redeem: &mut impl FnMut(&str, &str) -> Redeemed,
        events: &mut Vec<Value>,
        (records, decks): Outbox<'_>,
    ) -> Option<&'static str> {
        if self.character.is_none() && now_ms.saturating_sub(self.opened_ms) > LOGIN_DEADLINE_MS {
            return Some("deadline");
        }
        if now_ms.saturating_sub(self.heard_ms.max(self.opened_ms)) > IDLE_DEADLINE_MS {
            return Some("idle");
        }
        // At most 64 KiB per poll, so the inbox stays near one frame (`client_frame` refuses
        // longer ones).
        let mut chunk = [0; 4096];
        for _ in 0..16 {
            match self.stream.read(&mut chunk) {
                Ok(0) => return Some("eof"),
                Ok(n) => {
                    self.heard_ms = now_ms;
                    self.inbox.extend_from_slice(&chunk[..n]);
                }
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
            match self.lobby_command(&body, lobby, events, (&mut *records, &mut *decks)) {
                Some(Ok(reply)) if reply.is_empty() || self.stream.write_all(&reply).is_ok() => {
                    continue;
                }
                Some(Ok(_)) => return Some("write_failed"),
                Some(Err(_)) => return Some("bad_message"),
                None => {}
            }
            let (mut login, mut name, mut scenario, mut earned, mut built, mut headshot) =
                (None, String::new(), None, Vec::new(), Vec::new(), Vec::new());
            let logged_in = self.character.is_some();
            let reply = card_service::reply(&body, host, port, logged_in, |l| {
                let redeemed = redeem(&l.ticket, &l.character_id);
                let redeemed = redeemed.map(|(character, for_row, cards, decks, face)| {
                    (scenario, earned, built, headshot) = (for_row, cards, decks, face);
                    character
                });
                // T805: the 411's shape for its event: lengths only, never a value.
                let lengths = json!({
                    "session_id":l.session_id.len(),"character_name":l.character_name.len(),
                    "character_id":l.character_id.len(),"ticket":l.ticket.len()
                });
                login = Some((redeemed, l.version, lengths));
                name = l.character_name.clone();
                self.name.clone_from(&name);
                redeemed.ok()
            });
            match reply {
                // F824 (T813 review): frames stay small (the largest is the 92 carrying the
                // player's own 411 name, itself bounded by the 64 KiB frame limit), so a
                // nonblocking write_all is used directly. A peer that cannot take a frame is
                // closed as `write_failed` and must reconnect.
                Ok(Some(frame)) => {
                    let redeemed = login.as_ref().map(|(redeemed, ..)| *redeemed);
                    self.log_answer(&body, login, host, port, events);
                    if self.stream.write_all(&frame).is_err() {
                        return Some("write_failed");
                    }
                    match redeemed {
                        // T808: the refused 81 is written; the client shows it, nothing follows.
                        // (R) Closing after a refusal is server policy (one ticket guess per
                        // connection), not a recovered client fact.
                        Some(Err(_)) => return Some("login_refused"),
                        Some(Ok(character)) if self.character.is_none() => {
                            self.character = Some(character);
                            self.headshot = std::mem::take(&mut headshot);
                            if let Some(lobby) = lobby {
                                self.cards = osfr_tools::card_game::collection_with(
                                    &lobby.collection,
                                    &earned,
                                );
                                if !self.enter_lobby(character, scenario, &built, lobby, &name, events)
                                {
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
    /// three frames of `card_service::lobby_entry`, then one `card_lobby` event. A deck row's
    /// `scenario` swaps the lobby screen for its fight. False when the peer cannot take them.
    fn enter_lobby(
        &mut self,
        character: u64,
        scenario: Option<i32>,
        built: &[osfr_tools::card_game::BuiltDeck],
        lobby: &osfr_tools::card_game::Lobby,
        name: &str,
        events: &mut Vec<Value>,
    ) -> bool {
        let account = card_service::account_id(character);
        // The fight waits for a saved deck, so without cards it would never start.
        let scenario = scenario.filter(|_| !lobby.collection.is_empty());
        let mut bodies = card_service::lobby_entry(account, lobby, name).to_vec();
        // A deck row's fight replaces the lobby: ahead of the 92, whose map screen starts it.
        if let Some(scenario) = scenario {
            bodies.pop();
            bodies.insert(1, card_service::initiate_challenge(account, scenario));
            bodies.push(card_service::online_decks(&lobby.collection, built));
        }
        // T825: the collection sync starts right after the lobby.
        if !self.cards.is_empty() {
            bodies.push(card_service::sync_start(account, &self.cards));
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
        let mut event = json!({
            "event":"card_lobby","peer":self.peer.to_string(),"character":character,
            "account":account,"group":lobby.group_id
        });
        if let Some(scenario) = scenario {
            event["scenario"] = scenario.into();
        }
        events.push(event);
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
    /// - L935 487 single-player record (a Tutorial, Scenario or Skirmish the DLL plays itself):
    ///   queued for `Server::poll_card` to settle in the character's session; no answer. A record
    ///   for another account is dropped, an unreadable one is logged as unhandled.
    fn lobby_command(
        &mut self,
        body: &[u8],
        lobby: Option<&osfr_tools::card_game::Lobby>,
        events: &mut Vec<Value>,
        (records, decks): Outbox<'_>,
    ) -> Option<io::Result<Vec<u8>>> {
        let character = self.character?;
        let account = card_service::account_id(character);
        let reply = match card_service::class_id(body).ok()? {
            card_service::CHANGE_STATUS => card_service::change_status(account, body),
            card_service::JOIN => card_service::join_request(body).and_then(|(group, _)| {
                self.log_answer(body, None, "", 0, events);
                if let Some(root) = lobby.filter(|_| group == card_service::CASUAL_GROUP) {
                    for frame in card_service::casual_lobby(account, root, &self.name) {
                        self.stream.write_all(&card_service::server_frame(&frame))?;
                    }
                }
                Ok(Vec::new())
            }),
            card_service::SYNC_UPDATE => {
                let (next, step) = match self.sync {
                    SyncStep::Started => (
                        card_service::sync_start_instances(account, &self.cards),
                        SyncStep::InstancesStarted,
                    ),
                    SyncStep::InstancesStarted => (
                        card_service::sync_send_instances(&self.cards),
                        SyncStep::InstancesSent,
                    ),
                    SyncStep::InstancesSent => (card_service::sync_complete(), SyncStep::Done),
                    SyncStep::Off | SyncStep::Done => return Some(Ok(Vec::new())),
                };
                if step == SyncStep::Done {
                    self.synced("sent", events);
                }
                self.sync = step;
                Ok(next)
            }
            card_service::SYNC_COMPLETE if self.sync == SyncStep::Started => {
                self.sync = SyncStep::Done;
                self.synced("cached", events);
                Ok(Vec::new())
            }
            card_service::RECORD_SINGLE_PLAYER_GAME => {
                // A record this decoder cannot read stays an unhandled command (logged), never a
                // closed window: the layout has no native sample yet.
                let record = card_service::SinglePlayerRecord::decode(body).ok()?;
                self.log_answer(body, None, "", 0, events);
                if record.account == account && records.len() < MAX_RECORDS_PER_POLL {
                    records.push((character, record));
                }
                Ok(Vec::new())
            }
            card_service::ADD_ONLINE_DECK => {
                self.deck_commands = self.deck_commands.saturating_add(1);
                let (log, allowed) =
                    (self.deck_commands <= LOGGED_PER_KIND, self.deck_commands <= MAX_DECK_COMMANDS);
                let Ok(deck) = card_service::add_online_deck(body) else {
                    if log {
                        let head = osfr_tools::trace::hex_prefix_len(body, false, DECK_LOG_BYTES);
                        events.push(json!({"event":"card_deck_unread","character":character,"head":head}));
                    }
                    return Some(Ok(Vec::new()));
                };
                let kept = deck.storable(&self.cards);
                if log {
                    events.push(json!({
                        "event":"card_deck_saved","character":character,"cards":deck.cards.len(),
                        "kept":kept,"playable":deck.playable()
                    }));
                }
                if kept && allowed && decks.len() < MAX_RECORDS_PER_POLL {
                    decks.push((character, osfr_tools::card_game::DeckChange::Save(deck)));
                }
                Ok(Vec::new())
            }
            card_service::REQUEST_AVATAR_IMAGES => {
                let ids = card_service::avatar_request(body).ok();
                self.avatar_requests = self.avatar_requests.saturating_add(1);
                if self.avatar_requests <= LOGGED_PER_KIND {
                    let head = ids
                        .is_none()
                        .then(|| osfr_tools::trace::hex_prefix_len(body, false, DECK_LOG_BYTES))
                        .flatten();
                    events.push(json!({"event":"card_avatar_request","character":character,"account":account,"ids":ids,"head":head}));
                }
                let own = ids.is_some_and(|ids| ids.contains(&account));
                Ok(if own && !self.headshot.is_empty() {
                    card_service::send_avatar_image(account, &self.headshot)
                } else {
                    Vec::new()
                })
            }
            card_service::DELETE_ONLINE_DECK => {
                self.deck_commands = self.deck_commands.saturating_add(1);
                if let Ok(id) = card_service::delete_online_deck(body)
                    && self.deck_commands <= MAX_DECK_COMMANDS
                    && decks.len() < MAX_RECORDS_PER_POLL
                {
                    decks.push((character, osfr_tools::card_game::DeckChange::Delete(id)));
                }
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
        &self,
        how: &str,
        events: &mut Vec<Value>,
    ) {
        events.push(json!({
            "event":"card_sync","peer":self.peer.to_string(),"character":self.character,
            "collection":how,"archetypes":self.cards.len(),
            "instances":self.cards.iter().map(|card| card.copies).sum::<i32>()
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
            (Ok(card_service::RECORD_SINGLE_PLAYER_GAME), _) => AnsweredKind::SinglePlayer,
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
            AnsweredKind::SinglePlayer => {
                let Ok(record) = card_service::SinglePlayerRecord::decode(body) else {
                    return;
                };
                json!({
                    "event":"card_single_player","peer":peer,"character":self.character,
                    "phase":record.phase,"type":record.game_type,"result":record.result
                })
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
        let records: Vec<_> = card.records.drain(..).collect();
        let decks: Vec<_> = card.decks.drain(..).collect();
        self.card = Some(card);
        for (character, record) in records {
            self.settle_card_record(character, &record, now_ms);
        }
        for (character, change) in decks {
            self.change_card_deck(character, change);
        }
    }

    /// Saves a deck the card window built or removed, staged like a single-player record.
    fn change_card_deck(&mut self, character: u64, change: osfr_tools::card_game::DeckChange) {
        // L1177: never an unbound handshake that only carries the template's guid.
        let local = self.options.account_store.is_none();
        let Some(session) = self
            .sessions
            .values_mut()
            .find(|session| session.plays(character, local))
        else {
            return;
        };
        use osfr_tools::card_game::{DeckChange, MAX_BUILT_DECKS, MIN_DECK_CARDS};
        let mut staged = session.gateway.clone();
        let Some(game) = staged.gameplay_mut() else {
            return;
        };
        // The client saves any deck without comment, so the player hears it here.
        let notice = match &change {
            DeckChange::Save(deck) if !game.can_keep_deck(deck) => Some(format!(
                "Your deck was not saved: you already have {MAX_BUILT_DECKS} decks. Delete one first."
            )),
            DeckChange::Save(deck) if !deck.playable() => Some(format!(
                "Your deck is saved with {} of {MIN_DECK_CARDS} cards. It needs {MIN_DECK_CARDS} before you can duel with it.",
                deck.cards.len()
            )),
            _ => None,
        };
        if game.change_built_deck(change) {
            if save(&session.state_path, &staged.snapshot()).is_err() {
                println!("{}", json!({"event":"card_deck_save_failed","character":character}));
                return;
            }
            session.gateway = staged;
        }
        if let Some(line) = notice {
            // A full queue only loses the notice.
            let _ = session.queue_shared_message(&wrap_application(&osfr_tools::chat::system(&line)));
        }
    }

    /// L935: settle one single-player record in the character's session, in the order of every
    /// other server-initiated payout: stage the change on a copy, save it, adopt it, then queue
    /// the replies. The copy is adopted whenever the save is not needed, because a start record
    /// only moves the session's tracker. A record for a character with no session is dropped.
    fn settle_card_record(
        &mut self,
        character: u64,
        record: &card_service::SinglePlayerRecord,
        now_ms: u64,
    ) {
        let local = self.options.account_store.is_none();
        let Some(session) = self
            .sessions
            .values_mut()
            .find(|session| session.plays(character, local))
        else {
            return;
        };
        let mut staged = session.gateway.clone();
        let settled = match staged
            .gameplay_mut()
            .map(|game| game.card_single_player(character, record, now_ms))
        {
            Some(Ok(settled)) => settled,
            Some(Err(error)) => {
                println!(
                    "{}",
                    json!({"event":"card_duel_refused","character":character,"reason":error.to_string()})
                );
                return;
            }
            None => return,
        };
        if settled.as_ref().is_some_and(|(_, out)| out.changed)
            && save(&session.state_path, &staged.snapshot()).is_err()
        {
            println!(
                "{}",
                json!({"event":"card_duel_save_failed","character":character})
            );
            return;
        }
        session.gateway = staged;
        let Some((win, out)) = settled else {
            return;
        };
        for reply in out.replies {
            if session
                .queue_shared_message(&wrap_application(&reply))
                .is_err()
            {
                break;
            }
        }
        println!(
            "{}",
            json!({
                "event":"card_duel_settled","character":character,"minigame":win.minigame_id,
                "seconds":win.elapsed_seconds,"coins":win.coins
            })
        );
    }

    /// The live session that minted a card login's launch ticket redeems it. T805: the retail
    /// 411 carries no character id, so the ticket alone finds its session; `None` from a
    /// session's store means that session never held it (or spent it already), so the scan moves
    /// on. A session that holds it but refuses (expired, or an id that is not the owner's) names
    /// the refusal, and a ticket no live session holds is `unknown_ticket`.
    // ponytail: one scan of the live sessions per card login (one login per launch); a
    // server-wide ticket index if the player count ever makes it show.
    fn redeem_card_ticket(
        &mut self,
        ticket: &str,
        character: &str,
        now_ms: u64,
    ) -> Redeemed {
        let portraits = self.world.portraits.clone();
        for session in self.sessions.values_mut() {
            let Some(game) = session.gateway.gameplay_mut() else {
                continue;
            };
            if let Some(redeemed) = game.redeem_card_ticket(ticket, character, now_ms) {
                return redeemed.map(|guid| {
                    let face = portraits.as_deref().map_or_else(Vec::new, |root| {
                        osfr_tools::portrait_upload::image(root, guid, "Headshot")
                    });
                    (guid, game.card_scenario(), game.earned_cards(), game.built_decks(), face)
                });
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
    use super::card_packets::{
        ROUTING, frame, gateway_request, headers, int, ping, read_results, session_login,
        single_player_record, text, ticket_in,
    };
    use super::*;
    use std::time::Duration;

    fn listener() -> CardListener {
        CardListener::bind("127.0.0.1".parse().unwrap(), 0).unwrap()
    }

    /// A redeem that knows no ticket.
    fn refuse(_: &str, _: &str) -> Redeemed {
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
        answer_with(|| card.poll(now, refuse), client)
    }

    /// `answer` for a caller that drives another poll (the gateway's `Server::poll_card`).
    fn answer_with(mut poll: impl FnMut(), client: &mut TcpStream) -> Vec<u8> {
        for _ in 0..200 {
            poll();
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
        log_in_to(card, client, character_id, ticket, accept.map(|guid| (guid, None, Vec::new(), Vec::new(), Vec::new())))
    }

    /// [`log_in`] for a redeem that also names the row's client scenario.
    fn log_in_to(
        card: &mut CardListener,
        client: &mut TcpStream,
        character_id: &str,
        ticket: &str,
        accept: Redeemed,
    ) -> (bool, i32, i32, Vec<(String, String)>) {
        client
            .write_all(&frame(Some(ROUTING), &session_login(character_id, ticket)))
            .unwrap();
        let mut seen = Vec::new();
        let mut body = None;
        for _ in 0..200 {
            card.poll(2, |t, c| {
                seen.push((t.to_owned(), c.to_owned()));
                accept.clone()
            });
            if !seen.is_empty() {
                body = Some(answer(card, client, 2));
                break;
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        let body = body.expect("the login was read");
        let (success, code, account, _) = read_results(&body);
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
            log_in(&mut card, &mut refused, "5041", "77", Err(card_service::UNKNOWN_TICKET));
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

    /// A listener opened through the production `open` path from a content directory whose
    /// `card_game.json` is the shipped rows plus `launch`. `open` binds the content's port, so a
    /// free port is probed first; another test taking it between probe and bind is retried.
    fn open_with_launch(launch: Value) -> CardListener {
        let scratch = test_support::Scratch::new("f824-card-lobby");
        let content = scratch.path().join("content");
        fs::create_dir_all(content.join("npcs")).unwrap();
        fs::write(content.join("npc_spawns_extra.json"), b"{}").unwrap();
        let shipped = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content/card_game.json");
        let cards: Value = serde_json::from_slice(&fs::read(shipped).unwrap()).unwrap();
        let mut options = test_support::options(scratch.state());
        options.content = Some(content.clone());
        for _ in 0..8 {
            let probe = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
            let port = probe.local_addr().unwrap().port();
            drop(probe);
            let mut cards = cards.clone();
            let mut launch = launch.clone();
            launch["port"] = json!(port);
            cards["launch"] = launch;
            fs::write(
                content.join("card_game.json"),
                serde_json::to_vec(&cards).unwrap(),
            )
            .unwrap();
            if let Some(card) = open(&options).unwrap() {
                return card;
            }
        }
        panic!("no free port for the card service");
    }

    /// T813: login -> lobby over the real socket: after the 81 come 94, 92 and 291. F824 (T813
    /// review findings 6 and 9): the listener comes from the production `open` path and the
    /// content names the lobby's type and role, while the expected bytes are spelled out again
    /// here in the readers' order, so this socket path is not checked against `lobby_entry`
    /// itself. Then one `card_lobby` event with ids only. A refused login gets no lobby.
    #[test]
    fn a_card_login_enters_the_lobby_in_order() {
        if !test_support::bin_test_target("card") {
            return;
        }
        let mut card = open_with_launch(json!({
            "ticket_ttl_ms": 60_000,
            "lobby": {"group_id": 1, "lobby_type": 5, "role": 2},
            "source": [{"kind":"reconstructed","evidence":"F824 test"}]
        }));
        let mut refused = client(&card);
        log_in(&mut card, &mut refused, "", "77", Err(card_service::UNKNOWN_TICKET));
        let guid = 5041u64;
        let mut player = client(&card);
        let (ok, ..) = log_in(&mut card, &mut player, "", "78", Ok(guid));
        assert!(ok);
        let account = card_service::account_id(guid);
        let pushed: Vec<_> = (0..3).map(|_| answer(&mut card, &mut player, 3)).collect();
        // 94 v1 (`101583f0`): headers, account, one `Lobby` (`10155710`): class 103 v3, five
        // absent property sets, five empty role lists, containing group 0, group 1, type 5, no
        // deck, tournament round 0, sub-state 0.
        let lobby_object = [
            &[0x01][..],
            &[0x27, 0x06, 0x03],
            &[0x01; 5],
            &[0x00; 5],
            &[0x00, 0x01, 0x05, 0x01, 0x00, 0x00],
        ]
        .concat();
        let expected = [
            [
                &headers(&[0x2E, 0x05], 1)[..],
                &int(account)[..],
                &lobby_object,
            ]
            .concat(),
            // 92 v4 (`10163b20`): headers, account, group 1, role 2, empty string, empty int
            // list, the 411's name, a present empty `PropertySet` (23 v1), the trailing int.
            [
                &headers(&[0x2C, 0x05], 4)[..],
                &int(account)[..],
                &[0x01, 0x02],
                &text(""),
                &[0x00],
                &text("Tester"),
                &[0x00, 0x27, 0x01, 0x01, 0x00],
                &[0x00],
            ]
            .concat(),
            // 291 v1 (`1015e5f0`): headers, account, group 1.
            [&headers(&[0x23, 0x12], 1)[..], &int(account)[..], &[0x01]].concat(),
        ];
        assert_eq!(
            pushed, expected,
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

    /// A deck row's login: its challenge lands before the 92 whose map screen consumes it, and
    /// no 291 follows to swap that screen for the lobby.
    #[test]
    fn a_deck_login_queues_its_challenge_before_the_map_screen() {
        if !test_support::bin_test_target("card") {
            return;
        }
        let mut card = open_with_launch(json!({
            "port": 1, "ticket_ttl_ms": 60_000,
            "lobby": {"group_id": 1, "lobby_type": 5, "role": 2,
                "collection": [{"archetype_id": 100_009_946, "copies": 3}]},
            "source": [{"kind":"reconstructed","evidence":"deck login test"}]
        }));
        let guid = 5041u64;
        let earned = osfr_tools::card_game::StarterCard { archetype_id: 7, copies: 1 };
        let built = osfr_tools::card_game::BuiltDeck { id: "d".into(), name: "Mine".into(), cards: vec![7] };
        let mut player = client(&card);
        assert!(log_in_to(&mut card, &mut player, "", "78", Ok((guid, Some(6), vec![earned.clone()], vec![built.clone()], Vec::new()))).0);
        let pushed: Vec<_> = (0..5).map(|_| answer(&mut card, &mut player, 3)).collect();
        let classes: Vec<_> = pushed.iter().map(|b| card_service::class_id(b).unwrap()).collect();
        assert_eq!(classes, [94, 486, 92, 363, 83], "the deck, then the collection sync; no 291");
        let account = card_service::account_id(guid);
        assert_eq!(pushed[1], card_service::initiate_challenge(account, 6));
        let starter = osfr_tools::card_game::StarterCard { archetype_id: 100_009_946, copies: 3 };
        assert_eq!(pushed[3], card_service::online_decks(std::slice::from_ref(&starter), &[built]), "starter, built deck");
        assert_eq!(pushed[4], card_service::sync_start(account, &[starter, earned]), "earned cards sync");
        assert!(card.events.iter().any(|e| e["event"] == "card_lobby" && e["scenario"] == 6));
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
    /// its own account; its Join of the root (92) gets no answer, only a `card_join_request` event,
    /// and its Join of the casual group gets that group.
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
        // The client's 87 Update v1 (`101db190`): the three class headers alone.
        let update = headers(&[0x27, 0x05], 1);
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
        // 137 v1 (reader `1015ea00`): the headers, the account it names (ignored) and the status.
        let mut status = headers(&[0x29, 0x08], 1);
        status.extend(int(99));
        status.extend(int(7));
        player.write_all(&frame(Some(ROUTING), &status)).unwrap();
        assert_eq!(
            answer(&mut card, &mut player, 3),
            card_service::change_status(account, &status).unwrap()
        );
        // Its own Join request (92 v4, reader `10163b20`): headers, account, group, role, the
        // request's empty tail. No answer, one event with the group and role.
        let mut join = headers(&[0x2C, 0x05], 4);
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
        // Its avatar picture (503 v1, reader `10016810`: account, byte count, bytes); no answer
        // either.
        let mut avatar = headers(&[0x27, 0x1F], 1);
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
        // "Play a Game": a Join of group 2 is answered with the group, the Join and the screen.
        let mut casual = headers(&[0x2C, 0x05], 4);
        for value in [
            int(account),
            int(card_service::CASUAL_GROUP),
            int(1),
            text(""),
            int(0),
            text(""),
            int(1),
            int(0),
        ] {
            casual.extend(value);
        }
        player.write_all(&frame(Some(ROUTING), &casual)).unwrap();
        let bodies: Vec<_> = (0..3).map(|_| answer(&mut card, &mut player, 3)).collect();
        assert!(bodies[0].ends_with(&[1, 2, 2, 1, 0, 0]), "{:?}", bodies[0]);
        assert!(!waiting(&player));
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
        // 86 Complete v2 (reader `101d77c0`): headers and the delivery id; 87 Update
        // (`101db190`): headers only.
        let mut complete = headers(&[0x26, 0x05], 2);
        complete.extend(int(0));
        send(&mut card, &mut player, &complete);
        let update = headers(&[0x27, 0x05], 1);
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

    /// F824 (T813 review finding 6): `open` reads the launch block through `npc_content` and
    /// keeps its lobby. Port 0 keeps the service off; a launch without a lobby still opens it.
    #[test]
    fn the_card_listener_reads_the_launch_lobby_from_content() {
        if !test_support::bin_test_target("card") {
            return;
        }
        let scratch = test_support::Scratch::new("f824-card-open");
        let content = scratch.path().join("content");
        fs::create_dir_all(content.join("npcs")).unwrap();
        fs::write(content.join("npc_spawns_extra.json"), b"{}").unwrap();
        let shipped = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content/card_game.json");
        let mut cards: Value = serde_json::from_slice(&fs::read(shipped).unwrap()).unwrap();
        let write = |cards: &Value| {
            fs::write(
                content.join("card_game.json"),
                serde_json::to_vec(cards).unwrap(),
            )
            .unwrap();
        };
        cards["launch"] = json!({
            "ticket_ttl_ms": 60_000,
            "port": 17_782,
            "lobby": {"group_id": 1, "lobby_type": 5, "role": 2},
            "source": [{"kind":"reconstructed","evidence":"F824 test"}]
        });
        write(&cards);
        let mut options = test_support::options(scratch.state());
        options.content = Some(content.clone());
        let (port, lobby) = launch_settings(&options)
            .unwrap()
            .expect("a launch block asks for the card service");
        assert_eq!(port, 17_782);
        assert_eq!(
            lobby,
            Some(osfr_tools::card_game::Lobby {
                group_id: 1,
                lobby_type: 5,
                role: 2,
                collection: Vec::new()
            }),
            "the listener gets the content's lobby, not a default"
        );

        // Port 0 is the off switch; the lobby stays in the file.
        cards["launch"]["port"] = json!(0);
        write(&cards);
        assert_eq!(launch_settings(&options).unwrap(), None);

        // A launch block without a lobby still opens the service; the login stops after its 81.
        cards["launch"] = json!({
            "ticket_ttl_ms": 60_000,
            "port": 17_783,
            "source": [{"kind":"reconstructed","evidence":"F824 test"}]
        });
        write(&cards);
        assert_eq!(
            launch_settings(&options).unwrap().map(|(port, _)| port),
            Some(17_783)
        );
        assert_eq!(
            launch_settings(&options)
                .unwrap()
                .and_then(|(_, lobby)| lobby),
            None
        );
    }

    /// No launch block, no card service: a login, whatever it names, is `unknown_ticket`.
    #[test]
    fn a_card_login_without_a_card_launch_block_is_refused() {
        if !test_support::bin_test_target("card") {
            return;
        }
        let scratch = test_support::Scratch::new("t770-card-login");
        let mut options = test_support::options(scratch.state());
        if !test_support::fixtures_present(&options) {
            return;
        }
        // L935: the shipped content opens the card service, so this case runs on a vendor copy
        // that has no card file.
        let content = test_support::vendor_content(&scratch);
        assert!(!content.join("card_game.json").exists());
        options.content = Some(content);
        let mut server = Server::new(options).unwrap();
        for character in ["5041", "", "not-a-guid"] {
            assert_eq!(
                server.redeem_card_ticket("77", character, 1),
                Err(card_service::UNKNOWN_TICKET)
            );
        }
        assert!(server.card.is_none(), "no launch block, no card service");
    }

    /// T805 (native 2026-10-27, T804 review finding 3): the retail 411 names no character id, so
    /// the server finds the session that minted a launch ticket by the ticket alone. Two in-world
    /// sessions each mint one through the real Play -> 39/37 path; the retail-shaped 411s then
    /// cross the bound listener and `Server::poll_card`, and each 81 is a success (the 81's flag
    /// is the `redeemed.is_ok()` the `card_login` event records). Each ticket is held by one
    /// session only, a refusal changes nothing, and a ticket found under a wrong character id
    /// names `character_mismatch`, never the next session's answer or a bare `unknown_ticket`.
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
            scene.exchange(options, &test_support::portal_play(490), 1_100);
            let mut ask = vec![39, 0, 37];
            for value in [osfr_tools::card_game::FIRST_INSTANCE_ID, -1, -1] {
                ask.extend(value.to_le_bytes());
            }
            let ticket = ticket_in(&scene.exchange(options, &ask, 1_200));
            let key = SessionKey {
                peer: scene.session.peer,
                connection_id: scene.session.request.id,
            };
            minted.push((ticket.to_string(), key));
            server.sessions.insert(key, scene.session);
        }
        // Each ticket is held by its own session only.
        for (index, (_, key)) in minted.iter().enumerate() {
            let session = server.sessions.get_mut(key).unwrap();
            let gameplay = session.gateway.gameplay_mut().unwrap();
            assert_eq!(
                gameplay.redeem_card_ticket(&minted[1 - index].0, "", 1_800),
                None,
                "another session's ticket is not this session's"
            );
        }
        // The first session's ticket under another character id: found and refused by name, and
        // the scan does not fall through to an `unknown_ticket`. A malformed field keeps its own
        // word too.
        assert_eq!(
            server.redeem_card_ticket(&minted[0].0, "2", 1_900),
            Err("character_mismatch")
        );
        assert_eq!(
            server.redeem_card_ticket("not-a-ticket", "", 1_900),
            Err("ticket_not_decimal")
        );

        let card = listener();
        server.card = Some(card);
        let login = |server: &mut Server, ticket: &str, now: u64| {
            let mut client = client(server.card.as_ref().unwrap());
            client
                .write_all(&frame(Some(ROUTING), &session_login("", ticket)))
                .unwrap();
            read_results(&answer_with(|| server.poll_card(now), &mut client))
        };
        assert_eq!(
            login(&mut server, "77", 2_000),
            (false, 63, 0, String::new()),
            "a ticket no session holds is refused"
        );
        for (number, (ticket, _)) in minted.iter().enumerate() {
            assert_eq!(
                login(&mut server, ticket, 2_001 + number as u64),
                (true, 0, card_service::account_id(1), "w-session".to_owned()),
                "the minting session's ticket logs in (the fixture's character)"
            );
        }
        assert_eq!(
            login(&mut server, &minted[0].0, 2_010),
            (false, 63, 0, String::new()),
            "single use, and the refusal above spent nothing"
        );
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
    /// L935: a logged-in card window's single-player records are logged and queued for their
    /// session; another account's record is dropped, and an unreadable one is an unhandled command,
    /// never a closed window (the layout has no native sample yet).
    #[test]
    fn single_player_records_are_queued_for_the_session() {
        if !test_support::bin_test_target("card") {
            return;
        }
        let mut card = listener();
        let guid = 5041_u64;
        let mut window = client(&card);
        assert!(log_in(&mut card, &mut window, "", "271828182845", Ok(guid)).0);
        let account = card_service::account_id(guid);
        let started = 1_793_000_000;
        let mut send = |body: &[u8]| window.write_all(&frame(Some(ROUTING), body)).unwrap();
        send(&single_player_record(account, (1, 3, 0), started));
        send(&single_player_record(account, (2, 3, 1), started));
        send(&single_player_record(account + 1, (2, 3, 1), started + 1));
        send(&single_player_record(account, (2, 3, 1), started)[..7]);
        for _ in 0..40 {
            card.poll(3, refuse);
            std::thread::sleep(Duration::from_millis(5));
        }
        let record = |phase, result| {
            (
                guid,
                card_service::SinglePlayerRecord {
                    account,
                    phase,
                    game_type: 3,
                    result,
                    started,
                },
            )
        };
        assert_eq!(card.records, [record(1, 0), record(2, 1)]);
        assert_eq!(
            count(&card, "card_single_player"),
            3,
            "the foreign one is logged, not kept"
        );
        assert_eq!(
            count(&card, "card_command_unhandled"),
            1,
            "the truncated record"
        );
        assert!(closed(&card).is_empty(), "the window stays open");
    }

    /// L935: the whole path. A player presses Play on a card row and opens the card window; the
    /// window's start and end records (a Skirmish win 70 s apart on the server's clock) cross the
    /// socket and `Server::poll_card`, and the session that launched it is paid and saved once.
    #[test]
    fn a_card_window_win_pays_the_session_that_launched_it() {
        if !test_support::bin_test_target("card") {
            return;
        }
        let scratch = test_support::Scratch::new("l935-card-duel");
        let mut options = test_support::options(scratch.state());
        if !test_support::fixtures_present(&options) {
            return;
        }
        let content = test_support::vendor_content(&scratch);
        let shipped = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content");
        let mut cards: Value =
            serde_json::from_slice(&fs::read(shipped.join("card_game.json")).unwrap()).unwrap();
        // The shipped launch block (with its duel purse) minus its port: the test binds its own.
        cards["launch"]["port"] = json!(0);
        fs::write(content.join("card_game.json"), cards.to_string()).unwrap();
        options.content = Some(content);
        let mut server = Server::new(options).unwrap();
        let options = &server.options;
        let mut scene = test_support::Scene::new(options, 935);
        scene.send(options, &[13, 0], 1_000).unwrap();
        scene.send(options, &[10, 0], 1_001).unwrap();
        scene.exchange(options, &test_support::portal_play(490), 1_100);
        let mut ask = vec![39, 0, 37];
        for value in [osfr_tools::card_game::FIRST_INSTANCE_ID, -1, -1] {
            ask.extend(value.to_le_bytes());
        }
        let ticket = ticket_in(&scene.exchange(options, &ask, 1_200));
        let key = SessionKey {
            peer: scene.session.peer,
            connection_id: scene.session.request.id,
        };
        server.sessions.insert(key, scene.session);
        let mut service = listener();
        service.lobby = Some(serde_json::from_value(cards["launch"]["lobby"].clone()).unwrap());
        server.card = Some(service);

        let mut window = client(server.card.as_ref().unwrap());
        window
            .write_all(&frame(
                Some(ROUTING),
                &session_login("", &ticket.to_string()),
            ))
            .unwrap();
        let (success, _, account, _) =
            read_results(&answer_with(|| server.poll_card(2_000), &mut window));
        assert!(success, "the minting session's ticket logs in");
        answer_with(|| server.poll_card(2_001), &mut window);
        assert_eq!(
            answer_with(|| server.poll_card(2_002), &mut window),
            card_service::initiate_challenge(account, 1),
            "row 490 is Sam's first script"
        );

        let coins = |server: &Server| {
            server.sessions[&key]
                .gateway
                .gameplay()
                .unwrap()
                .state()
                .inventory
                .coins
        };
        let before = coins(&server);
        let started = 1_793_000_000;
        let mut report = |server: &mut Server, record: Vec<u8>, now: u64| {
            window.write_all(&frame(Some(ROUTING), &record)).unwrap();
            for _ in 0..40 {
                server.poll_card(now);
                std::thread::sleep(Duration::from_millis(5));
            }
        };
        report(
            &mut server,
            single_player_record(account, (1, 3, 0), started),
            10_000,
        );
        assert_eq!(coins(&server), before, "a start record pays nothing");
        report(
            &mut server,
            single_player_record(account, (2, 3, 1), started),
            80_000,
        );
        assert_eq!(coins(&server), before + 25, "the shipped duel purse");
        // The client repeating its end record is not paid again.
        report(
            &mut server,
            single_player_record(account, (2, 3, 1), started),
            90_000,
        );
        assert_eq!(coins(&server), before + 25);
        // The save on disk carries the purse.
        let saved: Value =
            serde_json::from_slice(&fs::read(&server.sessions[&key].state_path).unwrap()).unwrap();
        assert_eq!(
            saved["gameplay"]["inventory"]["coins"],
            json!(before + 25),
            "the settled win was saved"
        );
    }
}
