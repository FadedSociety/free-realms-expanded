//! G3: character and inventory editing through the production path, in account-store mode: a
//! real `Server`, sessions bound by login tickets, chat through `Session::dispatch_message`,
//! logout through `Server::end_session`, and the facade run on the server's own session map. The
//! characters are synthetic; the content is the bundled opening reference (lab).
use super::super::super::tests::test_support::{self, Scratch};
use super::*;
use osfr_tools::account::{AccountFlags, SlotPolicy, create_character};
use osfr_tools::character_admin::{Amount, Pick};
use osfr_tools::identity::NativeCreation;

const HANDOFF: &[u8] = b"g3-synthetic-handoff";
/// A stacking item of the opening catalogue (`max_stack` 50 or more) and a one-per-stack one.
const STACKING: i32 = 70454;
const SINGLE: i32 = 48316;
/// An enabled two-step Adventurer quest of the bundled catalogue (`w64_regional_quest_tests`).
const QUEST: u32 = 3_000_016;

struct Rig {
    _scratch: Scratch,
    server: Server,
    accounts: PathBuf,
    guids: BTreeMap<&'static str, u64>,
    keys: BTreeMap<u64, SessionKey>,
    next_id: u64,
}

impl Rig {
    /// One account per `(first name, role)`, each with one character named "<first> Test".
    fn new(tag: &str, people: &[(&'static str, AccountRole)]) -> Self {
        let scratch = Scratch::new(tag);
        let resources = test_support::creator_resources(scratch.path());
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
            guids.insert(*first, reply.guid);
        }
        store.save(&accounts).unwrap();
        let mut options = test_support::options(scratch.state());
        options.account_store = Some(accounts.clone());
        options.handoff_key = HANDOFF.to_vec();
        let server = Server::new(options).unwrap();
        Self {
            _scratch: scratch,
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

    fn actor(&self, first: &str, role: AccountRole) -> Actor {
        let store = AccountStore::load(&self.accounts).unwrap();
        Actor {
            account_id: store.character(self.guid(first)).unwrap().owner_account_id,
            role,
        }
    }

    /// Log a character in through its binding ticket, then Ready and Loaded.
    fn login(&mut self, first: &str) {
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
            format!("127.0.0.1:{}", 44_000 + id).parse().unwrap(),
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
        self.dispatch(guid, &login, 100);
        self.dispatch(guid, &wrap_application(&[13, 0]), 200);
        self.dispatch(guid, &wrap_application(&[10, 0]), 300);
        assert!(self.session(guid).gateway.is_world_loaded());
        self.drain(guid);
    }

    fn logout(&mut self, first: &str) {
        let key = self.keys.remove(&self.guid(first)).unwrap();
        let session = self.server.sessions.remove(&key).unwrap();
        self.server.end_session(key, session, "logout");
    }

    fn dispatch(&mut self, guid: u64, message: &[u8], now: u64) {
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
        result.unwrap();
    }

    fn session(&self, guid: u64) -> &Session {
        &self.server.sessions[&self.keys[&guid]]
    }

    /// Every message queued to the client since the last drain.
    fn drain(&mut self, guid: u64) -> Vec<Vec<u8>> {
        let session = self.server.sessions.get_mut(&self.keys[&guid]).unwrap();
        super::super::super::f120_fishing_hour_reconnect::drain_replies(session, 2_000)
    }

    /// `realm <text>` typed by `first`; the system lines it got back.
    fn realm(&mut self, first: &str, text: &str) -> Vec<String> {
        let guid = self.guid(first);
        let inner = osfr_tools::chat::local_request(&format!("realm {text}"));
        self.dispatch(guid, &wrap_application(&inner), 1_000);
        system_lines(&self.drain(guid))
    }

    fn host(&mut self) -> Host<'_> {
        Host {
            options: &self.server.options,
            sessions: &mut self.server.sessions,
            world: &mut self.server.world,
            membership: &mut self.server.membership,
            now_ms: 5_000,
            me: None,
        }
    }

    fn save_path(&self, first: &str) -> PathBuf {
        character_state_path(&self.server.options.state, self.guid(first))
    }

    /// The saved gameplay state of a character.
    fn saved(&self, first: &str) -> osfr_tools::gameplay::GameplayState {
        let document: serde_json::Value =
            serde_json::from_slice(&fs::read(self.save_path(first)).unwrap()).unwrap();
        serde_json::from_value(document["gameplay"].clone()).unwrap()
    }

    /// Units of `item` in the online character's live bag.
    fn live_units(&self, first: &str, item: i32) -> u32 {
        units(
            &self
                .session(self.guid(first))
                .gateway
                .gameplay()
                .unwrap()
                .state()
                .inventory,
            item,
        )
    }
}

fn units(inventory: &osfr_tools::inventory::InventoryState, item: i32) -> u32 {
    inventory
        .items
        .iter()
        .filter(|i| i.definition_id == item)
        .map(|i| i.count)
        .sum()
}

/// The inner application heads (family, subtype) of the messages a client got.
fn heads(messages: &[Vec<u8>]) -> Vec<[u8; 4]> {
    messages
        .iter()
        .filter_map(|m| m.get(7..11)?.try_into().ok())
        .collect()
}

fn system_lines(messages: &[Vec<u8>]) -> Vec<String> {
    messages
        .iter()
        .filter(|message| message.get(7..11) == Some(&[15, 0, 3, 0]))
        .map(|message| {
            let length = u32::from_le_bytes(message[11..15].try_into().unwrap()) as usize;
            String::from_utf8(message[15..15 + length].to_vec()).unwrap()
        })
        .collect()
}

fn rig(tag: &str) -> Option<Rig> {
    if !test_support::bin_test_target("g3") {
        return None;
    }
    let mut rig = Rig::new(
        tag,
        &[
            ("Alpha", AccountRole::Admin),
            ("Bravo", AccountRole::Player),
            ("Mona", AccountRole::Moderator),
        ],
    );
    // Every character enters once, so each has a gameplay save.
    for first in ["Alpha", "Bravo", "Mona"] {
        rig.login(first);
        rig.logout(first);
    }
    Some(rig)
}

/// The panel's Add item on an online player: the client gets the 38/2 new stack and the GM line
/// at once, the live bag holds it, the commit saved it, and the read view (a moderator may read)
/// names the row.
#[test]
fn g3_online_add_sends_the_bag_packet_and_lands_in_the_bag() {
    let Some(mut rig) = rig("g3-online-add") else {
        return;
    };
    rig.login("Bravo");
    let (bravo, admin) = (rig.guid("Bravo"), rig.actor("Alpha", AccountRole::Admin));
    let receipt = add_items(&mut rig.host(), &admin, bravo, STACKING, 3, false).unwrap();
    assert!(receipt.online && receipt.backup.is_none());
    assert_eq!(receipt.record.target, bravo);
    let sent = rig.drain(bravo);
    assert!(heads(&sent).contains(&[38, 0, 2, 0]), "{:?}", heads(&sent));
    assert!(
        system_lines(&sent)
            .iter()
            .any(|line| line.starts_with("A GM gave you 3 x ")),
        "{:?}",
        system_lines(&sent)
    );
    assert_eq!(rig.live_units("Bravo", STACKING), 3);
    assert_eq!(
        units(&rig.saved("Bravo").inventory, STACKING),
        3,
        "the commit saves"
    );

    let moderator = rig.actor("Mona", AccountRole::Moderator);
    let view = inventory(&rig.host(), &moderator, bravo).unwrap();
    let row = view
        .bags
        .iter()
        .find(|row| row.item_id == STACKING)
        .unwrap();
    assert_eq!((row.count, row.name.as_str()), (3, "#70454"));
    // A second add stacks onto the same slot: the client gets an update, not a new stack.
    add_items(&mut rig.host(), &admin, bravo, STACKING, 2, false).unwrap();
    let again = inventory(&rig.host(), &moderator, bravo).unwrap();
    let rows: Vec<_> = again
        .bags
        .iter()
        .filter(|r| r.item_id == STACKING)
        .collect();
    assert_eq!((rows.len(), rows[0].slot, rows[0].count), (1, row.slot, 5));
}

/// An offline add: the save holds the item, the old save is kept byte for byte as a timestamped
/// backup, and the next login brings the item.
#[test]
fn g3_offline_add_lands_in_the_save_and_keeps_a_backup() {
    let Some(mut rig) = rig("g3-offline-add") else {
        return;
    };
    let (bravo, admin) = (rig.guid("Bravo"), rig.actor("Alpha", AccountRole::Admin));
    let old = fs::read(rig.save_path("Bravo")).unwrap();
    let receipt = add_items(&mut rig.host(), &admin, bravo, STACKING, 7, false).unwrap();
    assert!(!receipt.online);
    let backup = receipt.backup.expect("an offline edit keeps a backup");
    assert_eq!(
        fs::read(&backup).unwrap(),
        old,
        "the backup is the old save"
    );
    assert!(
        backup
            .file_name()
            .unwrap()
            .to_string_lossy()
            .starts_with(&format!("{bravo}-"))
    );
    assert_eq!(units(&rig.saved("Bravo").inventory, STACKING), 7);
    // The read view of an offline character reads the save.
    let view = inventory(&rig.host(), &admin, bravo).unwrap();
    assert_eq!(units_in(&view, STACKING), 7);
    rig.login("Bravo");
    assert_eq!(
        rig.live_units("Bravo", STACKING),
        7,
        "the next login has it"
    );
}

fn units_in(view: &osfr_tools::character_admin::InventoryView, item: i32) -> u32 {
    view.bags
        .iter()
        .filter(|r| r.item_id == item)
        .map(|r| r.count)
        .sum()
}

/// `realm take <name> <item> all` (the chat front end) and the facade's slot removal both clear
/// the stack; the client gets 38/4 for the gone stack.
#[test]
fn g3_removing_all_of_an_item_clears_it() {
    let Some(mut rig) = rig("g3-remove") else {
        return;
    };
    rig.login("Alpha");
    rig.login("Bravo");
    let (bravo, admin) = (rig.guid("Bravo"), rig.actor("Alpha", AccountRole::Admin));
    add_items(&mut rig.host(), &admin, bravo, STACKING, 6, false).unwrap();
    rig.drain(bravo);
    let lines = rig.realm("Alpha", &format!("take bravo test {STACKING} all"));
    assert!(
        lines
            .iter()
            .any(|l| l.starts_with(&format!("Done for character {bravo}: remove"))),
        "{lines:?}"
    );
    assert_eq!(rig.live_units("Bravo", STACKING), 0);
    assert!(heads(&rig.drain(bravo)).contains(&[38, 0, 4, 0]));

    // Offline, by slot and count, then the rest.
    let receipt = add_items(&mut rig.host(), &admin, bravo, STACKING, 4, false).unwrap();
    let slot = receipt.record.change.stacks[0].id;
    rig.logout("Bravo");
    let take = |amount| {
        vec![Removal {
            pick: Pick::Slot(slot),
            amount,
        }]
    };
    remove_items(&mut rig.host(), &admin, bravo, take(Amount::Count(3))).unwrap();
    assert_eq!(units(&rig.saved("Bravo").inventory, STACKING), 1);
    let over = remove_items(&mut rig.host(), &admin, bravo, take(Amount::Count(5)));
    assert!(matches!(over, Err(AdminError::Refused(_))), "{over:?}");
    remove_items(&mut rig.host(), &admin, bravo, take(Amount::All)).unwrap();
    let saved = rig.saved("Bravo").inventory;
    assert!(
        !saved
            .items
            .iter()
            .any(|i| i.id == slot || i.definition_id == STACKING)
    );
}

/// Unknown ids, zero, negative and over-stack counts, an empty wallet and a bad chat line are
/// refused with a reason, and nothing changes.
#[test]
fn g3_an_unknown_id_and_bad_counts_are_refused() {
    let Some(mut rig) = rig("g3-refuse") else {
        return;
    };
    rig.login("Alpha");
    rig.login("Bravo");
    let (bravo, admin) = (rig.guid("Bravo"), rig.actor("Alpha", AccountRole::Admin));
    let before = rig.saved("Bravo");
    let unknown = add_items(&mut rig.host(), &admin, bravo, 999_999_999, 1, false);
    assert_eq!(
        unknown,
        Err(AdminError::Refused(
            "item 999999999 is not in the item tables".into()
        ))
    );
    for (item, count) in [(STACKING, 0), (-5, 1), (SINGLE, 2)] {
        let result = add_items(&mut rig.host(), &admin, bravo, item, count, false);
        assert!(matches!(result, Err(AdminError::Refused(_))), "{result:?}");
    }
    let wallet = rig.saved("Bravo").inventory.coins;
    let take = coins(&mut rig.host(), &admin, bravo, CoinOp::Take, wallet + 1);
    assert!(matches!(take, Err(AdminError::Refused(_))), "{take:?}");
    let huge = coins(&mut rig.host(), &admin, bravo, CoinOp::Give, 2_000_000_000);
    assert!(matches!(huge, Err(AdminError::Refused(_))), "{huge:?}");
    let missing = add_items(&mut rig.host(), &admin, 424_242, STACKING, 1, false);
    assert!(
        matches!(missing, Err(AdminError::NotFound(_))),
        "{missing:?}"
    );
    // `chat::system` escapes `<` and `>` for the client's chat markup.
    let lines = rig.realm("Alpha", "give bravo test 70454 -5");
    let usage = osfr_tools::character_admin::GIVE_USAGE
        .replace('<', "&lt;")
        .replace('>', "&gt;");
    assert_eq!(lines, [usage]);
    let lines = rig.realm("Alpha", "take nobody here 70454 1");
    assert!(lines[0].contains("no single character"), "{lines:?}");
    assert_eq!(rig.saved("Bravo"), before, "a refusal changes nothing");
}

/// A bag at the reward capacity (100 stacks, R) refuses a new stack, offline as a quest reward
/// would; nothing is written and no backup is made.
#[test]
fn g3_a_full_bag_refuses() {
    let Some(mut rig) = rig("g3-full") else {
        return;
    };
    let (bravo, admin) = (rig.guid("Bravo"), rig.actor("Alpha", AccountRole::Admin));
    let path = rig.save_path("Bravo");
    let character = serde_json::from_value(
        serde_json::from_slice::<serde_json::Value>(&fs::read(&path).unwrap()).unwrap()
            ["character"]
            .clone(),
    )
    .unwrap();
    let mut snapshot = load_or_create(&path, character).unwrap();
    let inventory = &mut snapshot.gameplay.as_mut().unwrap().inventory;
    let template = serde_json::json!({"definition_id": SINGLE, "tint": 0, "id": 0, "count": 1,
        "consumed_count": 0, "last_cast_time": i64::from(i32::MAX), "rental": null,
        "ability_count": 0, "activate_enabled": false});
    let mut next = 9_000_000;
    while inventory.items.len() < 100 {
        let mut item: osfr_tools::inventory::InventoryItem =
            serde_json::from_value(template.clone()).unwrap();
        item.id = next;
        next += 1;
        inventory.items.push(item);
    }
    inventory.next_instance_id = Some(i64::from(next));
    save(&path, &snapshot).unwrap();
    let full = fs::read(&path).unwrap();

    let result = add_items(&mut rig.host(), &admin, bravo, SINGLE, 1, false);
    assert_eq!(
        result,
        Err(AdminError::Refused("the bag is full (100 stacks)".into()))
    );
    assert_eq!(fs::read(&path).unwrap(), full, "nothing was written");
    assert!(!path.parent().unwrap().join("admin-backups").exists());
    // Topping up a stack that is already there needs no new slot.
    add_items(&mut rig.host(), &admin, bravo, STACKING, 1, false).unwrap_err();
}

/// Undo reverses an add, a coin change and a removal, online, through the record each write
/// returned; a second undo of the same record is refused as stale and changes nothing.
#[test]
fn g3_undo_restores() {
    let Some(mut rig) = rig("g3-undo") else {
        return;
    };
    rig.login("Bravo");
    let (bravo, admin) = (rig.guid("Bravo"), rig.actor("Alpha", AccountRole::Admin));
    let start = rig.saved("Bravo");
    let added = add_items(&mut rig.host(), &admin, bravo, STACKING, 4, false).unwrap();
    let paid = coins(&mut rig.host(), &admin, bravo, CoinOp::Give, 50).unwrap();
    assert_eq!(
        paid.record.change.coins.unwrap()[1],
        start.inventory.coins + 50
    );
    undo(&mut rig.host(), &admin, &paid.record).unwrap();
    undo(&mut rig.host(), &admin, &added.record).unwrap();
    let back = rig.saved("Bravo");
    assert_eq!(back.inventory.coins, start.inventory.coins);
    assert_eq!(units(&back.inventory, STACKING), 0);
    assert_eq!(rig.live_units("Bravo", STACKING), 0);
    let again = undo(&mut rig.host(), &admin, &added.record);
    assert!(matches!(again, Err(AdminError::Conflict(_))), "{again:?}");
    assert_eq!(rig.saved("Bravo"), back);

    // A removal comes back as the same stack, offline.
    let kept = add_items(&mut rig.host(), &admin, bravo, STACKING, 2, false).unwrap();
    let slot = kept.record.change.stacks[0].id;
    rig.logout("Bravo");
    let gone = remove_items(
        &mut rig.host(),
        &admin,
        bravo,
        vec![Removal {
            pick: Pick::Item(STACKING),
            amount: Amount::All,
        }],
    )
    .unwrap();
    undo(&mut rig.host(), &admin, &gone.record).unwrap();
    let restored = rig.saved("Bravo").inventory;
    let stack = restored.items.iter().find(|i| i.id == slot).unwrap();
    assert_eq!((stack.definition_id, stack.count), (STACKING, 2));
}

/// A moderator may read but not write, by the facade or by chat; a player may do neither.
#[test]
fn g3_a_moderator_cannot_write() {
    let Some(mut rig) = rig("g3-moderator") else {
        return;
    };
    rig.login("Mona");
    rig.login("Bravo");
    let bravo = rig.guid("Bravo");
    let before = rig.saved("Bravo");
    let moderator = rig.actor("Mona", AccountRole::Moderator);
    let writes = [
        add_items(&mut rig.host(), &moderator, bravo, STACKING, 1, false),
        coins(&mut rig.host(), &moderator, bravo, CoinOp::Give, 5),
        quests(&mut rig.host(), &moderator, bravo, QuestOp::Grant, QUEST),
        jobs(&mut rig.host(), &moderator, bravo, 1, JobSet::Level(3)),
    ];
    for result in writes {
        assert_eq!(result, Err(AdminError::Forbidden));
    }
    assert!(inventory(&rig.host(), &moderator, bravo).is_ok());
    assert!(profile(&rig.host(), &moderator, bravo).is_ok());
    let player = rig.actor("Bravo", AccountRole::Player);
    assert_eq!(
        inventory(&rig.host(), &player, bravo),
        Err(AdminError::Forbidden)
    );
    let lines = rig.realm("Mona", &format!("take bravo test {STACKING} all"));
    assert_eq!(lines.len(), 1, "{lines:?}");
    assert!(lines[0].starts_with("Cannot do that"), "{lines:?}");
    let lines = rig.realm("Mona", "coins bravo test set 0");
    assert!(lines[0].starts_with("Cannot do that"), "{lines:?}");
    assert_eq!(rig.saved("Bravo"), before);
}

/// Quest complete, reset, grant and abandon move the regional quest record, show in the
/// profile view and the save, and send the journal messages (49/12, 49/3).
#[test]
fn g3_quest_complete_and_reset_show_in_the_quest_state() {
    let Some(mut rig) = rig("g3-quests") else {
        return;
    };
    rig.login("Bravo");
    let (bravo, admin) = (rig.guid("Bravo"), rig.actor("Alpha", AccountRole::Admin));
    let ids = |rows: &[osfr_tools::character_admin::QuestRow]| -> Vec<u32> {
        rows.iter().map(|r| r.quest_id).collect()
    };
    rig.drain(bravo);
    quests(&mut rig.host(), &admin, bravo, QuestOp::Complete, QUEST).unwrap();
    let view = profile(&rig.host(), &admin, bravo).unwrap();
    assert!(ids(&view.quests_done).contains(&QUEST), "{view:?}");
    assert!(heads(&rig.drain(bravo)).contains(&[49, 0, 12, 0]));
    let saved = rig.saved("Bravo").npc_quests.unwrap();
    assert_eq!(
        saved.quests[&QUEST].status,
        osfr_tools::quest_engine::QuestStatus::Completed
    );
    let twice = quests(&mut rig.host(), &admin, bravo, QuestOp::Complete, QUEST);
    assert!(matches!(twice, Err(AdminError::Refused(_))), "{twice:?}");

    quests(&mut rig.host(), &admin, bravo, QuestOp::Reset, QUEST).unwrap();
    let view = profile(&rig.host(), &admin, bravo).unwrap();
    assert!(!ids(&view.quests_done).contains(&QUEST));
    assert!(!ids(&view.quests_active).contains(&QUEST));
    let saved = rig.saved("Bravo").npc_quests.unwrap_or_default_state();
    assert!(!saved.contains_key(&QUEST));

    rig.drain(bravo);
    let granted = quests(&mut rig.host(), &admin, bravo, QuestOp::Grant, QUEST).unwrap();
    assert!(heads(&rig.drain(bravo)).contains(&[49, 0, 3, 0]));
    let view = profile(&rig.host(), &admin, bravo).unwrap();
    assert!(ids(&view.quests_active).contains(&QUEST));
    quests(&mut rig.host(), &admin, bravo, QuestOp::Abandon, QUEST).unwrap();
    let view = profile(&rig.host(), &admin, bravo).unwrap();
    assert!(!ids(&view.quests_active).contains(&QUEST));
    // The grant's record undoes only from where it left the quest: abandoned since, so stale.
    let stale = undo(&mut rig.host(), &admin, &granted.record);
    assert!(matches!(stale, Err(AdminError::Conflict(_))), "{stale:?}");
}

/// A job's level set by the facade: the profile view shows it, the client gets 38/14, and the
/// record undoes it.
#[test]
fn g3_job_level_set_shows_and_undoes() {
    let Some(mut rig) = rig("g3-jobs") else {
        return;
    };
    rig.login("Bravo");
    let (bravo, admin) = (rig.guid("Bravo"), rig.actor("Alpha", AccountRole::Admin));
    let view = profile(&rig.host(), &admin, bravo).unwrap();
    let job = view
        .jobs
        .iter()
        .find(|j| j.active)
        .expect("an active job")
        .clone();
    rig.drain(bravo);
    let set = jobs(&mut rig.host(), &admin, bravo, job.job_id, JobSet::Level(5)).unwrap();
    assert!(heads(&rig.drain(bravo)).contains(&[38, 0, 14, 0]));
    let now = profile(&rig.host(), &admin, bravo).unwrap();
    let row = now.jobs.iter().find(|j| j.job_id == job.job_id).unwrap();
    assert_eq!(row.level, 5);
    let locked = jobs(&mut rig.host(), &admin, bravo, 999, JobSet::Xp(10));
    assert!(matches!(locked, Err(AdminError::Refused(_))), "{locked:?}");
    undo(&mut rig.host(), &admin, &set.record).unwrap();
    let back = profile(&rig.host(), &admin, bravo).unwrap();
    let row = back.jobs.iter().find(|j| j.job_id == job.job_id).unwrap();
    assert_eq!((row.level, row.xp), (job.level, job.xp));
}

/// Add worn: the active job (the Adventurer) wears it (38/5) and the view lists it worn; the
/// undo takes it off (38/6) and out of the bag (38/4).
#[test]
fn g3_add_worn_equips_and_undo_takes_it_off() {
    /// A one-per-stack slot-1 piece the Adventurer (profile 1) may wear.
    const HAT: i32 = 9052;
    let Some(mut rig) = rig("g3-worn") else {
        return;
    };
    rig.login("Bravo");
    let (bravo, admin) = (rig.guid("Bravo"), rig.actor("Alpha", AccountRole::Admin));
    let active = profile(&rig.host(), &admin, bravo).unwrap().jobs;
    assert!(
        active.iter().any(|j| j.active && j.job_id == 1),
        "{active:?}"
    );
    rig.drain(bravo);
    let worn = add_items(&mut rig.host(), &admin, bravo, HAT, 1, true).unwrap();
    assert!(heads(&rig.drain(bravo)).contains(&[38, 0, 5, 0]));
    assert_eq!(
        worn.record.change.equipment.len(),
        1,
        "{:?}",
        worn.record.change
    );
    let view = inventory(&rig.host(), &admin, bravo).unwrap();
    assert!(
        view.equipped
            .iter()
            .any(|r| r.item_id == HAT && r.profile_id == 1)
    );
    undo(&mut rig.host(), &admin, &worn.record).unwrap();
    let sent = heads(&rig.drain(bravo));
    assert!(
        sent.contains(&[38, 0, 6, 0]) && sent.contains(&[38, 0, 4, 0]),
        "{sent:?}"
    );
    let view = inventory(&rig.host(), &admin, bravo).unwrap();
    assert!(!view.equipped.iter().any(|r| r.item_id == HAT));
    assert_eq!(units_in(&view, HAT), 0);
    assert_eq!(units(&rig.saved("Bravo").inventory, HAT), 0);
}

/// The item search finds a definition by id and by name substring, a page at a time.
#[test]
fn g3_item_search_pages_the_item_tables() {
    let Some(mut rig) = rig("g3-search") else {
        return;
    };
    let admin = rig.actor("Alpha", AccountRole::Admin);
    let by_id = search_items(&rig.host(), &admin, "70454", 0).unwrap();
    assert_eq!(by_id.total, 1);
    assert_eq!(by_id.rows[0].item_id, STACKING);
    let all = search_items(&rig.host(), &admin, "", 0).unwrap();
    assert_eq!(all.rows.len(), osfr_tools::character_admin::SEARCH_PAGE);
    assert!(all.total > 1_000, "{}", all.total);
    // Without a locale every name is `#id`, so a name substring finds by that text.
    let named = search_items(&rig.host(), &admin, "#7045", 0).unwrap();
    assert!(named.rows.iter().any(|r| r.item_id == STACKING));
    let last = search_items(&rig.host(), &admin, "", all.total / 50).unwrap();
    assert!(last.rows.len() <= 50 && !last.rows.is_empty());
    let player = rig.actor("Bravo", AccountRole::Player);
    assert_eq!(
        search_items(&rig.host(), &player, "", 0),
        Err(AdminError::Forbidden)
    );
}

/// `realm coins <name> give|take|set` moves an online player's wallet (38/19) at once.
#[test]
fn g3_chat_coins_set_moves_the_wallet() {
    let Some(mut rig) = rig("g3-chat-coins") else {
        return;
    };
    rig.login("Alpha");
    rig.login("Bravo");
    let bravo = rig.guid("Bravo");
    let lines = rig.realm("Alpha", "coins Bravo Test set 1234");
    assert!(
        lines[0].starts_with(&format!("Done for character {bravo}")),
        "{lines:?}"
    );
    assert!(heads(&rig.drain(bravo)).contains(&[38, 0, 19, 0]));
    assert_eq!(rig.saved("Bravo").inventory.coins, 1234);
    let lines = rig.realm("Alpha", "coins bravo test take 34");
    assert!(lines[0].starts_with("Done"), "{lines:?}");
    assert_eq!(rig.saved("Bravo").inventory.coins, 1200);
    // The sender's own character goes through the sender's session.
    let alpha = rig.guid("Alpha");
    let lines = rig.realm("Alpha", "give alpha test 70454 2");
    assert!(
        lines
            .iter()
            .any(|l| l.starts_with(&format!("Done for character {alpha}")))
    );
    assert_eq!(rig.live_units("Alpha", STACKING), 2);
}

trait QuestMapOrEmpty {
    fn unwrap_or_default_state(self) -> BTreeMap<u32, osfr_tools::quest_engine::QuestProgress>;
}

impl QuestMapOrEmpty for Option<osfr_tools::quest_engine::QuestState> {
    fn unwrap_or_default_state(self) -> BTreeMap<u32, osfr_tools::quest_engine::QuestProgress> {
        self.map(|state| state.quests).unwrap_or_default()
    }
}
