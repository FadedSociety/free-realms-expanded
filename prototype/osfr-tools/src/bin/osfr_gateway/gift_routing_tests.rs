//! gifts-codes through the production path: the L042 cast's account store, sessions bound by a
//! real handoff ticket, Claim window packets and `realm redeem` through
//! `Session::dispatch_message` (the gateway's gift stop before gameplay), and `Server::step` for
//! the world tick that tells players. The content is the lab fixture with the release item tables
//! laid over it, so the shipped catalog's items resolve; the catalog itself is the shipped
//! `data/content/gift_catalog.json`. Each session's event clock is pinned, so a test never
//! depends on today's date. Synthetic names and ids only; every file lives in a scratch folder.
// Relative paths, as the other gateway test modules use: `tests/playtest_gateway_smoke.rs` and
// `tests/bot_smoke.rs` include the whole bin by `#[path]`, where `crate::` is their own crate.
use super::super::super::moderation::tests::{ADA, Files, HANDOFF, MO, PIP, account, guid, said};
use super::super::super::tests::test_support;
use super::super::super::*;
use osfr_tools::gifts::{self, CLAIM_TAG, Catalog, Via, admin as gift_admin};
use osfr_tools::inventory::InventoryItem;
use osfr_tools::redeem_codes::{self, Codes};
use std::collections::BTreeMap;

/// Another player of the L042 cast ("Ann Lee Moss").
const ANN: usize = 5;
/// A stackable item of the fixture's own inventory (max stack 50), to fill a bag.
const STACKABLE: i32 = 62_529;
const DOMO_HAT: i32 = 10_401;
const FIREFROG_LEGS: i32 = 77_418;
const ROBGOBLIN_COSTUME: i32 = 69_115;

/// Unix ms at noon UTC on a day: 2026-09-20 runs no event; 2026-10-15 is in Halloween.
fn noon(y: i64, m: i64, d: i64) -> u64 {
    let day = redeem_codes::days_from_civil(y, m, d);
    u64::try_from(day * 86_400_000 + 43_200_000).unwrap()
}

fn quiet() -> u64 {
    noon(2026, 9, 20)
}

fn halloween() -> u64 {
    noon(2026, 10, 15)
}

fn copy_tree(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for entry in fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let target = to.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_tree(&entry.path(), &target);
        } else if fs::hard_link(entry.path(), &target).is_err() {
            fs::copy(entry.path(), &target).unwrap();
        }
    }
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

fn has(messages: &[Vec<u8>], bytes: &[u8]) -> bool {
    messages
        .iter()
        .any(|message| message.windows(bytes.len()).any(|window| window == bytes))
}

/// A real server over the L042 store, with its own world folder (event toggles never touch the
/// shared content) and the release item tables over the fixture content.
struct Rig {
    files: Files,
    server: Server,
    next: u32,
    /// The client side of each session's reliable stream: a whole Claim window list is several
    /// fragments (`f120::drain_replies` reads unfragmented messages only).
    receivers: BTreeMap<SessionKey, ReliableReceiver>,
}

impl Rig {
    /// `None` off this PC's lab fixtures.
    fn new(tag: &str) -> Option<Self> {
        if !test_support::bin_test_target("gc_gifts") {
            return None;
        }
        let files = Files::new(&format!("gc-{tag}"));
        let mut options = test_support::options(files.dir().join("state.json"));
        if !test_support::fixtures_present(&options) {
            return None;
        }
        let content = files.dir().join("content");
        copy_tree(options.content.as_deref().unwrap(), &content);
        // The release item table and its supplement (B3's retired rows the catalog gives).
        let release = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/release");
        for table in ["item-table.json", "item-table-supplement.json"] {
            fs::copy(release.join(table), content.join(table)).unwrap();
        }
        options.content = Some(content);
        options.world = Some(files.dir().join("world"));
        fs::create_dir_all(files.dir().join("world")).unwrap();
        options.account_store = Some(files.store());
        options.handoff_key = HANDOFF.to_vec();
        let server = Server::new(options).unwrap();
        Some(Self { files, server, next: 0, receivers: BTreeMap::new() })
    }

    fn store(&self) -> PathBuf {
        self.files.store()
    }

    /// Bind `index`'s character, enter the world and pin its event clock to `clock`.
    fn join(&mut self, index: usize, clock: u64) -> SessionKey {
        self.next += 1;
        let id = 7_600 + self.next;
        let peer: SocketAddr = format!("127.0.0.1:{}", 32_000 + id).parse().unwrap();
        let request = SessionRequest { id, crc_length: 3, buffer_size: 1400 };
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
        let key = SessionKey { peer, connection_id: id };
        self.server.world.session_keys.insert(session.id, key);
        self.server.sessions.insert(key, session);
        let store = AccountStore::load(&self.store()).unwrap();
        let ticket =
            login::gateway_binding_ticket(HANDOFF, store.account(account(index)).unwrap(), guid(index));
        self.send(key, &bind(guid(index), &ticket));
        self.send(key, &wrap_application(&[13, 0]));
        self.send(key, &wrap_application(&[10, 0]));
        self.drain(key);
        self.clock(key, clock);
        key
    }

    fn clock(&mut self, key: SessionKey, unix_ms: u64) {
        let session = self.server.sessions.get_mut(&key).unwrap();
        session.gateway.gameplay_mut().unwrap().set_event_clock(Some(unix_ms));
    }

    fn send(&mut self, key: SessionKey, message: &[u8]) {
        let now = self.server.start.elapsed().as_millis() as u64;
        let mut session = self.server.sessions.remove(&key).expect("live session");
        let result = session.dispatch_message(
            message,
            &self.server.options,
            now,
            &mut self.server.world,
            &mut self.server.membership,
            &mut self.server.sessions,
            None,
        );
        self.server.sessions.insert(key, session);
        result.unwrap();
    }

    /// Every message the session has queued, fragments reassembled, each batch acknowledged as
    /// the client does so the sender window keeps draining.
    fn drain(&mut self, key: SessionKey) -> Vec<Vec<u8>> {
        let now = self.server.start.elapsed().as_millis() as u64;
        let session = self.server.sessions.get_mut(&key).unwrap();
        let receiver = (self.receivers.entry(key))
            .or_insert_with(|| ReliableReceiver::new(0, 512, 1 << 20, 2 << 20).unwrap());
        let mut messages = Vec::new();
        loop {
            let wires = session.sender.poll(now).unwrap();
            let mut last = None;
            for wire in wires {
                let packet = session.codec.decode(&wire).unwrap();
                if !matches!(packet.opcode, RELIABLE_DATA | RELIABLE_FRAGMENT) || packet.payload.len() < 3 {
                    continue;
                }
                last = Some(u16::from_be_bytes([packet.payload[0], packet.payload[1]]));
                // A resend of a packet already taken is a duplicate, not a message.
                if let Ok(received) = receiver.receive(&packet) {
                    messages.extend(received.messages);
                }
            }
            let Some(sequence) = last else {
                return messages;
            };
            let ack = osfr_tools::soe::Packet { opcode: CUMULATIVE_ACK, payload: sequence.to_be_bytes().to_vec() };
            session.sender.acknowledge(&ack).unwrap();
        }
    }

    /// One world tick (`mail_routing::tick` runs `gift_routing::tick`), then what `key` was sent.
    fn step(&mut self, key: SessionKey) -> Vec<Vec<u8>> {
        test_support::step(&mut self.server).unwrap();
        std::thread::sleep(Duration::from_millis(300));
        let mut got = Vec::new();
        for other in self.server.sessions.keys().copied().collect::<Vec<_>>() {
            let replies = self.drain(other);
            if other == key {
                got = replies;
            }
        }
        got
    }

    fn packet(&mut self, key: SessionKey, inner: &[u8]) -> Vec<Vec<u8>> {
        self.send(key, &wrap_application(inner));
        self.drain(key)
    }

    fn command(&mut self, key: SessionKey, text: &str) -> Vec<Vec<u8>> {
        self.packet(key, &osfr_tools::chat::local_request(text))
    }

    fn gameplay(&self, key: SessionKey) -> &osfr_tools::gameplay::Gameplay {
        self.server.sessions[&key].gateway.gameplay().unwrap()
    }

    fn count(&self, key: SessionKey, definition: i32) -> u32 {
        (self.gameplay(key).inventory_state().items.iter())
            .filter(|item| item.definition_id == definition)
            .map(|item| item.count)
            .sum()
    }

    fn gift_box(&self, index: usize) -> gifts::GiftBox {
        gifts::load(&gifts::box_path(&self.store(), account(index))).unwrap()
    }

    /// The pending gift of `index`'s account made from catalog row `key`.
    fn pending(&self, index: usize, key: &str) -> Option<gifts::Pending> {
        let found = self.gift_box(index).pending.into_iter();
        found.into_iter().find(|pending| pending.key.as_deref() == Some(key))
    }
}

fn catalog() -> Catalog {
    Catalog::load(&Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content")).unwrap()
}

fn claim(tracker: u32, ids: &[u64]) -> Vec<u8> {
    let mut packet = vec![108, 0, 3, 0];
    packet.extend(tracker.to_le_bytes());
    packet.extend((ids.len() as u32).to_le_bytes());
    for id in ids {
        packet.extend(id.to_le_bytes());
    }
    packet
}

/// The head of a 108/4 for `tracker` with its success byte.
fn answer(tracker: u32, ok: bool) -> Vec<u8> {
    [vec![108, 0, 4, 0], tracker.to_le_bytes().to_vec(), vec![u8::from(ok)]].concat()
}

/// One row's claim id as the row carries it: a count of one, then the id.
fn listed(id: u64) -> Vec<u8> {
    [1u32.to_le_bytes().to_vec(), id.to_le_bytes().to_vec()].concat()
}

#[test]
fn gc_gifts_sent_from_the_panel_list_claim_once_and_a_full_bag_refuses() {
    let Some(mut rig) = Rig::new("claim") else {
        return;
    };
    let pip = rig.join(PIP, quiet());
    // Login: the standing offers land in the account's box, the whole list goes out (108/2,
    // which posts the client's own ClaimNotification per row) with a chat line.
    let entered = rig.step(pip);
    assert!(has(&entered, &[108, 0, 2, 0, 1]), "the login list");
    assert!(said(&entered, "gifts are waiting in your Claim window"));
    let offers = rig.gift_box(PIP).pending;
    assert!(offers.len() >= 30 && offers.iter().all(|p| p.via == Via::Offer), "{}", offers.len());
    assert!(rig.pending(PIP, "profile-witch-pail").is_none(), "no Halloween row off its event");
    assert!(!said(&rig.step(pip), "waiting"), "told once per login");

    // The panel's send (the facade `POST /admin/api/gifts/send` calls), to an online player.
    let catalog = catalog();
    let hat = catalog.get("promo-domo-hat").unwrap();
    let (to, id) = gift_admin::send(&rig.store(), hat, guid(PIP), "Ada Admin", "event winner").unwrap();
    assert_eq!(to, account(PIP));
    let told = rig.step(pip);
    let claim_id = CLAIM_TAG | u64::from(id);
    assert!(has(&told, &listed(claim_id)), "the new row reaches the open session");
    assert!(said(&told, "waiting in your Claim window"));
    // The window's own request lists it too.
    assert!(has(&rig.packet(pip, &[108, 0, 1, 0]), &listed(claim_id)));

    let before = rig.count(pip, DOMO_HAT);
    let claimed = rig.packet(pip, &claim(77, &[claim_id]));
    assert!(has(&claimed, &answer(77, true)), "{claimed:?}");
    assert!(said(&claimed, "You claimed Domo Hat."));
    assert!(!has(&claimed, &listed(claim_id)), "the answer lists what is still waiting");
    assert_eq!(rig.count(pip, DOMO_HAT), before + 1, "into the bag");
    // Once: a second claim of the same id is refused and pays nothing.
    let again = rig.packet(pip, &claim(78, &[claim_id]));
    assert!(has(&again, &answer(78, false)) && said(&again, "no longer waiting"));
    assert_eq!(rig.count(pip, DOMO_HAT), before + 1);

    // A full bag: 100 shown stacks, and the hat needs a new one.
    let (_, id) = gift_admin::send(&rig.store(), hat, guid(PIP), "Ada Admin", "").unwrap();
    let mut full = rig.gameplay(pip).state();
    let first = full.inventory.items.iter().map(|item| item.id).max().unwrap_or(0) + 1;
    let pad = osfr_tools::inventory::BAG_STACKS - full.inventory.bag_stacks();
    full.inventory.items.extend((0..pad).map(|n| InventoryItem {
        definition_id: STACKABLE,
        tint: 0,
        id: first + n as i32,
        count: 50,
        consumed_count: 0,
        stored_count: 0,
        last_cast_time: i64::from(i32::MAX),
        rental: None,
        ability_count: 0,
        activate_enabled: false,
    }));
    // The instance counter stays past every id, as the allocator keeps it.
    full.inventory.next_instance_id = Some(i64::from(first) + pad as i64);
    let content = rig.server.options.content.clone().unwrap();
    let full = osfr_tools::gameplay::Gameplay::load(&content, Some(full)).unwrap();
    let session = rig.server.sessions.get_mut(&pip).unwrap();
    let own = std::mem::replace(session.gateway.gameplay_mut().unwrap(), full);
    rig.clock(pip, quiet());
    let held = rig.count(pip, DOMO_HAT);
    let refused = rig.packet(pip, &claim(79, &[CLAIM_TAG | u64::from(id)]));
    assert!(has(&refused, &answer(79, false)), "{refused:?}");
    assert!(said(&refused, "bag is full"), "{refused:?}");
    assert_eq!(rig.count(pip, DOMO_HAT), held, "nothing paid");
    assert!(rig.gift_box(PIP).pending.iter().any(|p| p.id == id), "the gift keeps waiting");
    // Room again: the same gift claims.
    *rig.server.sessions.get_mut(&pip).unwrap().gateway.gameplay_mut().unwrap() = own;
    rig.clock(pip, quiet());
    assert!(has(&rig.packet(pip, &claim(80, &[CLAIM_TAG | u64::from(id)])), &answer(80, true)));
}

#[test]
fn gc_gifts_codes_classic_single_use_expired_and_bound() {
    let Some(mut rig) = Rig::new("codes") else {
        return;
    };
    let [pip, ann, ada] = [PIP, ANN, ADA].map(|index| rig.join(index, quiet()));
    rig.step(pip);
    // Classic: the catalog's original code, once per account, into the Claim window.
    let redeemed = rig.command(pip, "realm redeem getspooky");
    assert!(said(&redeemed, "Redeemed code GETSPOOKY"), "{redeemed:?}");
    let costume = rig.pending(PIP, "code-getspooky").expect("the code's gift waits");
    assert!(has(&redeemed, &listed(costume.claim_id())), "the list comes with it");
    assert!(said(&rig.command(pip, "realm redeem GETSPOOKY"), "You already redeemed that code."));
    let before = rig.count(pip, ROBGOBLIN_COSTUME);
    assert!(has(&rig.packet(pip, &claim(5, &[costume.claim_id()])), &answer(5, true)));
    assert_eq!(rig.count(pip, ROBGOBLIN_COSTUME), before + 1);
    assert!(said(&rig.command(pip, "realm redeem getspooky"), "You already redeemed that code."), "claimed: still once");
    assert!(said(&rig.command(pip, "realm redeem NOTACODE"), "That code is not valid."));

    // Admin single use: the first player gets it, the next is refused.
    let catalog = catalog();
    let hat = catalog.get("promo-domo-hat").unwrap();
    let single = gift_admin::NewCode {
        gifts: vec![hat],
        uses: 1,
        expires_days: None,
        bound_character: None,
        note: "Costume contest".into(),
    };
    let (code, _) = gift_admin::make_code(&rig.store(), &single).unwrap();
    assert!(said(&rig.command(pip, &format!("realm redeem {code}")), "your gift is waiting"));
    assert!(rig.gift_box(PIP).pending.iter().any(|p| p.via == Via::Code && p.name == "Costume contest"));
    assert!(said(&rig.command(ann, &format!("realm redeem {code}")), "That code has been used up."));
    assert!(rig.gift_box(ANN).pending.iter().all(|p| p.via == Via::Offer), "nothing for Ann");

    // Expired: a code whose last day has passed.
    let (late, _) = gift_admin::make_code(&rig.store(), &gift_admin::NewCode { uses: 5, ..single }).unwrap();
    let path = gifts::codes_path(&rig.store());
    let mut codes = Codes::load(&path).unwrap();
    codes.codes.get_mut(&late).unwrap().expires = Some(redeem_codes::today() - 1);
    codes.save(&path).unwrap();
    assert!(said(&rig.command(ann, &format!("realm redeem {late}")), "That code has expired."));

    // Bound to another player: refused for Pip, taken by its owner.
    let bound = gift_admin::NewCode {
        gifts: vec![hat],
        uses: 1,
        expires_days: Some(30),
        bound_character: Some(guid(ADA)),
        note: String::new(),
    };
    let (mine, row) = gift_admin::make_code(&rig.store(), &bound).unwrap();
    assert_eq!(row.bound, Some(account(ADA)));
    assert!(said(&rig.command(pip, &format!("realm redeem {mine}")), "That code belongs to someone else."));
    assert_eq!(Codes::load(&path).unwrap().codes[&mine].uses, 0, "no use recorded");
    assert!(said(&rig.command(ada, &format!("realm redeem {mine}")), "your gift is waiting"));
}

#[test]
fn gc_gifts_an_event_offer_shows_only_during_its_event() {
    let Some(mut rig) = Rig::new("event") else {
        return;
    };
    let pip = rig.join(PIP, quiet());
    rig.step(pip);
    assert!(rig.pending(PIP, "profile-witch-pail").is_none());
    // Halloween starts for this session: the tick sees it and the row arrives.
    rig.clock(pip, halloween());
    let started = rig.step(pip);
    let pail = rig.pending(PIP, "profile-witch-pail").expect("the event's offer");
    assert!(has(&started, &listed(pail.claim_id())), "pushed when the event starts");
    assert!(said(&started, "waiting in your Claim window"));
    // Halloween ends: the row is hidden from the list and cannot be claimed...
    rig.clock(pip, quiet());
    let listing = rig.packet(pip, &[108, 0, 1, 0]);
    assert!(has(&listing, &[108, 0, 2, 0, 1]) && !has(&listing, &listed(pail.claim_id())));
    let refused = rig.packet(pip, &claim(3, &[pail.claim_id()]));
    assert!(has(&refused, &answer(3, false)));
    // ...and comes back with its event, still claimable once.
    rig.clock(pip, halloween());
    assert!(has(&rig.packet(pip, &[108, 0, 1, 0]), &listed(pail.claim_id())));
    assert!(has(&rig.packet(pip, &claim(4, &[pail.claim_id()])), &answer(4, true)));
}

#[test]
fn gc_gifts_a_repeating_offer_comes_back_after_its_hours() {
    let Some(mut rig) = Rig::new("repeat") else {
        return;
    };
    let pip = rig.join(PIP, quiet());
    rig.step(pip);
    let legs = rig.pending(PIP, "profile-firefrog-legs").expect("a daily offer");
    let before = rig.count(pip, FIREFROG_LEGS);
    assert!(has(&rig.packet(pip, &claim(1, &[legs.claim_id()])), &answer(1, true)));
    assert_eq!(rig.count(pip, FIREFROG_LEGS), before + 5, "the offer's stack of five");
    // Within its 24 hours it does not come back...
    rig.packet(pip, &[108, 0, 1, 0]);
    assert!(rig.pending(PIP, "profile-firefrog-legs").is_none());
    // ...after them it does (the claim moved back 25 hours in the account's box).
    let path = gifts::box_path(&rig.store(), account(PIP));
    gifts::update(&path, |gifts| {
        let taken = gifts.taken.get_mut("profile-firefrog-legs").unwrap();
        *taken -= 25 * 3600;
        Ok(())
    })
    .unwrap();
    let again = rig.packet(pip, &[108, 0, 1, 0]);
    let next = rig.pending(PIP, "profile-firefrog-legs").expect("offered again");
    assert_ne!(next.id, legs.id);
    assert!(has(&again, &listed(next.claim_id())));
    // A once-only offer never comes back.
    let hat = rig.pending(PIP, "profile-knitted-fish-hat").unwrap();
    assert!(has(&rig.packet(pip, &claim(2, &[hat.claim_id()])), &answer(2, true)));
    gifts::update(&path, |gifts| {
        *gifts.taken.get_mut("profile-knitted-fish-hat").unwrap() -= 400 * 24 * 3600;
        Ok(())
    })
    .unwrap();
    rig.packet(pip, &[108, 0, 1, 0]);
    assert!(rig.pending(PIP, "profile-knitted-fish-hat").is_none());
    // A moderator's character sees its own account's offers, not Pip's.
    let mo = rig.join(MO, quiet());
    rig.step(mo);
    assert!(rig.pending(MO, "profile-firefrog-legs").is_some());
}
