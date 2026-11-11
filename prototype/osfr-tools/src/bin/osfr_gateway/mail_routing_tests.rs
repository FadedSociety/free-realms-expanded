//! gm-panel G2: mail through the production path. Account-store sessions bound by a real handoff
//! ticket, chat and dialog packets through `Session::dispatch_message` (the gateway's mail stop
//! before gameplay), and `Server::step` for the world tick that starts staged sends and tells
//! players. Synthetic names, ids and text only; every file lives in a `test_support::Scratch`.
use super::super::tests::test_support::{self, Scratch};
use super::super::*;
use osfr_tools::account::{AccountFlags, SlotPolicy};
use osfr_tools::account_store::AccountRole;
use osfr_tools::gameplay::GameplayState;
use osfr_tools::inventory::InventoryItem;
use osfr_tools::mail::{
    self,
    admin::{self, JobState},
    chat::{self as mail_chat, Action},
};
use osfr_tools::npc_interaction::NpcInteractRequest;

const HANDOFF: &[u8] = b"g2-mail-synthetic-handoff-key";
const DENIED: &str = "authorized account role";

/// One account and one character each: account id = index + 1, character guid = index + 101.
const CAST: [(AccountRole, &str, &str); 5] = [
    (AccountRole::Admin, "Ada", "Admin"),
    (AccountRole::Moderator, "Mo", "Moderator"),
    (AccountRole::Player, "Pip", "Player"),
    (AccountRole::Player, "Oz", "Offline"),
    (AccountRole::Builder, "Bea", "Builder"),
];
const ADA: usize = 0;
const MO: usize = 1;
const PIP: usize = 2;
const OZ: usize = 3;
const BEA: usize = 4;
/// A stackable catalogue item of the bundled test content (max stack 50).
const STACKABLE: i32 = 62_529;
/// Sanctuary's postbox group (`data/content/mail.json`) and its Mail Sorting game.
const SANCTUARY_POSTBOX: u64 = 100_000_002_375;
const SANCTUARY_POSITION: [f32; 3] = [-1_100.981, 5.187_627, 223.413_1];
const SANCTUARY_GAME: u32 = 376;

fn account(index: usize) -> u64 {
    index as u64 + 1
}

fn guid(index: usize) -> u64 {
    index as u64 + 101
}

/// The account store and saves of one test.
struct Files(Scratch);

impl Files {
    /// `extra` more offline players beyond the cast (guids 1001 and up).
    fn new(tag: &str, extra: usize) -> Self {
        let files = Self(Scratch::new(&format!("g2-mail-{tag}")));
        write_store(&files.store(), extra);
        files
    }

    fn dir(&self) -> &Path {
        self.0.path()
    }

    fn store(&self) -> PathBuf {
        self.dir().join("accounts.json")
    }

    fn mail(&self, index: usize) -> Vec<admin::MailView> {
        admin::list_for(&self.store(), guid(index)).unwrap()
    }

    fn audit(&self) -> Vec<serde_json::Value> {
        fs::read_to_string(admin::audit_path(&self.store()))
            .unwrap_or_default()
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect()
    }
}

/// The public store API cannot add characters, so they are written into the saved document and
/// the result is checked by the production loader (as `l042_moderation` does).
fn write_store(path: &Path, extra: usize) {
    let mut store = AccountStore::new();
    let roles = CAST.iter().map(|(role, ..)| *role);
    let extras = std::iter::repeat_n(AccountRole::Player, extra);
    for (index, role) in roles.chain(extras).enumerate() {
        let flags = AccountFlags {
            open_chat: true,
            ..AccountFlags::default()
        };
        let mut login = [0u8; 32];
        login[..8].copy_from_slice(&(index as u64 + 1).to_le_bytes());
        let id = store
            .create_account(login, flags, SlotPolicy { max_characters: 2 })
            .unwrap();
        store.set_account_role_offline(id, role).unwrap();
    }
    store.save(path).unwrap();
    let mut document: serde_json::Value = serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/fixtures/account/legacy-identity-v1.json"
    ))
    .unwrap();
    let cast = CAST
        .iter()
        .enumerate()
        .map(|(index, (_, first, last))| (guid(index), account(index), first.to_string(), last.to_string()));
    // Names are letters only (the store refuses digits): Exaa, Exab, ...
    let letter = |n: usize| char::from(b'a' + (n % 26) as u8);
    let extras = (0..extra).map(|n| {
        let index = CAST.len() + n;
        let first = format!("Ex{}{}", letter(n / 26), letter(n));
        (1_001 + n as u64, account(index), first, "Offline".to_owned())
    });
    let mut last_guid = 0;
    for (guid, account, first, last) in cast.chain(extras) {
        let mut identity = fixture["character"].clone();
        identity["first_name"] = json!(first);
        identity["last_name"] = json!(last);
        identity["creation"]["first_name"] = json!(first);
        identity["creation"]["last_name"] = json!(last);
        identity["creation"]["temporary_first_name"] = json!("");
        identity["creation"]["temporary_last_name"] = json!("");
        document["store"]["characters"][guid.to_string()] = json!({
            "guid": guid, "owner_account_id": account,
            "identity": identity, "world_state": null,
        });
        document["store"]["accounts"][account.to_string()]["character_guids"]
            .as_array_mut()
            .unwrap()
            .push(json!(guid));
        last_guid = last_guid.max(guid);
    }
    document["store"]["next_character_guid"] = json!(last_guid + 1);
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

fn has(messages: &[Vec<u8>], bytes: &[u8]) -> bool {
    messages
        .iter()
        .any(|message| message.windows(bytes.len()).any(|window| window == bytes))
}

/// A 26/3 whose body is `text_id` (the first dword after the header).
fn dialog_body(text_id: u32) -> Vec<u8> {
    [vec![26, 0, 3, 0], text_id.to_le_bytes().to_vec()].concat()
}

/// A 26/3 row: its response id, then the label (`dialogue_choice_row`'s first three dwords).
fn dialog_row(action: Action, value: u32, label: u32) -> Vec<u8> {
    let id = mail_chat::response(action, value);
    [
        id.to_le_bytes().to_vec(),
        17u32.to_le_bytes().to_vec(),
        label.to_le_bytes().to_vec(),
    ]
    .concat()
}

fn answer(action: Action, value: u32) -> Vec<u8> {
    [vec![26, 0, 6, 0], mail_chat::response(action, value).to_le_bytes().to_vec()].concat()
}

/// The job id a staff member was told ("... (job 12).").
fn job_of(messages: &[Vec<u8>]) -> u64 {
    let text: String = messages
        .iter()
        .map(|message| String::from_utf8_lossy(message).into_owned())
        .collect();
    let at = text.find("(job ").expect("a queued job reply") + "(job ".len();
    text[at..]
        .chars()
        .take_while(char::is_ascii_digit)
        .collect::<String>()
        .parse()
        .unwrap()
}

fn wait(job: u64) -> admin::JobView {
    let deadline = Instant::now() + Duration::from_secs(60);
    loop {
        let view = admin::job(job).expect("a known job");
        if view.state == JobState::Done {
            return view;
        }
        assert!(Instant::now() < deadline, "job {job} never finished: {view:?}");
        std::thread::sleep(Duration::from_millis(20));
    }
}

/// A real server (socket, template) whose sessions are driven directly (`l042_moderation`'s rig).
struct Rig {
    server: Server,
    next: u32,
}

impl Rig {
    fn new(files: &Files) -> Self {
        let mut options = test_support::options(files.dir().join("state.json"));
        options.account_store = Some(files.store());
        options.handoff_key = HANDOFF.to_vec();
        Self {
            server: Server::new(options).unwrap(),
            next: 0,
        }
    }

    /// `Server::step` (the world tick that runs `mail_routing::tick`), then what each session
    /// was sent, after the sender's 250 ms retry interval (see `l042_moderation::Rig::step`).
    fn step(&mut self) -> BTreeMap<SessionKey, Vec<Vec<u8>>> {
        test_support::step(&mut self.server).unwrap();
        std::thread::sleep(Duration::from_millis(300));
        let keys: Vec<_> = self.server.sessions.keys().copied().collect();
        keys.into_iter().map(|key| (key, self.drain(key))).collect()
    }

    fn join(&mut self, index: usize) -> SessionKey {
        self.join_guid(index, guid(index))
    }

    fn join_guid(&mut self, index: usize, character: u64) -> SessionKey {
        self.next += 1;
        let id = 7_200 + self.next;
        let peer: SocketAddr = format!("127.0.0.1:{}", 31_000 + id).parse().unwrap();
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
            self.server.template.as_deref(),
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
        let ticket =
            login::gateway_binding_ticket(HANDOFF, store.account(account(index)).unwrap(), character);
        self.send(key, &bind(character, &ticket)).unwrap();
        self.send(key, &wrap_application(&[13, 0])).unwrap();
        self.send(key, &wrap_application(&[10, 0])).unwrap();
        self.drain(key);
        key
    }

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

    /// One inner application packet; what the sender saw.
    fn packet(&mut self, key: SessionKey, inner: &[u8]) -> Vec<Vec<u8>> {
        self.send(key, &wrap_application(inner)).unwrap();
        self.drain(key)
    }

    fn command(&mut self, key: SessionKey, text: &str) -> Vec<Vec<u8>> {
        self.packet(key, &osfr_tools::chat::local_request(text))
    }

    fn drain(&mut self, key: SessionKey) -> Vec<Vec<u8>> {
        let now = self.now();
        let session = self.server.sessions.get_mut(&key).unwrap();
        super::super::f120_fishing_hour_reconnect::drain_replies(session, now)
    }

    fn gameplay(&self, key: SessionKey) -> &osfr_tools::gameplay::Gameplay {
        self.server.sessions[&key].gateway.gameplay().unwrap()
    }

    /// Stack count of `definition` in the session's bag.
    fn count(&self, key: SessionKey, definition: i32) -> u32 {
        let inventory = self.gameplay(key).inventory_state();
        (inventory.items.iter())
            .filter(|item| item.definition_id == definition)
            .map(|item| item.count)
            .sum()
    }

    fn coins(&self, key: SessionKey) -> u32 {
        self.gameplay(key).inventory_state().coins
    }

    /// A staff send through chat, the world tick that starts it and one more once its job has
    /// finished: the job, and what every session was told meanwhile (a short job can land
    /// within the first tick, so its notice comes then).
    fn mail(
        &mut self,
        staff: SessionKey,
        text: &str,
    ) -> (admin::JobView, BTreeMap<SessionKey, Vec<Vec<u8>>>) {
        let staged = self.command(staff, text);
        assert!(!said(&staged, "Cannot") && !said(&staged, "Usage"), "{text}");
        let mut told = self.step();
        let view = wait(job_of(&told[&staff]));
        for (key, messages) in self.step() {
            told.entry(key).or_default().extend(messages);
        }
        (view, told)
    }

    /// Swap in `state` as this session's live gameplay (no save is written), returning the old.
    fn swap_gameplay(
        &mut self,
        key: SessionKey,
        game: osfr_tools::gameplay::Gameplay,
    ) -> osfr_tools::gameplay::Gameplay {
        let session = self.server.sessions.get_mut(&key).unwrap();
        std::mem::replace(session.gateway.gameplay_mut().unwrap(), game)
    }
}

fn skip() -> bool {
    !test_support::bin_test_target("g2_mail")
}

#[test]
fn g2_mail_to_one_online_player_notifies_and_reads_back() {
    if skip() {
        return;
    }
    let files = Files::new("one", 0);
    let mut rig = Rig::new(&files);
    let [ada, pip] = [ADA, PIP].map(|index| rig.join(index));
    let entered = rig.step();
    assert!(!said(&entered[&pip], "new messages"), "an empty mailbox says nothing");

    let (job, told) = rig.mail(ada, "realm mail Pip Player | Welcome | Hello Pip, the realm is open.");
    assert_eq!((job.total, job.done, job.skipped), (1, 1, 0));
    assert_eq!(job.to_summary, "Pip Player");
    assert!(said(&told[&ada], "Mail queued for 1 character (job "));
    assert!(said(&told[&pip], "You have 1 new messages"), "the client's own notice row");
    assert!(!said(&told[&ada], "new messages"), "only the recipient hears it");

    let read = rig.command(pip, "realm mail");
    assert!(said(&read, "Mail 1 of 1 (new): \"Welcome\" from Ada Admin, today."));
    assert!(said(&read, "Hello Pip, the realm is open."));
    assert!(has(&read, &dialog_body(mail_chat::MAIL_TEXT_ID)), "the dialog window opens");
    assert!(has(&read, &[26, 0, 4, 0]), "a shown dialog is closed first (T844)");
    assert_eq!(mail::unread(&mail::mailbox_path(&files.store(), guid(PIP))), 0);
    // The slash form, for a client that sends it, reads the same mailbox, now read.
    let again = rig.command(pip, "/mail");
    assert!(said(&again, "Mail 1 of 1: \"Welcome\""));
    assert!(said(&rig.command(pip, "realm mail 2"), "that was the last"));
    // Nothing more to tell on later ticks.
    assert!(!said(&rig.step()[&pip], "new messages"));
}

#[test]
fn g2_mail_to_everyone_reaches_an_offline_save_and_its_next_login() {
    if skip() {
        return;
    }
    let files = Files::new("everyone", 0);
    let mut rig = Rig::new(&files);
    let mo = rig.join(MO);
    rig.step();
    let (job, told) = rig.mail(mo, "realm mail everyone | Patch notes | The realm was updated today.");
    assert_eq!((job.total, job.done), (CAST.len(), CAST.len()));
    assert!(said(&told[&mo], "You have 1 new messages"), "the sender is one of everyone");
    assert_eq!(job.to_summary, "everyone");
    // Oz never logged in: the letter waits in the mailbox beside the account store, and the
    // character save was never written (mail touches no save).
    let oz_mail = files.mail(OZ);
    assert_eq!(oz_mail.len(), 1);
    assert_eq!((oz_mail[0].subject.as_str(), oz_mail[0].from.as_str()), ("Patch notes", "Mo Moderator"));
    assert!(files.dir().join("mail").join(format!("{}.json", guid(OZ))).is_file());
    assert!(!paths::character_state_path(&files.dir().join("state.json"), guid(OZ)).exists());
    // The audit row: who sent what to whom.
    let audit = files.audit();
    assert_eq!(audit.len(), 1);
    assert_eq!(audit[0]["action"], "mail_send");
    assert_eq!(audit[0]["staff_account"], account(MO));
    assert_eq!(audit[0]["staff"], "Mo Moderator");
    assert_eq!(audit[0]["to"], "everyone");
    assert_eq!(audit[0]["recipients"], CAST.len());
    assert_eq!(audit[0]["subject"], "Patch notes");

    let oz = rig.join(OZ);
    let entered = rig.step();
    assert!(said(&entered[&oz], "You have 1 new messages"), "the login notice");
    assert!(!said(&rig.step()[&oz], "new messages"), "once per login");
    assert!(said(&rig.command(oz, "realm mail"), "The realm was updated today."));
}

#[test]
fn g2_mail_attachment_is_claimed_once_and_a_full_bag_refuses() {
    if skip() {
        return;
    }
    let files = Files::new("claim", 0);
    let mut rig = Rig::new(&files);
    let [ada, pip, oz] = [ADA, PIP, OZ].map(|index| rig.join(index));
    rig.step();
    let (job, _) = rig.mail(ada, &format!("realm mail Pip Player, Oz Offline | A gift | For you. | 25c {STACKABLE}x2"));
    assert_eq!((job.total, job.done), (2, 2));

    let (coins, items) = (rig.coins(pip), rig.count(pip, STACKABLE));
    let read = rig.command(pip, "realm mail");
    assert!(said(&read, &format!("Attached: 25 coins, 2 x item {STACKABLE}")));
    let id = files.mail(PIP)[0].mail_id;
    assert!(has(&read, &dialog_row(Action::Claim, id, mail_chat::CLAIM_TEXT_ID)));
    // The dialog's Claim button: through the reward path into the bag and the wallet.
    let claimed = rig.packet(pip, &answer(Action::Claim, id));
    assert!(said(&claimed, &format!("You claimed 25 coins, 2 x item {STACKABLE}.")));
    assert!(has(&claimed, &[38, 0, 19, 0]), "the wallet's coin count packet");
    assert_eq!((rig.coins(pip), rig.count(pip, STACKABLE)), (coins + 25, items + 2));
    assert!(files.mail(PIP)[0].claimed);
    // Claimed once: the chat form and the button again both refuse, and nothing more is paid.
    assert!(said(&rig.command(pip, "realm mail claim 1"), "nothing to claim"));
    assert!(said(&rig.packet(pip, &answer(Action::Claim, id)), "nothing to claim"));
    assert_eq!((rig.coins(pip), rig.count(pip, STACKABLE)), (coins + 25, items + 2));

    // A full bag: Oz's live bag holds 4,096 full stacks (`inventory::MAX_ITEMS`). A save that
    // large exceeds the save cap, so the full bag is loaded into the live session only, through
    // the production `Gameplay::load` of a state; nothing is saved while it is in.
    let mut full: GameplayState = rig.gameplay(oz).state();
    let first_id = full.inventory.items.iter().map(|item| item.id).max().unwrap_or(0) + 1;
    let pad = osfr_tools::inventory::MAX_ITEMS - full.inventory.items.len();
    full.inventory.items.extend((0..pad).map(|n| InventoryItem {
        definition_id: STACKABLE,
        tint: 0,
        id: first_id + n as i32,
        count: 50,
        consumed_count: 0,
        stored_count: 0,
        last_cast_time: i64::from(i32::MAX),
        rental: None,
        ability_count: 0,
        activate_enabled: false,
    }));
    let content = rig.server.options.content.clone().unwrap();
    let full = osfr_tools::gameplay::Gameplay::load(&content, Some(full)).unwrap();
    let own = rig.swap_gameplay(oz, full);
    assert_eq!(rig.gameplay(oz).inventory_state().items.len(), osfr_tools::inventory::MAX_ITEMS);
    let (oz_coins, oz_items) = (rig.coins(oz), rig.count(oz, STACKABLE));
    let refused = rig.command(oz, "realm mail claim 1");
    assert!(said(&refused, "Cannot claim this mail: inventory is full."), "{refused:?}");
    assert_eq!((rig.coins(oz), rig.count(oz, STACKABLE)), (oz_coins, oz_items));
    assert!(!files.mail(OZ)[0].claimed, "the letter keeps its attachment");
    // And an unclaimed letter cannot be deleted away.
    assert!(said(&rig.command(oz, "realm mail delete 1"), "claim that message's attachments"));
    assert_eq!(files.mail(OZ).len(), 1);
    // Room again: the same letter claims.
    rig.swap_gameplay(oz, own);
    let oz_coins = rig.coins(oz);
    assert!(said(&rig.command(oz, "realm mail claim 1"), "You claimed 25 coins"));
    assert_eq!(rig.coins(oz), oz_coins + 25);
}

#[test]
fn g2_mail_players_and_builders_cannot_send() {
    if skip() {
        return;
    }
    let files = Files::new("denied", 0);
    let mut rig = Rig::new(&files);
    let [pip, bea] = [PIP, BEA].map(|index| rig.join(index));
    rig.step();
    for key in [pip, bea] {
        let refused = rig.command(key, "realm mail everyone | Free coins | Click here.");
        assert!(said(&refused, DENIED), "{refused:?}");
    }
    let told = rig.step();
    assert!(told.values().all(|messages| !said(messages, "Mail queued")));
    for index in 0..CAST.len() {
        assert!(files.mail(index).is_empty(), "nothing was delivered");
    }
    assert!(files.audit().is_empty(), "no send, no audit row");
    // Reading stays every player's.
    assert!(said(&rig.command(pip, "realm mail"), "You have no mail."));
}

#[test]
fn g2_mail_mass_send_never_stalls_the_tick() {
    if skip() {
        return;
    }
    const EXTRA: usize = 200;
    let files = Files::new("mass", EXTRA);
    let mut rig = Rig::new(&files);
    let ada = rig.join(ADA);
    rig.step();
    let staged = rig.command(ada, "realm mail everyone | Hello all | One letter each.");
    assert!(!said(&staged, "Cannot"));
    let started = Instant::now();
    let told = rig.step();
    let job = job_of(&told[&ada]);
    let total = CAST.len() + EXTRA;
    // The job is paced (`send_rate_per_second`) on its own thread: while it is still writing,
    // the world tick keeps its rhythm.
    let (mut steps, mut slowest, mut slowest_mail) = (0, Duration::ZERO, Duration::ZERO);
    while admin::job(job).unwrap().state != JobState::Done {
        let tick = Instant::now();
        test_support::step(&mut rig.server).unwrap();
        slowest = slowest.max(tick.elapsed());
        // The mail share of a world tick on its own (`Server::step` also runs it).
        let mail_tick = Instant::now();
        super::tick(&mut rig.server.sessions, &rig.server.options);
        slowest_mail = slowest_mail.max(mail_tick.elapsed());
        steps += 1;
        std::thread::sleep(Duration::from_millis(50));
    }
    let view = wait(job);
    eprintln!(
        "g2 mass send: {total} letters in {:?}, {steps} world steps meanwhile, slowest step \
         {slowest:?}, slowest mail tick {slowest_mail:?}",
        started.elapsed()
    );
    assert_eq!((view.total, view.done, view.skipped), (total, total, 0));
    assert!(steps >= 20, "the tick ran throughout the job ({steps} steps)");
    // The mail tick only drains a list: far inside the 50 ms tick while 205 mailboxes are written.
    assert!(slowest_mail < Duration::from_millis(50), "the mail tick waited: {slowest_mail:?}");
    // A whole step also saves characters (not mail's); it must never wait out the job.
    assert!(slowest < Duration::from_millis(500), "a step waited on the job: {slowest:?}");
    assert_eq!(admin::list_for(&files.store(), 1_001 + EXTRA as u64 - 1).unwrap().len(), 1);
}

#[test]
fn g2_mail_postbox_click_offers_read_mail_and_mail_sorting() {
    if skip() {
        return;
    }
    let files = Files::new("postbox", 0);
    let mut rig = Rig::new(&files);
    let ada = rig.join(ADA);
    rig.step();
    let [x, y, z] = SANCTUARY_POSITION;
    let moved = rig.command(ada, &format!("realm goto {} {y} {}", x + 2.0, z + 2.0));
    assert!(said(&moved, "Developer teleport"), "{moved:?}");
    assert_eq!(rig.server.sessions[&ada].gateway.state().zone_id, 0, "the open world");
    let click = |target: u64| {
        NpcInteractRequest {
            target,
            trailing_flag: false,
        }
        .encode()
    };
    let menu = rig.packet(ada, &click(SANCTUARY_POSTBOX));
    let body = [dialog_body(mail_chat::MAILBOX_TEXT_ID), vec![0; 4]].concat();
    assert!(has(&menu, &[body, SANCTUARY_POSTBOX.to_le_bytes().to_vec()].concat()), "spoken by the postbox");
    assert!(has(&menu, &dialog_row(Action::Read, 0, mail_chat::READ_MAIL_TEXT_ID)));
    assert!(has(&menu, &dialog_row(Action::Sort, SANCTUARY_GAME, mail_chat::MAIL_SORTING_TEXT_ID)));
    // A postbox out of reach (Seaside's) answers nothing.
    let far = rig.packet(ada, &click(100_000_001_881));
    assert!(!has(&far, &dialog_body(mail_chat::MAILBOX_TEXT_ID)));
    // Read mail reads this character's mailbox.
    assert!(said(&rig.packet(ada, &answer(Action::Read, 0)), "You have no mail."));
    // Mail Sorting goes to gameplay and gets the Games window row's own gate: this character has
    // no Postman job yet, so the client's missing-job alert (the row's refusal), after the
    // dialog closes.
    let sort = rig.packet(ada, &answer(Action::Sort, SANCTUARY_GAME));
    assert!(has(&sort, &[26, 0, 4, 0]));
    assert!(
        has(&sort, &osfr_tools::activity_portal::join_error(osfr_tools::activity_portal::MISSING_JOB_TEXT)),
        "{sort:?}"
    );
}
