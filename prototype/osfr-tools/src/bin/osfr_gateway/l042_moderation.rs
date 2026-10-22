//! L042 moderation through the production path: account-store sessions bound by a real
//! handoff ticket, `realm` chat through `Session::dispatch_message` (gateway role gate, staged
//! command, post-commit host run) and `Server::step` for the close path. Synthetic names,
//! ids and chat text only; every file lives in a `test_support::Scratch` directory.
use super::super::tests::test_support::{self, Scratch};
use super::super::*;
use osfr_tools::account::{AccountFlags, SlotPolicy};
use osfr_tools::account_store::{AccountRole, AccountStoreLock};
use osfr_tools::moderation::{Command, MAX_MINUTES, Sanctions, UNTIL_LIFTED, run, unix_now};

const HANDOFF: &[u8] = b"l042-synthetic-handoff-key";
const MUTED: &str = "cannot chat right now";
const DENIED: &str = "authorized account role";

/// One account and one character each: account id = index + 1, character guid = index + 101.
/// "Ann Lee" "Moss" and "Ann" "Lee Moss" are distinct name pairs on two accounts that display
/// alike (an older store; creation now refuses the second). Quill's account also holds
/// `SPARE`, a look-alike of its own name.
const CAST: [(AccountRole, &str, &str); 8] = [
    (AccountRole::Admin, "Ada", "Admin"),
    (AccountRole::Moderator, "Mo", "Moderator"),
    (AccountRole::Moderator, "Mia", "Moderator"),
    (AccountRole::Player, "Pip", "Player"),
    (AccountRole::Builder, "Bea", "Builder"),
    (AccountRole::Player, "Ann Lee", "Moss"),
    (AccountRole::Player, "Ann", "Lee Moss"),
    (AccountRole::Player, "Quill Zed", "Vox"),
];
const ADA: usize = 0;
const MO: usize = 1;
const MIA: usize = 2;
const PIP: usize = 3;
const BEA: usize = 4;
const QUILL: usize = 7;
/// Guid, first and last name of the second character on Quill's account.
const SPARE: (u64, &str, &str) = (201, "Quill", "Zed Vox");

fn account(index: usize) -> u64 {
    index as u64 + 1
}

fn guid(index: usize) -> u64 {
    index as u64 + 101
}

/// The account store, saves and security log of one test, kept across server restarts.
struct Files(Scratch);

impl Files {
    fn new(tag: &str) -> Self {
        let files = Self(Scratch::new(&format!("l042-{tag}")));
        write_store(&files.store());
        files
    }

    fn dir(&self) -> &Path {
        self.0.path()
    }

    fn store(&self) -> PathBuf {
        self.dir().join("accounts.json")
    }

    fn sanctions(&self, index: usize) -> Sanctions {
        AccountStore::load(&self.store())
            .unwrap()
            .sanctions(account(index))
    }

    /// Moderation rows in the security log.
    fn audit(&self) -> Vec<serde_json::Value> {
        fs::read_to_string(self.dir().join("security.jsonl"))
            .unwrap_or_default()
            .lines()
            .map(|line| serde_json::from_str::<serde_json::Value>(line).unwrap())
            .filter(|row| row["kind"] == "moderation_action")
            .collect()
    }
}

/// The public store API cannot add characters, so they are written into the saved document
/// and the result is checked by the production loader.
fn write_store(path: &Path) {
    let mut store = AccountStore::new();
    for (index, (role, ..)) in CAST.iter().enumerate() {
        let flags = AccountFlags {
            open_chat: true,
            ..AccountFlags::default()
        };
        let id = store
            .create_account(
                [index as u8 + 1; 32],
                flags,
                SlotPolicy { max_characters: 2 },
            )
            .unwrap();
        store.set_account_role_offline(id, *role).unwrap();
    }
    store.save(path).unwrap();
    let mut document: serde_json::Value = serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/fixtures/account/legacy-identity-v1.json"
    ))
    .unwrap();
    let (spare, spare_first, spare_last) = SPARE;
    let characters = CAST
        .iter()
        .enumerate()
        .map(|(index, (_, first, last))| (guid(index), index, *first, *last))
        .chain([(spare, QUILL, spare_first, spare_last)]);
    for (guid, index, first, last) in characters {
        let mut identity = fixture["character"].clone();
        identity["first_name"] = json!(first);
        identity["last_name"] = json!(last);
        identity["creation"]["first_name"] = json!(first);
        identity["creation"]["last_name"] = json!(last);
        identity["creation"]["temporary_first_name"] = json!("");
        identity["creation"]["temporary_last_name"] = json!("");
        document["store"]["characters"][guid.to_string()] = json!({
            "guid": guid, "owner_account_id": account(index),
            "identity": identity, "world_state": null,
        });
        document["store"]["accounts"][account(index).to_string()]["character_guids"]
            .as_array_mut()
            .unwrap()
            .push(json!(guid));
    }
    document["store"]["next_character_guid"] = json!(spare + 1);
    fs::write(path, serde_json::to_vec(&document).unwrap()).unwrap();
    AccountStore::load(path).unwrap();
}

fn bind(guid: u64, ticket: &str) -> Vec<u8> {
    let mut bytes = 1u16.to_le_bytes().to_vec();
    for (index, text) in [ticket, "1.910.1.530630", ""].into_iter().enumerate() {
        bytes.extend((text.len() as u32).to_le_bytes());
        bytes.extend(text.as_bytes());
        if index == 0 {
            bytes.extend(guid.to_le_bytes());
        }
    }
    bytes
}

fn said(messages: &[Vec<u8>], text: &str) -> bool {
    messages.iter().any(|message| {
        message
            .windows(text.len())
            .any(|bytes| bytes == text.as_bytes())
    })
}

/// A typed chat request on `channel` (15/1); the mute guard runs before any body field.
fn chat_on(channel: u16, text: &str) -> Vec<u8> {
    let mut inner = osfr_tools::chat::local_request(text);
    inner[4..6].copy_from_slice(&channel.to_le_bytes());
    inner
}

/// A real server (socket, template, security log) whose sessions are driven directly.
struct Rig {
    server: Server,
    next: u32,
}

impl Rig {
    fn new(files: &Files) -> Self {
        let mut options = test_support::options(files.dir().join("state.json"));
        options.account_store = Some(files.store());
        options.handoff_key = HANDOFF.to_vec();
        options.security_log = Some(files.dir().join("security.jsonl"));
        Self {
            server: Server::new(options).unwrap(),
            next: 0,
        }
    }

    /// `Server::step`, the production close path. It ticks and flushes every session, and
    /// nothing here acknowledges those sends: wait out the sender's 250 ms retry interval
    /// (`Session::new`) so `drain` takes them back as resends and acknowledges them.
    fn step(&mut self) {
        self.server.step().unwrap();
        std::thread::sleep(Duration::from_millis(300));
        for key in self.server.sessions.keys().copied().collect::<Vec<_>>() {
            self.drain(key);
        }
    }

    /// A new connection binds `index`'s character with a valid ticket and enters the world.
    /// A refused binding ends the session through the production close path.
    fn join(&mut self, index: usize) -> Result<SessionKey, String> {
        self.next += 1;
        let id = 7_000 + self.next;
        let peer: SocketAddr = format!("127.0.0.1:{}", 30_000 + id).parse().unwrap();
        let request = SessionRequest {
            id,
            crc_length: 3,
            buffer_size: 1400,
        };
        let session = Session::new(
            SessionId(u64::from(id)),
            peer,
            request,
            &self.server.options,
            CRC_SEED,
            None,
            self.server.template.as_ref(),
        )
        .unwrap();
        let key = SessionKey {
            peer,
            connection_id: id,
        };
        self.server.world.session_keys.insert(session.id, key);
        self.server.sessions.insert(key, session);
        let store =
            AccountStore::load(&self.server.options.account_store.clone().unwrap()).unwrap();
        let ticket = login::gateway_binding_ticket(
            HANDOFF,
            store.account(account(index)).unwrap(),
            guid(index),
        );
        let result = self
            .send(key, &bind(guid(index), &ticket))
            .and_then(|()| self.send(key, &wrap_application(&[13, 0])))
            .and_then(|()| self.send(key, &wrap_application(&[10, 0])));
        if let Err(error) = result {
            let session = self.server.sessions.remove(&key).unwrap();
            self.server.end_session(key, session, &error);
            return Err(error);
        }
        self.drain(key);
        Ok(key)
    }

    /// The server's own dispatch clock, so `step` and these calls never run backwards.
    fn now(&self) -> u64 {
        self.server.start.elapsed().as_millis() as u64
    }

    fn send(&mut self, key: SessionKey, message: &[u8]) -> Result<(), String> {
        let now = self.now();
        let mut session = self.server.sessions.remove(&key).expect("live session");
        let result = session
            .dispatch_message(
                message,
                &self.server.options,
                now,
                &mut self.server.world,
                &mut self.server.membership,
                &mut self.server.sessions,
                None,
            )
            .map_err(|error| error.to_string());
        self.server.sessions.insert(key, session);
        result
    }

    /// Send one typed chat line and return what the sender saw.
    fn say(&mut self, key: SessionKey, inner: &[u8]) -> Vec<Vec<u8>> {
        self.send(key, &wrap_application(inner)).unwrap();
        self.drain(key)
    }

    fn command(&mut self, key: SessionKey, text: &str) -> Vec<Vec<u8>> {
        self.say(key, &osfr_tools::chat::local_request(text))
    }

    fn drain(&mut self, key: SessionKey) -> Vec<Vec<u8>> {
        let now = self.now();
        let session = self.server.sessions.get_mut(&key).unwrap();
        super::super::f120_fishing_hour_reconnect::drain_replies(session, now)
    }

    fn live(&self, key: SessionKey) -> bool {
        self.server
            .sessions
            .get(&key)
            .is_some_and(|session| !session.kicked)
    }
}

#[test]
fn moderator_cannot_target_admin_moderator_or_self() {
    let files = Files::new("hierarchy");
    let mut rig = Rig::new(&files);
    let [ada, mo, mia] = [ADA, MO, MIA].map(|index| rig.join(index).unwrap());
    let before = fs::read(files.store()).unwrap();
    for (target, refusal) in [
        ("Ada Admin", "equal to or above yours"),
        ("Mia Moderator", "equal to or above yours"),
        ("Mo Moderator", "your own account"),
    ] {
        for action in ["mute", "ban", "kick"] {
            let replies = rig.command(mo, &format!("realm {action} {target}"));
            assert!(said(&replies, refusal), "{action} {target}");
        }
    }
    assert_eq!(fs::read(files.store()).unwrap(), before);
    assert!([ada, mo, mia].iter().all(|key| rig.live(*key)));
    rig.step();
    assert!([ada, mo, mia].iter().all(|key| rig.live(*key)));
    // An admin strictly outranks a moderator; the same path then applies the ban.
    assert!(said(
        &rig.command(ada, "realm ban Mia Moderator 5"),
        "Player banned."
    ));
    assert!(files.sanctions(MIA).banned(unix_now()));
    assert_eq!(files.audit().len(), 1);
    // `realm role` stays admin-only; an admin can grant the moderator role.
    assert!(said(&rig.command(mo, "realm role 4 moderator"), DENIED));
    assert!(said(
        &rig.command(ada, "realm role 4 moderator"),
        "Account role saved."
    ));
    let store = AccountStore::load(&files.store()).unwrap();
    assert_eq!(
        store.account_role(account(PIP)),
        Some(AccountRole::Moderator)
    );
}

#[test]
fn moderation_refusals_change_nothing() {
    let files = Files::new("refusals");
    let mut rig = Rig::new(&files);
    let [mo, pip, bea] = [MO, PIP, BEA].map(|index| rig.join(index).unwrap());
    let before = fs::read(files.store()).unwrap();
    let over = format!("realm ban Pip Player {}", *MAX_MINUTES + 1);
    for (actor, text, refusal) in [
        (pip, "realm mute Mo Moderator", DENIED),
        (pip, "realm ban Bea Builder 5", DENIED),
        (bea, "realm kick Pip Player", DENIED),
        (mo, "realm mute Nobody Here", "no single character"),
        (mo, "realm mute Pip", "no single character"),
        (mo, "realm mute Ann Lee Moss", "no single character"),
        (mo, "realm mute Pip Player 0", "minutes must be"),
        (mo, over.as_str(), "minutes must be"),
        (mo, "realm ban Pip Player 4294967296", "minutes must be"),
        (mo, "realm kick Pip Player 5", "Usage:"),
        (mo, "realm mute", "Usage:"),
        (mo, "realm unmute Pip Player", "not muted"),
        (mo, "realm unban Pip Player", "not banned"),
    ] {
        assert!(said(&rig.command(actor, text), refusal), "{text}");
        assert_eq!(fs::read(files.store()).unwrap(), before, "{text}");
        assert!([mo, pip, bea].iter().all(|key| rig.live(*key)), "{text}");
    }
    // Another writer holds the store lock past the lock's bounded wait (a stale lock): the
    // write is refused as busy and changes nothing. `step` waits out the burst
    // the loop spent of the moderator's action budget.
    rig.step();
    let lock = AccountStoreLock::acquire(&files.store()).unwrap();
    assert!(said(&rig.command(mo, "realm ban Pip Player 5"), "busy"));
    drop(lock);
    assert_eq!(fs::read(files.store()).unwrap(), before);
    rig.step();
    assert!([mo, pip, bea].iter().all(|key| rig.live(*key)));
    assert!(files.audit().is_empty());
    // A builder ranks with players, so a moderator may act on one.
    assert!(said(
        &rig.command(mo, "realm mute bea BUILDER 5"),
        "Player muted."
    ));
    assert!(files.sanctions(BEA).muted(unix_now()));
}

#[test]
fn mute_blocks_chat_and_survives_relog() {
    let files = Files::new("mute");
    let mut rig = Rig::new(&files);
    let [mo, pip] = [MO, PIP].map(|index| rig.join(index).unwrap());
    rig.say(pip, &chat_on(0, "l042 before the mute"));
    assert!(said(&rig.drain(mo), "l042 before the mute"));

    assert!(said(
        &rig.command(mo, "realm mute Pip Player 60"),
        "Player muted."
    ));
    let now = unix_now();
    let until = files.sanctions(PIP).muted_until.unwrap();
    assert!((now + 3_590..=now + 3_610).contains(&until), "{until}");
    // (R) A shorter second mute keeps the later expiry.
    assert!(said(
        &rig.command(mo, "realm mute Pip Player 1"),
        "Player muted."
    ));
    assert_eq!(files.sanctions(PIP).muted_until, Some(until));

    let mut quick = vec![67, 0, 3, 0];
    quick.extend([0; 24]);
    let mut tell = vec![67, 0, 2, 0];
    tell.extend([0; 24]);
    // 129/11 guild MOTD: guild GUID, then the text; answered before the guild's own checks.
    let mut motd = vec![129, 0, 11, 0];
    motd.extend(0u64.to_le_bytes());
    motd.extend(4u32.to_le_bytes());
    motd.extend(b"l042");
    for (inner, text) in [
        (chat_on(0, "l042 muted say"), "l042 muted say"),
        (chat_on(1, "l042 muted tell"), "l042 muted tell"),
        (chat_on(4, "l042 muted group"), "l042 muted group"),
        (chat_on(9, "l042 muted guild"), "l042 muted guild"),
        (quick.clone(), "l042"),
        (tell.clone(), "l042"),
        (motd, "l042"),
    ] {
        assert!(said(&rig.say(pip, &inner), MUTED), "{text}");
        assert!(!said(&rig.drain(mo), text), "{text}");
    }
    // Ordinary realm commands are not chat and still answer a muted player.
    assert!(!said(&rig.command(pip, "realm help"), MUTED));

    // Relog on a restarted server: the mute is read back from the account store.
    drop(rig);
    let mut rig = Rig::new(&files);
    let [mo, pip] = [MO, PIP].map(|index| rig.join(index).unwrap());
    assert!(said(&rig.say(pip, &chat_on(0, "l042 after relog")), MUTED));
    assert!(!said(&rig.drain(mo), "l042 after relog"));

    // Expiry by wall clock: lift, then mute again with a clock two minutes behind.
    assert!(said(
        &rig.command(mo, "realm unmute Pip Player"),
        "Player unmuted."
    ));
    let store = files.store();
    let command = Command::parse("mute pip player 1").unwrap();
    run(&store, account(MO), &command, now - 120).unwrap();
    assert!(files.sanctions(PIP).muted_until.is_some());
    rig.say(pip, &chat_on(0, "l042 after expiry"));
    assert!(said(&rig.drain(mo), "l042 after expiry"));
    // An expired mute is "not muted" and the command changes nothing.
    let before = fs::read(&store).unwrap();
    assert!(said(
        &rig.command(mo, "realm unmute Pip Player"),
        "not muted"
    ));
    assert_eq!(fs::read(&store).unwrap(), before);
    // (R) An unreadable account store refuses chat, as it drops the role to player.
    fs::write(&store, b"{").unwrap();
    assert!(said(&rig.say(mo, &chat_on(0, "l042 unreadable")), MUTED));
    fs::write(&store, &before).unwrap();
    assert!(!said(&rig.say(mo, &chat_on(0, "l042 readable")), MUTED));
}

#[test]
fn ban_kicks_only_the_target() {
    let files = Files::new("ban");
    let mut rig = Rig::new(&files);
    let [ada, mo, pip, bea] = [ADA, MO, PIP, BEA].map(|index| rig.join(index).unwrap());
    assert!(said(
        &rig.command(mo, "realm ban Pip Player 30"),
        "Player banned."
    ));
    assert!(!rig.live(pip));
    // Until `step` ends it, the banned session's messages are dropped unread.
    rig.say(pip, &chat_on(0, "l042 after the ban"));
    assert!(!said(&rig.drain(mo), "l042 after the ban"));
    rig.step();
    assert!(!rig.server.sessions.contains_key(&pip));
    assert!([ada, mo, bea].iter().all(|key| rig.live(*key)));
    assert!(!rig.server.world.active_sessions.contains_key(&guid(PIP)));
    // The close path saved the kicked character.
    assert!(character_state_path(&files.dir().join("state.json"), guid(PIP)).is_file());

    // The ban refuses the gateway binding and changes nothing.
    let before = fs::read(files.store()).unwrap();
    assert_eq!(rig.join(PIP).unwrap_err(), "account is banned");
    assert_eq!(fs::read(files.store()).unwrap(), before);

    // After it expires the binding works again.
    assert!(said(
        &rig.command(mo, "realm unban Pip Player"),
        "Player unbanned."
    ));
    let command = Command::parse("ban pip player 1").unwrap();
    let past = unix_now() - 120;
    run(&files.store(), account(MO), &command, past).unwrap();
    let pip = rig.join(PIP).unwrap();

    // A kick ends the live session only; the player may come straight back.
    assert!(said(
        &rig.command(mo, "realm kick Pip Player"),
        "Player kicked."
    ));
    rig.step();
    assert!(!rig.server.sessions.contains_key(&pip));
    assert!([ada, mo, bea].iter().all(|key| rig.live(*key)));
    let pip = rig.join(PIP).unwrap();
    assert!(rig.live(pip));
    // A kick is audited; once the player is gone the same kick is refused and not audited.
    let rows = files.audit().len();
    assert!(said(
        &rig.command(mo, "realm kick Bea Builder"),
        "Player kicked."
    ));
    rig.step();
    assert!(said(
        &rig.command(mo, "realm kick Bea Builder"),
        "not online"
    ));
    assert_eq!(files.audit().len(), rows + 1);
}

/// A player whose spare slot holds a look-alike of their own name ("Quill Zed" "Vox" and
/// "Quill" "Zed Vox") is still one target: sanctions are per account.
#[test]
fn one_accounts_look_alike_names_stay_mutable_kickable_and_bannable() {
    let files = Files::new("look-alike");
    let mut rig = Rig::new(&files);
    let [mo, quill] = [MO, QUILL].map(|index| rig.join(index).unwrap());
    assert!(said(
        &rig.command(mo, "realm mute Quill Zed Vox 5"),
        "Player muted."
    ));
    assert!(files.sanctions(QUILL).muted(unix_now()));
    assert!(said(&rig.say(quill, &chat_on(0, "l042 look-alike")), MUTED));
    assert!(said(
        &rig.command(mo, "realm kick Quill Zed Vox"),
        "Player kicked."
    ));
    rig.step();
    assert!(!rig.server.sessions.contains_key(&quill));
    let quill = rig.join(QUILL).unwrap();
    assert!(said(
        &rig.command(mo, "realm ban quill zed VOX 5"),
        "Player banned."
    ));
    assert!(files.sanctions(QUILL).banned(unix_now()));
    assert!(!rig.live(quill));
    let rows = files.audit();
    assert_eq!(rows.len(), 3, "{rows:?}");
    assert!(
        rows.iter()
            .all(|row| row["target_account_id"] == account(QUILL))
    );
}

#[test]
fn moderation_audit_has_ids_only() {
    let files = Files::new("audit");
    let mut rig = Rig::new(&files);
    let [mo, _pip] = [MO, PIP].map(|index| rig.join(index).unwrap());
    rig.command(mo, "realm mute Pip Player 15");
    rig.command(mo, "realm ban Pip Player");
    rig.step();
    let rows = files.audit();
    assert_eq!(rows.len(), 2, "{rows:?}");
    for (row, action, minutes) in [
        (&rows[0], "mute", json!(15)),
        (&rows[1], "ban", json!(null)),
    ] {
        let mut keys: Vec<_> = row.as_object().unwrap().keys().cloned().collect();
        keys.sort();
        assert_eq!(
            keys,
            [
                "account_id",
                "action",
                "kind",
                "minutes",
                "outcome",
                "target_account_id",
                "timestamp_ms"
            ]
        );
        assert_eq!(row["account_id"], account(MO));
        assert_eq!(row["target_account_id"], account(PIP));
        assert_eq!(
            (&row["action"], &row["minutes"]),
            (&json!(action), &minutes)
        );
    }
    let log = fs::read_to_string(files.dir().join("security.jsonl")).unwrap();
    for (_, first, last) in CAST {
        assert!(!log.contains(first) && !log.contains(last));
    }
    // The store keeps ids and times only; a ban without minutes lasts until lifted.
    let document: serde_json::Value =
        serde_json::from_slice(&fs::read(files.store()).unwrap()).unwrap();
    assert_eq!(
        document["store"]["sanctions"][account(PIP).to_string()]["banned_until"],
        json!(UNTIL_LIFTED)
    );
}
