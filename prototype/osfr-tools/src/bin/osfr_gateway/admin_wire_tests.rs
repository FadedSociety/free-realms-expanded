//! gm-panel integration: every wired route family through the production path (G1's panel rig:
//! a real `Server`, L042 sessions, real HTTP, the world loop run by `Server::poll_admin`). Each
//! test checks that the call reaches its builder's facade, the role gates, the statuses errors
//! map to, and the staff audit row. Synthetic names, keys and text only.
use super::*;
use osfr_tools::mail;

/// A stacking item of the rig's opening catalogue (G3's `g3_character_admin` uses it too).
const STACKING: i32 = 70454;

fn ok(reply: Response) -> Value {
    assert_eq!(
        reply.status,
        200,
        "{}",
        String::from_utf8_lossy(&reply.body)
    );
    reply.json()
}

fn count(inventory: &Value, item: i32) -> u64 {
    (inventory["bags"].as_array().into_iter().flatten())
        .filter(|row| row["item_id"] == item)
        .filter_map(|row| row["count"].as_u64())
        .sum()
}

/// `Server::step` with every session's replies kept (the rig's own `step` drops them).
fn told(rig: &mut Rig) -> BTreeMap<SessionKey, Vec<Vec<u8>>> {
    test_support::step(&mut rig.server).unwrap();
    std::thread::sleep(Duration::from_millis(300));
    let keys: Vec<SessionKey> = rig.server.sessions.keys().copied().collect();
    keys.into_iter().map(|key| (key, rig.drain(key))).collect()
}

fn rows_of(files: &Files, action: &str) -> Vec<audit::Row> {
    audit_rows(files)
        .into_iter()
        .filter(|row| row.action == action)
        .collect()
}

#[test]
fn admin_inventory_coins_and_undo_go_through_g3_with_audit_rows() {
    let Some((files, mut rig)) = panel("edit", SESSION_TTL, None) else {
        return;
    };
    let pip_key = rig.join(PIP).unwrap();
    let pip = guid(PIP);
    let (ada, mo) = (sign_in(&mut rig, ADA_HANDLE), sign_in(&mut rig, MO_HANDLE));
    let base = format!("/admin/api/players/{pip}");
    // A moderator reads the bag but may not change it or the coins.
    let before = ok(get(&mut rig, &format!("{base}/inventory"), Some(&mo)));
    assert!(
        before["bags"].is_array() && before["equipped"].is_array(),
        "{before}"
    );
    let add = json!({"item_id": STACKING, "count": 3});
    for (path, body) in [
        ("inventory/add", add.clone()),
        ("coins", json!({"op": "give", "amount": 5})),
    ] {
        assert_eq!(
            post(&mut rig, &format!("{base}/{path}"), body, &mo).status,
            403,
            "{path}"
        );
    }
    // The admin's add reaches the online player's own session.
    let added = ok(post(&mut rig, &format!("{base}/inventory/add"), add, &ada));
    assert_eq!(count(&added, STACKING), count(&before, STACKING) + 3);
    assert_eq!(added["online"], true);
    let live = rig.server.sessions[&pip_key]
        .gateway
        .gameplay()
        .unwrap()
        .inventory_state();
    assert_eq!(
        live.items
            .iter()
            .filter(|item| item.definition_id == STACKING)
            .map(|item| u64::from(item.count))
            .sum::<u64>(),
        count(&added, STACKING),
        "the session holds what the route answered"
    );
    let removed = ok(post(
        &mut rig,
        &format!("{base}/inventory/remove"),
        json!({"items": [{"item_id": STACKING, "count": "all"}]}),
        &ada,
    ));
    assert_eq!(count(&removed, STACKING), 0);
    // Undo puts the stacks back, once; the undo is an edit of its own.
    let key = removed["undo"].as_str().unwrap().to_owned();
    let undone = ok(post(
        &mut rig,
        "/admin/api/undo",
        json!({"undo": key}),
        &ada,
    ));
    assert_eq!(count(&undone, STACKING), count(&added, STACKING));
    assert_eq!(
        post(&mut rig, "/admin/api/undo", json!({"undo": key}), &ada).status,
        409,
        "used"
    );
    assert_eq!(
        post(
            &mut rig,
            "/admin/api/undo",
            json!({"undo": "0".repeat(32)}),
            &ada
        )
        .status,
        404
    );
    assert_eq!(
        post(
            &mut rig,
            "/admin/api/undo",
            json!({"undo": undone["undo"]}),
            &mo
        )
        .status,
        403
    );
    // Coins: give, then a take past the balance is the game's refusal (422), changing nothing.
    let coins = ok(post(
        &mut rig,
        &format!("{base}/coins"),
        json!({"op": "set", "amount": 40}),
        &ada,
    ));
    assert_eq!(coins["coins"], 40);
    let take = post(
        &mut rig,
        &format!("{base}/coins"),
        json!({"op": "take", "amount": 41}),
        &ada,
    );
    assert_eq!(take.status, 422, "{}", String::from_utf8_lossy(&take.body));
    let given = ok(post(
        &mut rig,
        &format!("{base}/coins"),
        json!({"op": "give", "amount": 5}),
        &ada,
    ));
    assert_eq!(given["coins"], 45);
    // Errors keep their meaning: an unknown item (422), no such character (404), a bad body (400).
    let unknown = json!({"item_id": 2_000_000_000, "count": 1});
    assert_eq!(
        post(&mut rig, &format!("{base}/inventory/add"), unknown, &ada).status,
        422
    );
    assert_eq!(
        post(
            &mut rig,
            "/admin/api/players/999999/coins",
            json!({"op": "give", "amount": 1}),
            &ada
        )
        .status,
        404
    );
    assert_eq!(
        post(
            &mut rig,
            &format!("{base}/coins"),
            json!({"op": "steal", "amount": 1}),
            &ada
        )
        .status,
        400
    );
    assert_eq!(
        post(
            &mut rig,
            &format!("{base}/inventory/remove"),
            json!({"items": [{"count": 1}]}),
            &ada
        )
        .status,
        400
    );
    // One row per applied edit, newest first, each with what it changed and its undo key.
    let actions: Vec<_> = audit_rows(&files)
        .into_iter()
        .filter(|row| row.action != "login")
        .map(|row| (row.action, row.via))
        .collect();
    let panel = |action: &str| (action.to_owned(), "panel".to_owned());
    assert_eq!(
        actions,
        [
            panel("coins"),
            panel("coins"),
            panel("undo"),
            panel("inventory_remove"),
            panel("inventory_add")
        ]
    );
    let give = &rows_of(&files, "coins")[0];
    assert_eq!(
        (give.before["coins"].as_u64(), give.after["coins"].as_u64()),
        (Some(40), Some(45))
    );
    assert_eq!(give.target.character_id, Some(pip));
    assert_eq!(give.target.name.as_deref(), Some("Pip Player"));
    assert!(
        give.after["undo"].is_string() && give.after["change"].is_object(),
        "{}",
        give.after
    );
    assert_eq!(rows_of(&files, "undo")[0].after["undid"], json!(key));
    // Offline: the save is edited after a backup, through the same route.
    let session = rig.server.sessions.remove(&pip_key).unwrap();
    rig.server.end_session(pip_key, session, "test_leave");
    let offline = ok(post(
        &mut rig,
        &format!("{base}/coins"),
        json!({"op": "give", "amount": 10}),
        &ada,
    ));
    assert_eq!(
        (offline["coins"].as_u64(), offline["online"].as_bool()),
        (Some(55), Some(false))
    );
    let backups = paths::character_state_path(&rig.server.options.state, pip)
        .parent()
        .unwrap()
        .join("admin-backups");
    assert_eq!(
        fs::read_dir(backups).unwrap().count(),
        1,
        "one backup before the offline write"
    );
}

#[test]
fn admin_items_and_names_answer_from_the_content() {
    let Some((_files, mut rig)) = panel("items", SESSION_TTL, None) else {
        return;
    };
    let mo = sign_in(&mut rig, MO_HANDLE);
    let hits = ok(get(
        &mut rig,
        &format!("/admin/api/items?q={STACKING}"),
        Some(&mo),
    ));
    assert_eq!(hits[0]["item_id"], STACKING, "{hits}");
    let page = ok(get(&mut rig, "/admin/api/items?q=", Some(&mo)));
    assert_eq!(
        page.as_array().map(Vec::len),
        Some(osfr_tools::character_admin::SEARCH_PAGE)
    );
    assert_eq!(
        get(
            &mut rig,
            &format!("/admin/api/items?q={}", "x".repeat(65)),
            Some(&mo)
        )
        .status,
        400
    );
    let names = ok(get(&mut rig, "/admin/api/names", Some(&mo)));
    assert!(
        names["jobs"].is_object() && names["quests"].is_object(),
        "{names}"
    );
    assert_eq!(get(&mut rig, "/admin/api/items", None).status, 401);
}

#[test]
fn admin_mail_goes_through_g2_and_tells_online_players() {
    let Some((files, mut rig)) = panel("mail", SESSION_TTL, None) else {
        return;
    };
    let pip_key = rig.join(PIP).unwrap();
    told(&mut rig);
    let (ada, mo) = (sign_in(&mut rig, ADA_HANDLE), sign_in(&mut rig, MO_HANDLE));
    let pip = guid(PIP);
    let letter =
        |to: Value| json!({"to": to, "subject": "Welcome", "body": "Hello from the staff."});
    // A moderator mails one player; the job delivers, and the world tick tells the player.
    let sent = ok(post(
        &mut rig,
        "/admin/api/mail",
        letter(json!({"characters": [pip]})),
        &mo,
    ));
    assert_eq!(sent["recipients"], 1);
    let job = sent["job_id"].as_u64().unwrap();
    let deadline = Instant::now() + Duration::from_secs(10);
    while mail::admin::job(job).is_none_or(|view| view.state != mail::admin::JobState::Done) {
        assert!(Instant::now() < deadline, "the mail job finishes");
        std::thread::sleep(Duration::from_millis(20));
    }
    let heard = told(&mut rig);
    assert!(
        said(&heard[&pip_key], "You have 1 new messages"),
        "the online player is told"
    );
    let mailbox = ok(get(
        &mut rig,
        &format!("/admin/api/players/{pip}/mail"),
        Some(&mo),
    ));
    assert_eq!(
        (mailbox[0]["subject"].as_str(), mailbox[0]["from"].as_str()),
        (Some("Welcome"), Some("Free Realms"))
    );
    assert_eq!(mailbox[0]["attachments"], json!({"items": [], "coins": 0}));
    let jobs = ok(get(&mut rig, "/admin/api/mail/jobs", Some(&mo)));
    assert!(
        jobs.as_array()
            .unwrap()
            .iter()
            .any(|row| row["job_id"] == job && row["state"] == "done"),
        "{jobs}"
    );
    // Everyone means every character in the store, online or not.
    let everyone = ok(post(
        &mut rig,
        "/admin/api/mail",
        letter(json!({"everyone": true})),
        &mo,
    ));
    let characters = AccountStore::load(&files.store())
        .unwrap()
        .characters()
        .count();
    assert_eq!(everyone["recipients"].as_u64(), Some(characters as u64));
    // Attachments make coins and items: admins only. Bad recipients and letters are refused.
    let gift = json!({"to": {"characters": [pip]}, "subject": "Gift", "body": "For you.", "attachments": {"coins": 25}});
    assert_eq!(
        post(&mut rig, "/admin/api/mail", gift.clone(), &mo).status,
        403
    );
    ok(post(&mut rig, "/admin/api/mail", gift, &ada));
    assert_eq!(
        post(
            &mut rig,
            "/admin/api/mail",
            letter(json!({"nobody": 1})),
            &mo
        )
        .status,
        400
    );
    assert_eq!(
        post(
            &mut rig,
            "/admin/api/mail",
            letter(json!({"characters": [999999]})),
            &mo
        )
        .status,
        422
    );
    assert_eq!(
        get(&mut rig, "/admin/api/players/999999/mail", Some(&mo)).status,
        404
    );
    // One audit row per send, in the staff log the Audit view reads.
    let mails = rows_of(&files, "mail");
    assert_eq!(mails.len(), 3);
    assert_eq!(
        mails[2].target.character_id,
        Some(pip),
        "a send to one names its recipient"
    );
    assert_eq!(
        (mails[2].staff.name.as_str(), mails[2].via.as_str()),
        (MO_HANDLE, "panel")
    );
    assert_eq!(mails[1].after["to"], "everyone");
    assert_eq!(mails[0].after["attachments"]["coins"], 25);
}

#[test]
fn admin_powers_use_the_staff_character_or_act_on_others_alone() {
    let Some((files, mut rig)) = panel("powers", SESSION_TTL, None) else {
        return;
    };
    let pip_key = rig.join(PIP).unwrap();
    let pip = guid(PIP);
    let (ada, mo) = (sign_in(&mut rig, ADA_HANDLE), sign_in(&mut rig, MO_HANDLE));
    // Moderators reach neither powers nor moves on the panel (admin only, PLAN 4.8).
    assert_eq!(
        post(
            &mut rig,
            "/admin/api/powers",
            json!({"op": "freeze", "target": pip}),
            &mo
        )
        .status,
        403
    );
    assert_eq!(
        post(
            &mut rig,
            &format!("/admin/api/players/{pip}/move"),
            json!({"op": "goto"}),
            &mo
        )
        .status,
        403
    );
    // With no character in the world, a power on the admin's own character is refused, clearly.
    let refused = post(
        &mut rig,
        "/admin/api/powers",
        json!({"op": "invisible"}),
        &ada,
    );
    assert_eq!(refused.status, 409);
    assert!(
        refused.json()["error"]
            .as_str()
            .unwrap()
            .contains("sign in to the game"),
        "{}",
        refused.json()
    );
    // Freeze acts on the other player alone: it runs without the admin's character.
    let frozen = ok(post(
        &mut rig,
        "/admin/api/powers",
        json!({"op": "freeze", "target": pip}),
        &ada,
    ));
    assert_eq!(frozen["state"]["frozen"], json!([pip]));
    assert!(rig.server.world.gm.frozen.contains(&pip));
    assert!(said(&rig.drain(pip_key), "A GM has frozen you in place."));
    assert_eq!(
        post(
            &mut rig,
            "/admin/api/powers",
            json!({"op": "freeze", "target": pip}),
            &ada
        )
        .status,
        409
    );
    ok(post(
        &mut rig,
        "/admin/api/powers",
        json!({"op": "unfreeze", "target": pip}),
        &ada,
    ));
    assert!(rig.server.world.gm.frozen.is_empty());
    // In the world, the admin's own powers run through its session (and it is told in chat).
    let ada_key = rig.join(ADA).unwrap();
    let hidden = ok(post(
        &mut rig,
        "/admin/api/powers",
        json!({"op": "invisible"}),
        &ada,
    ));
    assert_eq!(hidden["state"]["invisible"], true);
    assert!(rig.server.world.players.is_hidden(guid(ADA)));
    assert!(said(&rig.drain(ada_key), "You are invisible to players"));
    let state = ok(get(&mut rig, "/admin/api/powers", Some(&mo)));
    assert_eq!(
        state["state"]["character_id"],
        json!(null),
        "the moderator has no character in the world"
    );
    ok(post(
        &mut rig,
        "/admin/api/powers",
        json!({"op": "visible"}),
        &ada,
    ));
    assert!(!rig.server.world.players.is_hidden(guid(ADA)));
    assert_eq!(
        post(
            &mut rig,
            "/admin/api/powers",
            json!({"op": "teleport"}),
            &ada
        )
        .status,
        400
    );
    assert_eq!(
        post(
            &mut rig,
            "/admin/api/powers",
            json!({"op": "spectate"}),
            &ada
        )
        .status,
        400,
        "no target"
    );
    // Chat powers and chat edits write the same staff rows, by the character's name.
    rig.command(ada_key, "realm invisible");
    assert!(said(
        &rig.command(ada_key, "realm coins Pip Player give 5"),
        "Done for character"
    ));
    // One row per applied power; the refusals left none.
    let rows: Vec<_> = audit_rows(&files)
        .into_iter()
        .filter(|row| row.action != "login")
        .map(|row| (row.action, row.via, row.target.character_id))
        .collect();
    let row = |action: &str, via: &str, target| (action.to_owned(), via.to_owned(), target);
    assert_eq!(
        rows,
        [
            row("coins", "chat", Some(pip)),
            row("invisible", "chat", None),
            row("visible", "panel", None),
            row("invisible", "panel", None),
            row("unfreeze", "panel", Some(pip)),
            row("freeze", "panel", Some(pip)),
        ]
    );
    assert_eq!(rows_of(&files, "invisible")[0].staff.name, "Ada Admin");
    assert!(
        rows_of(&files, "coins")[0].after["undo"].is_string(),
        "a chat edit can be undone from the panel"
    );
    assert_eq!(
        rows_of(&files, "freeze")[0].after["result"],
        "Froze Pip Player."
    );
}

#[test]
fn admin_moves_send_goto_and_bring_go_through_g5() {
    let Some((files, mut rig)) = panel("move", SESSION_TTL, None) else {
        return;
    };
    let pip_key = rig.join(PIP).unwrap();
    let pip = guid(PIP);
    let ada = sign_in(&mut rig, ADA_HANDLE);
    let path = format!("/admin/api/players/{pip}/move");
    let at = rig.server.sessions[&pip_key].gateway.state().clone();
    assert_eq!(at.zone_id, 0, "the rig's players arrive in the open world");
    // Send moves the player alone, so it needs no character of the admin's.
    let spot = [at.position[0] + 3.0, at.position[1], at.position[2] + 4.0];
    let sent = post(
        &mut rig,
        &path,
        json!({"op": "send", "to": {"zone": 0, "position": spot}}),
        &ada,
    );
    let sent = ok(sent);
    let now = rig.server.sessions[&pip_key].gateway.state().position;
    assert!(
        (now[0] - spot[0]).abs() < 0.1 && (now[2] - spot[2]).abs() < 0.1,
        "{now:?} vs {spot:?}"
    );
    assert_eq!(sent["zone"], 0);
    assert_eq!(rows_of(&files, "send")[0].target.character_id, Some(pip));
    assert_eq!(
        post(
            &mut rig,
            &path,
            json!({"op": "send", "to": {"zone": 3, "position": spot}}),
            &ada
        )
        .status,
        422
    );
    assert_eq!(
        post(&mut rig, &path, json!({"op": "send"}), &ada).status,
        400
    );
    // Goto and bring move the admin's own character, which is not in the world yet.
    for op in ["goto", "bring"] {
        let reply = post(&mut rig, &path, json!({"op": op}), &ada);
        assert_eq!(reply.status, 409, "{op}");
        assert!(
            reply.json()["error"]
                .as_str()
                .unwrap()
                .contains("sign in to the game")
        );
    }
    let ada_key = rig.join(ADA).unwrap();
    let goto = post(&mut rig, &path, json!({"op": "goto"}), &ada);
    let pip_at = rig.server.sessions[&pip_key].gateway.state().position;
    let goto = ok(goto);
    let here = rig.server.sessions[&ada_key].gateway.state().position;
    assert!(
        (here[0] - pip_at[0]).abs() < 0.1 && (here[2] - pip_at[2]).abs() < 0.1,
        "{here:?} vs {pip_at:?}"
    );
    assert!(
        goto["message"].as_str().unwrap().starts_with("Moved you"),
        "{goto}"
    );
    // Bring moves the player to the admin's spot.
    let send_back = json!({"op": "send", "to": {"position": spot}});
    ok(post(&mut rig, &path, send_back, &ada));
    let brought = ok(post(&mut rig, &path, json!({"op": "bring"}), &ada));
    let there = rig.server.sessions[&ada_key].gateway.state().position;
    let now = rig.server.sessions[&pip_key].gateway.state().position;
    assert!(
        (now[0] - there[0]).abs() < 0.1 && (now[2] - there[2]).abs() < 0.1,
        "{now:?} vs {there:?}"
    );
    assert_eq!(brought["zone"], 0);
    let rows: Vec<_> = audit_rows(&files)
        .into_iter()
        .filter(|row| row.action != "login")
        .map(|row| row.action)
        .collect();
    assert_eq!(rows, ["bring", "send", "goto", "send"]);
}

#[test]
fn admin_jail_holds_a_player_until_release() {
    let Some((files, mut rig)) = panel("jail", SESSION_TTL, None) else {
        return;
    };
    let pip_key = rig.join(PIP).unwrap();
    let pip = guid(PIP);
    let mo = sign_in(&mut rig, MO_HANDLE);
    let moderate = format!("/admin/api/players/{pip}/moderate");
    // The hierarchy holds: a moderator cannot jail an admin.
    let admin = format!("/admin/api/players/{}/moderate", guid(ADA));
    assert_eq!(
        post(&mut rig, &admin, json!({"op": "jail", "reason": "x"}), &mo).status,
        403
    );
    let jailed = ok(post(
        &mut rig,
        &moderate,
        json!({"op": "jail", "reason": "griefing"}),
        &mo,
    ));
    assert_eq!(jailed["row"]["action"], "jail");
    let spot = admin::moderation::jail_spot().unwrap();
    let now = rig.server.sessions[&pip_key].gateway.state().position;
    assert!(
        (now[0] - spot[0]).abs() < 0.1 && (now[2] - spot[2]).abs() < 0.1,
        "{now:?} vs {spot:?}"
    );
    assert!(rig.server.world.gm.jailed.contains(&pip));
    assert!(said(&rig.drain(pip_key), "A GM has jailed you"));
    assert_eq!(
        post(&mut rig, &moderate, json!({"op": "jail"}), &mo).status,
        409,
        "already jailed"
    );
    // A relog does not walk out: the next tick holds the character again, at the jail.
    let session = rig.server.sessions.remove(&pip_key).unwrap();
    rig.server.end_session(pip_key, session, "test_leave");
    let back = rig.join(PIP).unwrap();
    told(&mut rig);
    let held = Some(osfr_tools::movement::StaffMotion::Held);
    assert_eq!(rig.server.sessions[&back].gateway.staff_motion(), held);
    let now = rig.server.sessions[&back].gateway.state().position;
    assert!(
        (now[0] - spot[0]).abs() < 0.1 && (now[2] - spot[2]).abs() < 0.1,
        "{now:?} vs {spot:?}"
    );
    let released = ok(post(&mut rig, &moderate, json!({"op": "release"}), &mo));
    assert_eq!(released["moderation"][0]["action"], "release");
    assert!(rig.server.world.gm.jailed.is_empty());
    assert_eq!(rig.server.sessions[&back].gateway.staff_motion(), None);
    assert!(said(&rig.drain(back), "released you from jail"));
    let rows = rows_of(&files, "jail");
    assert_eq!(
        (rows[0].reason.as_deref(), rows[0].target.character_id),
        (Some("griefing"), Some(pip))
    );
}

#[test]
fn admin_the_map_lists_zone_players_and_the_install_map() {
    let install = test_support::Scratch::new("admin-map-install");
    let Some((_files, mut rig)) = panel("map", SESSION_TTL, Some(install.path().to_owned())) else {
        return;
    };
    let pip_key = rig.join(PIP).unwrap();
    let mo = sign_in(&mut rig, MO_HANDLE);
    let zone = rig.server.sessions[&pip_key]
        .gateway
        .runtime_zone()
        .unwrap();
    // A one-tile map for the player's zone, in a one-file pack (the icons test's layout).
    let tile = "Test_Tile_0.dds";
    let list = format!("{tile}\t-100\t0\t-50\t200\t100\n");
    let mut pack = 0u32.to_be_bytes().to_vec();
    let entries = [
        (
            format!("{}_TileInfo.txt", zone.name).to_lowercase(),
            list.into_bytes(),
        ),
        (tile.to_lowercase(), super::dxt1()),
    ];
    pack.extend((entries.len() as u32).to_be_bytes());
    let head: usize = pack.len()
        + entries
            .iter()
            .map(|(name, _)| 16 + name.len())
            .sum::<usize>();
    let mut at = head;
    let mut bodies: Vec<u8> = Vec::new();
    for (name, bytes) in &entries {
        pack.extend((name.len() as u32).to_be_bytes());
        pack.extend(name.as_bytes());
        pack.extend((at as u32).to_be_bytes());
        pack.extend((bytes.len() as u32).to_be_bytes());
        pack.extend(0u32.to_be_bytes());
        at += bytes.len();
        bodies.extend(bytes);
    }
    pack.extend(bodies);
    fs::write(install.path().join("Assets_000.pack"), pack).unwrap();
    let map = ok(get(
        &mut rig,
        &format!("/admin/api/map/{}", zone.id),
        Some(&mo),
    ));
    assert_eq!(map["zone_name"], json!(zone.name));
    assert_eq!(
        map["bounds"],
        json!([-100.0, -50.0, 0.0, 150.0]),
        "the tile's box, x then z"
    );
    let players = map["players"].as_array().unwrap();
    assert_eq!(players.len(), 1);
    assert_eq!(players[0]["character_id"], guid(PIP));
    let image = map["image"].as_str().unwrap().to_owned();
    assert_eq!(get(&mut rig, &image, None).status, 401, "staff only");
    let png = get(&mut rig, &image, Some(&mo));
    assert_eq!(
        (png.status, png.header("content-type")),
        (200, Some("image/png"))
    );
    assert_eq!(&png.body[..8], b"\x89PNG\r\n\x1a\n");
    // Another zone: nobody there, no map art, no box.
    let empty = ok(get(
        &mut rig,
        &format!("/admin/api/map/{}", zone.id + 1000),
        Some(&mo),
    ));
    assert_eq!(
        (
            empty["players"].as_array().map(Vec::len),
            empty["image"].is_null()
        ),
        (Some(0), true)
    );
    assert!(empty["bounds"].is_null());
    assert_eq!(get(&mut rig, "/admin/api/map/x", Some(&mo)).status, 400);
}

/// rc-0136 F1: the panel gives a job the way a trainer does and switches to it the way the job
/// list does (the client gets the 38/21 activation), admins only, one `jobs` row with the learned
/// jobs and the active job before and after, undone by its key. Complete is not Grant: it
/// finishes the quests before the one named, so the next quest of the chain is offered.
#[test]
fn admin_jobs_learn_and_switch_and_complete_finishes_the_chain() {
    let Some((files, mut rig)) = panel("jobs", SESSION_TTL, None) else {
        return;
    };
    let pip_key = rig.join(PIP).unwrap();
    let pip = guid(PIP);
    let (ada, mo) = (sign_in(&mut rig, ADA_HANDLE), sign_in(&mut rig, MO_HANDLE));
    let base = format!("/admin/api/players/{pip}");
    let gameplay = |rig: &Rig| {
        rig.server.sessions[&pip_key]
            .gateway
            .gameplay()
            .unwrap()
            .state()
    };
    let before = gameplay(&rig).profiles.unwrap();
    // A job with a starter outfit in the rig's opening content that Pip has not learned.
    let job = [14, 11, 16, 35, 45]
        .into_iter()
        .find(|job| !before.unlocked.contains(job))
        .unwrap();
    let learn = json!({"job_id": job, "op": "learn", "active": true});
    assert_eq!(
        post(&mut rig, &format!("{base}/jobs"), learn.clone(), &mo).status,
        403
    );
    told(&mut rig);
    let learned = ok(post(&mut rig, &format!("{base}/jobs"), learn.clone(), &ada));
    let row = |answer: &Value| {
        (answer["jobs"].as_array().into_iter().flatten())
            .find(|row| row["job_id"] == job)
            .cloned()
    };
    assert_eq!(
        row(&learned).map(|row| row["active"].clone()),
        Some(json!(true)),
        "{learned}"
    );
    let after = gameplay(&rig).profiles.unwrap();
    assert!(after.unlocked.contains(&job) && after.active == job);
    let sent = told(&mut rig).remove(&pip_key).unwrap_or_default();
    assert!(
        sent.iter()
            .any(|m| m.windows(4).any(|w| w == [38, 0, 21, 0])),
        "the switch reached the player's client as the job list's 38/21"
    );
    // Refusals change nothing and write no row: known already, a job never learned.
    assert_eq!(
        post(&mut rig, &format!("{base}/jobs"), learn, &ada).status,
        422
    );
    let never = json!({"job_id": 137, "op": "activate"});
    assert_eq!(
        post(&mut rig, &format!("{base}/jobs"), never, &ada).status,
        422
    );
    let mixed = json!({"job_id": job, "op": "learn", "level": 3});
    assert_eq!(
        post(&mut rig, &format!("{base}/jobs"), mixed, &ada).status,
        400
    );
    let jobs = rows_of(&files, "jobs");
    assert_eq!(jobs.len(), 1, "one row for the applied edit");
    assert_eq!(jobs[0].before["active_job"], json!(before.active));
    assert_eq!(jobs[0].after["active_job"], json!(job));
    assert!(
        !jobs[0].before["jobs_learned"]
            .as_array()
            .unwrap()
            .contains(&json!(job))
    );
    assert!(
        jobs[0].after["jobs_learned"]
            .as_array()
            .unwrap()
            .contains(&json!(job))
    );
    // A second job given without the switch, then switched to through the job list's path; that
    // switch's undo switches back. (The rig's synthetic Adventurer wears no creation clothes, so
    // the switch back to it is refused as the client's profile swap would be: 422.)
    let second = [14, 11, 16, 35, 45]
        .into_iter()
        .find(|other| *other != job && !before.unlocked.contains(other))
        .unwrap();
    let quiet = json!({"job_id": second, "op": "learn"});
    ok(post(&mut rig, &format!("{base}/jobs"), quiet, &ada));
    assert_eq!(
        gameplay(&rig).profiles.unwrap().active,
        job,
        "learn alone does not switch"
    );
    let back = json!({"job_id": before.active, "op": "activate"});
    assert_eq!(
        post(&mut rig, &format!("{base}/jobs"), back, &ada).status,
        422
    );
    let switch = json!({"job_id": second, "op": "activate"});
    let switched = ok(post(&mut rig, &format!("{base}/jobs"), switch, &ada));
    assert_eq!(gameplay(&rig).profiles.unwrap().active, second);
    let undo = switched["undo"].as_str().unwrap().to_owned();
    ok(post(
        &mut rig,
        "/admin/api/undo",
        json!({"undo": undo}),
        &ada,
    ));
    assert_eq!(
        gameplay(&rig).profiles.unwrap().active,
        job,
        "the undo switched back"
    );
    // The first learn's undo is stale now (a job was learned since): refused, nothing changes.
    let undo = learned["undo"].as_str().unwrap().to_owned();
    assert_eq!(
        post(&mut rig, "/admin/api/undo", json!({"undo": undo}), &ada).status,
        409
    );
    assert_eq!(gameplay(&rig).profiles.unwrap().active, job);
    // Complete finishes the chain before the quest: Smoggo's Sword, Slash and Smash, then
    // Fighting is for the Birds! (the bundled catalogue). Grant would leave them open.
    let complete = json!({"op": "complete", "quest_id": 3_957_750});
    let done = ok(post(&mut rig, &format!("{base}/quests"), complete, &ada));
    let ids: Vec<u64> = (done["done"].as_array().into_iter().flatten())
        .filter_map(|row| row["quest_id"].as_u64())
        .collect();
    for quest in [3_955_917, 3_955_914, 3_957_750] {
        assert!(ids.contains(&quest), "{quest} done: {done}");
    }
    let row = &rows_of(&files, "quests")[0];
    assert_eq!(row.after["quests"]["3955917"], "completed", "{}", row.after);
}
