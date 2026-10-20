//! L011 (Security Model S12/L8): a vendor purchase whose character save fails changes
//! nothing and ends only that session. Test-only; no production code changes.
//!
//! Production path: `Session::commit_outcome` stages the whole 66/43 purchase on a cloned
//! `Gateway` and a cloned `SharedWorld`, calls `save` on the staged snapshot, and only then
//! installs the staged gateway/world (`self.gateway = staged_gateway`, `*world = staged_world`).
//! A save error returns `Err` from `commit_outcome`; `dispatch_message_inner` propagates it
//! with `?`, and `Server::step`'s `contain` error arm ends that one session through
//! `end_session`. The test writes no test-only hook: the next save targets a path under a
//! file, exactly as `isolation.rs` and T199 break a save.
//!
//! 66/43 request shape: client serializer `00b3ab20`; the shared vendor helpers in
//! `tests/common/gateway_vendor.rs` encode the same `MerchantBuyRequestV2` fields. The
//! content pack, prices and starting state are the shared `tests/common/vendor_fixture.rs`.

use super::test_support::{self, Scratch};
use super::*;
use osfr_tools::game_store::Snapshot;

// The shared test kit: this host uses a subset of its helpers.
#[allow(dead_code)]
#[path = "../../../tests/common/gateway_vendor.rs"]
mod gateway_vendor;
#[allow(dead_code)]
#[path = "../../../tests/common/vendor_fixture.rs"]
mod vendor_fixture;

use gateway_vendor::{
    buy_back_packet, count_definition, interact, login, merchant_status, select, sell_packet,
    vendor_buy_packet,
};
use vendor_fixture::{REPRESENTATIVE_ITEM, REPRESENTATIVE_VENDOR, SEEDED_ITEM, VendorFixture};

const BUY_TRANSACTION: u32 = 108_901;

fn options(fixture: &VendorFixture, state: PathBuf) -> Options {
    let mut options = test_support::options(state);
    options.reference = fixture.reference().to_owned();
    options.content = Some(fixture.content().to_owned());
    options.dev_tools = false;
    options
}

/// One session plus the shared containers a `Server` owns; `Server::new` is not needed
/// because this case asserts the transaction, not the teardown.
struct Scene {
    session: Session,
    world: SharedWorld,
    membership: World,
    sessions: SessionMap,
}

impl Scene {
    fn new(options: &Options, id: u32) -> Self {
        let session = Session::new(
            SessionId(u64::from(id)),
            format!("127.0.0.1:{}", 34_000 + id).parse().unwrap(),
            SessionRequest {
                id,
                crc_length: 3,
                buffer_size: 1400,
            },
            options,
            CRC_SEED,
            None,
            None,
        )
        .expect("vendor session loads");
        Self {
            session,
            world: SharedWorld::new(),
            membership: World::runtime(),
            sessions: SessionMap::new(),
        }
    }

    fn send(&mut self, options: &Options, inner: &[u8], now: u64) -> Result<(), String> {
        self.session
            .dispatch_message(
                &wrap(inner),
                options,
                now,
                &mut self.world,
                &mut self.membership,
                &mut self.sessions,
                None,
            )
            .map_err(|error| error.to_string())
    }

    fn drain(&mut self, now: u64) -> Vec<Vec<u8>> {
        super::super::f120_fishing_hour_reconnect::drain_replies(&mut self.session, now)
    }

    /// Login, Ready and Loaded, the three messages every world session sends first.
    fn enter(&mut self, options: &Options, now: u64) {
        let guid = self.session.gateway.state().guid;
        for (index, message) in [login(guid), wrap(&[13, 0]), wrap(&[10, 0])]
            .into_iter()
            .enumerate()
        {
            self.session
                .dispatch_message(
                    &message,
                    options,
                    now + index as u64,
                    &mut self.world,
                    &mut self.membership,
                    &mut self.sessions,
                    None,
                )
                .expect("initialization message handles");
        }
    }

    fn open_shop(&mut self, fixture: &VendorFixture, options: &Options, now: u64) {
        self.send(options, &interact(REPRESENTATIVE_VENDOR), now)
            .expect("vendor interaction handles");
        self.send(
            options,
            &select(REPRESENTATIVE_VENDOR, fixture.option()),
            now,
        )
        .expect("shop selection handles");
        let replies = self.drain(now);
        assert!(
            replies
                .iter()
                .any(|packet| packet[7..].starts_with(&[66, 0, 42, 0])),
            "the real vendor did not open its shop: {replies:?}"
        );
    }

    fn snapshot(&self) -> Snapshot {
        self.session.gateway.snapshot()
    }

    /// Break only the next save: the good save file is a component of the target path,
    /// so `save` cannot create its temporary file beside it. The good bytes stay put.
    fn break_next_save(&mut self, good_save: &Path) {
        self.session.state_path = good_save.join("blocked.json");
    }

    fn repair_save(&mut self, good_save: &Path) {
        self.session.state_path = good_save.to_owned();
    }
}

#[test]
fn a_failed_vendor_save_changes_nothing_and_preserves_buy_back_state() {
    let scratch = Scratch::new("l011-failed-save");
    let fixture = VendorFixture::new(scratch.path(), "content");
    let options = options(&fixture, scratch.state());
    let character = fixture.character();
    test_support::save_character(
        &options.state,
        &character,
        &fixture.state(fixture.price() + 1_000, false),
    );
    let mut scene = Scene::new(&options, 81);
    scene.enter(&options, 1_000);
    scene.open_shop(&fixture, &options, 1_100);

    // Seed the Coin Shop's buy-back window with a real 165/5 sale.
    scene
        .send(&options, &sell_packet(0, 1, 1), 1_200)
        .expect("the seeded sale handles");
    let sold = scene.drain(1_200);
    let transaction = sold
        .iter()
        .find(|packet| packet[7..].starts_with(&[165, 0, 6, 0]))
        .map(|packet| u32::from_le_bytes(packet[31..35].try_into().unwrap()))
        .expect("the sale completion carries its transaction id");
    // Completion record: header + result + item, kind, merchant, transaction at inner
    // offset 24 (client readers 00c74ef0 -> 00c71f30).
    assert_eq!(
        sold.iter()
            .filter(|packet| packet[7..].starts_with(&[165, 0, 6, 0]))
            .count(),
        1,
        "one sale completion"
    );

    let before = scene.snapshot();
    let before_inventory = before
        .gameplay
        .as_ref()
        .expect("pre-purchase gameplay exists")
        .inventory
        .clone();
    assert_eq!(
        before_inventory.coins,
        fixture.price() + 1_000 + fixture.sell_price(),
        "fixture setup: the sale paid its price"
    );
    assert_eq!(count_definition(&before_inventory, SEEDED_ITEM), 0);
    let good_save = fs::read(&options.state).expect("the good save is readable");
    scene.break_next_save(&options.state);

    // Check 1: the purchase transaction fails when its save cannot be written.
    let error = scene
        .send(
            &options,
            &vendor_buy_packet(BUY_TRANSACTION, REPRESENTATIVE_ITEM, 1),
            1_300,
        )
        .expect_err("a vendor purchase with a failed save must not commit");
    assert!(
        error.contains("os error"),
        "the transaction must fail in the save, not earlier: {error}"
    );
    let failed_replies = scene.drain(1_300);
    assert!(
        failed_replies.is_empty(),
        "a purchase whose save failed still queued a client success reply: {failed_replies:?}"
    );
    scene.repair_save(&options.state);

    // Check 2: the in-memory character is unchanged. `snapshot()` carries the character
    // pose and the inventory; the session-only Coin Shop idempotency cache is not part of
    // that type, so the retry below proves it behaviourally.
    assert_eq!(
        scene.snapshot(),
        before,
        "a failed save installed part of the purchase"
    );
    let after_inventory = scene
        .snapshot()
        .gameplay
        .expect("post-purchase gameplay exists")
        .inventory;
    assert_eq!(
        after_inventory.coins, before_inventory.coins,
        "a failed purchase spent coins"
    );
    assert_eq!(
        count_definition(&after_inventory, REPRESENTATIVE_ITEM),
        count_definition(&before_inventory, REPRESENTATIVE_ITEM),
        "a failed purchase granted the item"
    );

    // Check 3: the on-disk good save is byte-identical.
    assert_eq!(
        fs::read(&options.state).expect("the good save is still readable"),
        good_save,
        "a failed purchase wrote the character save"
    );

    // Check 4: the same 66/43 transaction is not cached as a success. A regression that
    // installs the staged Coin Shop before the save error is handled would answer this
    // retry from `vendor_transactions` without charging or granting.
    scene
        .send(
            &options,
            &vendor_buy_packet(BUY_TRANSACTION, REPRESENTATIVE_ITEM, 1),
            1_400,
        )
        .expect("the same transaction must commit once the save is writable");
    let retried = scene.drain(1_400);
    let completion = retried
        .iter()
        .find(|packet| packet[7..].starts_with(&[66, 0, 44, 0]))
        .expect("the retried purchase answers 66/44");
    assert_eq!(
        merchant_status(&completion[7..]),
        1,
        "the retried purchase was served as a refusal"
    );
    let committed = scene
        .snapshot()
        .gameplay
        .expect("committed gameplay exists")
        .inventory;
    assert_eq!(
        committed.coins,
        before_inventory.coins - fixture.price(),
        "the retried purchase was served from the failed transaction's cache"
    );
    assert_eq!(
        count_definition(&committed, REPRESENTATIVE_ITEM),
        count_definition(&before_inventory, REPRESENTATIVE_ITEM) + 1,
        "the retried purchase did not grant the item"
    );

    // The buy-back window is part of the same staged Coin Shop state: the sale record
    // survives with the same id and price, and the shop still serves its open session.
    scene
        .send(&options, &buy_back_packet(transaction, 0), 1_500)
        .expect("the buy-back handles after the failed purchase");
    let after_buy_back = scene
        .snapshot()
        .gameplay
        .expect("buy-back gameplay exists")
        .inventory;
    assert_eq!(
        after_buy_back.coins,
        before_inventory.coins - fixture.price() - fixture.sell_price(),
        "the failed purchase consumed or repriced the buy-back sale"
    );
    assert_eq!(
        count_definition(&after_buy_back, SEEDED_ITEM),
        1,
        "the buy-back sale record was lost or rewritten"
    );
}

#[test]
fn control_a_working_save_commits_the_same_purchase() {
    let scratch = Scratch::new("l011-control");
    let fixture = VendorFixture::new(scratch.path(), "content");
    let options = options(&fixture, scratch.state());
    let character = fixture.character();
    test_support::save_character(
        &options.state,
        &character,
        &fixture.state(fixture.price() + 1_000, false),
    );
    let mut scene = Scene::new(&options, 82);
    scene.enter(&options, 1_000);
    scene.open_shop(&fixture, &options, 1_100);

    let before = scene.snapshot();
    let before_inventory = before
        .gameplay
        .as_ref()
        .expect("pre-purchase gameplay exists")
        .inventory
        .clone();
    let good_save = fs::read(&options.state).expect("the good save is readable");
    scene
        .send(
            &options,
            &vendor_buy_packet(BUY_TRANSACTION, REPRESENTATIVE_ITEM, 1),
            1_200,
        )
        .expect("a vendor purchase with a working save commits");
    let after = scene.snapshot();
    let after_inventory = after
        .gameplay
        .as_ref()
        .expect("post-purchase gameplay exists")
        .inventory
        .clone();
    assert_eq!(
        after_inventory.coins,
        before_inventory.coins - fixture.price(),
        "the control purchase did not spend the authoritative price"
    );
    assert_eq!(
        count_definition(&after_inventory, REPRESENTATIVE_ITEM),
        count_definition(&before_inventory, REPRESENTATIVE_ITEM) + 1,
        "the control purchase did not grant the item"
    );
    assert_ne!(
        fs::read(&options.state).expect("the committed save is readable"),
        good_save,
        "the control purchase did not write the character save"
    );
}

/// Assertion 5: the save failure inside `Server::step` ends only that session, the victim
/// leaves no world or membership row, and the surviving sessions still answer a datagram.
#[test]
fn a_failed_vendor_save_ends_only_that_session() {
    let scratch = Scratch::new("l011-server");
    let fixture = VendorFixture::new(scratch.path(), "content");
    let (mut server, keys, server_scratch) = super::super::t020_group_routing::server_rig("l011");
    let victim_id = server.sessions[&keys[0]].id;
    let victim_guid = server.sessions[&keys[0]].character_guid;

    // The vendor character reuses the seeded world row's guid.
    let mut character = fixture.character();
    character.guid = victim_guid;
    let save_path = scratch.state();
    let gameplay = fixture.state(fixture.price() + 1_000, false);
    test_support::save_character(&save_path, &character, &gameplay);
    let mut gateway = Gateway::load_gameplay(
        &server.options.reference,
        character,
        fixture.content(),
        Some(gameplay),
    )
    .expect("vendor gateway loads");
    gateway.set_dev_tools(false);
    gateway.set_role(osfr_tools::account_store::AccountRole::Player);

    // A real client socket at the session's peer address; the buy arrives over UDP.
    let client = UdpSocket::bind("127.0.0.1:0").expect("client socket binds");
    let mut victim = server
        .sessions
        .remove(&keys[0])
        .expect("the seeded victim session exists");
    victim.peer = client.local_addr().expect("client address");
    victim.gateway = gateway;
    victim.state_path = save_path.clone();
    let key = SessionKey {
        peer: victim.peer,
        connection_id: victim.request.id,
    };
    // Handshake through the production dispatch: the committed login/Ready/Loaded is what
    // registers the victim's membership row that `end_session` must remove.
    for (index, message) in [login(victim_guid), wrap(&[13, 0]), wrap(&[10, 0])]
        .into_iter()
        .enumerate()
    {
        victim
            .dispatch_message(
                &message,
                &server.options,
                1_000 + index as u64,
                &mut server.world,
                &mut server.membership,
                &mut server.sessions,
                None,
            )
            .expect("the victim handshake handles");
    }
    assert_eq!(
        victim.character_guid, victim_guid,
        "the committed handshake must keep the fixture guid"
    );
    assert!(
        server.membership.player_location(victim_guid).is_some(),
        "fixture setup: the committed handshake registers membership"
    );
    let mut opened = Vec::new();
    for inner in [
        interact(REPRESENTATIVE_VENDOR),
        select(REPRESENTATIVE_VENDOR, fixture.option()),
    ] {
        opened.extend(
            victim
                .gateway
                .handle_at(&wrap(&inner), 1_003)
                .expect("the vendor interaction handles")
                .replies,
        );
    }
    assert!(
        opened
            .iter()
            .any(|packet| packet[7..].starts_with(&[66, 0, 42, 0])),
        "the vendor shop did not open on the real gateway"
    );
    server.world.session_keys.insert(victim.id, key);
    server.sessions.insert(key, victim);

    let good_save = fs::read(&save_path).expect("the good save is readable");
    let survivors: Vec<u64> = server
        .world
        .active_sessions
        .keys()
        .copied()
        .filter(|guid| *guid != victim_guid)
        .collect();
    assert_eq!(survivors.len(), 2, "the seeded world has two survivors");
    // Give every survivor a real socket. The rig seeds them with one shared fake peer, so
    // the ending session's leave fanout would hit a closed UDP port; the ICMP reset then
    // makes the next `Server::step` receive stop before its queued datagram.
    let mut survivor_sockets = Vec::new();
    let mut survivor_keys = Vec::new();
    for guid in &survivors {
        let id = server.world.active_sessions[guid];
        let seeded = server.world.session_keys[&id];
        let socket = UdpSocket::bind("127.0.0.1:0").expect("survivor socket binds");
        socket
            .set_read_timeout(Some(Duration::from_millis(100)))
            .expect("survivor read timeout sets");
        let mut session = server
            .sessions
            .remove(&seeded)
            .expect("the seeded survivor session exists");
        session.peer = socket.local_addr().expect("survivor address");
        let key = SessionKey {
            peer: session.peer,
            connection_id: session.request.id,
        };
        server.world.session_keys.insert(session.id, key);
        server.sessions.insert(key, session);
        survivor_sockets.push(socket);
        survivor_keys.push(key);
    }
    server
        .sessions
        .get_mut(&key)
        .expect("the victim session is present")
        .state_path = save_path.join("blocked.json");
    let wire = {
        let session = &server.sessions[&key];
        let mut payload = 0u16.to_be_bytes().to_vec();
        payload.extend(wrap(&vendor_buy_packet(
            BUY_TRANSACTION,
            REPRESENTATIVE_ITEM,
            1,
        )));
        session
            .codec
            .encode(RELIABLE_DATA, &payload)
            .expect("buy datagram encodes")
    };
    client
        .send_to(&wire, server.local_addr().expect("server address"))
        .expect("buy datagram sends");
    server
        .step()
        .expect("one failed session must not stop the server");

    assert!(
        !server.sessions.contains_key(&key),
        "the save failure did not end the victim session"
    );
    assert!(
        !server.world.active_sessions.contains_key(&victim_guid),
        "the ended session left a world ghost"
    );
    assert!(!server.world.joined.contains(&victim_id));
    assert!(!server.world.players.contains(victim_id));
    assert!(server.world.chat_session(victim_id).is_none());
    assert!(!server.world.session_keys.contains_key(&victim_id));
    assert!(
        server.membership.player_location(victim_guid).is_none(),
        "the ended session left a membership row"
    );
    assert_eq!(
        fs::read(&save_path).expect("the good save is still readable"),
        good_save,
        "the ended session wrote a partial save"
    );
    for (guid, key) in survivors.iter().zip(&survivor_keys) {
        let id = server.world.active_sessions[guid];
        assert_eq!(
            server.world.session_keys.get(&id),
            Some(key),
            "the ended session changed survivor {guid}'s session key"
        );
        assert!(
            server.sessions.contains_key(key),
            "another session ended with the victim"
        );
    }
    assert_eq!(
        server.world.active_sessions.len(),
        2,
        "the ended session left a world ghost"
    );

    // Drain whatever the ending session's leave fanout queued to each survivor, then prove
    // every survivor's next real datagram is still handled and answered on its own socket.
    let mut buffer = [0; 65_535];
    for (survivor_client, survivor_key) in survivor_sockets.iter().zip(&survivor_keys) {
        while survivor_client.recv_from(&mut buffer).is_ok() {}
        let survivor_login_guid = server.sessions[survivor_key].gateway.state().guid;
        let wire = {
            let session = &server.sessions[survivor_key];
            let mut payload = 0u16.to_be_bytes().to_vec();
            payload.extend(login(survivor_login_guid));
            session
                .codec
                .encode(RELIABLE_DATA, &payload)
                .expect("survivor login datagram encodes")
        };
        survivor_client
            .send_to(&wire, server.local_addr().expect("server address"))
            .expect("survivor datagram sends");
        server.next_tick_ms = 0;
        server.step().expect("the surviving sessions keep stepping");
        assert!(
            server.sessions[survivor_key].gateway.is_logged_in(),
            "survivor session {} did not handle its next datagram",
            survivor_key.connection_id
        );
        let mut answered = false;
        loop {
            match survivor_client.recv_from(&mut buffer) {
                Ok((length, _)) => {
                    let packet = server.sessions[survivor_key]
                        .codec
                        .decode(&buffer[..length])
                        .expect("the survivor's reply decodes");
                    answered |= matches!(packet.opcode, RELIABLE_DATA | RELIABLE_FRAGMENT);
                }
                Err(error)
                    if matches!(
                        error.kind(),
                        io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut
                    ) =>
                {
                    break;
                }
                Err(error) => panic!("survivor reply read fails: {error}"),
            }
        }
        assert!(
            answered,
            "survivor session {} did not answer its next datagram",
            survivor_key.connection_id
        );
    }
    assert_eq!(server.sessions.len(), 2);

    drop(server);
    let _ = fs::remove_dir_all(server_scratch);
}
