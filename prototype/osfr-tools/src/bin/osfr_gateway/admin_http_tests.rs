//! GM panel (G1) through the production path: a real `Server` with account-store sessions bound
//! by handoff tickets (the L042 rig), the panel's listener on an ephemeral loopback port, real
//! HTTP over a socket from a client thread, and the world loop driven by `Server::poll_admin`
//! exactly as `Server::step` drives it. Staff sign in with Ed25519 keys enrolled in the store.
//! Synthetic names, keys and text only; every file lives in the rig's scratch folder.
use super::super::moderation::tests::{
    ADA, Files, MIA, MO, MUTED, PIP, QUILL, Rig, account, chat_on, guid, said,
};
use super::super::tests::test_support;
use super::*;
use ed25519_dalek::{Signer, SigningKey};
use osfr_tools::moderation::Sanctions;

const ADA_HANDLE: &str = "ada_admin";
const MO_HANDLE: &str = "mo_moderator";
const PIP_HANDLE: &str = "pip_player";
const SEEDS: [(usize, &str, u8); 3] = [(ADA, ADA_HANDLE, 11), (MO, MO_HANDLE, 12), (PIP, PIP_HANDLE, 14)];

fn key(seed: u8) -> SigningKey {
    SigningKey::from_bytes(&[seed; 32])
}

fn seed(handle: &str) -> u8 {
    SEEDS.iter().find(|(_, name, _)| *name == handle).unwrap().2
}

/// Give an account a real login handle (the store keeps its SHA-256) and one launcher key.
fn enroll(files: &Files, index: usize, handle: &str, seed: u8) {
    let path = files.store();
    let mut document: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    let hash: [u8; 32] = Sha256::digest(handle.as_bytes()).into();
    document["store"]["accounts"][account(index).to_string()]["login_name_hash"] = json!(hash);
    fs::write(&path, serde_json::to_vec(&document).unwrap()).unwrap();
    let mut store = AccountStore::load(&path).unwrap();
    store
        .add_public_key(account(index), key(seed).verifying_key().as_bytes())
        .unwrap();
    store.save(&path).unwrap();
}

/// The L042 cast with three enrolled keys and a running panel; `None` off this PC's fixtures.
fn panel(tag: &str, session_ttl: Duration, client: Option<PathBuf>) -> Option<(Files, Rig)> {
    if !test_support::bin_test_target("admin") {
        return None;
    }
    let files = Files::new(&format!("admin-{tag}"));
    if !test_support::fixtures_present(&test_support::options(files.dir().join("state.json"))) {
        return None;
    }
    for (index, handle, seed) in SEEDS {
        enroll(&files, index, handle, seed);
    }
    let mut rig = Rig::new(&files);
    let web = files.dir().join("web");
    fs::create_dir_all(&web).unwrap();
    fs::write(web.join("index.html"), "<!doctype html><title>GM panel</title>").unwrap();
    fs::write(web.join("app.js"), "console.log('panel');").unwrap();
    fs::write(files.dir().join("secret.txt"), "TOP-SECRET").unwrap();
    let config = Config {
        bind: "127.0.0.1:0".parse().unwrap(),
        web,
        client,
        session_ttl,
    };
    rig.server.admin = open(config, &rig.server.options).unwrap();
    assert!(rig.server.admin.is_some(), "the panel listens");
    Some((files, rig))
}

struct Response {
    status: u16,
    head: String,
    body: Vec<u8>,
}

impl Response {
    fn json(&self) -> Value {
        serde_json::from_slice(&self.body).unwrap_or(Value::Null)
    }

    fn header(&self, name: &str) -> Option<&str> {
        self.head.lines().find_map(|line| {
            let (key, value) = line.split_once(':')?;
            key.eq_ignore_ascii_case(name).then(|| value.trim())
        })
    }
}

/// Send raw bytes to the panel and run the world loop until the answer is back.
fn exchange(rig: &mut Rig, raw: Vec<u8>) -> Response {
    let address = rig.server.admin.as_ref().unwrap().address;
    let client = std::thread::spawn(move || {
        let mut stream = TcpStream::connect(address).unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(10)))
            .unwrap();
        let _ = stream.write_all(&raw);
        let mut out = Vec::new();
        let _ = stream.read_to_end(&mut out);
        out
    });
    while !client.is_finished() {
        rig.server.poll_admin();
        std::thread::sleep(Duration::from_millis(1));
    }
    let out = client.join().unwrap();
    let end = out
        .windows(4)
        .position(|w| w == b"\r\n\r\n")
        .map_or(out.len(), |at| at + 4);
    let head = String::from_utf8_lossy(&out[..end]).into_owned();
    let status = head
        .split(' ')
        .nth(1)
        .and_then(|code| code.parse().ok())
        .unwrap_or(0);
    Response {
        status,
        head,
        body: out[end..].to_vec(),
    }
}

struct Login {
    token: String,
    csrf: String,
}

fn host(rig: &Rig) -> String {
    format!("127.0.0.1:{}", rig.server.admin.as_ref().unwrap().address.port())
}

/// One request as a browser on the panel's page sends it: the session cookie and the CSRF echo.
fn call(rig: &mut Rig, method: &str, path: &str, body: Option<Value>, login: Option<&Login>) -> Response {
    let body = body.map_or_else(Vec::new, |body| serde_json::to_vec(&body).unwrap());
    let mut raw = format!(
        "{method} {path} HTTP/1.1\r\nHost: {}\r\nOrigin: http://{}\r\nContent-Length: {}\r\n",
        host(rig),
        host(rig),
        body.len()
    );
    if let Some(login) = login {
        raw.push_str(&format!(
            "Cookie: theme=dark; osfr_admin={}\r\nX-CSRF-Token: {}\r\n",
            login.token, login.csrf
        ));
    }
    raw.push_str("\r\n");
    let mut raw = raw.into_bytes();
    raw.extend(body);
    exchange(rig, raw)
}

fn get(rig: &mut Rig, path: &str, login: Option<&Login>) -> Response {
    call(rig, "GET", path, None, login)
}

fn post(rig: &mut Rig, path: &str, body: Value, login: &Login) -> Response {
    call(rig, "POST", path, Some(body), Some(login))
}

/// The panel's two-step login: a challenge, then the launcher key's signature over it.
fn login(rig: &mut Rig, handle: &str, seed: u8) -> Response {
    let challenge = call(rig, "POST", "/admin/api/login/challenge", Some(json!({"account": handle})), None);
    assert_eq!(challenge.status, 200, "{}", String::from_utf8_lossy(&challenge.body));
    let challenge = challenge.json();
    let nonce: [u8; 32] = unhex(challenge["nonce"].as_str().unwrap())
        .unwrap()
        .try_into()
        .unwrap();
    let server_id: [u8; 32] = unhex(challenge["server_id"].as_str().unwrap())
        .unwrap()
        .try_into()
        .unwrap();
    let message = osfr_tools::login_auth::signing_message(&server_id, &nonce, handle).unwrap();
    let key = key(seed);
    let body = json!({
        "account": handle, "nonce": hex(&nonce),
        "public_key": hex(key.verifying_key().as_bytes()),
        "signature": hex(&key.sign(&message).to_bytes()),
    });
    call(rig, "POST", "/admin/api/login", Some(body), None)
}

fn sign_in(rig: &mut Rig, handle: &str) -> Login {
    let reply = login(rig, handle, seed(handle));
    assert_eq!(reply.status, 200, "{}", String::from_utf8_lossy(&reply.body));
    let body = reply.json();
    Login {
        token: body["token"].as_str().unwrap().to_owned(),
        csrf: body["csrf"].as_str().unwrap().to_owned(),
    }
}

fn audit_rows(files: &Files) -> Vec<audit::Row> {
    let mut rows = Vec::new();
    audit::scan(&audit::dir_for(&files.store()), |row| {
        rows.push(row);
        true
    })
    .unwrap();
    rows
}

#[test]
fn admin_login_gives_a_token_that_expires() {
    let Some((files, mut rig)) = panel("login", Duration::from_millis(1500), None) else {
        return;
    };
    assert_eq!(get(&mut rig, "/admin/api/me", None).status, 401, "no session");
    assert_eq!(
        login(&mut rig, ADA_HANDLE, seed(MO_HANDLE)).status,
        401,
        "another account's key"
    );
    assert_eq!(
        login(&mut rig, PIP_HANDLE, seed(PIP_HANDLE)).status,
        403,
        "a player's real key is not staff"
    );
    let reply = login(&mut rig, ADA_HANDLE, seed(ADA_HANDLE));
    assert_eq!(reply.status, 200);
    let cookie = reply.header("set-cookie").unwrap();
    assert!(cookie.contains("HttpOnly") && cookie.contains("SameSite=Strict"), "{cookie}");
    let body = reply.json();
    assert_eq!(body["staff"]["role"], "admin");
    assert_eq!(body["staff"]["name"], ADA_HANDLE);
    let ada = Login {
        token: body["token"].as_str().unwrap().into(),
        csrf: body["csrf"].as_str().unwrap().into(),
    };
    let me = get(&mut rig, "/admin/api/me", Some(&ada));
    assert_eq!(me.status, 200);
    let me = me.json();
    assert_eq!((me["role"].as_str(), me["account"].as_u64()), (Some("admin"), Some(account(ADA))));
    assert!(me["permissions"].as_array().unwrap().contains(&json!("announce")));
    // A non-browser client may send the same token as a bearer header.
    let raw = format!(
        "GET /admin/api/me HTTP/1.1\r\nHost: {}\r\nAuthorization: Bearer {}\r\n\r\n",
        host(&rig),
        ada.token
    );
    assert_eq!(exchange(&mut rig, raw.into_bytes()).status, 200);
    std::thread::sleep(Duration::from_millis(1600));
    assert_eq!(get(&mut rig, "/admin/api/me", Some(&ada)).status, 401, "expired");
    // Logout ends a fresh session at once.
    let mo = sign_in(&mut rig, MO_HANDLE);
    assert_eq!(post(&mut rig, "/admin/api/logout", json!({}), &mo).status, 200);
    assert_eq!(get(&mut rig, "/admin/api/me", Some(&mo)).status, 401);
    let logins: Vec<_> = audit_rows(&files)
        .into_iter()
        .filter(|row| row.action == "login")
        .map(|row| row.staff.name)
        .collect();
    assert_eq!(logins, [MO_HANDLE, ADA_HANDLE], "one row per successful login, newest first");
}

#[test]
fn admin_a_moderator_is_refused_admin_routes() {
    let Some((files, mut rig)) = panel("roles", SESSION_TTL, None) else {
        return;
    };
    let pip = guid(PIP);
    let mo = sign_in(&mut rig, MO_HANDLE);
    let announce = json!({"to": "all", "text": "hello", "style": "chat"});
    for (path, body) in [
        ("/admin/api/announce".to_owned(), announce.clone()),
        (format!("/admin/api/players/{pip}/inventory/add"), json!({"item_id": 1, "count": 1})),
        (format!("/admin/api/players/{pip}/coins"), json!({"op": "give", "amount": 5})),
        ("/admin/api/powers".to_owned(), json!({"op": "invisible"})),
    ] {
        let reply = post(&mut rig, &path, body, &mo);
        assert_eq!(reply.status, 403, "{path}: {}", String::from_utf8_lossy(&reply.body));
    }
    for path in ["/admin/api/status", "/admin/api/players/online", "/admin/api/audit", "/admin/api/players"] {
        assert_eq!(get(&mut rig, path, Some(&mo)).status, 200, "{path}");
    }
    // The moderation hierarchy holds on the panel too.
    let ban_ada = post(&mut rig, &format!("/admin/api/players/{}/moderate", guid(ADA)), json!({"op": "ban"}), &mo);
    assert_eq!(ban_ada.status, 403);
    assert_eq!(files.sanctions(ADA), Sanctions::default());
    // An admin reaches the same routes: real ones answer, other builders' stubs say 501.
    let ada = sign_in(&mut rig, ADA_HANDLE);
    assert_eq!(post(&mut rig, "/admin/api/announce", announce, &ada).status, 200);
    let stub = post(&mut rig, &format!("/admin/api/players/{pip}/inventory/add"), json!({}), &ada);
    assert_eq!(stub.status, 501);
    assert!(stub.json()["error"].as_str().unwrap().contains("not built yet"));
    let status = get(&mut rig, "/admin/api/status", Some(&ada)).json();
    assert_eq!(status["online"], 0);
    assert!(status["tick_ms"]["max"].as_f64().is_some() && status["saves_ok"] == true, "{status}");
    // Refused requests left no audit row: two logins and the admin's announcement.
    let actions: Vec<_> = audit_rows(&files).into_iter().map(|row| row.action).collect();
    assert_eq!(actions, ["announce", "login", "login"]);
}

#[test]
fn admin_the_online_list_matches_the_sessions() {
    let Some((_files, mut rig)) = panel("online", SESSION_TTL, None) else {
        return;
    };
    let pip_key = rig.join(PIP).unwrap();
    rig.join(MO).unwrap();
    let ada = sign_in(&mut rig, ADA_HANDLE);
    let listed = |rig: &mut Rig| -> BTreeSet<u64> {
        let reply = get(rig, "/admin/api/players/online", Some(&ada));
        assert_eq!(reply.status, 200);
        reply.json().as_array().unwrap().iter().map(|row| row["character_id"].as_u64().unwrap()).collect()
    };
    let live: BTreeSet<u64> = rig.server.world.active_sessions.keys().copied().collect();
    assert_eq!(live, BTreeSet::from([guid(PIP), guid(MO)]));
    assert_eq!(listed(&mut rig), live);
    let rows = get(&mut rig, "/admin/api/players/online", Some(&ada)).json();
    let pip = rows.as_array().unwrap().iter().find(|row| row["character_id"] == guid(PIP)).unwrap().clone();
    assert_eq!(pip["name"], "Pip Player");
    assert_eq!(pip["account"], account(PIP));
    assert_eq!(pip["role"], "player");
    let at = rig.server.world.players.get(rig.server.sessions[&pip_key].id).unwrap();
    assert_eq!(pip["zone"], at.zone.zone_id);
    // `!gm players` reads the same rows.
    assert_eq!(online_rows(&rig.server.world).len(), 2);
    // A session that ends leaves the list.
    let session = rig.server.sessions.remove(&pip_key).unwrap();
    rig.server.end_session(pip_key, session, "test_leave");
    assert_eq!(listed(&mut rig), BTreeSet::from([guid(MO)]));
}

#[test]
fn admin_search_finds_an_offline_character_by_substring() {
    let Some((_files, mut rig)) = panel("search", SESSION_TTL, None) else {
        return;
    };
    rig.join(ADA).unwrap();
    let mo = sign_in(&mut rig, MO_HANDLE);
    let names = |rig: &mut Rig, query: &str| -> Vec<(String, bool)> {
        let reply = get(rig, &format!("/admin/api/players?{query}"), Some(&mo));
        assert_eq!(reply.status, 200, "{query}");
        let body = reply.json();
        let rows = body["rows"].as_array().unwrap();
        assert_eq!(body["total"].as_u64(), Some(rows.len() as u64));
        rows.iter()
            .map(|row| (row["name"].as_str().unwrap().to_owned(), row["online"].as_bool().unwrap()))
            .collect()
    };
    // Offline, by a substring in the middle of the name, and by its letters in order.
    let lee = [("Ann Lee Moss".to_owned(), false), ("Ann Lee Moss".to_owned(), false)];
    assert_eq!(names(&mut rig, "q=lee%20mo"), lee);
    let quill = [("Quill Zed Vox".to_owned(), false), ("Quill Zed Vox".to_owned(), false)];
    assert_eq!(names(&mut rig, "q=ill%20z"), quill);
    assert_eq!(names(&mut rig, "q=qzv"), quill, "fuzzy");
    assert_eq!(names(&mut rig, "q=PIP"), [("Pip Player".to_owned(), false)]);
    assert_eq!(names(&mut rig, &format!("q={}", account(QUILL))), quill, "by account id");
    assert_eq!(names(&mut rig, "online=true"), [("Ada Admin".to_owned(), true)]);
    assert_eq!(
        names(&mut rig, "role=moderator"),
        [("Mia Moderator".to_owned(), false), ("Mo Moderator".to_owned(), false)]
    );
    assert!(names(&mut rig, "q=nobody").is_empty());
    assert_eq!(get(&mut rig, "/admin/api/players?online=maybe", Some(&mo)).status, 400);
    // And the profile reads the offline character from the store and its (absent) save.
    let profile = get(&mut rig, &format!("/admin/api/players/{}", guid(MIA)), Some(&mo)).json();
    assert_eq!(profile["identity"]["name"], "Mia Moderator");
    assert_eq!(profile["identity"]["role"], "moderator");
    assert_eq!(profile["location"]["online"], false);
    assert_eq!(get(&mut rig, "/admin/api/players/999999", Some(&mo)).status, 404);
}

#[test]
fn admin_mute_through_the_api_equals_mute_through_chat() {
    let Some((files, mut rig)) = panel("mute", SESSION_TTL, None) else {
        return;
    };
    let [mo_key, pip_key] = [MO, PIP].map(|index| rig.join(index).unwrap());
    // Chat.
    assert!(said(&rig.command(mo_key, "realm mute Pip Player 30"), "Player muted."));
    let by_chat = files.sanctions(PIP);
    assert!(said(&rig.say(pip_key, &chat_on(1, "hello there")), MUTED));
    assert!(said(&rig.command(mo_key, "realm unmute Pip Player"), "Player unmuted."));
    assert_eq!(files.sanctions(PIP), Sanctions::default());
    // The panel, the same moderator, the same target and minutes.
    let mo = sign_in(&mut rig, MO_HANDLE);
    let reply = post(
        &mut rig,
        &format!("/admin/api/players/{}/moderate", guid(PIP)),
        json!({"op": "mute", "minutes": 30, "reason": "spam"}),
        &mo,
    );
    assert_eq!(reply.status, 200, "{}", String::from_utf8_lossy(&reply.body));
    let by_panel = files.sanctions(PIP);
    let (chat_until, panel_until) = (by_chat.muted_until.unwrap(), by_panel.muted_until.unwrap());
    assert!(
        (chat_until..=chat_until + 30).contains(&panel_until),
        "both last 30 minutes from when they were given: {chat_until} vs {panel_until}"
    );
    assert_eq!(by_panel.banned_until, by_chat.banned_until);
    assert!(said(&rig.say(pip_key, &chat_on(1, "hello again")), MUTED), "the muted player is told");
    // Both wrote the same security-log row and the same audit row, apart from the front end.
    let security: Vec<_> = files.audit().into_iter().filter(|row| row["action"] == "mute").collect();
    assert_eq!(security.len(), 2);
    for field in ["account_id", "target_account_id", "minutes", "outcome"] {
        assert_eq!(security[0][field], security[1][field], "{field}");
    }
    let mutes: Vec<_> = audit_rows(&files).into_iter().filter(|row| row.action == "mute").collect();
    assert_eq!(mutes.iter().map(|row| row.via.as_str()).collect::<Vec<_>>(), ["panel", "chat"]);
    let (panel, chat) = (&mutes[0], &mutes[1]);
    assert_eq!(panel.staff.account_id, chat.staff.account_id);
    assert_eq!(panel.target.account_id, Some(account(PIP)));
    assert_eq!(panel.target.account_id, chat.target.account_id);
    assert_eq!(panel.target.name.as_deref(), Some("Pip Player"));
    assert_eq!(panel.before, chat.before);
    assert_eq!(panel.after["minutes"], chat.after["minutes"]);
    assert_eq!(panel.reason.as_deref(), Some("spam"));
}

#[test]
fn admin_every_write_leaves_an_audit_row() {
    let Some((files, mut rig)) = panel("audit", SESSION_TTL, None) else {
        return;
    };
    let pip_key = rig.join(PIP).unwrap();
    let ada = sign_in(&mut rig, ADA_HANDLE);
    let mo = sign_in(&mut rig, MO_HANDLE);
    let pip = guid(PIP);
    let moderate = format!("/admin/api/players/{pip}/moderate");
    let ok = |reply: Response| {
        assert_eq!(reply.status, 200, "{}", String::from_utf8_lossy(&reply.body));
        reply.json()
    };
    ok(post(&mut rig, &moderate, json!({"op": "mute", "minutes": 5}), &ada));
    ok(post(&mut rig, &moderate, json!({"op": "unmute"}), &ada));
    let warned = ok(post(&mut rig, &moderate, json!({"op": "warn", "reason": "mind your language"}), &mo));
    assert_eq!(warned["row"]["after"]["delivered"], true);
    assert!(said(&rig.drain(pip_key), "Warning from the staff: mind your language"));
    let notes = ok(post(&mut rig, &format!("/admin/api/players/{pip}/notes"), json!({"text": "second warning"}), &mo));
    assert_eq!(notes[0]["after"]["text"], "second warning");
    let announced = ok(post(&mut rig, "/admin/api/announce", json!({"to": "all", "text": "Restart in five minutes"}), &ada));
    assert_eq!(announced["delivered"], 1);
    assert!(said(&rig.drain(pip_key), "Restart in five minutes"));
    ok(post(&mut rig, &moderate, json!({"op": "kick", "reason": "cool off"}), &mo));
    rig.step();
    assert!(!rig.server.sessions.contains_key(&pip_key), "kicked");
    // A kick of nobody online is refused and recorded nowhere.
    assert_eq!(post(&mut rig, &moderate, json!({"op": "kick"}), &mo).status, 409);
    ok(post(&mut rig, &moderate, json!({"op": "ban", "minutes": 60}), &ada));
    ok(post(&mut rig, &moderate, json!({"op": "unban"}), &ada));
    // Refusals write nothing: a role too low, a missing CSRF echo, a bad op.
    assert_eq!(post(&mut rig, "/admin/api/announce", json!({"to": "all", "text": "x"}), &mo).status, 403);
    let forged = Login { token: mo.token.clone(), csrf: "0".repeat(32) };
    assert_eq!(post(&mut rig, &moderate, json!({"op": "mute"}), &forged).status, 403);
    assert_eq!(post(&mut rig, &moderate, json!({"op": "explode"}), &mo).status, 400);
    let actions: Vec<_> = audit_rows(&files).into_iter().map(|row| row.action).collect();
    assert_eq!(
        actions,
        ["unban", "ban", "kick", "announce", "note", "warn", "unmute", "mute", "login", "login"]
    );
    // The panel's own reads agree: the audit page, its filters and the player's profile.
    let page = get(&mut rig, "/admin/api/audit", Some(&ada)).json();
    assert_eq!(page["total"], 10);
    let by_mo = get(&mut rig, &format!("/admin/api/audit?staff={MO_HANDLE}"), Some(&ada)).json();
    assert_eq!(by_mo["total"], 4, "warn, note, kick and the login");
    let kicks = get(&mut rig, &format!("/admin/api/audit?action=kick&player={pip}"), Some(&ada)).json();
    assert_eq!(kicks["rows"][0]["reason"], "cool off");
    let profile = get(&mut rig, &format!("/admin/api/players/{pip}"), Some(&mo)).json();
    let history: Vec<_> = profile["moderation"].as_array().unwrap().iter().map(|row| row["action"].clone()).collect();
    assert_eq!(history, ["unban", "ban", "kick", "warn", "unmute", "mute"]);
    assert_eq!(profile["notes"][0]["staff"]["name"], MO_HANDLE);
}

#[test]
fn admin_an_oversized_body_is_refused() {
    let Some((files, mut rig)) = panel("size", SESSION_TTL, None) else {
        return;
    };
    let ada = sign_in(&mut rig, ADA_HANDLE);
    let raw = format!(
        "POST /admin/api/players/{}/notes HTTP/1.1\r\nHost: {}\r\nContent-Length: {}\r\nCookie: osfr_admin={}\r\nX-CSRF-Token: {}\r\n\r\n",
        guid(PIP),
        host(&rig),
        MAX_BODY + 1,
        ada.token,
        ada.csrf
    );
    assert_eq!(exchange(&mut rig, raw.into_bytes()).status, 413);
    let huge = format!("GET /admin/ HTTP/1.1\r\nHost: {}\r\nX-Pad: {}\r\n\r\n", host(&rig), "a".repeat(9000));
    assert_eq!(exchange(&mut rig, huge.into_bytes()).status, 431);
    let chunked = format!("POST /admin/api/login HTTP/1.1\r\nHost: {}\r\nTransfer-Encoding: chunked\r\n\r\n", host(&rig));
    assert_eq!(exchange(&mut rig, chunked.into_bytes()).status, 411);
    // A note longer than the text bound is refused inside the body cap too.
    let long = post(&mut rig, &format!("/admin/api/players/{}/notes", guid(PIP)), json!({"text": "x".repeat(600)}), &ada);
    assert_eq!(long.status, 400);
    assert!(audit_rows(&files).iter().all(|row| row.action == "login"));
}

#[test]
fn admin_a_static_path_escape_is_refused() {
    let Some((_files, mut rig)) = panel("static", SESSION_TTL, None) else {
        return;
    };
    let index = get(&mut rig, "/admin/", None);
    assert_eq!(index.status, 200);
    assert_eq!(index.header("content-type"), Some("text/html; charset=utf-8"));
    assert!(String::from_utf8_lossy(&index.body).contains("GM panel"));
    let script = get(&mut rig, "/admin/app.js", None);
    assert_eq!((script.status, script.header("content-type")), (200, Some("text/javascript; charset=utf-8")));
    assert_eq!(get(&mut rig, "/admin", None).status, 301);
    for path in [
        "/admin/../secret.txt",
        "/admin/%2e%2e/secret.txt",
        "/admin/..%2fsecret.txt",
        "/admin/..\\secret.txt",
        "/admin/sub/../../secret.txt",
        "/admin//secret.txt",
        "/admin/.htaccess",
        "/admin/C:/Windows/win.ini",
        "/admin/missing.js",
    ] {
        let reply = get(&mut rig, path, None);
        assert_eq!(reply.status, 404, "{path}");
        assert!(!String::from_utf8_lossy(&reply.body).contains("TOP-SECRET"), "{path}");
    }
    // Same origin only: a rebound DNS name and a foreign Origin are refused before routing.
    let port = rig.server.admin.as_ref().unwrap().address.port();
    let rebound = format!("GET /admin/ HTTP/1.1\r\nHost: evil.example:{port}\r\n\r\n");
    assert_eq!(exchange(&mut rig, rebound.into_bytes()).status, 403);
    let foreign = format!("GET /admin/ HTTP/1.1\r\nHost: {}\r\nOrigin: http://evil.example\r\n\r\n", host(&rig));
    assert_eq!(exchange(&mut rig, foreign.into_bytes()).status, 403);
    let cross = format!("GET /admin/ HTTP/1.1\r\nHost: {}\r\nSec-Fetch-Site: cross-site\r\n\r\n", host(&rig));
    assert_eq!(exchange(&mut rig, cross.into_bytes()).status, 403);
    let localhost = format!("GET /admin/ HTTP/1.1\r\nHost: localhost:{port}\r\n\r\n");
    assert_eq!(exchange(&mut rig, localhost.into_bytes()).status, 200);
}

/// A 4x4 DXT1 file: red and blue endpoints, picks 0..3 along the first row.
fn dxt1() -> Vec<u8> {
    let mut file = b"DDS ".to_vec();
    for value in [124u32, 0, 4, 4, 0, 0, 1] {
        file.extend(value.to_le_bytes());
    }
    file.resize(76, 0);
    file.extend(32u32.to_le_bytes());
    file.extend(4u32.to_le_bytes());
    file.extend(b"DXT1");
    file.resize(128, 0);
    file.extend([0x00, 0xf8, 0x1f, 0x00, 0b1110_0100, 0, 0, 0]);
    file
}

#[test]
fn admin_icons_come_from_the_client_install() {
    if !test_support::bin_test_target("admin") {
        return;
    }
    let options = test_support::options(PathBuf::from("unused"));
    if !test_support::fixtures_present(&options) {
        return;
    }
    // A real item of the rig's content and its icon set, served from a one-file install.
    let inventory = osfr_tools::inventory::Inventory::load(options.content.as_deref().unwrap(), None).unwrap();
    let (item, set) = inventory
        .definition_ids()
        .find_map(|id| {
            let set = osfr_tools::le::u32(inventory.definition_record(id)?, 12)?;
            (set != 0).then_some((id, set))
        })
        .unwrap();
    let install = test_support::Scratch::new("admin-install");
    let images = install.path().join("Resources").join("Images");
    fs::create_dir_all(&images).unwrap();
    fs::write(images.join("Images.txt"), "#ID^FILE_NAME^\r\n9001^Test_Icon_64.dds^0^\r\n").unwrap();
    fs::write(images.join("ImageSetMappings.txt"), format!("#SET^TYPE^IMAGE^\r\n{set}^6^9001^\r\n")).unwrap();
    let name = b"test_icon_64.dds";
    let mut pack = 0u32.to_be_bytes().to_vec();
    pack.extend(1u32.to_be_bytes());
    pack.extend((name.len() as u32).to_be_bytes());
    pack.extend(name);
    let file = dxt1();
    pack.extend((pack.len() as u32 + 12).to_be_bytes());
    pack.extend((file.len() as u32).to_be_bytes());
    pack.extend(0u32.to_be_bytes());
    pack.extend(file);
    fs::write(install.path().join("Assets_000.pack"), pack).unwrap();
    let Some((_files, mut rig)) = panel("icons", SESSION_TTL, Some(install.path().to_owned())) else {
        return;
    };
    assert_eq!(get(&mut rig, &format!("/admin/icon/{item}"), None).status, 401, "staff only");
    let mo = sign_in(&mut rig, MO_HANDLE);
    let icon = get(&mut rig, &format!("/admin/icon/{item}"), Some(&mo));
    assert_eq!(icon.status, 200);
    assert_eq!(icon.header("content-type"), Some("image/png"));
    assert!(icon.header("cache-control").unwrap().contains("max-age"));
    assert_eq!(&icon.body[..8], b"\x89PNG\r\n\x1a\n");
    assert_eq!(&icon.body[16..24], [0, 0, 0, 4, 0, 0, 0, 4]);
    assert_eq!(get(&mut rig, "/admin/icon/2147483647", Some(&mo)).status, 404);
    assert_eq!(get(&mut rig, "/admin/icon/abc", Some(&mo)).status, 400);
}
