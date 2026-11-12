//! L071: the portrait upload listener. TCP on the gateway's own address and port number (UDP and
//! TCP share it, so a hosted realm forwards nothing new); the launcher writes
//! `http://<gateway>/image` into the client's `[Portrait] UploadUrl`. The request is parsed and
//! its PNGs rebuilt by `osfr_tools::portrait_upload`; this file decides who may upload.
//!
//! (R) An upload is taken only from the address of the live session that plays the character, at
//! most once a minute per character. The stored images then reach that player and everyone who
//! can see them as 156/3 Headshot and Portrait, through the 156/2 visibility rule.
use super::*;
use osfr_tools::{fotomat, portrait_upload};
use std::io::{ErrorKind, Read, Write};
use std::net::{IpAddr, TcpStream};

/// (R) The gateway's own ceilings: a stranger cannot hold more sockets than a few players.
const MAX_CONNECTIONS: usize = 8;
const MAX_PER_ADDRESS: usize = 2;
/// (R) A request that has not fully arrived this long after opening is closed.
const DEADLINE_MS: u64 = 10_000;
const COOLDOWN_MS: u64 = 60_000;

pub(super) struct PortraitHttp {
    listener: TcpListener,
    root: Arc<Path>,
    connections: Vec<Connection>,
    last_upload: BTreeMap<u64, u64>,
}

struct Connection {
    stream: TcpStream,
    peer: SocketAddr,
    opened_ms: u64,
    input: Vec<u8>,
}

/// Bind beside the UDP socket; images live in `portraits/` beside the state file. A port already
/// taken leaves uploads off, never the server.
pub(super) fn open(options: &Options, udp: SocketAddr) -> Option<PortraitHttp> {
    let root: Arc<Path> = Arc::from(
        options
            .state
            .parent()
            .unwrap_or(Path::new("."))
            .join("portraits"),
    );
    let listener = TcpListener::bind(udp).and_then(|listener| {
        listener.set_nonblocking(true)?;
        Ok(listener)
    });
    match listener {
        Ok(listener) => {
            println!(
                "{}",
                json!({"event":"portrait_listening","address":udp.to_string()})
            );
            Some(PortraitHttp {
                listener,
                root,
                connections: Vec::new(),
                last_upload: BTreeMap::new(),
            })
        }
        Err(error) => {
            println!(
                "{}",
                json!({"event":"portrait_unavailable","reason":error.to_string()})
            );
            None
        }
    }
}

impl PortraitHttp {
    pub(super) fn root(&self) -> Arc<Path> {
        self.root.clone()
    }

    /// Accept and read what has arrived; return each complete request with its peer.
    fn poll(&mut self, now_ms: u64, allow: impl Fn(IpAddr) -> bool) -> Vec<(Connection, Vec<u8>)> {
        for _ in 0..4 {
            let Ok((stream, peer)) = self.listener.accept() else {
                break;
            };
            let same = self
                .connections
                .iter()
                .filter(|c| c.peer.ip() == peer.ip())
                .count();
            if !allow(peer.ip())
                || self.connections.len() >= MAX_CONNECTIONS
                || same >= MAX_PER_ADDRESS
                || stream.set_nonblocking(true).is_err()
            {
                continue;
            }
            self.connections.push(Connection {
                stream,
                peer,
                opened_ms: now_ms,
                input: Vec::new(),
            });
        }
        let mut done = Vec::new();
        let mut index = 0;
        while index < self.connections.len() {
            let connection = &mut self.connections[index];
            let mut buffer = [0; 16 * 1024];
            // L1177 (security): the deadline holds while bytes keep trickling in too; it was
            // checked only when a read would block, so a slow sender kept its slot.
            let expired = now_ms.saturating_sub(connection.opened_ms) > DEADLINE_MS;
            let finished = match connection.stream.read(&mut buffer) {
                _ if expired => Some(None),
                Ok(0) => Some(None),
                Ok(n) => {
                    connection.input.extend_from_slice(&buffer[..n]);
                    match portrait_upload::request_length(&connection.input) {
                        Ok(Some(total)) if connection.input.len() >= total => {
                            connection.input.truncate(total);
                            Some(Some(Ok(())))
                        }
                        Ok(_) => None,
                        Err(reason) => Some(Some(Err(reason))),
                    }
                }
                Err(error) if error.kind() == ErrorKind::WouldBlock => None,
                Err(_) => Some(None),
            };
            match finished {
                None => index += 1,
                Some(outcome) => {
                    let mut connection = self.connections.swap_remove(index);
                    match outcome {
                        Some(Ok(())) => {
                            let input = std::mem::take(&mut connection.input);
                            done.push((connection, input));
                        }
                        Some(Err(reason)) => answer(connection, 400, reason),
                        None => {}
                    }
                }
            }
        }
        done
    }
}

/// One short reply, then the socket closes (`Connection: close`).
fn answer(mut connection: Connection, status: u16, reason: &str) {
    let text = match status {
        200 => "OK",
        400 => "Bad Request",
        403 => "Forbidden",
        429 => "Too Many Requests",
        _ => "Internal Server Error",
    };
    let _ = connection.stream.set_nonblocking(false);
    let _ = connection
        .stream
        .set_write_timeout(Some(Duration::from_millis(500)));
    let _ = write!(
        connection.stream,
        "HTTP/1.1 {status} {text}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
    );
    println!(
        "{}",
        json!({"event":"portrait_upload","status":status,"reason":reason,
        "peer":connection.peer.ip().to_string()})
    );
}

impl Server {
    /// L071: serve portrait uploads once per step.
    pub(super) fn poll_portraits(&mut self, now_ms: u64) {
        let Some(mut http) = self.portraits.take() else {
            return;
        };
        let (bind, allow_lan) = (self.options.bind.ip(), self.options.allow_lan);
        let requests = http.poll(now_ms, |ip| {
            osfr_tools::net_policy::admits(bind, ip, allow_lan)
        });
        for (connection, input) in requests {
            let upload = match portrait_upload::parse_request(&input) {
                Ok(upload) => upload,
                Err(reason) => {
                    answer(connection, 400, reason);
                    continue;
                }
            };
            let guid = upload.character_guid;
            // L1177 (security review 2026-11-12, findings 2 and 3): the owner is the session
            // that selected this character (an unbound handshake carries the startup template's
            // guid), at this address, whose client announced this portrait's CRC in its 115
            // (`00b4fd60` sends it just before the POST). A player behind the same NAT cannot.
            let local = self.options.account_store.is_none();
            let crc = upload.portrait.as_ref().map(|(_, crc)| *crc);
            let owner = self.sessions.iter().find(|(_, session)| {
                session.plays(guid, local)
                    && session.peer.ip() == connection.peer.ip()
                    && crc.is_some()
                    && session.portrait_crc == crc
            });
            let Some((owner_key, owner)) = owner.map(|(key, session)| (*key, session.id)) else {
                answer(connection, 403, "not this address's character");
                continue;
            };
            if http
                .last_upload
                .get(&guid)
                .is_some_and(|last| now_ms.saturating_sub(*last) < COOLDOWN_MS)
            {
                answer(connection, 429, "one upload a minute");
                continue;
            }
            if let Err(error) = portrait_upload::store(&http.root, &upload) {
                eprintln!("portrait store: {error}");
                answer(connection, 500, "store failed");
                continue;
            }
            http.last_upload.insert(guid, now_ms);
            if let Some(session) = self.sessions.get_mut(&owner_key) {
                session.portrait_crc = None; // one 115 admits one upload
            }
            answer(connection, 200, "stored");
            self.show_portrait(owner, guid, &http.root);
        }
        self.portraits = Some(http);
    }

    /// The fresh images to their owner and to every nearby session that knows the character.
    fn show_portrait(&mut self, owner: SessionId, guid: u64, root: &Path) {
        let Some(player) = self.world.players.get(owner) else {
            return;
        };
        let mut recipients = vec![owner];
        recipients.extend(
            self.world
                .players
                .nearby_sessions(player.zone, player.position)
                .into_iter()
                .filter(|other| *other != owner),
        );
        for recipient in recipients {
            for provider in ["Headshot", "Portrait"] {
                let body = fotomat::request_body(guid, provider);
                let Some(reply) =
                    fotomat::request_with(&body, recipient, &self.world.players, Some(root))
                else {
                    continue;
                };
                if let Some(session) = self
                    .world
                    .session_keys
                    .get(&recipient)
                    .and_then(|key| self.sessions.get_mut(key))
                    && let Err(error) = session
                        .sender
                        .queue_message(&delivery::wrap_application(&reply))
                {
                    eprintln!("portrait delivery: {error}");
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::f120_fishing_hour_reconnect::drain_replies;
    use super::super::tests::test_support::{self, Scene, Scratch};
    use super::*;

    /// A 70x70 RGB headshot, the smallest upload the client makes.
    fn headshot() -> Vec<u8> {
        png(70, 70)
    }

    /// The 180x330 portrait the client always sends beside the headshot (`00b4fd60`).
    fn portrait() -> Vec<u8> {
        png(180, 330)
    }

    fn png(width: u32, height: u32) -> Vec<u8> {
        let stride = 1 + width as usize * 3;
        let rows = vec![0; stride * height as usize];
        let mut out = vec![137, 80, 78, 71, 13, 10, 26, 10];
        let mut ihdr = width.to_be_bytes().to_vec();
        ihdr.extend(height.to_be_bytes());
        ihdr.extend([8, 2, 0, 0, 0]);
        for (kind, data) in [
            (b"IHDR", ihdr),
            (
                b"IDAT",
                miniz_oxide::deflate::compress_to_vec_zlib(&rows, 6),
            ),
            (b"IEND", Vec::new()),
        ] {
            out.extend((data.len() as u32).to_be_bytes());
            let start = out.len();
            out.extend(kind);
            out.extend(&data);
            let crc = portrait_upload::crc32(&out[start..]);
            out.extend(crc.to_be_bytes());
        }
        out
    }

    fn upload(guid: u64) -> Vec<u8> {
        let mut body = format!(
            "--AaBb432101234bBaA\r\nContent-Disposition: form-data; name=\"imageType\"\r\n\r\nportrait\r\n\
             --AaBb432101234bBaA\r\nContent-Disposition: form-data; name=\"characterId\"\r\n\r\n{guid}\r\n\
             --AaBb432101234bBaA\r\nContent-Disposition: form-data; name=\"thumbnailFile\"; filename=\"headshot.png\"\r\nContent-Type: image/png\r\n\r\n"
        )
        .into_bytes();
        body.extend(headshot());
        body.extend(
            b"\r\n--AaBb432101234bBaA\r\nContent-Disposition: form-data; name=\"imageFile\"; \
              filename=\"portrait.png\"\r\nContent-Type: image/png\r\n\r\n",
        );
        body.extend(portrait());
        body.extend(b"\r\n--AaBb432101234bBaA--\r\n\r\n");
        let mut raw = format!(
            "POST /image HTTP/1.1\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n",
            portrait_upload::CONTENT_TYPE,
            body.len()
        )
        .into_bytes();
        raw.extend(body);
        raw
    }

    /// Send one request to the gateway's port and poll until its status line comes back.
    fn post(server: &mut Server, raw: &[u8], now: u64) -> u16 {
        let address = server.local_addr().unwrap();
        let mut client = TcpStream::connect(address).unwrap();
        client.write_all(raw).unwrap();
        client.set_nonblocking(true).unwrap();
        let mut reply = Vec::new();
        for _ in 0..400 {
            server.poll_portraits(now);
            let mut buffer = [0; 256];
            match client.read(&mut buffer) {
                Ok(0) => break,
                Ok(n) => reply.extend_from_slice(&buffer[..n]),
                Err(error) if error.kind() == ErrorKind::WouldBlock => {
                    std::thread::sleep(Duration::from_millis(5))
                }
                Err(_) => break,
            }
        }
        let text = String::from_utf8_lossy(&reply);
        text.split(' ')
            .nth(1)
            .and_then(|code| code.parse().ok())
            .unwrap_or(0)
    }

    /// L071: only the address that plays a character may upload its portrait, once a minute; the
    /// rebuilt image is stored by guid and reaches the player at once as a 156/3.
    #[test]
    fn only_the_players_own_address_uploads_and_the_portrait_is_shown() {
        if !test_support::bin_test_target("portrait") {
            return;
        }
        let scratch = Scratch::new("l071-portrait");
        let options = test_support::options(scratch.state());
        if !test_support::fixtures_present(&options) {
            return;
        }
        let mut server = Server::new(options).unwrap();
        let root = server
            .portraits
            .as_ref()
            .expect("listening on the gateway port")
            .root();
        assert_eq!(server.world.portraits.as_deref(), Some(&*root));
        let mut scene = Scene::new(&server.options, 71);
        test_support::enter_world(&mut scene, &server.options);
        let _ = scene.drain(300);
        let guid = scene.session.character_guid;
        let key = SessionKey {
            peer: scene.session.peer,
            connection_id: scene.session.request.id,
        };
        assert!(
            key.peer.ip().is_loopback(),
            "the test client shares the session's address"
        );
        let session_id = scene.session.id;
        server.world = scene.world;
        server.world.portraits = Some(root.clone());
        server.world.session_keys.insert(session_id, key);
        server.sessions.insert(key, scene.session);

        // The client's 115 (u16 115, u32 CRC-32 of portrait.png) through the session's real
        // route, as `00b4fd60` sends it just before the POST.
        let arm = |server: &mut Server| {
            let mut notice = fotomat::CRC_OPCODE.to_le_bytes().to_vec();
            notice.extend(portrait_upload::crc32(&portrait()).to_le_bytes());
            let session = server.sessions.get_mut(&key).unwrap();
            assert!(session.route_application(&notice, &mut server.world, 1_000).is_empty());
        };

        assert_eq!(
            post(&mut server, &upload(guid + 1), 1_000),
            403,
            "not this address's character"
        );
        assert!(portrait_upload::image(&root, guid + 1, "Headshot").is_empty());
        assert_eq!(
            post(
                &mut server,
                b"POST /image HTTP/1.1\r\nContent-Length: 3\r\n\r\nabc",
                1_000
            ),
            400
        );
        // L1177: no 115 from this session, no upload (another player behind the same address).
        assert_eq!(post(&mut server, &upload(guid), 1_000), 403, "no announced CRC");
        assert!(portrait_upload::image(&root, guid, "Headshot").is_empty());
        arm(&mut server);
        assert_eq!(post(&mut server, &upload(guid), 1_000), 200);
        let stored = portrait_upload::image(&root, guid, "Headshot");
        assert_eq!(
            stored,
            portrait_upload::rebuild_png(&headshot(), (70, 70)).unwrap()
        );
        let replies = drain_replies(server.sessions.get_mut(&key).unwrap(), 1_100);
        assert!(
            replies
                .iter()
                .any(|reply| reply.windows(stored.len()).any(|w| w == stored)),
            "the player sees their new headshot in a 156/3"
        );
        assert_eq!(
            post(&mut server, &upload(guid), 61_001),
            403,
            "one 115 admits one upload"
        );
        arm(&mut server);
        assert_eq!(
            post(&mut server, &upload(guid), 30_000),
            429,
            "one upload a minute"
        );
        assert_eq!(post(&mut server, &upload(guid), 61_001), 200);

        // L1177 (finding 2): on a realm with an account store, only a session that selected
        // its character owns it; an unbound handshake carries the startup template's guid.
        server.options.account_store = Some(scratch.state().with_file_name("accounts.json"));
        server.sessions.get_mut(&key).unwrap().selected_character = false;
        arm(&mut server);
        assert_eq!(post(&mut server, &upload(guid), 130_000), 403, "not selected");
        server.sessions.get_mut(&key).unwrap().selected_character = true;
        assert_eq!(post(&mut server, &upload(guid), 130_000), 200);
    }

    /// L1177 (security review 2026-11-12, finding 4): a request still arriving byte by byte is
    /// closed at the deadline; the deadline was checked only when a read would block.
    #[test]
    fn a_trickling_request_is_closed_at_its_deadline() {
        if !test_support::bin_test_target("portrait") {
            return;
        }
        let scratch = Scratch::new("l1177-portrait-deadline");
        let options = test_support::options(scratch.state());
        if !test_support::fixtures_present(&options) {
            return;
        }
        let mut server = Server::new(options).unwrap();
        let address = server.local_addr().unwrap();
        let mut client = TcpStream::connect(address).unwrap();
        client.write_all(b"POST /image HTTP/1.1\r\n").unwrap();
        let mut accepted = false;
        for _ in 0..400 {
            server.poll_portraits(1_000);
            if server.portraits.as_ref().unwrap().connections.len() == 1 {
                accepted = true;
                break;
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        assert!(accepted, "the connection is open");
        client.write_all(b"X").unwrap();
        std::thread::sleep(Duration::from_millis(100));
        server.poll_portraits(1_000 + DEADLINE_MS + 1);
        assert!(
            server.portraits.as_ref().unwrap().connections.is_empty(),
            "closed although a byte was readable"
        );
    }
}
