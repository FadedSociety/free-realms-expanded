//! GM panel (G1): the web admin panel's HTTP API and files, served on their own threads.
//!
//! - **Where:** off unless asked for. `--admin-port N` listens on 127.0.0.1:N; `--admin-bind
//!   IP:PORT` on any address, for remote use (PLAN 4.8). Both need `--account-store`. The panel's
//!   files come from `--admin-web DIR` (default `admin-web/` beside the executable, else the
//!   crate's own `admin-web/`) and item icons from the client install, `--admin-client DIR`
//!   (default: the folder above `--client-logs`).
//! - **Login:** there are no passwords (owner: launcher-held Ed25519 keys; the store keeps public
//!   keys only). `POST /admin/api/login/challenge` gives a nonce; the panel signs
//!   `login_auth::signing_message(server_id, nonce, handle)` with the staff member's launcher key
//!   (`key.ed25519`, the 32-byte seed), where `server_id` is the panel's own (sha256 of
//!   `osfr-admin-panel-v1`), so a panel signature never opens a game login nor the reverse; and
//!   `POST /admin/api/login` checks it through `login_auth::LoginAuth` against the account store.
//!   Moderators and admins only, and never a banned account. The answer is a random session token
//!   with an expiry (cookie `osfr_admin`, HttpOnly, SameSite=Strict, or `Authorization: Bearer`)
//!   and a CSRF token that every later POST echoes in `X-CSRF-Token`. Role and ban are re-read
//!   from the store on every request, as `Gateway::refresh_role` does for chat.
//! - **Checks:** a loopback bind takes only loopback `Host` names (DNS rebinding); an `Origin`
//!   must match the `Host`; a cross-site `Sec-Fetch-Site` is refused; heads and bodies are
//!   capped; logins are rate limited per address and per handle (`LoginFailures`).
//! - **The world:** these threads never touch game state. A route that needs it sends a closure
//!   to `Server::poll_admin`, which runs a few per step on the world loop and answers at once.
//!   File work (the account store, saves, the audit log, icons) stays on the HTTP thread.
//! - **Routes:** `lab/autopilot/opus/pieces/gm-panel/api-served.md`. Mail, inventory, coins,
//!   quests, jobs, undo, moves, powers, items, names and the map call G2, G3 and G5's facades
//!   (`admin_wire.rs`).
use super::*;
use osfr_tools::account_store::AccountRole;
use osfr_tools::admin::{self, Staff, audit, moderation::Op, roles, roster};
use osfr_tools::login_auth::{AUTH_LIFETIME, AuthError, LoginAuth, LoginFailures, login_handle_hash};
use osfr_tools::moderation::{self as rules, unix_now};
use osfr_tools::net_policy::KeyedBuckets;
use serde::Deserialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::VecDeque;
use std::io::{Read, Write};
use std::net::{IpAddr, TcpStream};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{MutexGuard, PoisonError, mpsc};
use subtle::ConstantTimeEq;

#[path = "admin_wire.rs"]
mod wire;

/// (R) Local bounds: request head and body, connections at once, staff sessions, a panel file.
const MAX_HEAD: usize = 8 * 1024;
pub(super) const MAX_BODY: usize = 64 * 1024;
const MAX_HEADERS: usize = 64;
const MAX_CONNECTIONS: usize = 16;
const MAX_SESSIONS: usize = 64;
const MAX_FILE: u64 = 8 * 1024 * 1024;
const IO_TIMEOUT: Duration = Duration::from_secs(5);
/// How long a request waits for the world loop before answering 503.
const WORLD_WAIT: Duration = Duration::from_secs(3);
/// (R) A staff session lasts this long after login.
pub(super) const SESSION_TTL: Duration = Duration::from_secs(8 * 60 * 60);
/// World jobs run per step, and step gaps kept for the status page.
const JOBS_PER_STEP: usize = 16;
const STEP_SAMPLES: usize = 1024;
/// (R) Login attempts per address: a burst of 10, then one every two seconds.
const LOGIN_BURST: f64 = 10.0;
const LOGIN_RATE: f64 = 0.5;
const COOKIE: &str = "osfr_admin";
const HEX: &[u8; 16] = b"0123456789abcdef";

type Job = Box<dyn FnOnce(&mut Server) + Send>;
type Answer = Result<Reply, Reply>;

/// Where the panel listens and what it serves.
pub(super) struct Config {
    pub(super) bind: SocketAddr,
    pub(super) web: PathBuf,
    pub(super) client: Option<PathBuf>,
    pub(super) session_ttl: Duration,
}

impl Config {
    /// The `--admin-*` options; `None` when neither `--admin-port` nor `--admin-bind` is given.
    pub(super) fn from_args(
        mut args: impl Iterator<Item = String>,
        options: &Options,
    ) -> Result<Option<Self>, String> {
        let (mut bind, mut web, mut client) = (None, None, None);
        while let Some(key) = args.next() {
            if !matches!(
                key.as_str(),
                "--admin-port" | "--admin-bind" | "--admin-web" | "--admin-client"
            ) {
                continue;
            }
            let value = args.next().ok_or(format!("{key} needs a value"))?;
            match key.as_str() {
                "--admin-port" => {
                    let port: u16 = value.parse().map_err(|_| "--admin-port takes a port")?;
                    bind = Some(SocketAddr::from(([127, 0, 0, 1], port)));
                }
                "--admin-bind" => {
                    bind = Some(value.parse().map_err(|_| "--admin-bind takes IP:PORT")?)
                }
                "--admin-web" => web = Some(PathBuf::from(value)),
                _ => client = Some(PathBuf::from(value)),
            }
        }
        let Some(bind) = bind else { return Ok(None) };
        let beside_exe = env::current_exe()
            .ok()
            .and_then(|exe| Some(exe.parent()?.join("admin-web")))
            .filter(|dir| dir.is_dir());
        let web = web
            .or(beside_exe)
            .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("admin-web"));
        let client = client.or_else(|| Some(options.client_logs.as_ref()?.parent()?.to_owned()));
        Ok(Some(Self {
            bind,
            web,
            client,
            session_ttl: SESSION_TTL,
        }))
    }
}

/// The world loop's end of the panel: jobs from the HTTP threads and the step clock.
pub(super) struct AdminHttp {
    jobs: mpsc::Receiver<Job>,
    pub(super) address: SocketAddr,
    stop: Arc<AtomicBool>,
    last_step: Option<Instant>,
    /// Microseconds between consecutive steps (work plus at most 10 ms of idle socket wait).
    steps: VecDeque<u32>,
}

impl Drop for AdminHttp {
    /// Stop accepting: set the flag, then wake the blocking `accept` with one connection.
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        let mut wake = self.address;
        if wake.ip().is_unspecified() {
            wake.set_ip(IpAddr::from([127, 0, 0, 1]));
        }
        let _ = TcpStream::connect_timeout(&wake, Duration::from_millis(200));
    }
}

/// One signed-in staff member.
struct StaffSession {
    digest: [u8; 32],
    csrf: String,
    account: u64,
    name: String,
    expires: Instant,
}

/// What the HTTP threads share.
struct Shared {
    session_ttl: Duration,
    web: PathBuf,
    /// The address actually bound (a test binds port 0).
    address: SocketAddr,
    store: PathBuf,
    saves: PathBuf,
    content: Option<PathBuf>,
    audit: PathBuf,
    jobs: mpsc::Sender<Job>,
    sessions: Mutex<Vec<StaffSession>>,
    auth: Mutex<LoginAuth>,
    failures: Mutex<LoginFailures>,
    addresses: Mutex<KeyedBuckets<IpAddr>>,
    icons: Option<admin::icons::Icons>,
    /// The item tables, loaded on first use (item search, names in audit rows, mail attachments).
    catalogue: std::sync::OnceLock<Option<osfr_tools::inventory::Inventory>>,
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

/// The panel's own login server id: `login_auth` signs it into every transcript, so a panel
/// signature and a game-login signature can never stand in for each other.
fn panel_server_id() -> [u8; 32] {
    Sha256::digest(b"osfr-admin-panel-v1").into()
}

/// Bind a listener and print `<name>_listening` or `<name>_unavailable` (Studio uses it too). A
/// port already taken leaves that listener off, never the server.
pub(super) fn bind_logged(address: SocketAddr, name: &str, nonblocking: bool) -> Option<TcpListener> {
    let bound = TcpListener::bind(address).and_then(|listener| {
        listener.set_nonblocking(nonblocking)?;
        let at = listener.local_addr()?;
        Ok((listener, at))
    });
    match bound {
        Ok((listener, at)) => {
            println!("{}", json!({"event": format!("{name}_listening"), "address": at.to_string()}));
            Some(listener)
        }
        Err(error) => {
            println!("{}", json!({"event": format!("{name}_unavailable"), "reason": error.to_string()}));
            None
        }
    }
}

/// `run`'s entry: the panel from the command line, or `None` when it is off or cannot bind.
pub(super) fn open_from_args(
    options: &Options,
) -> Result<Option<Box<AdminHttp>>, Box<dyn std::error::Error>> {
    match Config::from_args(env::args().skip(1), options)? {
        Some(config) => Ok(open(config, options)?),
        None => Ok(None),
    }
}

/// Start the listener thread. `Err` for a configuration the panel cannot run with.
pub(super) fn open(config: Config, options: &Options) -> Result<Option<Box<AdminHttp>>, String> {
    let store = options
        .account_store
        .clone()
        .ok_or("the admin panel needs --account-store")?;
    let audit_dir = audit::dir_for(&store);
    fs::create_dir_all(&audit_dir).map_err(|error| format!("admin audit folder: {error}"))?;
    let Some(listener) = bind_logged(config.bind, "admin", false) else {
        return Ok(None);
    };
    let address = listener.local_addr().map_err(|error| error.to_string())?;
    let saves = paths::character_state_path(&options.state, 0)
        .parent()
        .map_or_else(|| PathBuf::from("."), Path::to_owned);
    let (jobs, receiver) = mpsc::channel();
    let shared = Arc::new(Shared {
        session_ttl: config.session_ttl,
        web: config.web,
        address,
        store,
        saves,
        content: options.content.clone(),
        audit: audit_dir,
        jobs,
        sessions: Mutex::new(Vec::new()),
        auth: Mutex::new(LoginAuth::new(panel_server_id())),
        failures: Mutex::new(LoginFailures::default()),
        addresses: Mutex::new(KeyedBuckets::new(LOGIN_RATE, LOGIN_BURST, 1024)),
        icons: config
            .client
            .map(|install| admin::icons::Icons::new(install, options.content.clone())),
        catalogue: std::sync::OnceLock::new(),
    });
    let stop = Arc::new(AtomicBool::new(false));
    let stopping = stop.clone();
    std::thread::Builder::new()
        .name("admin-http".into())
        .spawn(move || serve(listener, shared, stopping))
        .map_err(|error| format!("admin listener thread: {error}"))?;
    Ok(Some(Box::new(AdminHttp {
        jobs: receiver,
        address,
        stop,
        last_step: None,
        steps: VecDeque::with_capacity(STEP_SAMPLES),
    })))
}

/// Accept connections; each gets its own short-lived thread, at most `MAX_CONNECTIONS` at once
/// (the rest are closed unanswered).
fn serve(listener: TcpListener, shared: Arc<Shared>, stop: Arc<AtomicBool>) {
    let active = Arc::new(AtomicUsize::new(0));
    for stream in listener.incoming() {
        if stop.load(Ordering::Relaxed) {
            return;
        }
        let Ok(stream) = stream else { continue };
        if active.fetch_add(1, Ordering::AcqRel) >= MAX_CONNECTIONS {
            active.fetch_sub(1, Ordering::AcqRel);
            continue;
        }
        let (shared, done) = (shared.clone(), active.clone());
        let spawned = std::thread::Builder::new()
            .name("admin-request".into())
            .spawn(move || {
                handle(stream, &shared);
                done.fetch_sub(1, Ordering::AcqRel);
            });
        if spawned.is_err() {
            active.fetch_sub(1, Ordering::AcqRel);
        }
    }
}

fn handle(mut stream: TcpStream, shared: &Shared) {
    let _ = stream.set_read_timeout(Some(IO_TIMEOUT));
    let _ = stream.set_write_timeout(Some(IO_TIMEOUT));
    let peer = stream
        .peer_addr()
        .map_or(IpAddr::from([0, 0, 0, 0]), |peer| peer.ip());
    let reply = match read_request(&mut stream) {
        Ok(request) => route(shared, &request, peer),
        Err(reply) => reply,
    };
    let _ = reply.send(&mut stream);
}

pub(super) struct Request {
    method: String,
    path: String,
    query: String,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
}

impl Request {
    fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value.as_str())
    }

    /// A query parameter, percent-decoded; blank counts as absent.
    fn param(&self, name: &str) -> Option<String> {
        self.query
            .split('&')
            .filter_map(|pair| pair.split_once('='))
            .find(|(key, _)| *key == name)
            .map(|(_, value)| studio_decode(value))
            .filter(|value| !value.trim().is_empty())
    }
}

/// One request: the head up to `MAX_HEAD`, then a `Content-Length` body up to `MAX_BODY`.
fn read_request(stream: &mut impl Read) -> Result<Request, Reply> {
    let mut data = Vec::new();
    let mut chunk = [0u8; 4096];
    let head_end = loop {
        if let Some(at) = data.windows(4).position(|w| w == b"\r\n\r\n") {
            break at + 4;
        }
        if data.len() > MAX_HEAD {
            return Err(error(431, "request head too large"));
        }
        let read = stream
            .read(&mut chunk)
            .map_err(|_| error(408, "request timed out"))?;
        if read == 0 {
            return Err(error(400, "connection closed before the request head"));
        }
        data.extend_from_slice(&chunk[..read]);
    };
    if head_end > MAX_HEAD {
        return Err(error(431, "request head too large"));
    }
    let head = std::str::from_utf8(&data[..head_end]).map_err(|_| error(400, "request head is not UTF-8"))?;
    let mut lines = head.split("\r\n");
    let mut first = lines.next().unwrap_or_default().split(' ');
    let (method, target) = (first.next().unwrap_or_default(), first.next().unwrap_or_default());
    let mut headers = Vec::new();
    for line in lines.filter(|line| !line.is_empty()) {
        let (name, value) = line
            .split_once(':')
            .ok_or_else(|| error(400, "malformed header"))?;
        headers.push((name.trim().to_ascii_lowercase(), value.trim().to_owned()));
        if headers.len() > MAX_HEADERS {
            return Err(error(431, "too many headers"));
        }
    }
    if headers.iter().any(|(name, _)| name == "transfer-encoding") {
        return Err(error(411, "send the body with a Content-Length"));
    }
    let length = match headers.iter().find(|(name, _)| name == "content-length") {
        Some((_, value)) => value
            .parse::<usize>()
            .map_err(|_| error(400, "bad Content-Length"))?,
        None => 0,
    };
    if length > MAX_BODY {
        return Err(error(413, "request body too large"));
    }
    let mut body = data[head_end..].to_vec();
    while body.len() < length {
        let read = stream
            .read(&mut chunk)
            .map_err(|_| error(408, "request timed out"))?;
        if read == 0 {
            return Err(error(400, "connection closed before the body"));
        }
        body.extend_from_slice(&chunk[..read]);
    }
    body.truncate(length);
    let (path, query) = target.split_once('?').unwrap_or((target, ""));
    Ok(Request {
        method: method.to_owned(),
        path: path.to_owned(),
        query: query.to_owned(),
        headers,
        body,
    })
}

pub(super) struct Reply {
    status: u16,
    kind: &'static str,
    body: Vec<u8>,
    cache: &'static str,
    headers: Vec<String>,
}

fn json_reply(status: u16, value: Value) -> Reply {
    Reply {
        status,
        kind: "application/json",
        body: serde_json::to_vec(&value).unwrap_or_default(),
        cache: "no-store",
        headers: Vec::new(),
    }
}

fn error(status: u16, message: &str) -> Reply {
    json_reply(status, json!({ "error": message }))
}

impl Reply {
    fn send(&self, stream: &mut impl Write) -> io::Result<()> {
        let reason = match self.status {
            200 => "OK",
            301 => "Moved Permanently",
            400 => "Bad Request",
            401 => "Unauthorized",
            403 => "Forbidden",
            404 => "Not Found",
            405 => "Method Not Allowed",
            408 => "Request Timeout",
            409 => "Conflict",
            411 => "Length Required",
            413 => "Payload Too Large",
            429 => "Too Many Requests",
            431 => "Request Header Fields Too Large",
            501 => "Not Implemented",
            503 => "Service Unavailable",
            _ => "Error",
        };
        let mut head = format!(
            "HTTP/1.1 {} {reason}\r\nContent-Type: {}\r\nContent-Length: {}\r\nCache-Control: {}\r\n\
             X-Content-Type-Options: nosniff\r\nX-Frame-Options: DENY\r\nReferrer-Policy: no-referrer\r\n\
             Content-Security-Policy: default-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; frame-ancestors 'none'; base-uri 'none'; form-action 'self'\r\n\
             Connection: close\r\n",
            self.status,
            self.kind,
            self.body.len(),
            self.cache,
        );
        for header in &self.headers {
            head.push_str(header);
            head.push_str("\r\n");
        }
        head.push_str("\r\n");
        stream.write_all(head.as_bytes())?;
        stream.write_all(&self.body)?;
        stream.flush()
    }
}

fn route(shared: &Shared, request: &Request, peer: IpAddr) -> Reply {
    if let Err(reply) = same_origin(shared, request) {
        return reply;
    }
    let path = request.path.as_str();
    if path == "/admin" {
        let mut reply = error(301, "moved to /admin/");
        reply.headers.push("Location: /admin/".into());
        return reply;
    }
    let answer = if let Some(rest) = path.strip_prefix("/admin/api/") {
        api(shared, request, rest, peer)
    } else if let Some(id) = path.strip_prefix("/admin/icon/") {
        icon(shared, request, id)
    } else if let Some(zone) = path.strip_prefix("/admin/map-image/") {
        wire::map_image(shared, request, zone)
    } else if request.method != "GET" {
        Err(error(405, "the panel's files are GET only"))
    } else if path.starts_with("/admin/") {
        Ok(static_file(&shared.web, path))
    } else {
        Err(error(404, "not found"))
    };
    answer.unwrap_or_else(|reply| reply)
}

/// A loopback bind answers loopback `Host` names only (a rebound DNS name is refused); an
/// `Origin` must be this host; a browser's cross-site fetch is refused.
fn same_origin(shared: &Shared, request: &Request) -> Result<(), Reply> {
    let refused = || error(403, "cross-origin request refused");
    let host = request
        .header("host")
        .ok_or_else(|| error(400, "Host header required"))?;
    if shared.address.ip().is_loopback() {
        let (name, port) = host.rsplit_once(':').unwrap_or((host, ""));
        if !matches!(name, "127.0.0.1" | "localhost" | "[::1]")
            || port != shared.address.port().to_string()
        {
            return Err(refused());
        }
    }
    if let Some(origin) = request.header("origin")
        && origin != format!("http://{host}")
        && origin != format!("https://{host}")
    {
        return Err(refused());
    }
    if request.header("sec-fetch-site") == Some("cross-site") {
        return Err(refused());
    }
    Ok(())
}

/// A staff member's request, after the session and the store's current role both check out.
struct Caller {
    account: u64,
    name: String,
    role: AccountRole,
    csrf: String,
    expires_in: Duration,
    digest: [u8; 32],
}

impl Caller {
    fn staff(&self) -> Staff {
        Staff {
            account_id: self.account,
            name: self.name.clone(),
        }
    }

    fn need(&self, action: &str) -> Result<(), Reply> {
        if roles::allowed(self.role, action) {
            Ok(())
        } else {
            Err(error(403, "your role cannot do this"))
        }
    }
}

fn session_token(request: &Request) -> Option<&str> {
    if let Some(token) = request
        .header("authorization")
        .and_then(|value| value.strip_prefix("Bearer "))
    {
        return Some(token.trim());
    }
    request
        .header("cookie")?
        .split(';')
        .find_map(|pair| pair.trim().strip_prefix("osfr_admin="))
}

fn load_store(shared: &Shared) -> Result<AccountStore, Reply> {
    AccountStore::load(&shared.store).map_err(|_| error(503, "the account store is busy or unreadable"))
}

fn authenticate(shared: &Shared, request: &Request) -> Result<Caller, Reply> {
    let unauthorized = || error(401, "sign in first");
    let token = session_token(request)
        .filter(|token| token.len() == 64)
        .ok_or_else(unauthorized)?;
    let digest: [u8; 32] = Sha256::digest(token.as_bytes()).into();
    let now = Instant::now();
    let found = {
        let mut sessions = lock(&shared.sessions);
        sessions.retain(|session| session.expires > now);
        // Every session is compared, in constant time each, so the scan leaks no prefix.
        let mut found = None;
        for session in sessions.iter() {
            if bool::from(session.digest.ct_eq(&digest)) {
                found = Some((session.account, session.name.clone(), session.csrf.clone(), session.expires));
            }
        }
        found
    };
    let (account, name, csrf, expires) = found.ok_or_else(unauthorized)?;
    let store = load_store(shared)?;
    let role = store
        .account_role(account)
        .filter(|role| roles::is_staff(*role))
        .filter(|_| !store.sanctions(account).banned(unix_now()));
    let Some(role) = role else {
        lock(&shared.sessions).retain(|session| session.digest != digest);
        return Err(error(403, "this account is not staff any more"));
    };
    Ok(Caller {
        account,
        name,
        role,
        csrf,
        expires_in: expires.saturating_duration_since(now),
        digest,
    })
}

fn parse<T: serde::de::DeserializeOwned>(request: &Request) -> Result<T, Reply> {
    serde_json::from_slice(&request.body).map_err(|error_| error(400, &format!("bad JSON body: {error_}")))
}

pub(super) fn hex(bytes: &[u8]) -> String {
    bytes
        .iter()
        .flat_map(|byte| [HEX[usize::from(byte >> 4)] as char, HEX[usize::from(byte & 15)] as char])
        .collect()
}

fn unhex(text: &str) -> Option<Vec<u8>> {
    if text.len() % 2 != 0 || text.len() > 256 {
        return None;
    }
    (0..text.len())
        .step_by(2)
        .map(|at| u8::from_str_radix(text.get(at..at + 2)?, 16).ok())
        .collect()
}

fn random_hex(bytes: usize) -> Result<String, Reply> {
    let mut random = vec![0u8; bytes];
    getrandom::getrandom(&mut random).map_err(|_| error(503, "secure randomness unavailable"))?;
    Ok(hex(&random))
}

/// Run `work` on the world loop (`Server::poll_admin`) and wait for its answer.
fn world<T: Send + 'static>(
    shared: &Shared,
    work: impl FnOnce(&mut Server) -> T + Send + 'static,
) -> Result<T, Reply> {
    world_within(shared, WORLD_WAIT, work)
}

/// `world` with its own wait: a job that edits or loads a character (a save written, a backup
/// taken) waits longer, so its answer and audit row are not lost to a slow step.
fn world_within<T: Send + 'static>(
    shared: &Shared,
    patience: Duration,
    work: impl FnOnce(&mut Server) -> T + Send + 'static,
) -> Result<T, Reply> {
    let (answer, wait) = mpsc::channel();
    shared
        .jobs
        .send(Box::new(move |server: &mut Server| {
            let _ = answer.send(work(server));
        }))
        .map_err(|_| error(503, "the server is stopping"))?;
    wait.recv_timeout(patience)
        .map_err(|_| error(503, "the world loop did not answer in time"))
}

/// Append one audit row; the action already happened, so a failure is reported, not hidden.
fn record(shared: &Shared, row: &audit::Row) -> Result<(), Reply> {
    audit::record(&shared.audit, row).map_err(|failure| {
        eprintln!("admin audit: {failure}");
        error(500, "the action was applied but its audit row was not written")
    })
}

/// The status for a refusal string from the moderation rules.
fn refusal(message: &'static str) -> Reply {
    let status = if message.contains("no single character") {
        404
    } else if message.contains("busy") {
        503
    } else if message.contains("minutes") {
        400
    } else if message.contains("role") || message.contains("own account") {
        403
    } else {
        409
    };
    error(status, message)
}

fn character_id(text: &str) -> Result<u64, Reply> {
    text.parse::<u64>()
        .ok()
        .filter(|id| *id > 0)
        .ok_or_else(|| error(400, "the player id is a character id"))
}

fn api(shared: &Shared, request: &Request, rest: &str, peer: IpAddr) -> Answer {
    let method = request.method.as_str();
    match (method, rest) {
        ("POST", "login/challenge") => return challenge(shared, request, peer),
        ("POST", "login") => return login(shared, request, peer),
        _ => {}
    }
    let caller = authenticate(shared, request)?;
    if method == "POST"
        && !request
            .header("x-csrf-token")
            .is_some_and(|token| bool::from(token.as_bytes().ct_eq(caller.csrf.as_bytes())))
    {
        return Err(error(403, "missing or wrong X-CSRF-Token"));
    }
    let words: Vec<&str> = rest.split('/').collect();
    match (method, words.as_slice()) {
        ("POST", ["logout"]) => Ok(logout(shared, &caller)),
        ("GET", ["me"]) => Ok(json_reply(200, json!({
            "name": caller.name, "account": caller.account, "role": roles::name(caller.role),
            "permissions": roles::permissions(caller.role), "csrf": caller.csrf,
            "expires_s": caller.expires_in.as_secs(),
        }))),
        ("GET", ["status"]) => caller.need("view").and_then(|()| status(shared)),
        ("GET", ["players", "online"]) => {
            caller.need("view")?;
            let mut rows = world(shared, online_now)?;
            roster::sort_online(&mut rows);
            Ok(json_reply(200, json!(rows)))
        }
        ("GET", ["players"]) => caller.need("view").and_then(|()| players(shared, request)),
        ("GET", ["players", id]) => caller.need("view").and_then(|()| profile(shared, id)),
        ("POST", ["players", id, "moderate"]) => moderate(shared, &caller, request, id),
        ("POST", ["players", id, "notes"]) => note(shared, &caller, request, id),
        ("POST", ["announce"]) => announce(shared, &caller, request),
        ("GET", ["audit"]) => caller.need("view").and_then(|()| audit_page(shared, request)),
        // The other builders' facades (`admin_wire.rs`).
        ("GET", ["players", id, "inventory"]) => wire::inventory(shared, &caller, id),
        ("POST", ["players", id, "inventory", op @ ("add" | "remove")]) => {
            wire::inventory_edit(shared, &caller, request, id, op)
        }
        ("POST", ["players", id, "coins"]) => wire::coins(shared, &caller, request, id),
        ("POST", ["players", id, "quests"]) => wire::quests(shared, &caller, request, id),
        ("POST", ["players", id, "jobs"]) => wire::jobs(shared, &caller, request, id),
        ("POST", ["undo"]) => wire::undo(shared, &caller, request),
        ("GET", ["items"]) => wire::items(shared, &caller, request),
        ("GET", ["names"]) => wire::names(shared, &caller),
        ("GET", ["players", id, "mail"]) => wire::mailbox(shared, &caller, id),
        ("GET", ["mail", "jobs"]) => wire::mail_jobs(&caller),
        ("POST", ["mail"]) => wire::send_mail(shared, &caller, request),
        ("POST", ["players", id, "move"]) => wire::move_player(shared, &caller, request, id),
        ("GET", ["powers"]) => wire::powers_now(shared, &caller),
        ("POST", ["powers"]) => wire::power(shared, &caller, request),
        ("GET", ["map", zone]) => wire::map(shared, &caller, zone),
        ("GET", ["events"]) => stub(&caller, "view", "the event stream (poll every 2 s instead)"),
        _ => Err(error(404, "no such admin route")),
    }
}

/// A registered route with nothing behind it: role-checked first, then 501.
fn stub(caller: &Caller, action: &str, what: &str) -> Answer {
    caller.need(action)?;
    Err(error(501, &format!("{what} is not built yet; it is wired in at integration")))
}

/// Per-address login budget; every login and challenge request pays one.
fn allow_address(shared: &Shared, peer: IpAddr) -> Result<(), Reply> {
    if lock(&shared.addresses).allow(peer, Instant::now()) {
        Ok(())
    } else {
        Err(error(429, "too many login attempts; wait a minute"))
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ChallengeBody {
    account: String,
}

fn challenge(shared: &Shared, request: &Request, peer: IpAddr) -> Answer {
    allow_address(shared, peer)?;
    let body: ChallengeBody = parse(request)?;
    let store = load_store(shared)?;
    match lock(&shared.auth).issue_challenge(&store, &body.account, Instant::now()) {
        Ok(nonce) => Ok(json_reply(200, json!({
            "nonce": hex(&nonce), "server_id": hex(&panel_server_id()),
            "expires_s": AUTH_LIFETIME.as_secs(),
        }))),
        Err(AuthError::InvalidCredentials) => Err(error(
            400,
            "an account name is 3 to 32 letters, digits, - or _",
        )),
        Err(_) => Err(error(503, "login is busy; try again shortly")),
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LoginBody {
    account: String,
    nonce: String,
    public_key: String,
    signature: String,
}

fn login(shared: &Shared, request: &Request, peer: IpAddr) -> Answer {
    allow_address(shared, peer)?;
    let body: LoginBody = parse(request)?;
    let refused = || error(401, "login refused");
    let hash = login_handle_hash(&body.account).map_err(|_| refused())?;
    let now = Instant::now();
    if !lock(&shared.failures).allow(hash, now) {
        return Err(error(429, "too many failed logins for this account; wait a minute"));
    }
    let failed = |shared: &Shared| {
        lock(&shared.failures).failed(hash, now);
        refused()
    };
    let (Some(nonce), Some(key), Some(signature)) = (
        unhex(&body.nonce),
        unhex(&body.public_key),
        unhex(&body.signature),
    ) else {
        return Err(failed(shared));
    };
    let store = load_store(shared)?;
    let verified = {
        let mut auth = lock(&shared.auth);
        auth.verify(&store, &nonce, &key, &signature, now)
            .and_then(|token| auth.redeem_identity(&store, token.as_bytes(), now))
    };
    let (account, handle) = verified
        .ok()
        .filter(|(_, handle)| *handle == body.account)
        .ok_or_else(|| failed(shared))?;
    lock(&shared.failures).succeeded(&hash);
    let role = store
        .account_role(account)
        .filter(|role| roles::is_staff(*role))
        .filter(|_| !store.sanctions(account).banned(unix_now()))
        .ok_or_else(|| error(403, "the panel is for moderators and admins"))?;
    let (token, csrf) = (random_hex(32)?, random_hex(16)?);
    let expires = now + shared.session_ttl;
    {
        let mut sessions = lock(&shared.sessions);
        sessions.retain(|session| session.expires > now);
        if sessions.len() >= MAX_SESSIONS {
            sessions.remove(0);
        }
        sessions.push(StaffSession {
            digest: Sha256::digest(token.as_bytes()).into(),
            csrf: csrf.clone(),
            account,
            name: handle.clone(),
            expires,
        });
    }
    let staff = Staff {
        account_id: account,
        name: handle,
    };
    let unix = unix_now();
    record(shared, &audit::Row {
        time: unix,
        staff: staff.clone(),
        action: "login".into(),
        target: audit::Target::default(),
        before: Value::Null,
        after: json!({"role": roles::name(role)}),
        reason: None,
        via: "panel".into(),
    })?;
    event(json!({"event": "admin_login", "account_id": account}));
    let ttl = shared.session_ttl.as_secs();
    let mut reply = json_reply(200, json!({
        "token": token, "csrf": csrf,
        "staff": {"name": staff.name, "role": roles::name(role), "account": account},
        "expires": unix.saturating_add(ttl), "expires_s": ttl,
    }));
    reply.headers.push(format!(
        "Set-Cookie: {COOKIE}={token}; Path=/admin; HttpOnly; SameSite=Strict; Max-Age={ttl}"
    ));
    Ok(reply)
}

fn logout(shared: &Shared, caller: &Caller) -> Reply {
    lock(&shared.sessions).retain(|session| session.digest != caller.digest);
    let mut reply = json_reply(200, json!({"ok": true}));
    reply.headers.push(format!(
        "Set-Cookie: {COOKIE}=; Path=/admin; HttpOnly; SameSite=Strict; Max-Age=0"
    ));
    reply
}

/// (R) Resident memory where the OS shows it cheaply (Linux `/proc`); null elsewhere.
fn memory_mb() -> Option<u64> {
    let statm = fs::read_to_string("/proc/self/statm").ok()?;
    let pages: u64 = statm.split_whitespace().nth(1)?.parse().ok()?;
    Some(pages.saturating_mul(4096) / (1024 * 1024))
}

fn status(shared: &Shared) -> Answer {
    let (online, uptime_s, tick) = world(shared, |server| {
        let tick = server.admin.as_deref().map_or((0.0, 0.0), AdminHttp::step_ms);
        (server.world.active_sessions.len(), server.start.elapsed().as_secs(), tick)
    })?;
    let saves_ok = AccountStore::load(&shared.store).is_ok()
        && (!shared.saves.exists() || shared.saves.is_dir());
    let content = shared
        .content
        .as_deref()
        .and_then(Path::file_name)
        .map(|name| name.to_string_lossy().into_owned());
    Ok(json_reply(200, json!({
        "online": online, "uptime_s": uptime_s,
        "tick_ms": {"mean": tick.0, "max": tick.1},
        "memory_mb": memory_mb(),
        "content": {"server": env!("CARGO_PKG_VERSION"), "dir": content},
        "saves_ok": saves_ok,
    })))
}

/// Every joined character as the panel lists it, from the same world view `!gm players` reads
/// (`gm_panel_routing`): identity, registry place, public job and level.
pub(super) fn online_rows(world: &SharedWorld) -> Vec<roster::OnlineRow> {
    world
        .active_sessions
        .iter()
        .filter_map(|(guid, id)| {
            let identity = world.identities.get(guid)?;
            let player = world.players.get(*id);
            let look = world.public_looks.get(guid);
            let job = look.map(|look| look.active_profile);
            Some(roster::OnlineRow {
                character_id: *guid,
                name: format!("{} {}", identity.name.first, identity.name.last)
                    .trim()
                    .to_owned(),
                account: identity.account_id,
                role: None,
                zone: player.map(|player| player.zone.zone_id),
                zone_name: None,
                position: player.map(|player| player.position),
                job,
                level: look.and_then(|look| {
                    look.profiles
                        .iter()
                        .find(|(profile, _)| Some(*profile) == job)
                        .map(|(_, level)| *level)
                }),
                session_s: 0,
                client: None,
            })
        })
        .collect()
}

/// `online_rows` with what only the sessions know: the account role, the session's length and
/// the zone's name.
fn online_now(server: &mut Server) -> Vec<roster::OnlineRow> {
    let live: BTreeMap<u64, (AccountRole, u64, Option<String>)> = server
        .sessions
        .values()
        .filter(|session| session.selected_character)
        .map(|session| {
            let length = session.session_start.elapsed().as_secs();
            let zone = session.gateway.runtime_zone().ok().map(|zone| zone.name);
            (session.character_guid, (session.gateway.role(), length, zone))
        })
        .collect();
    let mut rows = online_rows(&server.world);
    for row in &mut rows {
        if let Some((role, length, zone)) = live.get(&row.character_id) {
            row.role = Some(roles::name(*role));
            row.session_s = *length;
            row.zone_name.clone_from(zone);
        }
    }
    rows
}

fn flag(request: &Request, name: &str) -> Result<Option<bool>, Reply> {
    match request.param(name).as_deref() {
        None => Ok(None),
        Some("true" | "1") => Ok(Some(true)),
        Some("false" | "0") => Ok(Some(false)),
        Some(_) => Err(error(400, &format!("{name} is true or false"))),
    }
}

fn page_param(request: &Request) -> Result<usize, Reply> {
    request.param("page").map_or(Ok(1), |page| {
        page.parse::<usize>()
            .ok()
            .filter(|page| (1..=100_000).contains(page))
            .ok_or_else(|| error(400, "page is a number from 1"))
    })
}

fn players(shared: &Shared, request: &Request) -> Answer {
    let q: String = request.param("q").unwrap_or_default().chars().take(64).collect();
    let role = match request.param("role") {
        Some(role) => Some(AccountRole::parse(&role).ok_or_else(|| error(400, "unknown role"))?),
        None => None,
    };
    let sort = roster::Sort::parse(&request.param("sort").unwrap_or_default())
        .ok_or_else(|| error(400, "sort is name, last_seen or created"))?;
    let query = roster::Query {
        q,
        online: flag(request, "online")?,
        role,
        banned: flag(request, "banned")?,
        muted: flag(request, "muted")?,
        sort,
        page: page_param(request)?,
    };
    let online: BTreeSet<u64> = world(shared, |server| {
        server.world.active_sessions.keys().copied().collect::<BTreeSet<u64>>()
    })?;
    let store = load_store(shared)?;
    let (total, rows) = roster::search(&store, &shared.saves, &online, &query, unix_now());
    Ok(json_reply(200, json!({
        "total": total, "page": query.page, "page_size": roster::PAGE, "rows": rows,
    })))
}

fn profile(shared: &Shared, id: &str) -> Answer {
    let guid = character_id(id)?;
    let online = world(shared, move |server| {
        online_now(server).into_iter().find(|row| row.character_id == guid)
    })?;
    let store = load_store(shared)?;
    let sources = roster::Sources {
        store: &store,
        saves: &shared.saves,
        content: shared.content.as_deref(),
        audit: &shared.audit,
    };
    roster::profile(&sources, guid, online.as_ref(), unix_now())
        .map(|profile| json_reply(200, profile))
        .ok_or_else(|| error(404, "no such character"))
}

/// The audit row's target for a stored character.
fn target(store: &AccountStore, guid: u64) -> Option<audit::Target> {
    let record = store.character(guid)?;
    Some(audit::Target {
        character_id: Some(guid),
        account_id: Some(record.owner_account_id),
        name: Some(record.identity.display_name()),
    })
}

/// The audit row's target for `guid` read from the account store at `store`: its account and
/// name when the store has it, the id alone otherwise, empty for none.
pub(super) fn audit_target(store: &Path, guid: Option<u64>) -> audit::Target {
    let Some(guid) = guid else {
        return audit::Target::default();
    };
    AccountStore::load(store)
        .ok()
        .and_then(|store| target(&store, guid))
        .unwrap_or(audit::Target {
            character_id: Some(guid),
            ..audit::Target::default()
        })
}

/// The staff member behind a chat command as the audit log names chat moderation: the account
/// and its character's name.
pub(super) fn chat_staff(session: &Session) -> Staff {
    let account = session.account_guid;
    let name = session.social_identity.as_ref().map_or_else(
        || format!("account {account}"),
        |identity| format!("{} {}", identity.name.first, identity.name.last).trim().to_owned(),
    );
    Staff {
        account_id: account,
        name,
    }
}

/// One applied staff action typed in chat, in the staff audit log; best effort like the chat
/// moderation row (a failure is printed, never undoes the action).
pub(super) fn audit_chat(options: &Options, staff: &Session, action: &str, guid: Option<u64>, after: Value) {
    let Some(store) = options.account_store.as_deref() else {
        return;
    };
    audit::record_beside(store, &audit::Row {
        time: unix_now(),
        staff: chat_staff(staff),
        action: action.to_owned(),
        target: audit_target(store, guid),
        before: Value::Null,
        after,
        reason: None,
        via: "chat".into(),
    });
}

/// One system line to every selected session of `account`; whether any received it.
fn tell_account(server: &mut Server, account: u64, line: &str) -> bool {
    let bytes = wrap_application(&osfr_tools::chat::system(line));
    let mut told = false;
    for session in server.sessions.values_mut() {
        if session.selected_character
            && session.account_guid == account
            && session.sender.queue_message(&bytes).is_ok()
        {
            told = true;
        }
    }
    told
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ModerateBody {
    op: String,
    #[serde(default)]
    minutes: Option<u32>,
    #[serde(default)]
    reason: Option<String>,
}

fn moderate(shared: &Shared, caller: &Caller, request: &Request, id: &str) -> Answer {
    let guid = character_id(id)?;
    let body: ModerateBody = parse(request)?;
    let op = Op::parse(&body.op).ok_or_else(|| {
        error(400, "op is mute, unmute, kick, ban, unban, warn, jail or release")
    })?;
    caller.need(op.word())?;
    let reason = admin::clean_text(body.reason.as_deref()).map_err(|message| error(400, message))?;
    let now = unix_now();
    let staff = caller.staff();
    let row = match op {
        Op::Jail | Op::Release => wire::jail(shared, caller, guid, op == Op::Jail, reason)?,
        Op::Warn => {
            let reason = reason.ok_or_else(|| error(400, "a warning needs a reason"))?;
            let store = load_store(shared)?;
            let (account, line) =
                admin::moderation::warn(&store, caller.account, guid, &reason).map_err(refusal)?;
            let delivered = world(shared, move |server| tell_account(server, account, &line))?;
            audit::Row {
                time: now,
                staff,
                action: "warn".into(),
                target: target(&store, guid).unwrap_or_default(),
                before: Value::Null,
                after: json!({"delivered": delivered}),
                reason: Some(reason),
                via: "panel".into(),
            }
        }
        Op::Sanction(action) => {
            let command =
                rules::Command::for_character(action, guid, body.minutes).map_err(refusal)?;
            let change = rules::run_with_change(&shared.store, caller.account, &command, now)
                .map_err(refusal)?;
            let (applied, actor) = (change.0, caller.account);
            world(shared, move |server| {
                moderation::finish(&mut server.sessions, &server.world, actor, applied)
            })?
            .map_err(refusal)?;
            let store = AccountStore::load(&shared.store).ok();
            admin::moderation::sanction_row(&staff, &command, change, store.as_ref(), reason, "panel", now)
        }
    };
    record(shared, &row)?;
    let (history, _) = roster::history(&shared.audit, row.target.account_id.unwrap_or(0), guid);
    Ok(json_reply(200, json!({"op": op.word(), "row": row, "moderation": history})))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct NoteBody {
    text: String,
}

fn note(shared: &Shared, caller: &Caller, request: &Request, id: &str) -> Answer {
    caller.need("note")?;
    let guid = character_id(id)?;
    let body: NoteBody = parse(request)?;
    let text = admin::clean_text(Some(&body.text))
        .map_err(|message| error(400, message))?
        .ok_or_else(|| error(400, "a note needs text"))?;
    let store = load_store(shared)?;
    let target = target(&store, guid).ok_or_else(|| error(404, "no such character"))?;
    let account = target.account_id.unwrap_or(0);
    record(shared, &audit::Row {
        time: unix_now(),
        staff: caller.staff(),
        action: "note".into(),
        target,
        before: Value::Null,
        after: json!({ "text": text }),
        reason: None,
        via: "panel".into(),
    })?;
    let (_, notes) = roster::history(&shared.audit, account, guid);
    Ok(json_reply(200, json!(notes)))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AnnounceBody {
    to: Value,
    text: String,
    #[serde(default)]
    style: Option<String>,
}

enum Audience {
    All,
    Zone(u32),
    Role(AccountRole),
}

fn audience(to: &Value) -> Option<Audience> {
    if to.as_str() == Some("all") {
        return Some(Audience::All);
    }
    if let Some(zone) = to.get("zone") {
        return Some(Audience::Zone(u32::try_from(zone.as_u64()?).ok()?));
    }
    AccountRole::parse(to.get("role")?.as_str()?).map(Audience::Role)
}

/// `realm broadcast`'s own line and recipient list (`delivery::broadcast_deliveries`), narrowed
/// to the audience. Both styles are the 77/2 broadcast: on screen and in chat (rc-0136 F4).
fn announce_now(server: &mut Server, audience: &Audience, text: &str) -> usize {
    let mut delivered = 0;
    for delivery in delivery::broadcast_deliveries(text, &server.world) {
        let zone = server
            .world
            .players
            .get(delivery.recipient)
            .map(|player| player.zone.zone_id);
        let Some(key) = server.world.session_keys.get(&delivery.recipient).copied() else {
            continue;
        };
        let Some(session) = server.sessions.get_mut(&key) else {
            continue;
        };
        let wanted = match audience {
            Audience::All => true,
            Audience::Zone(wanted) => zone == Some(*wanted),
            Audience::Role(role) => session.gateway.role() == *role,
        };
        if wanted && session.sender.queue_message(&delivery.bytes).is_ok() {
            delivered += 1;
        }
    }
    delivered
}

fn announce(shared: &Shared, caller: &Caller, request: &Request) -> Answer {
    caller.need("announce")?;
    let body: AnnounceBody = parse(request)?;
    let audience = audience(&body.to)
        .ok_or_else(|| error(400, "to is \"all\", {\"zone\": id} or {\"role\": name}"))?;
    let style = body.style.unwrap_or_else(|| "chat".into());
    if !matches!(style.as_str(), "chat" | "banner") {
        return Err(error(400, "style is chat or banner"));
    }
    let text = osfr_tools::world_notice::parse(&format!("broadcast {}", body.text))
        .map_err(|refused| error(400, &refused.to_string()))?
        .to_owned();
    let sent = text.clone();
    let delivered = world(shared, move |server| announce_now(server, &audience, &sent))?;
    record(shared, &audit::Row {
        time: unix_now(),
        staff: caller.staff(),
        action: "announce".into(),
        target: audit::Target::default(),
        before: Value::Null,
        after: json!({"to": body.to, "style": style, "text": text, "delivered": delivered}),
        reason: None,
        via: "panel".into(),
    })?;
    Ok(json_reply(200, json!({ "delivered": delivered })))
}

fn audit_page(shared: &Shared, request: &Request) -> Answer {
    let since = match request.param("since") {
        Some(since) => Some(since.parse::<u64>().map_err(|_| error(400, "since is unix seconds"))?),
        None => None,
    };
    let filter = audit::Filter {
        staff: request.param("staff"),
        player: request.param("player"),
        action: request.param("action"),
        since,
    };
    let page = page_param(request)?;
    let (total, rows) = audit::page(&shared.audit, &filter, page)
        .map_err(|_| error(500, "the audit log could not be read"))?;
    Ok(json_reply(200, json!({
        "total": total, "page": page, "page_size": audit::PAGE, "rows": rows,
    })))
}

fn icon(shared: &Shared, request: &Request, id: &str) -> Answer {
    if request.method != "GET" {
        return Err(error(405, "icons are GET only"));
    }
    authenticate(shared, request)?.need("view")?;
    let item = id
        .parse::<i32>()
        .map_err(|_| error(400, "the icon id is an item definition id"))?;
    let png = shared
        .icons
        .as_ref()
        .and_then(|icons| icons.png(item))
        .ok_or_else(|| error(404, "no icon for that item in the client install"))?;
    Ok(Reply {
        status: 200,
        kind: "image/png",
        body: png.to_vec(),
        cache: "private, max-age=86400",
        headers: Vec::new(),
    })
}

/// A panel file under `root`. Only plain names reach the disk: every segment is letters,
/// digits, `-`, `_` and `.`, none starts with a dot, and the resolved file must still sit under
/// the resolved root (a link out is refused too).
fn static_file(root: &Path, path: &str) -> Reply {
    let relative = match path.strip_prefix("/admin/") {
        Some("") | None => "index.html",
        Some(rest) => rest,
    };
    let plain = relative.len() <= 200
        && relative.split('/').all(|part| {
            !part.is_empty()
                && !part.starts_with('.')
                && part
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.'))
        });
    let not_found = || error(404, "not found");
    if !plain {
        return not_found();
    }
    let (Ok(root), Ok(file)) = (root.canonicalize(), root.join(relative).canonicalize()) else {
        return not_found();
    };
    if !file.starts_with(&root) || !file.is_file() {
        return not_found();
    }
    let mut body = Vec::new();
    let read = fs::File::open(&file).and_then(|file| file.take(MAX_FILE + 1).read_to_end(&mut body));
    if read.is_err() || body.len() as u64 > MAX_FILE {
        return not_found();
    }
    let kind = match file.extension().and_then(|ext| ext.to_str()).unwrap_or_default() {
        "html" => "text/html; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "js" | "mjs" => "text/javascript; charset=utf-8",
        "json" => "application/json",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "webp" => "image/webp",
        "ico" => "image/x-icon",
        "woff2" => "font/woff2",
        "txt" | "md" => "text/plain; charset=utf-8",
        _ => "application/octet-stream",
    };
    Reply {
        status: 200,
        kind,
        body,
        cache: "no-cache",
        headers: Vec::new(),
    }
}

impl AdminHttp {
    /// Mean and worst gap between steps over the last samples, in milliseconds.
    fn step_ms(&self) -> (f64, f64) {
        let count = self.steps.len().max(1) as f64;
        let sum: f64 = self.steps.iter().map(|micros| f64::from(*micros)).sum();
        let max = self.steps.iter().copied().max().unwrap_or(0);
        ((sum / count / 100.0).round() / 10.0, (f64::from(max) / 100.0).round() / 10.0)
    }
}

impl Server {
    /// Once per step: time the step and run up to `JOBS_PER_STEP` panel requests' world work.
    /// A job that fails is contained like a faulted field; its request times out with 503.
    pub(super) fn poll_admin(&mut self) {
        let Some(admin) = self.admin.as_deref_mut() else {
            return;
        };
        let now = Instant::now();
        if let Some(last) = admin.last_step.replace(now) {
            if admin.steps.len() >= STEP_SAMPLES {
                admin.steps.pop_front();
            }
            let gap = now.saturating_duration_since(last).as_micros();
            admin.steps.push_back(u32::try_from(gap).unwrap_or(u32::MAX));
        }
        let jobs: Vec<Job> = std::iter::from_fn(|| admin.jobs.try_recv().ok())
            .take(JOBS_PER_STEP)
            .collect();
        for job in jobs {
            if contain(|| {
                job(self);
                Ok(())
            })
            .is_err()
            {
                event(json!({"event": "admin_job_failed"}));
            }
        }
    }
}

#[cfg(test)]
#[path = "admin_http_tests.rs"]
mod tests;
