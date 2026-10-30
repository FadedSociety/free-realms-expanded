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
            let finished = match connection.stream.read(&mut buffer) {
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
                Err(error) if error.kind() == ErrorKind::WouldBlock => {
                    (now_ms.saturating_sub(connection.opened_ms) > DEADLINE_MS).then_some(None)
                }
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
            let owner = self.sessions.values().find(|session| {
                session.character_guid == guid && session.peer.ip() == connection.peer.ip()
            });
            let Some(owner) = owner.map(|session| session.id) else {
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
        let stride = 1 + 70 * 3;
        let rows = vec![0; stride * 70];
        let mut out = vec![137, 80, 78, 71, 13, 10, 26, 10];
        let mut ihdr = 70u32.to_be_bytes().to_vec();
        ihdr.extend(70u32.to_be_bytes());
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
        body.extend(b"\r\n--AaBb432101234bBaA--\r\n");
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
            post(&mut server, &upload(guid), 30_000),
            429,
            "one upload a minute"
        );
        assert_eq!(post(&mut server, &upload(guid), 61_001), 200);
    }
}
