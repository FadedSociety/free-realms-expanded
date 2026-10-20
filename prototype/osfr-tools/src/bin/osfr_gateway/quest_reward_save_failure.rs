//! T258 (Security Model S12/L8): a quest-reward turn-in whose character save fails changes
//! nothing and ends only that session. Test-only; no production code changes.
//!
//! Production path, read in `src/bin/osfr-gateway.rs`: the 49/14 request (client
//! `QuestEndReplyPacket` constructor `00a864c0`, 10 bytes) reaches
//! `gameplay/npc.rs`, which stages `QuestEngine::finish_turnin` (`RewardState` +
//! `rewards::apply_grant`) on the `Gateway`-cloned `Gameplay`. `Session::commit_outcome` then
//! calls `save(&self.state_path, &staged_gateway.snapshot())?` BEFORE installing that gateway and
//! the staged world. A save error returns `Err` from `commit_outcome`, `dispatch_message_inner`
//! propagates it, and `Server::step`'s `contain` error arm calls `end_session` for that one
//! session. The test injects no production hook: it points the next save under an existing file,
//! the same mechanism `isolation.rs` and T199 use.
//!
//! Client field layouts (IDA, this client): 49/14 constructor `00a864c0`; quest-packet base
//! reader `00c7acd0` (u16 family + u32 subtype header); 49/13 reader `00c7b990` reads the
//! 8-byte speaker, then quest id / end text / animation, then the reward bundle consumed by
//! `008e7930`; `ShowQuestEndScreen` `00a95420` resolves the speaker through the client actor
//! table. The 26/8 (`008b6bc0`), 26/10 (`00a36f80`), 26/6 (`009f5890`) and 49/2 (`00a86400`)
//! bodies are the shared `tests/common/gateway_vendor.rs` encoders.
//!
//! Synthetic content: the shared round70 base (`tests/common/round70_content.rs`) plus one
//! giver NPC, one job row and two `talk_to_npc` quests. Each quest pays coins, one real
//! catalogue item (8, a `bowl_basic.adr` stack) and Adventurer (profile 1) job XP.

use super::test_support::{self, Scene, Scratch};
use super::*;
use osfr_tools::{
    character_store::CharacterState,
    gameplay::{Gameplay, GameplayState},
    inventory::InventoryItem,
    quest_engine::QuestStatus,
};

// The shared test kit: this host uses a subset of each module's helpers.
#[allow(dead_code)]
#[path = "../../../tests/common/gateway_vendor.rs"]
mod gateway_vendor;
#[allow(dead_code)]
#[path = "../../../tests/common/round70_content.rs"]
mod round70_content;

use gateway_vendor::{count_definition, interact, login, select};

/// The synthetic giver's content key. `NpcVisibility` maps a `legacy_data` regional row's actor
/// GUID to its own content key, so the client-facing GUID and the quest NPC key are equal.
const GIVER: u64 = 700;
const CONTROL_QUEST: u32 = 7_258_001;
const SECOND_QUEST: u32 = 7_258_002;
/// `bowl_basic.adr` in the copied inventory catalogue: a real definition with `max_stack` 1, so
/// the grant always adds one new stack.
const REWARD_ITEM: i32 = 8;
const CONTROL_COINS: u32 = 41;
const CONTROL_XP: u32 = 25;
const SECOND_COINS: u32 = 57;
const SECOND_XP: u32 = 35;
const STARTING_COINS: u32 = 1_000;
const PROFILE: i32 = 1;
/// `build_options` lists Talk first with id 1 (`npc_interaction.rs`).
const TALK_OPTION: u32 = 1;
/// Quest dialog row that opens the offer or releases the held end screen (`009f5890`).
const QUEST_CHOICE: i32 = 301;
/// Item the full-bag case stacks to the bag limit; also real and already in the inventory
/// catalogue.
const FILLER_ITEM: i32 = 75_060;
/// (R) The pre-existing production bag floor for a turn-in: `gameplay/npc.rs` computes
/// `items.len().max(100)`, not content data. If the lead moves that cap to content, this
/// fixture must follow it or the bag is no longer full.
const BAG_SLOTS: usize = 100;

/// The shared round70 content plus one giver NPC, one job row and two quests.
struct Fixture {
    scratch: Scratch,
    content: PathBuf,
    reference: PathBuf,
}

impl Fixture {
    /// Run only in the gateway binary's own test target, and only with the private `lab/`
    /// fixtures: an embedded copy and a clean checkout skip instead of failing in setup.
    fn ready() -> bool {
        if !test_support::bin_test_target("t258") {
            return false;
        }
        let source = round70_content::source();
        let present = [
            "inventory.json",
            "quests.json",
            "profiles.json",
            "item-catalogue.json",
        ]
        .iter()
        .all(|file| source.join(file).is_file())
            && super::super::t020_group_routing::fixtures_present();
        if !present {
            eprintln!("t258: private lab fixture absent; skipping");
        }
        present
    }

    fn new(tag: &str) -> Self {
        let scratch = Scratch::new(tag);
        let content = round70_content::build(scratch.path(), "content", &["profiles.json"]);
        fs::create_dir_all(content.join("quests")).expect("fixture quest directory creates");

        let provenance = json!([{
            "kind": "reconstructed",
            "evidence": "(R) T258 synthetic save-failure fixture: two talk-to-npc quests, local ids, 41/57 coins, one item 8 and 25/35 Adventurer XP. Values are test policy, not recovered retail rewards."
        }]);
        write_json(
            content.join("zones.json"),
            json!({"schema_version": 1, "records": [{"id": 0, "name": "FabledRealms"}]}),
        );
        write_json(
            content.join("npc_spawns_extra.json"),
            json!({"schema_version": 1, "records": []}),
        );
        write_json(
            content.join("vendors.json"),
            json!({"schema_version": 1, "vendors": []}),
        );
        write_json(
            content.join("stores.json"),
            json!({
                "schema_version": 1,
                "marketplace": {
                    "enabled_on_handshake": false,
                    "orders_enabled": false,
                    "payment_sources_enabled": false,
                    "not_available_message": "disabled"
                },
                "items": [{
                    "item_id": REWARD_ITEM,
                    "purchase_price_coins": 15,
                    "member_discount_percent": null,
                    "member_price_coins": null,
                    "sell_price_coins": 7,
                    "no_sale": false
                }]
            }),
        );
        write_json(
            content.join("npcs/quest_giver_258.json"),
            json!({"schema_version": 1, "records": [{
                "npc_id": GIVER,
                "name": "T258 Giver",
                "zone_id": 0,
                "position": [0.0, 0.0, 0.0],
                "placement": "legacy_data",
                "model_id": 1,
                "roles": ["quest_giver"],
                "quest_ids_given": [CONTROL_QUEST, SECOND_QUEST]
            }]}),
        );
        write_json(
            content.join("quests/synthetic.json"),
            json!({"schema_version": 1, "records": [
                quest(CONTROL_QUEST, CONTROL_COINS, CONTROL_XP, &provenance),
                quest(SECOND_QUEST, SECOND_COINS, SECOND_XP, &provenance)
            ]}),
        );
        write_json(
            content.join("jobs.json"),
            json!({"schema_version": 1, "records": [{
                "id": "synthetic",
                "name": "Synthetic job",
                "profile_id": PROFILE,
                "category": "freestyle",
                "max_level": 20,
                "members_only": false,
                "trainer_npc_ids": [],
                "quest_ids": [CONTROL_QUEST, SECOND_QUEST],
                "abilities": [],
                "source": provenance
            }]}),
        );
        Self {
            scratch,
            content,
            reference: round70_content::reference(),
        }
    }

    fn character(&self) -> CharacterState {
        let mut character =
            Gateway::reference_state(&self.reference).expect("reference character loads");
        character.zone_id = 0;
        character.position = [0.0, 0.0, 0.0];
        character.rotation = [0.0; 3];
        character
    }

    /// Empty bag, known coin balance and the profile content's default Adventurer state.
    fn state(&self) -> GameplayState {
        let mut state = Gameplay::load(&self.content, None)
            .expect("fixture gameplay content loads")
            .state();
        state.inventory.items.clear();
        for entry in &mut state.inventory.equipment {
            entry.item_id = 0;
        }
        state.inventory.coins = STARTING_COINS;
        state.inventory.next_instance_id = Some(1);
        state
    }

    /// The bag `gameplay/npc.rs` refuses a reward into: one occupied filler stack per slot,
    /// so adding the reward item makes one more than the capacity.
    fn full_bag_state(&self) -> GameplayState {
        let mut state = self.state();
        for offset in 0..BAG_SLOTS {
            state.inventory.items.push(InventoryItem {
                definition_id: FILLER_ITEM,
                tint: 0,
                id: 100_000 + offset as i32,
                count: 1,
                consumed_count: 0,
                stored_count: 0,
                last_cast_time: 0,
                rental: None,
                ability_count: 0,
                activate_enabled: false,
            });
        }
        state.inventory.next_instance_id = Some(100_000 + BAG_SLOTS as i64);
        state
    }

    fn options(&self, state: PathBuf) -> Options {
        let mut options = test_support::options(state);
        options.reference = self.reference.clone();
        options.content = Some(self.content.clone());
        options.dev_tools = false;
        options
    }
}

fn quest(id: u32, coins: u32, xp: u32, provenance: &serde_json::Value) -> serde_json::Value {
    json!({
        "id": id,
        "title_string_id": 1,
        "giver_npc_id": GIVER,
        "prerequisite_quest_ids": [],
        "objectives": [{
            "kind": "talk_to_npc",
            "id": 1,
            "npc_id": GIVER,
            "source": provenance
        }],
        "rewards": [{
            "coins": coins,
            "items": [{
                "definition_id": REWARD_ITEM,
                "count": 1,
                "source": provenance
            }],
            "job_xp": {PROFILE.to_string(): xp},
            "source": provenance
        }],
        "source": provenance,
        "enabled": true
    })
}

fn write_json(path: PathBuf, value: serde_json::Value) {
    fs::write(
        &path,
        serde_json::to_vec(&value).expect("fixture JSON serializes"),
    )
    .unwrap_or_else(|error| panic!("fixture {} writes: {error}", path.display()));
}

/// 26/6 (client constructor `009f5890`): one i32 response row.
fn dialogue_choice(row: i32) -> Vec<u8> {
    let mut packet = vec![26, 0, 6, 0];
    packet.extend(row.to_le_bytes());
    packet
}

/// 49/2 (client constructor `00a86400`): six-byte header, u32 quest id, u8 accept.
fn quest_reply(quest_id: u32, accept: bool) -> Vec<u8> {
    let mut packet = vec![49, 0, 2, 0, 0, 0];
    packet.extend(quest_id.to_le_bytes());
    packet.push(u8::from(accept));
    packet
}

/// 49/14 (client constructor `00a864c0`): six-byte header plus u32 quest id.
fn quest_end_reply(quest_id: u32) -> Vec<u8> {
    let mut packet = vec![49, 0, 14, 0, 0, 0];
    packet.extend(quest_id.to_le_bytes());
    packet
}

/// Ready and Loaded, the two messages every world session sends after login. `Scene::new`
/// already sent the login; the entry replies are drained so each case reads only its own.
fn enter(scene: &mut Scene, options: &Options, now: &mut u64) {
    for message in [[13, 0], [10, 0]] {
        scene
            .send(options, &message, *now)
            .expect("world entry commits");
        *now += 10;
    }
    scene.drain(*now);
}

/// The session's drained replies keep the client application wrapper (`005`, subtype 1,
/// payload length); every family check reads the inner message.
fn inner(packet: &[u8]) -> &[u8] {
    if packet.len() >= 7 && packet[..3] == [5, 0, 1] {
        &packet[7..]
    } else {
        packet
    }
}

fn has_quest_packet(packets: &[Vec<u8>], subtype: u16, quest_id: u32) -> bool {
    packets.iter().any(|packet| {
        let packet = inner(packet);
        packet.len() >= 10
            && packet[..4] == [49, 0, subtype as u8, 0]
            && u32::from_le_bytes(packet[6..10].try_into().unwrap()) == quest_id
    })
}

fn has_family(packets: &[Vec<u8>], family: u16, subtype: u16) -> bool {
    packets.iter().any(|packet| {
        let packet = inner(packet);
        packet.len() >= 4
            && packet[..2] == family.to_le_bytes()
            && packet[2..4] == subtype.to_le_bytes()
    })
}

fn coins(state: &GameplayState) -> u32 {
    state.inventory.coins
}

fn count_items(state: &GameplayState) -> usize {
    state.inventory.items.len()
}

fn profile_xp(state: &GameplayState) -> u32 {
    state
        .profiles
        .as_ref()
        .expect("the fixture has profile content")
        .experience
        .get(&PROFILE)
        .copied()
        .unwrap_or(0)
}

fn quest_status(state: &GameplayState, quest_id: u32) -> QuestStatus {
    state
        .npc_quests
        .as_ref()
        .expect("the session has NPC quest state")
        .quests
        .get(&quest_id)
        .unwrap_or_else(|| panic!("quest {quest_id} has no progress"))
        .status
        .clone()
}

fn pending_turnin(state: &GameplayState) -> Option<u32> {
    state
        .npc_quests
        .as_ref()
        .expect("the session has NPC quest state")
        .pending_turnin
}

fn gameplay(scene: &Scene) -> GameplayState {
    scene
        .snapshot()
        .gameplay
        .expect("the session has gameplay state")
}

/// Offer and accept one synthetic quest through the real 26/8 -> 26/10 -> 26/6 -> 49/2
/// interaction path.
fn accept_quest(scene: &mut Scene, options: &Options, quest_id: u32, now: &mut u64) {
    *now += 10;
    let menu = scene.exchange(options, &interact(GIVER), *now);
    assert!(
        has_family(&menu, 26, 9),
        "the giver did not return its 26/9 menu: {menu:?}"
    );
    *now += 10;
    let dialogue = scene.exchange(options, &select(GIVER, TALK_OPTION), *now);
    assert!(
        has_family(&dialogue, 26, 3),
        "Talk did not open the 26/3 dialogue: {dialogue:?}"
    );
    *now += 10;
    let offered = scene.exchange(options, &dialogue_choice(QUEST_CHOICE), *now);
    assert!(
        has_quest_packet(&offered, 1, quest_id),
        "quest {quest_id} was not offered: {offered:?}"
    );
    *now += 10;
    let accepted = scene.exchange(options, &quest_reply(quest_id, true), *now);
    assert!(
        has_quest_packet(&accepted, 3, quest_id),
        "quest {quest_id} did not send QuestAdd: {accepted:?}"
    );
    assert_eq!(
        quest_status(&gameplay(scene), quest_id),
        QuestStatus::Active
    );
}

/// Complete the quest's only Talk objective at the giver and release the held 49/13 end screen
/// with the client's 26/6 row.
fn complete_quest(scene: &mut Scene, options: &Options, quest_id: u32, now: &mut u64) {
    *now += 10;
    let menu = scene.exchange(options, &interact(GIVER), *now);
    assert!(
        has_family(&menu, 26, 9),
        "the giver did not return its 26/9 menu: {menu:?}"
    );
    *now += 10;
    let dialogue = scene.exchange(options, &select(GIVER, TALK_OPTION), *now);
    assert!(
        has_family(&dialogue, 26, 3),
        "the turn-in Talk did not open the 26/3 dialogue: {dialogue:?}"
    );
    let state = gameplay(scene);
    assert_eq!(pending_turnin(&state), Some(quest_id));
    *now += 10;
    let released = scene.exchange(options, &dialogue_choice(QUEST_CHOICE), *now);
    let end = released
        .iter()
        .find(|packet| {
            let packet = inner(packet);
            packet.len() >= 18
                && packet[..4] == [49, 0, 13, 0]
                && u32::from_le_bytes(packet[14..18].try_into().unwrap()) == quest_id
        })
        .unwrap_or_else(|| panic!("quest {quest_id} did not release its 49/13: {released:?}"));
    let end = inner(end);
    assert_eq!(
        u64::from_le_bytes(end[6..14].try_into().unwrap()),
        GIVER,
        "the end screen must name the clicked runtime actor"
    );
}

#[test]
fn a_working_turn_in_pays_coins_items_and_xp_once() {
    if !Fixture::ready() {
        return;
    }
    let fixture = Fixture::new("t258-control");
    let options = fixture.options(fixture.scratch.state());
    test_support::save_character(&options.state, &fixture.character(), &fixture.state());
    let mut scene = Scene::new(&options, 91);
    let mut now = 1_000;
    enter(&mut scene, &options, &mut now);
    let before = gameplay(&scene);

    accept_quest(&mut scene, &options, CONTROL_QUEST, &mut now);
    complete_quest(&mut scene, &options, CONTROL_QUEST, &mut now);
    let pending_save = fs::read(&options.state).expect("the pending save is readable");
    now += 10;
    scene
        .send(&options, &quest_end_reply(CONTROL_QUEST), now)
        .expect("a turn-in with a working save commits");

    let after = gameplay(&scene);
    assert_eq!(
        coins(&after),
        coins(&before) + CONTROL_COINS,
        "the turn-in did not pay its configured coins"
    );
    assert_eq!(
        count_definition(&after.inventory, REWARD_ITEM),
        count_definition(&before.inventory, REWARD_ITEM) + 1,
        "the turn-in did not grant its configured item"
    );
    assert_eq!(
        profile_xp(&after),
        profile_xp(&before) + CONTROL_XP,
        "the turn-in did not pay its configured job XP"
    );
    assert_eq!(quest_status(&after, CONTROL_QUEST), QuestStatus::Completed);
    assert_eq!(pending_turnin(&after), None, "the turn-in is still pending");
    let paid_save = fs::read(&options.state).expect("the committed save is readable");
    assert_ne!(
        paid_save, pending_save,
        "the control turn-in did not write the character save"
    );

    // A replayed 49/14 after the committed turn-in pays nothing and saves nothing.
    let completed = scene.snapshot();
    now += 10;
    scene
        .send(&options, &quest_end_reply(CONTROL_QUEST), now)
        .expect("a completed turn-in replay is refused without an error");
    assert_eq!(
        scene.snapshot(),
        completed,
        "a replayed turn-in paid a second reward"
    );
    assert_eq!(
        fs::read(&options.state).expect("the save is readable"),
        paid_save,
        "a replayed turn-in wrote the character save again"
    );
}

#[test]
fn a_failed_save_leaves_the_character_and_save_unchanged() {
    if !Fixture::ready() {
        return;
    }
    let fixture = Fixture::new("t258-failed-save");
    let options = fixture.options(fixture.scratch.state());
    test_support::save_character(&options.state, &fixture.character(), &fixture.state());
    let mut scene = Scene::new(&options, 92);
    let mut now = 1_000;
    enter(&mut scene, &options, &mut now);

    // A good save first: the control quest is paid and committed.
    accept_quest(&mut scene, &options, CONTROL_QUEST, &mut now);
    complete_quest(&mut scene, &options, CONTROL_QUEST, &mut now);
    now += 10;
    scene
        .send(&options, &quest_end_reply(CONTROL_QUEST), now)
        .expect("the control turn-in commits");
    assert_eq!(
        quest_status(&gameplay(&scene), CONTROL_QUEST),
        QuestStatus::Completed
    );

    // A second quest is ready, then only its save fails.
    accept_quest(&mut scene, &options, SECOND_QUEST, &mut now);
    complete_quest(&mut scene, &options, SECOND_QUEST, &mut now);
    let before = scene.snapshot();
    let before_state = before
        .gameplay
        .as_ref()
        .expect("pre-turn-in gameplay exists")
        .clone();
    let good_save = fs::read(&options.state).expect("the good save is readable");
    scene.break_next_save(&options.state);
    now += 10;
    let error = scene
        .send(&options, &quest_end_reply(SECOND_QUEST), now)
        .expect_err("a turn-in whose save fails must not commit");
    assert!(
        error.contains("os error"),
        "the transaction must fail in the save, not earlier: {error}"
    );
    let failed_replies = scene.drain(now);
    assert!(
        failed_replies.is_empty(),
        "a turn-in whose save failed still queued a client success reply: {failed_replies:?}"
    );
    scene.repair_save(&options.state);

    // Rollback covers every staged half: coins, item instances, profile XP and quest state.
    assert_eq!(
        scene.snapshot(),
        before,
        "a failed save installed part of the turn-in"
    );
    let after_state = gameplay(&scene);
    assert_eq!(
        coins(&after_state),
        coins(&before_state),
        "a failed turn-in spent or paid coins"
    );
    assert_eq!(
        count_definition(&after_state.inventory, REWARD_ITEM),
        count_definition(&before_state.inventory, REWARD_ITEM),
        "a failed turn-in granted its item"
    );
    assert_eq!(
        profile_xp(&after_state),
        profile_xp(&before_state),
        "a failed turn-in paid job XP"
    );
    assert_eq!(
        quest_status(&after_state, SECOND_QUEST),
        QuestStatus::Active,
        "a failed turn-in completed the quest"
    );
    assert_eq!(
        pending_turnin(&after_state),
        Some(SECOND_QUEST),
        "a failed turn-in consumed the pending end screen"
    );
    assert_eq!(
        quest_status(&after_state, CONTROL_QUEST),
        QuestStatus::Completed,
        "the failed save rewound the earlier committed quest"
    );
    assert_eq!(
        fs::read(&options.state).expect("the good save is still readable"),
        good_save,
        "a failed turn-in wrote the character save"
    );
}

/// The fixture's gateway for one seeded character, loaded through the production loader
/// before any handshake.
fn fixture_gateway(fixture: &Fixture, guid: u64) -> Gateway {
    let mut character = fixture.character();
    character.guid = guid;
    let mut gateway = Gateway::load_gameplay(
        &fixture.reference,
        character,
        &fixture.content,
        Some(fixture.state()),
    )
    .expect("the fixture gateway loads");
    gateway.set_dev_tools(false);
    gateway.set_role(osfr_tools::account_store::AccountRole::Player);
    gateway
}

/// Drive one session to the held 49/13 end screen through `Session::dispatch_message`: the
/// login/Ready/Loaded handshake that registers its membership row, then the offer/accept and
/// the Talk turn-in release.
fn stage_pending_turnin(server: &mut Server, session: &mut Session, guid: u64, now: &mut u64) {
    for message in [login(guid), wrap(&[13, 0]), wrap(&[10, 0])] {
        session
            .dispatch_message(
                &message,
                &server.options,
                *now,
                &mut server.world,
                &mut server.membership,
                &mut server.sessions,
                None,
            )
            .expect("the handshake commits");
        *now += 10;
    }
    assert!(
        server.membership.player_location(guid).is_some(),
        "the committed handshake must register the membership row this case removes"
    );
    for packet in [
        interact(GIVER),
        select(GIVER, TALK_OPTION),
        dialogue_choice(QUEST_CHOICE),
        quest_reply(CONTROL_QUEST, true),
        interact(GIVER),
        select(GIVER, TALK_OPTION),
        dialogue_choice(QUEST_CHOICE),
    ] {
        session
            .dispatch_message(
                &wrap(&packet),
                &server.options,
                *now,
                &mut server.world,
                &mut server.membership,
                &mut server.sessions,
                None,
            )
            .expect("the quest interaction commits");
        *now += 10;
    }
    let state = session
        .gateway
        .snapshot()
        .gameplay
        .expect("the staged session has gameplay state");
    assert_eq!(
        pending_turnin(&state),
        Some(CONTROL_QUEST),
        "the staged session must hold the turn-in end screen"
    );
}

/// The full server path: a real UDP 49/14 whose save fails ends A through `contain`, while B
/// stays in the world and later turns in its own quest with a working save.
#[test]
fn a_failed_save_ends_only_that_session() {
    if !Fixture::ready() {
        return;
    }
    let fixture = Fixture::new("t258-server");
    let (mut server, keys, server_scratch) = super::super::t020_group_routing::server_rig("t258");
    let a_guid = server.sessions[&keys[0]].character_guid;
    let b_guid = server.sessions[&keys[1]].character_guid;
    let a_save = fixture.scratch.path().join("a-character.json");
    let b_save = fixture.scratch.path().join("b-character.json");
    let mut now = 1_000;

    // Every survivor gets a real socket: the ending session's leave fanout must not hit a
    // closed port, or Windows reports the ICMP reset on the server socket's next receive.
    let b_socket = UdpSocket::bind("127.0.0.1:0").expect("B socket binds");
    b_socket
        .set_read_timeout(Some(Duration::from_millis(100)))
        .expect("B read timeout sets");
    let mut bystander = server
        .sessions
        .remove(&keys[1])
        .expect("the bystander session exists");
    bystander.peer = b_socket.local_addr().expect("B address");
    bystander.gateway = fixture_gateway(&fixture, b_guid);
    bystander.state_path = b_save.clone();
    bystander.character_guid = b_guid;
    let b_key = SessionKey {
        peer: bystander.peer,
        connection_id: bystander.request.id,
    };
    server.world.session_keys.insert(bystander.id, b_key);
    stage_pending_turnin(&mut server, &mut bystander, b_guid, &mut now);
    let b_good_save = fs::read(&b_save).expect("B's good save is readable");
    server.sessions.insert(b_key, bystander);

    let c_socket = UdpSocket::bind("127.0.0.1:0").expect("C socket binds");
    c_socket
        .set_read_timeout(Some(Duration::from_millis(100)))
        .expect("C read timeout sets");
    let mut survivor = server
        .sessions
        .remove(&keys[2])
        .expect("the second survivor exists");
    survivor.peer = c_socket.local_addr().expect("C address");
    let c_key = SessionKey {
        peer: survivor.peer,
        connection_id: survivor.request.id,
    };
    server.world.session_keys.insert(survivor.id, c_key);
    server.sessions.insert(c_key, survivor);

    // A real client socket at the victim session's peer address; the 49/14 arrives over UDP.
    let client = UdpSocket::bind("127.0.0.1:0").expect("client socket binds");
    let mut victim = server
        .sessions
        .remove(&keys[0])
        .expect("the seeded victim session exists");
    victim.peer = client.local_addr().expect("client address");
    victim.gateway = fixture_gateway(&fixture, a_guid);
    victim.state_path = a_save.clone();
    victim.character_guid = a_guid;
    let a_key = SessionKey {
        peer: victim.peer,
        connection_id: victim.request.id,
    };
    server.world.session_keys.insert(victim.id, a_key);
    stage_pending_turnin(&mut server, &mut victim, a_guid, &mut now);
    let a_good_save = fs::read(&a_save).expect("A's good save is readable");
    // Break only the next save: the good save file is a component of the target path.
    victim.state_path = a_save.join("blocked.json");
    let victim_id = victim.id;
    server.sessions.insert(a_key, victim);

    let survivors: Vec<u64> = server
        .world
        .active_sessions
        .keys()
        .copied()
        .filter(|guid| *guid != a_guid)
        .collect();
    assert_eq!(survivors.len(), 2, "the seeded world has two survivors");

    let wire = {
        let session = &server.sessions[&a_key];
        let mut payload = 0u16.to_be_bytes().to_vec();
        payload.extend(wrap(&quest_end_reply(CONTROL_QUEST)));
        session
            .codec
            .encode(RELIABLE_DATA, &payload)
            .expect("turn-in datagram encodes")
    };
    client
        .send_to(&wire, server.local_addr().expect("server address"))
        .expect("turn-in datagram sends");
    server
        .step()
        .expect("one failed session must not stop the server");

    assert!(
        !server.sessions.contains_key(&a_key),
        "the save failure did not end the victim session"
    );
    assert!(
        !server.world.active_sessions.contains_key(&a_guid),
        "the ended session left a world ghost"
    );
    assert!(!server.world.joined.contains(&victim_id));
    assert!(!server.world.players.contains(victim_id));
    assert!(server.world.chat_session(victim_id).is_none());
    assert!(!server.world.session_keys.contains_key(&victim_id));
    assert!(
        server.membership.player_location(a_guid).is_none(),
        "the ended session left a membership row"
    );
    assert_eq!(
        fs::read(&a_save).expect("A's good save is still readable"),
        a_good_save,
        "the ended session wrote a partial save"
    );
    for (guid, key) in survivors.iter().zip([b_key, c_key]) {
        let id = server.world.active_sessions[guid];
        assert_eq!(
            server.world.session_keys.get(&id),
            Some(&key),
            "the ended session changed survivor {guid}'s session key"
        );
        assert!(
            server.sessions.contains_key(&key),
            "another session ended with the victim"
        );
    }
    assert_eq!(
        fs::read(&b_save).expect("B's good save is readable"),
        b_good_save,
        "A's failed save touched B's character file"
    );

    // B is untouched by A's failure and its own turn-in still commits and saves.
    now += 10;
    let before_b = server.sessions[&b_key]
        .gateway
        .snapshot()
        .gameplay
        .expect("B has gameplay state");
    let mut bystander = server
        .sessions
        .remove(&b_key)
        .expect("the bystander session exists");
    bystander
        .dispatch_message(
            &wrap(&quest_end_reply(CONTROL_QUEST)),
            &server.options,
            now,
            &mut server.world,
            &mut server.membership,
            &mut server.sessions,
            None,
        )
        .expect("B's own turn-in keeps working after A's failed save");
    server.sessions.insert(b_key, bystander);
    let after_b = server.sessions[&b_key]
        .gateway
        .snapshot()
        .gameplay
        .expect("B has gameplay state");
    assert_eq!(
        coins(&after_b),
        coins(&before_b) + CONTROL_COINS,
        "B's turn-in did not pay after A's failure"
    );
    assert_eq!(
        count_definition(&after_b.inventory, REWARD_ITEM),
        count_definition(&before_b.inventory, REWARD_ITEM) + 1
    );
    assert_eq!(profile_xp(&after_b), profile_xp(&before_b) + CONTROL_XP);
    assert_eq!(
        quest_status(&after_b, CONTROL_QUEST),
        QuestStatus::Completed
    );
    assert_ne!(
        fs::read(&b_save).expect("B's save is readable"),
        b_good_save,
        "B's committed turn-in did not write B's own save"
    );
    assert!(server.sessions.contains_key(&b_key));
    assert!(server.sessions.contains_key(&c_key));

    // Drain each survivor's leave fanout, then prove the server keeps stepping and ends no
    // other session.
    let mut buffer = [0; 65_535];
    for socket in [&b_socket, &c_socket] {
        while socket.recv_from(&mut buffer).is_ok() {}
    }
    server.next_tick_ms = 0;
    server.step().expect("the surviving sessions keep stepping");
    assert_eq!(server.sessions.len(), 2);

    drop(server);
    let _ = fs::remove_dir_all(server_scratch);
}

#[test]
fn a_failed_save_does_not_consume_the_turn_in() {
    if !Fixture::ready() {
        return;
    }
    let fixture = Fixture::new("t258-reconnect");
    let options = fixture.options(fixture.scratch.state());
    test_support::save_character(&options.state, &fixture.character(), &fixture.state());
    let mut scene = Scene::new(&options, 94);
    let mut now = 1_000;
    enter(&mut scene, &options, &mut now);
    accept_quest(&mut scene, &options, CONTROL_QUEST, &mut now);
    complete_quest(&mut scene, &options, CONTROL_QUEST, &mut now);
    let before = gameplay(&scene);
    let good_save = fs::read(&options.state).expect("the good save is readable");
    scene.break_next_save(&options.state);
    now += 10;
    let error = scene
        .send(&options, &quest_end_reply(CONTROL_QUEST), now)
        .expect_err("a turn-in whose save fails must not commit");
    assert!(
        error.contains("os error"),
        "expected the save error: {error}"
    );
    scene.repair_save(&options.state);

    // A fresh session loads the good save through the production reconnect path.
    let mut reconnected = Scene::new(&options, 95);
    let mut later = now + 100;
    enter(&mut reconnected, &options, &mut later);
    let loaded = gameplay(&reconnected);
    assert_eq!(
        pending_turnin(&loaded),
        Some(CONTROL_QUEST),
        "the failed save consumed the pending turn-in"
    );
    assert_eq!(
        quest_status(&loaded, CONTROL_QUEST),
        QuestStatus::Active,
        "the failed save completed the quest"
    );
    assert_eq!(coins(&loaded), coins(&before), "the failed save paid coins");
    assert_eq!(
        count_definition(&loaded.inventory, REWARD_ITEM),
        count_definition(&before.inventory, REWARD_ITEM)
    );
    assert_eq!(profile_xp(&loaded), profile_xp(&before));

    // The working reconnect pays it exactly once.
    later += 10;
    reconnected
        .send(&options, &quest_end_reply(CONTROL_QUEST), later)
        .expect("the reconnected turn-in commits");
    let paid = gameplay(&reconnected);
    assert_eq!(coins(&paid), coins(&before) + CONTROL_COINS);
    assert_eq!(
        count_definition(&paid.inventory, REWARD_ITEM),
        count_definition(&before.inventory, REWARD_ITEM) + 1
    );
    assert_eq!(profile_xp(&paid), profile_xp(&before) + CONTROL_XP);
    assert_eq!(quest_status(&paid, CONTROL_QUEST), QuestStatus::Completed);
    assert_eq!(pending_turnin(&paid), None);
    let paid_save = fs::read(&options.state).expect("the paid save is readable");
    assert_ne!(
        paid_save, good_save,
        "the reconnected turn-in did not write the character save"
    );

    later += 10;
    reconnected
        .send(&options, &quest_end_reply(CONTROL_QUEST), later)
        .expect("the replayed turn-in is refused without an error");
    assert_eq!(
        reconnected.snapshot().gameplay.expect("gameplay exists"),
        paid,
        "the replay after reconnect paid twice"
    );
    assert_eq!(
        fs::read(&options.state).expect("the save is readable"),
        paid_save
    );
}

#[test]
fn a_replay_after_a_successful_turn_in_pays_nothing_more() {
    if !Fixture::ready() {
        return;
    }
    let fixture = Fixture::new("t258-replay");
    let options = fixture.options(fixture.scratch.state());
    test_support::save_character(&options.state, &fixture.character(), &fixture.state());
    let mut scene = Scene::new(&options, 96);
    let mut now = 1_000;
    enter(&mut scene, &options, &mut now);
    accept_quest(&mut scene, &options, CONTROL_QUEST, &mut now);
    complete_quest(&mut scene, &options, CONTROL_QUEST, &mut now);
    now += 10;
    scene
        .send(&options, &quest_end_reply(CONTROL_QUEST), now)
        .expect("the first turn-in commits");
    let paid = scene.snapshot();
    let paid_save = fs::read(&options.state).expect("the paid save is readable");

    for round in 0..2 {
        now += 10;
        scene
            .send(&options, &quest_end_reply(CONTROL_QUEST), now)
            .expect("a same-session replay is refused without an error");
        assert_eq!(
            scene.snapshot(),
            paid,
            "same-session replay {round} changed the character"
        );
        assert_eq!(
            fs::read(&options.state).expect("the save is readable"),
            paid_save,
            "same-session replay {round} wrote the character save"
        );
    }

    let mut reconnected = Scene::new(&options, 97);
    let mut later = now + 100;
    enter(&mut reconnected, &options, &mut later);
    let restored = reconnected.snapshot();
    let restored_save = fs::read(&options.state).expect("the restored save is readable");
    assert_eq!(
        quest_status(
            restored.gameplay.as_ref().expect("restored gameplay"),
            CONTROL_QUEST
        ),
        QuestStatus::Completed
    );
    later += 10;
    reconnected
        .send(&options, &quest_end_reply(CONTROL_QUEST), later)
        .expect("a replay after reconnect is refused without an error");
    assert_eq!(
        reconnected.snapshot(),
        restored,
        "the replay after reconnect changed the character"
    );
    assert_eq!(
        fs::read(&options.state).expect("the save is readable"),
        restored_save,
        "the replay after reconnect wrote the character save"
    );
}

#[test]
fn a_full_bag_refuses_the_turn_in_and_changes_nothing() {
    if !Fixture::ready() {
        return;
    }
    let fixture = Fixture::new("t258-full-bag");
    let options = fixture.options(fixture.scratch.state());
    let full = fixture.full_bag_state();
    test_support::save_character(&options.state, &fixture.character(), &full);
    let mut scene = Scene::new(&options, 98);
    let mut now = 1_000;
    enter(&mut scene, &options, &mut now);
    accept_quest(&mut scene, &options, CONTROL_QUEST, &mut now);
    complete_quest(&mut scene, &options, CONTROL_QUEST, &mut now);

    let before = scene.snapshot();
    let before_save = fs::read(&options.state).expect("the good save is readable");
    assert_eq!(
        count_items(before.gameplay.as_ref().expect("full-bag gameplay")),
        BAG_SLOTS,
        "fixture setup: the bag is full"
    );
    now += 10;
    scene
        .send(&options, &quest_end_reply(CONTROL_QUEST), now)
        .expect("a full bag is a refusal, not a save failure");
    let refused_replies = scene.drain(now);
    assert!(
        refused_replies.is_empty(),
        "a full-bag refusal must be silent, not a client-visible reply: {refused_replies:?}"
    );

    // The reward never became a quest completion, coin, item, XP or save change.
    assert_eq!(
        scene.snapshot(),
        before,
        "a full-bag refusal changed the character"
    );
    let after = gameplay(&scene);
    assert_eq!(quest_status(&after, CONTROL_QUEST), QuestStatus::Active);
    assert_eq!(pending_turnin(&after), Some(CONTROL_QUEST));
    assert_eq!(count_items(&after), BAG_SLOTS);
    assert_eq!(
        fs::read(&options.state).expect("the good save is readable"),
        before_save,
        "a full-bag refusal wrote the character save"
    );

    // The session is still healthy: the same giver reopens its menu.
    now += 10;
    let menu = scene.exchange(&options, &interact(GIVER), now);
    assert!(
        has_family(&menu, 26, 9),
        "the full-bag refusal ended the NPC session: {menu:?}"
    );
}

/// Return the body of the first `signature` in `source`, from its `{` to the matching `}`.
/// Braces inside the body's string literals matter only when they are unbalanced, and this
/// function is pinned against the real hub text.
fn function_body<'a>(source: &'a str, signature: &str) -> Option<&'a str> {
    let start = source.find(signature)?;
    let open = start + source[start..].find('{')?;
    let mut depth = 0usize;
    for (offset, byte) in source[open..].bytes().enumerate() {
        match byte {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(&source[open..=open + offset]);
                }
            }
            _ => {}
        }
    }
    None
}

/// Gate-only proof: the save under test is `commit_outcome`'s save, which runs inside that
/// function (the slice stops at its closing brace, before the next method) and before the
/// staged gateway is installed. The failure cases drive `Session::dispatch_message`.
#[test]
fn the_save_failure_harness_is_the_real_commit_path() {
    if !test_support::bin_test_target("t258") {
        return;
    }
    let production = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/bin/osfr-gateway.rs"
    ));
    let commit = function_body(production, "fn commit_outcome(")
        .expect("commit_outcome must exist in the gateway hub");
    let save = commit
        .find("save(&self.state_path, &staged_gateway.snapshot())?;")
        .expect("commit_outcome must save the staged character");
    let install = commit
        .find("self.gateway = staged_gateway;")
        .expect("commit_outcome must install the staged gateway");
    assert!(
        save < install,
        "the staged gateway is installed before its save: the transaction is not all-or-nothing"
    );
    assert!(
        production.contains("fn dispatch_message("),
        "the real session entry moved; the failure tests must follow it"
    );
    assert!(
        include_str!("quest_reward_save_failure.rs").contains(".dispatch_message("),
        "the save-failure cases must drive the real session dispatch"
    );
}
