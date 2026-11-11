//! L043: admin `realm move` through the production path, in account-store mode: a real
//! `Server` (its security log opened by `Server::new`), sessions bound by login tickets, chat
//! commands through `Session::dispatch_message`, logout through `Server::end_session`. The
//! characters are synthetic; the content is the bundled opening reference.
use super::test_support::{self, Scratch};
use super::*;
use osfr_tools::account::{AccountFlags, SlotPolicy, create_character};
use osfr_tools::account_store::AccountRole;
use osfr_tools::identity::{NativeCreation, Resources};

const HANDOFF: &[u8] = b"l043-synthetic-handoff";
/// Cobblestone Village's hub row (`piece_077` CURATED id 7): no captured facing, no terrain here.
const COBBLESTONE: [f32; 3] = [-1904.46, -39.79, 413.316];

/// The character creator's resource tables, shared with the friends rig in `test_support`.
fn resources(root: &Path) -> Resources {
    test_support::creator_resources(root)
}

struct Rig {
    scratch: Scratch,
    server: Server,
    accounts: PathBuf,
    /// Character guid by synthetic first name.
    guids: BTreeMap<&'static str, u64>,
    keys: BTreeMap<u64, SessionKey>,
    next_id: u64,
}

impl Rig {
    /// One account per `(first name, role)`, each with one character named "<first> Test".
    fn new(tag: &str, people: &[(&'static str, AccountRole)], security_log: bool) -> Self {
        let scratch = Scratch::new(tag);
        let resources = resources(scratch.path());
        let accounts = scratch.path().join("accounts.json");
        let mut store = AccountStore::new();
        let mut guids = BTreeMap::new();
        for (index, (first, role)) in people.iter().enumerate() {
            let account = store
                .create_account(
                    [index as u8 + 1; 32],
                    AccountFlags {
                        open_chat: true,
                        ..Default::default()
                    },
                    SlotPolicy { max_characters: 1 },
                )
                .unwrap();
            store.set_account_role_offline(account, *role).unwrap();
            let mut choices = [0; 21];
            for (slot, value) in [(0, 1), (1, 1), (2, 2), (5, 1), (7, 201), (8, 189), (9, 199)] {
                choices[slot] = value;
            }
            choices[20] = 1;
            let reply = create_character(
                &mut store,
                account,
                NativeCreation {
                    first_name: (*first).into(),
                    last_name: "Test".into(),
                    temporary_first_name: String::new(),
                    temporary_last_name: String::new(),
                    choices,
                },
                &resources,
            );
            assert_eq!(
                reply.status,
                osfr_tools::account::CharacterCreateStatus::Created
            );
            guids.insert(*first, reply.guid);
        }
        store.save(&accounts).unwrap();
        let mut options = test_support::options(scratch.state());
        options.account_store = Some(accounts.clone());
        options.handoff_key = HANDOFF.to_vec();
        options.security_log = security_log.then(|| scratch.path().join("security.jsonl"));
        let server = Server::new(options).unwrap();
        Self {
            scratch,
            server,
            accounts,
            guids,
            keys: BTreeMap::new(),
            next_id: 1,
        }
    }

    fn guid(&self, first: &str) -> u64 {
        self.guids[first]
    }

    /// Log a character in through its binding ticket; `enter` also sends Ready and Loaded.
    fn login(&mut self, first: &str, enter: bool) {
        let guid = self.guid(first);
        let store = AccountStore::load(&self.accounts).unwrap();
        let account = store
            .account(store.character(guid).unwrap().owner_account_id)
            .unwrap();
        let mut login = 1u16.to_le_bytes().to_vec();
        let ticket = login::gateway_binding_ticket(HANDOFF, account, guid);
        for (index, text) in [ticket.as_str(), "1.910.1.530630", ""]
            .into_iter()
            .enumerate()
        {
            login.extend((text.len() as u32).to_le_bytes());
            login.extend(text.as_bytes());
            if index == 0 {
                login.extend(guid.to_le_bytes());
            }
        }
        let id = self.next_id;
        self.next_id += 1;
        let session = Session::new(
            SessionId(id),
            format!("127.0.0.1:{}", 43_000 + id).parse().unwrap(),
            SessionRequest {
                id: id as u32,
                crc_length: 3,
                buffer_size: 1400,
            },
            &self.server.options,
            CRC_SEED,
            None,
            self.server.template.as_deref(),
        )
        .unwrap();
        let key = SessionKey {
            peer: session.peer,
            connection_id: session.request.id,
        };
        self.server.world.session_keys.insert(session.id, key);
        self.server.sessions.insert(key, session);
        self.keys.insert(guid, key);
        self.dispatch(guid, &login, 100).unwrap();
        if enter {
            self.dispatch(guid, &wrap_application(&[13, 0]), 200)
                .unwrap();
            self.dispatch(guid, &wrap_application(&[10, 0]), 300)
                .unwrap();
            assert!(self.session(guid).gateway.is_world_loaded());
        }
        self.drain(guid);
    }

    fn logout(&mut self, first: &str) {
        let key = self.keys.remove(&self.guid(first)).unwrap();
        let session = self.server.sessions.remove(&key).unwrap();
        self.server.end_session(key, session, "logout");
    }

    fn dispatch(&mut self, guid: u64, message: &[u8], now: u64) -> Result<(), String> {
        let key = self.keys[&guid];
        let server = &mut self.server;
        let mut session = server.sessions.remove(&key).unwrap();
        let result = session.dispatch_message(
            message,
            &server.options,
            now,
            &mut server.world,
            &mut server.membership,
            &mut server.sessions,
            None,
        );
        server.sessions.insert(key, session);
        result.map_err(|error| error.to_string())
    }

    fn session(&self, guid: u64) -> &Session {
        &self.server.sessions[&self.keys[&guid]]
    }

    fn drain(&mut self, guid: u64) -> Vec<Vec<u8>> {
        let key = self.keys[&guid];
        let session = self.server.sessions.get_mut(&key).unwrap();
        super::super::f120_fishing_hour_reconnect::drain_replies(session, 2_000)
    }

    /// `realm <text>` typed by `first`; returns the system lines it got back.
    fn realm(&mut self, first: &str, text: &str) -> Vec<String> {
        let guid = self.guid(first);
        let inner = osfr_tools::chat::local_request(&format!("realm {text}"));
        self.dispatch(guid, &wrap_application(&inner), 1_000)
            .unwrap();
        self.drain(guid)
            .iter()
            .filter(|message| message.get(7..11) == Some(&[15, 0, 3, 0]))
            .map(|message| {
                let length = u32::from_le_bytes(message[11..15].try_into().unwrap()) as usize;
                String::from_utf8(message[15..15 + length].to_vec()).unwrap()
            })
            .collect()
    }

    fn save_path(&self, first: &str) -> PathBuf {
        character_state_path(&self.server.options.state, self.guid(first))
    }

    fn audit_rows(&self, kind: &str) -> Vec<serde_json::Value> {
        fs::read_to_string(self.scratch.path().join("security.jsonl"))
            .unwrap_or_default()
            .lines()
            .map(|line| serde_json::from_str::<serde_json::Value>(line).unwrap())
            .filter(|row| row["kind"] == kind)
            .collect()
    }
}

fn json(path: &Path) -> serde_json::Value {
    serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
}

fn without_pose(mut document: serde_json::Value) -> serde_json::Value {
    let character = document["character"].as_object_mut().unwrap();
    character.remove("position");
    character.remove("rotation");
    document
}

/// The native gate in miniature: a logged-out character moved by name to a warpstone has only
/// its pose rewritten on disk, one audit row of numbers is written, and its next login puts it
/// at the warpstone.
#[test]
fn l043_offline_move_rewrites_only_the_pose_and_holds_at_next_login() {
    if !test_support::bin_test_target("l043") {
        return;
    }
    let mut rig = Rig::new(
        "l043-offline",
        &[
            ("Alpha", AccountRole::Admin),
            ("Bravo", AccountRole::Player),
        ],
        true,
    );
    rig.login("Bravo", true);
    rig.logout("Bravo");
    rig.login("Alpha", true);
    let before = json(&rig.save_path("Bravo"));

    let replies = rig.realm("Alpha", "move bravo TEST cobblestone village");
    let bravo = rig.guid("Bravo");
    assert!(
        replies
            .iter()
            .any(|line| line.starts_with(&format!("Moved character {bravo} "))),
        "{replies:?}"
    );
    let after = json(&rig.save_path("Bravo"));
    assert_eq!(
        serde_json::from_value::<[f32; 3]>(after["character"]["position"].clone()).unwrap(),
        COBBLESTONE
    );
    // No captured facing for this hub: the saved facing is kept, so only the position moved.
    assert_eq!(
        after["character"]["rotation"],
        before["character"]["rotation"]
    );
    assert_eq!(
        without_pose(after),
        without_pose(before),
        "only the pose may change"
    );

    let rows = rig.audit_rows("admin_move");
    assert_eq!(rows.len(), 1, "{rows:?}");
    let admin_account = AccountStore::load(&rig.accounts)
        .unwrap()
        .character(rig.guid("Alpha"))
        .unwrap()
        .owner_account_id;
    assert_eq!(rows[0]["account_id"], admin_account);
    assert_eq!(rows[0]["target_guid"], bravo);
    assert_eq!(rows[0]["online"], false);
    let log = fs::read_to_string(rig.scratch.path().join("security.jsonl")).unwrap();
    for text in ["Bravo", "bravo", "Test", "realm", "cobblestone"] {
        assert!(!log.contains(text), "the audit row names {text}");
    }

    rig.login("Bravo", true);
    assert_eq!(rig.session(bravo).gateway.state().position, COBBLESTONE);
}

/// The saved roster exactly as written: the top-level `"pets"` member is the document's last
/// (`game_store::Document` order, pretty-printed), so it runs from its key to the end.
fn saved_roster(document: &[u8]) -> &str {
    let text = std::str::from_utf8(document).unwrap();
    &text[text
        .find("\n  \"pets\": ")
        .expect("the save holds a roster")..]
}

/// L043 on S17: the roster rides in the document `replace_pose` rewrites, so an offline move of
/// a character that owns a pet keeps it byte for byte, and the pet is there at the next login.
#[test]
fn l043_offline_move_keeps_the_pet_roster_byte_for_byte() {
    if !test_support::bin_test_target("l043") {
        return;
    }
    let mut rig = Rig::new(
        "l043-pets",
        &[
            ("Alpha", AccountRole::Admin),
            ("Bravo", AccountRole::Player),
        ],
        true,
    );
    rig.login("Bravo", true);
    rig.logout("Bravo");
    // A real roster (one adopted pet, from the S17 fixture), saved through the one save path.
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/fixtures/save_schema/pets-save-with-pets.json"
    ))
    .unwrap();
    let roster = fixture["pets"].clone();
    let path = rig.save_path("Bravo");
    let character = serde_json::from_value(json(&path)["character"].clone()).unwrap();
    let mut saved = osfr_tools::game_store::load_or_create(&path, character).unwrap();
    saved.pets = Some(roster.clone());
    osfr_tools::game_store::save(&path, &saved).unwrap();
    let before = fs::read(&path).unwrap();
    assert_eq!(
        json(&path)["pets"],
        roster,
        "fixture: the roster was not saved"
    );

    rig.login("Alpha", true);
    let replies = rig.realm("Alpha", "move Bravo Test cobblestone village");
    let bravo = rig.guid("Bravo");
    assert!(
        replies
            .iter()
            .any(|line| line.starts_with(&format!("Moved character {bravo} "))),
        "{replies:?}"
    );
    let after = fs::read(&path).unwrap();
    assert_ne!(after, before, "the move did not rewrite the save");
    assert_eq!(
        saved_roster(&after),
        saved_roster(&before),
        "the offline move rewrote the pet roster"
    );
    assert_eq!(
        without_pose(serde_json::from_slice(&after).unwrap()),
        without_pose(serde_json::from_slice(&before).unwrap()),
        "only the pose may change"
    );

    rig.login("Bravo", true);
    let gateway = &rig.session(bravo).gateway;
    assert_eq!(gateway.state().position, COBBLESTONE);
    assert_eq!(gateway.snapshot().pets, Some(roster), "the pet was lost");
}

/// An online, loaded character gets the 38/12 relocation, is saved at once and is audited.
#[test]
fn l043_online_move_relocates_saves_and_audits() {
    if !test_support::bin_test_target("l043") {
        return;
    }
    let mut rig = Rig::new(
        "l043-online",
        &[
            ("Alpha", AccountRole::Admin),
            ("Bravo", AccountRole::Player),
        ],
        true,
    );
    rig.login("Alpha", true);
    rig.login("Bravo", true);
    let bravo = rig.guid("Bravo");
    let before = rig.session(bravo).gateway.state().clone();
    assert_eq!(
        before.zone_id, 0,
        "fixture: the opening starts in the open world"
    );
    let target = [
        before.position[0] + 7.0,
        before.position[1] + 1.0,
        before.position[2] - 3.0,
    ];

    let replies = rig.realm(
        "Alpha",
        &format!("move {bravo} {} {} {}", target[0], target[1], target[2]),
    );
    assert!(
        replies
            .iter()
            .any(|line| line.starts_with(&format!("Moved character {bravo} "))),
        "{replies:?}"
    );
    let sent = rig.drain(bravo);
    let packet = wrap_application(&osfr_tools::dev_teleport::location_packet(
        target,
        before.rotation,
    ));
    assert!(sent.contains(&packet), "the target got no 38/12 relocation");
    assert_eq!(rig.session(bravo).gateway.state().position, target);
    let saved = json(&rig.save_path("Bravo"));
    assert_eq!(
        serde_json::from_value::<[f32; 3]>(saved["character"]["position"].clone()).unwrap(),
        target,
        "an admin move is saved at once"
    );
    let rows = rig.audit_rows("admin_move");
    assert_eq!(rows.len(), 1, "{rows:?}");
    assert_eq!(rows[0]["online"], true);
    assert_eq!(rows[0]["target_guid"], bravo);

    // The admin's own session is out of the map while it dispatches, so it must not look
    // offline: moving yourself is refused and your save is left alone.
    let own = fs::read(rig.save_path("Alpha")).unwrap();
    let replies = rig.realm("Alpha", "move Alpha Test Snowhill");
    assert!(
        replies.iter().any(|line| line.contains("use realm goto")),
        "{replies:?}"
    );
    assert_eq!(fs::read(rig.save_path("Alpha")).unwrap(), own);
    assert_eq!(rig.audit_rows("admin_move").len(), 1);
}

/// A selected but not yet loaded character is refused with its save, its live state and the
/// audit log untouched.
#[test]
fn l043_loading_or_busy_account_is_refused_and_unchanged() {
    if !test_support::bin_test_target("l043") {
        return;
    }
    let mut rig = Rig::new(
        "l043-loading",
        &[
            ("Alpha", AccountRole::Admin),
            ("Bravo", AccountRole::Player),
        ],
        true,
    );
    rig.login("Alpha", true);
    rig.login("Bravo", false);
    let bravo = rig.guid("Bravo");
    assert!(!rig.session(bravo).gateway.is_world_loaded());
    let saved = fs::read(rig.save_path("Bravo")).unwrap();
    let state = rig.session(bravo).gateway.state().clone();

    let replies = rig.realm("Alpha", "move Bravo Test Snowhill");
    assert!(
        replies
            .iter()
            .any(|line| line.contains("still loading") && line.ends_with("Nothing changed.")),
        "{replies:?}"
    );
    assert_eq!(fs::read(rig.save_path("Bravo")).unwrap(), saved);
    assert_eq!(rig.session(bravo).gateway.state(), &state);
    assert!(
        rig.drain(bravo).is_empty(),
        "the loading client was sent something"
    );
    assert!(rig.audit_rows("admin_move").is_empty());
}

/// Players and builders get the ordinary role refusal whether or not the target exists, and
/// nothing changes. A server without a security log refuses an admin too (fail closed).
#[test]
fn l043_player_builder_and_unaudited_moves_are_refused_without_effect() {
    if !test_support::bin_test_target("l043") {
        return;
    }
    let mut rig = Rig::new(
        "l043-roles",
        &[
            ("Alpha", AccountRole::Builder),
            ("Bravo", AccountRole::Player),
            ("Charlie", AccountRole::Player),
        ],
        true,
    );
    rig.login("Bravo", true);
    rig.logout("Bravo");
    let saved = fs::read(rig.save_path("Bravo")).unwrap();
    rig.login("Alpha", true);
    rig.login("Charlie", true);
    for first in ["Alpha", "Charlie"] {
        let real = rig.realm(first, "move Bravo Test Snowhill");
        let absent = rig.realm(first, "move Nobody Here Snowhill");
        assert_eq!(
            real,
            ["Cannot do that: your account role does not allow this command."],
            "{first}"
        );
        assert_eq!(
            real, absent,
            "{first}: the refusal must not reveal a character"
        );
    }
    assert_eq!(fs::read(rig.save_path("Bravo")).unwrap(), saved);
    assert!(rig.audit_rows("admin_move").is_empty());

    let mut rig = Rig::new(
        "l043-unaudited",
        &[
            ("Alpha", AccountRole::Admin),
            ("Bravo", AccountRole::Player),
        ],
        false,
    );
    rig.login("Bravo", true);
    rig.logout("Bravo");
    let saved = fs::read(rig.save_path("Bravo")).unwrap();
    rig.login("Alpha", true);
    let replies = rig.realm("Alpha", "move Bravo Test Snowhill");
    assert!(
        replies.iter().any(|line| line.contains("no security log")),
        "{replies:?}"
    );
    assert_eq!(fs::read(rig.save_path("Bravo")).unwrap(), saved);
}

/// L081: what `first` got back since the last read: the 26/9 wheel row ids and system lines.
fn read(rig: &mut Rig, first: &str) -> (Vec<i32>, Vec<String>) {
    let (mut ids, mut lines) = (Vec::new(), Vec::new());
    for message in rig.drain(rig.guid(first)) {
        let Some(body) = message.get(7..) else {
            continue;
        };
        let at = |offset: usize| i32::from_le_bytes(body[offset..offset + 4].try_into().unwrap());
        if body.starts_with(&[26, 0, 9, 0]) {
            ids = (0..at(13) as usize).map(|row| at(17 + row * 28)).collect();
        } else if body.starts_with(&[15, 0, 3, 0]) {
            lines.push(String::from_utf8(body[8..8 + at(4) as usize].to_vec()).unwrap());
        }
    }
    (ids, lines)
}

fn send(rig: &mut Rig, first: &str, inner: &[u8], now: &mut u64) -> (Vec<i32>, Vec<String>) {
    *now += 100;
    // The account action budget (5/s, burst 10) runs on the real clock; refill it per step.
    rig.server.world.security.lock().unwrap().budgets = Default::default();
    rig.dispatch(rig.guid(first), &wrap_application(inner), *now)
        .unwrap();
    read(rig, first)
}

/// Alpha opens Bravo's wheel and picks `ids` in order; what the last step got back.
fn gm(rig: &mut Rig, now: &mut u64, ids: &[i32]) -> (Vec<i32>, Vec<String>) {
    let (alpha, bravo) = (rig.guid("Alpha"), rig.guid("Bravo"));
    let select = [
        &[26, 0, 19, 0][..],
        &alpha.to_le_bytes(),
        &bravo.to_le_bytes(),
    ]
    .concat();
    let mut last = send(rig, "Alpha", &select, now);
    for id in ids {
        let choice = [&[26, 0, 10, 0][..], &bravo.to_le_bytes(), &id.to_le_bytes()].concat();
        last = send(rig, "Alpha", &choice, now);
    }
    last
}

/// L081: the admin GM wheel through the production path. Only an admin's wheel on another
/// player carries the GM entry; its pages open in place; a chosen mute, coin gift, summon, go-to
/// and role land through the rules their chat commands use.
#[test]
fn l081_gm_wheel_mutes_gifts_moves_and_sets_a_role() {
    if !test_support::bin_test_target("l081") {
        return;
    }
    let mut rig = Rig::new(
        "l081-wheel",
        &[
            ("Alpha", AccountRole::Admin),
            ("Bravo", AccountRole::Player),
        ],
        true,
    );
    rig.login("Alpha", true);
    rig.login("Bravo", true);
    let (alpha, bravo) = (rig.guid("Alpha"), rig.guid("Bravo"));
    let bravo_account = AccountStore::load(&rig.accounts)
        .unwrap()
        .character(bravo)
        .unwrap()
        .owner_account_id;
    let mut now = 2_000;

    let select = [
        &[26, 0, 19, 0][..],
        &bravo.to_le_bytes(),
        &alpha.to_le_bytes(),
    ]
    .concat();
    let (wheel, _) = send(&mut rig, "Bravo", &select, &mut now);
    assert!(
        !wheel.contains(&1100),
        "a player's wheel has no GM entry: {wheel:?}"
    );
    assert!(gm(&mut rig, &mut now, &[]).0.contains(&1100));
    assert_eq!(
        gm(&mut rig, &mut now, &[1100]).0,
        (1101..=1108).collect::<Vec<_>>()
    );
    assert_eq!(
        gm(&mut rig, &mut now, &[1100, 1101]).0,
        [1111, 1112, 1113, 1119]
    );

    let (_, lines) = gm(&mut rig, &mut now, &[1100, 1101, 1111]);
    assert!(lines.iter().any(|l| l == "Player muted."), "{lines:?}");
    let store = AccountStore::load(&rig.accounts).unwrap();
    assert!(
        store
            .sanctions(bravo_account)
            .muted(osfr_tools::moderation::unix_now())
    );

    let coins =
        |rig: &Rig| rig.session(bravo).gateway.gameplay_summary().unwrap()["coins"].as_u64();
    let before = coins(&rig).unwrap();
    read(&mut rig, "Bravo");
    assert_eq!(
        gm(&mut rig, &mut now, &[1100, 1104]).0,
        [1131, 1132, 1133, 1134, 1139]
    );
    let (_, lines) = gm(&mut rig, &mut now, &[1100, 1104, 1131]);
    assert!(lines.iter().any(|l| l == "Gift sent."), "{lines:?}");
    assert_eq!(coins(&rig), Some(before + 1_000));
    let (_, told) = read(&mut rig, "Bravo");
    assert!(
        told.iter().any(|l| l == "A GM gave you 1000 coins."),
        "{told:?}"
    );

    // Summon: Bravo is moved off first, then the wheel brings it to Alpha.
    let here = rig.session(alpha).gateway.state().position;
    let away = [here[0] + 7.0, here[1] + 1.0, here[2] - 3.0];
    let text = format!("realm move {bravo} {} {} {}", away[0], away[1], away[2]);
    let request = osfr_tools::chat::local_request(&text);
    send(&mut rig, "Alpha", &request, &mut now);
    assert_eq!(rig.session(bravo).gateway.state().position, away);
    gm(&mut rig, &mut now, &[1100, 1107]);
    assert_eq!(rig.session(bravo).gateway.state().position, here);

    // Go to: Bravo is moved off again, and the wheel takes Alpha there.
    send(&mut rig, "Alpha", &request, &mut now);
    let (_, lines) = gm(&mut rig, &mut now, &[1100, 1106]);
    assert!(
        lines.iter().any(|l| l.starts_with("Moved you to")),
        "{lines:?}"
    );
    assert_eq!(rig.session(alpha).gateway.state().position, away);

    let (_, lines) = gm(&mut rig, &mut now, &[1100, 1108, 1153]);
    assert!(lines.iter().any(|l| l == "Role saved."), "{lines:?}");
    let store = AccountStore::load(&rig.accounts).unwrap();
    assert_eq!(
        store.account_role(bravo_account),
        Some(AccountRole::Moderator)
    );
    // L101 (review 281675b-L090): the saved change wrote one audit row, ids and role only.
    let alpha_account = store.character(alpha).unwrap().owner_account_id;
    let rows = rig.audit_rows("role_change");
    assert_eq!(rows.len(), 1, "{rows:?}");
    assert_eq!(rows[0]["account_id"], alpha_account);
    assert_eq!(rows[0]["target_account_id"], bravo_account);
    assert_eq!(rows[0]["role"], "moderator");

    let (_, lines) = gm(&mut rig, &mut now, &[1100, 1105, 1141]);
    assert!(lines.iter().any(|l| l == "Gift sent."), "{lines:?}");
    let (_, told) = read(&mut rig, "Bravo");
    assert!(
        told.iter().any(|l| l == "A GM gave you 100 Station Cash."),
        "{told:?}"
    );

    // The panel's Players tab: Summon, then Go to, by name.
    let here = rig.session(alpha).gateway.state().position;
    let off = [here[0] - 4.0, here[1], here[2] + 2.0];
    let text = format!("realm move {bravo} {} {} {}", off[0], off[1], off[2]);
    let request = osfr_tools::chat::local_request(&text);
    let panel = |text: &str| osfr_tools::chat::local_request(text);
    send(&mut rig, "Alpha", &request, &mut now);
    assert_eq!(rig.session(bravo).gateway.state().position, off);
    send(&mut rig, "Alpha", &panel("!gm summon bravo TEST"), &mut now);
    assert_eq!(rig.session(bravo).gateway.state().position, here);
    send(&mut rig, "Alpha", &request, &mut now);
    send(&mut rig, "Alpha", &panel("!gm goto Bravo Test"), &mut now);
    assert_eq!(rig.session(alpha).gateway.state().position, off);
    let (_, lines) = send(&mut rig, "Alpha", &panel("!gm goto Nobody Here"), &mut now);
    assert!(
        lines.iter().any(|l| l.contains("no single online player")),
        "{lines:?}"
    );

    // An admin outranks a moderator: the kick ends Bravo's session.
    let (_, lines) = gm(&mut rig, &mut now, &[1100, 1102]);
    assert!(lines.iter().any(|l| l == "Player kicked."), "{lines:?}");
    assert!(rig.session(bravo).kicked);
}

/// G5's gateway tests use this rig (`gm_powers_tests.rs`).
#[cfg(test)]
#[path = "gm_powers_tests.rs"]
mod g5_gm_powers;
