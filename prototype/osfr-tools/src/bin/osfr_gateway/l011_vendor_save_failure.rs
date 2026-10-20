//! L011 (Security Model S12/L8): a vendor purchase whose character save fails changes
//! nothing and ends only that session. Test-only; no production code changes.
//!
//! Production path (read in `Session::commit_outcome`, `src/bin/osfr-gateway.rs:1686`):
//! the 66/43 purchase runs on a cloned `Gateway`, then `save` runs at line 1936 BEFORE
//! `self.gateway = staged_gateway` (line 1968) and `*world = staged_world` (line 1969).
//! A save error returns `Err` from `commit_outcome`; `dispatch_message_inner` propagates
//! it (`?` at line 1677), and `Server::step`'s `contain` error arm ends that one session
//! (`step`, line 3747 -> `end_session`). The test writes no test-only hook: the next save
//! targets a path under a file, exactly as `isolation.rs` and T199 break a save.
//!
//! 66/43 request shape: client serializer `00b3ab20`; the gateway test helper in
//! `tests/common/gateway_vendor.rs` encodes the same `MerchantBuyRequestV2` fields.

use super::test_support::{self, Scratch};
use super::*;
use osfr_tools::{
    character_store::CharacterState,
    game_store::Snapshot,
    gameplay::{Gameplay, GameplayState},
    inventory::{InventoryItem, InventoryState},
    npc_interaction::{MerchantBuyRequestV2, NpcAction, NpcInteraction, encode_merchant_buy_66_43},
};

#[path = "../../../tests/common/gateway_packets.rs"]
mod gateway_packets;
use gateway_packets::{interact, select};

/// The shipped representative selected-hub vendor and new stock row used by
/// `tests/common/vendor_purchase.rs`.
const REPRESENTATIVE_VENDOR: u64 = 9_500_000_045;
const REPRESENTATIVE_ITEM: i32 = 76_427;
/// The fixture's seeded pre-login item, sold then bought back around the failed purchase.
const SEEDED_ITEM: i32 = 75_060;
const BUY_TRANSACTION: u32 = 108_901;

/// A content pack with one real vendor, built the way `tests/common/vendor_purchase.rs`
/// builds `w178-vendor-gateway`; only the NPC's zone/position moves to zone 0.
struct VendorFixture {
    scratch: Scratch,
    content: PathBuf,
    reference: PathBuf,
    zone: u32,
    position: [f32; 3],
    option: u32,
    price: u32,
    sell_price: u32,
}

impl VendorFixture {
    fn new(tag: &str) -> Self {
        let scratch = Scratch::new(tag);
        let lab = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../lab");
        let reference = lab.join("references/rust-gateway-reference");
        let source = lab.join("reports/round70-native-20261016T061000Z/content");
        let data = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content");
        let content = scratch.path().join("content");
        fs::create_dir_all(content.join("npcs")).expect("fixture content creates");
        for file in [
            "inventory.json",
            "quests.json",
            "quest-journal.bin",
            "combat.json",
            "combat-template.bin",
            "categories.json",
        ] {
            fs::copy(source.join(file), content.join(file))
                .unwrap_or_else(|error| panic!("fixture {file} copies: {error}"));
        }
        if fs::hard_link(
            source.join("item-catalogue.json"),
            content.join("item-catalogue.json"),
        )
        .is_err()
        {
            fs::copy(
                source.join("item-catalogue.json"),
                content.join("item-catalogue.json"),
            )
            .expect("item catalogue copies");
        }
        for file in ["stores.json", "zones.json"] {
            fs::copy(data.join(file), content.join(file))
                .unwrap_or_else(|error| panic!("fixture {file} copies: {error}"));
        }

        let vendors = read_json(&data.join("vendors.json"));
        let vendor = vendors["vendors"]
            .as_array()
            .expect("vendor records are an array")
            .iter()
            .find(|vendor| vendor["npc_id"].as_u64() == Some(REPRESENTATIVE_VENDOR))
            .expect("the representative vendor exists")
            .clone();
        fs::write(
            content.join("vendors.json"),
            serde_json::to_vec(&json!({"schema_version": 1, "vendors": [vendor]}))
                .expect("fixture vendors serialize"),
        )
        .expect("fixture vendors write");

        let npcs = read_json(&data.join("npcs/blackspore_swamp.json"));
        let mut npc = npcs["records"]
            .as_array()
            .expect("NPC records are an array")
            .iter()
            .find(|npc| npc["npc_id"].as_u64() == Some(REPRESENTATIVE_VENDOR))
            .expect("the representative NPC exists")
            .clone();
        npc["zone_id"] = json!(0);
        npc["zone"] = json!("FabledRealms");
        npc["position"] = json!([0.0, 0.0, 0.0]);
        npc["heading"] = json!(0.0);
        npc["placement"] = json!("legacy_data");
        fs::write(
            content.join("npcs/melninster.json"),
            serde_json::to_vec(&json!({"schema_version": 1, "records": [npc]}))
                .expect("fixture NPC serializes"),
        )
        .expect("fixture NPC writes");
        fs::write(
            content.join("npc_spawns_extra.json"),
            br#"{"schema_version":1,"records":[]}"#,
        )
        .expect("fixture spawn overlay writes");

        let mut service = NpcInteraction::load(&content.join("npcs"), &content.join("stores.json"))
            .expect("fixture NPC content loads");
        let (zone, position) = service
            .location(REPRESENTATIVE_VENDOR)
            .expect("the representative vendor has a location");
        let menu = service
            .interact(REPRESENTATIVE_VENDOR, zone, position, None)
            .expect("the representative vendor opens");
        let option = u32::try_from(
            menu.options
                .iter()
                .find(|option| option.action == NpcAction::Shop)
                .expect("the representative vendor exposes a shop option")
                .id,
        )
        .expect("the shop option id fits the client field");
        let price = store_price(
            &data.join("stores.json"),
            REPRESENTATIVE_ITEM,
            "purchase_price_coins",
        );
        let sell_price = store_price(&data.join("stores.json"), SEEDED_ITEM, "sell_price_coins");
        Self {
            scratch,
            content,
            reference,
            zone,
            position,
            option,
            price,
            sell_price,
        }
    }

    fn character(&self) -> CharacterState {
        let mut character =
            Gateway::reference_state(&self.reference).expect("reference character loads");
        character.zone_id = self.zone;
        character.position = self.position;
        character.rotation = [0.0; 3];
        character
    }

    fn state(&self) -> GameplayState {
        let mut state = Gameplay::load(&self.content, None)
            .expect("fixture gameplay content loads")
            .state();
        state.inventory.items.clear();
        for entry in &mut state.inventory.equipment {
            entry.item_id = 0;
        }
        state.inventory.coins = self.price + 1_000;
        state.inventory.items.push(InventoryItem {
            definition_id: SEEDED_ITEM,
            tint: 0,
            id: 1,
            count: 1,
            consumed_count: 0,
            stored_count: 0,
            last_cast_time: 0,
            rental: None,
            ability_count: 0,
            activate_enabled: false,
        });
        state.inventory.next_instance_id = Some(2);
        state
    }

    fn save(&self, path: &Path, character: &CharacterState, gameplay: &GameplayState) {
        osfr_tools::game_store::save(
            path,
            &Snapshot {
                character: character.clone(),
                gameplay: Some(gameplay.clone()),
            },
        )
        .expect("fixture save writes");
    }
}

fn read_json(path: &Path) -> serde_json::Value {
    serde_json::from_slice(&fs::read(path).expect("content file reads"))
        .unwrap_or_else(|error| panic!("{} parses: {error}", path.display()))
}

/// The authoritative Coin price for `item` from `stores.json` (`purchase_price_coins`
/// or `sell_price_coins`).
fn store_price(stores: &Path, item: i32, field: &str) -> u32 {
    let document = read_json(stores);
    let price = document["items"]
        .as_array()
        .expect("store items are an array")
        .iter()
        .find(|row| row["item_id"].as_i64() == Some(i64::from(item)))
        .and_then(|row| row[field].as_u64())
        .unwrap_or_else(|| panic!("item {item} has {field}"));
    u32::try_from(price).expect("the store price fits a Coin balance")
}

fn vendor_buy() -> Vec<u8> {
    // Client serializer 00b3ab20; the same library encoder is byte-pinned by
    // `vendor_stock_coverage::vendor_buy_wire_matches_the_client_reader_layout`.
    encode_merchant_buy_66_43(&MerchantBuyRequestV2 {
        initial_dwords: [REPRESENTATIVE_ITEM as u32, 0],
        context_dwords: [0; 3],
        context_bytes: Vec::new(),
        trailing_dwords: [1, 0, BUY_TRANSACTION],
    })
    .expect("66/43 request encodes")
}

/// 165/5 body (client 00c75a70 / 00c76dd0): u64 merchant (0 = the open shop) +
/// u32 item instance + u32 quantity.
fn sell(instance: i32) -> Vec<u8> {
    let mut packet = vec![165, 0, 5, 0];
    packet.extend(0u64.to_le_bytes());
    packet.extend(instance.to_le_bytes());
    packet.extend(1u32.to_le_bytes());
    packet
}

/// 165/12 body (client 00c76600): the sale transaction id plus a context dword.
fn buy_back(transaction: u32) -> Vec<u8> {
    let mut packet = vec![165, 0, 12, 0];
    packet.extend(transaction.to_le_bytes());
    packet.extend(0u32.to_le_bytes());
    packet
}

fn count_definition(state: &InventoryState, definition_id: i32) -> u32 {
    state
        .items
        .iter()
        .filter(|item| item.definition_id == definition_id)
        .map(|item| item.count)
        .sum()
}

/// The client's synthetic login body: u16 count, three length-prefixed strings, the
/// account guid after the ticket.
fn login(guid: u64) -> Vec<u8> {
    let mut bytes = 1u16.to_le_bytes().to_vec();
    for (index, text) in ["l011-synthetic-ticket", "1.910.1.530630", ""]
        .iter()
        .enumerate()
    {
        bytes.extend((text.len() as u32).to_le_bytes());
        bytes.extend(text.as_bytes());
        if index == 0 {
            bytes.extend(guid.to_le_bytes());
        }
    }
    bytes
}

fn options(fixture: &VendorFixture, state: PathBuf) -> Options {
    let mut options = test_support::options(state);
    options.reference = fixture.reference.clone();
    options.content = Some(fixture.content.clone());
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
        self.send(options, &select(REPRESENTATIVE_VENDOR, fixture.option), now)
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
    let fixture = VendorFixture::new("l011-failed-save");
    let options = options(&fixture, fixture.scratch.path().join("character.json"));
    let character = fixture.character();
    fixture.save(&options.state, &character, &fixture.state());
    let mut scene = Scene::new(&options, 81);
    scene.enter(&options, 1_000);
    scene.open_shop(&fixture, &options, 1_100);

    // Seed the Coin Shop's buy-back window with a real 165/5 sale.
    scene
        .send(&options, &sell(1), 1_200)
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
        fixture.price + 1_000 + fixture.sell_price,
        "fixture setup: the sale paid its price"
    );
    assert_eq!(count_definition(&before_inventory, SEEDED_ITEM), 0);
    let good_save = fs::read(&options.state).expect("the good save is readable");
    scene.break_next_save(&options.state);

    // Check 1: the purchase transaction fails when its save cannot be written.
    let error = scene
        .send(&options, &vendor_buy(), 1_300)
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
    // pose and the inventory; the session-only Coin Shop buy-back window is not part of
    // that type, so the buy-back probe at the end of this test covers it behaviourally.
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

    // The buy-back window is part of the same staged Coin Shop state: the sale record
    // survives with the same id and price, and the shop still serves its open session.
    scene
        .send(&options, &buy_back(transaction), 1_400)
        .expect("the buy-back handles after the failed purchase");
    let after_buy_back = scene
        .snapshot()
        .gameplay
        .expect("buy-back gameplay exists")
        .inventory;
    assert_eq!(
        after_buy_back.coins,
        before_inventory.coins - fixture.sell_price,
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
    let fixture = VendorFixture::new("l011-control");
    let options = options(&fixture, fixture.scratch.path().join("character.json"));
    let character = fixture.character();
    fixture.save(&options.state, &character, &fixture.state());
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
        .send(&options, &vendor_buy(), 1_200)
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
        before_inventory.coins - fixture.price,
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

/// Assertion 4: the save failure inside `Server::step` ends only that session, and the
/// shared world keeps no ghost row for it.
#[test]
fn a_failed_vendor_save_ends_only_that_session() {
    let fixture = VendorFixture::new("l011-server");
    let (mut server, keys, server_scratch) = super::super::t020_group_routing::server_rig("l011");
    let victim_id = server.sessions[&keys[0]].id;
    let victim_guid = server.sessions[&keys[0]].character_guid;

    // The vendor character reuses the seeded world row's guid.
    let mut character = fixture.character();
    character.guid = victim_guid;
    let gameplay = fixture.state();
    let save_path = fixture.scratch.path().join("character.json");
    fixture.save(&save_path, &character, &gameplay);
    let mut gateway = Gateway::load_gameplay(
        &server.options.reference,
        character,
        &fixture.content,
        Some(gameplay),
    )
    .expect("vendor gateway loads");
    gateway.set_dev_tools(false);
    gateway.set_role(osfr_tools::account_store::AccountRole::Player);
    for (index, message) in [
        login(victim_guid),
        wrap(&[13, 0]),
        wrap(&[10, 0]),
        wrap(&interact(REPRESENTATIVE_VENDOR)),
    ]
    .into_iter()
    .enumerate()
    {
        gateway
            .handle_at(&message, 1_000 + index as u64)
            .expect("vendor handshake handles");
    }
    let opened = gateway
        .handle_at(&wrap(&select(REPRESENTATIVE_VENDOR, fixture.option)), 1_004)
        .expect("the shop selection handles");
    assert!(
        opened
            .replies
            .iter()
            .any(|packet| packet[7..].starts_with(&[66, 0, 42, 0])),
        "the vendor shop did not open on the real gateway"
    );

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
    server
        .sessions
        .get_mut(&key)
        .expect("the victim session is present")
        .state_path = save_path.join("blocked.json");
    let wire = {
        let session = &server.sessions[&key];
        let mut payload = 0u16.to_be_bytes().to_vec();
        payload.extend(wrap(&vendor_buy()));
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
    assert_eq!(
        fs::read(&save_path).expect("the good save is still readable"),
        good_save,
        "the ended session wrote a partial save"
    );
    assert!(
        server.sessions.contains_key(&keys[1]) && server.sessions.contains_key(&keys[2]),
        "another session ended with the victim"
    );
    assert_eq!(
        server.world.active_sessions.len(),
        2,
        "the ended session left a world ghost"
    );
    assert!(
        survivors
            .iter()
            .all(|guid| server.world.active_sessions.contains_key(guid)),
        "the ended session removed a survivor's world row"
    );
    server.next_tick_ms = 0;
    server.step().expect("the surviving sessions keep stepping");
    assert_eq!(server.sessions.len(), 2);

    drop(server);
    let _ = fs::remove_dir_all(server_scratch);
}
