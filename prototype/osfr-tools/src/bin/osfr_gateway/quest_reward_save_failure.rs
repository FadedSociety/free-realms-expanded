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
//! table. 26/6 (`009f5890`), 26/8 (`008b6bc0`), 26/10 (`00a36f80`) and 49/2 (`00a86400`) are
//! the request shapes already pinned by `tests/quest_job_xp.rs`.
//!
//! Synthetic content: the round70 inventory/catalogue/profiles used by the T250 save-failure
//! rig, plus one giver NPC, one job row and two `talk_to_npc` quests. Each quest pays coins, one
//! real catalogue item (8, a `bowl_basic.adr` stack) and Adventurer (profile 1) job XP.

use super::test_support::{self, Scratch};
use super::*;
use osfr_tools::{
    character_store::CharacterState,
    game_store::Snapshot,
    gameplay::{Gameplay, GameplayState},
    inventory::InventoryItem,
    quest_engine::QuestStatus,
};

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
/// Item the full-bag case stacks to the 100-slot bag limit; also real and already in the
/// inventory catalogue.
const FILLER_ITEM: i32 = 75_060;

/// The round70 content package the T250 save-failure rig copies, minus `opening.json` and the
/// campaign files so a zone-0 character is supported, plus `profiles.json` for job XP.
struct Fixture {
    scratch: Scratch,
    content: PathBuf,
    reference: PathBuf,
}

impl Fixture {
    fn new(tag: &str) -> Self {
        let scratch = Scratch::new(tag);
        let lab = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../lab");
        let reference = lab.join("references/rust-gateway-reference");
        let source = lab.join("reports/round70-native-20261016T061000Z/content");
        let content = scratch.path().join("content");
        fs::create_dir_all(content.join("npcs")).expect("fixture NPC directory creates");
        fs::create_dir_all(content.join("quests")).expect("fixture quest directory creates");
        for file in [
            "inventory.json",
            "quests.json",
            "quest-journal.bin",
            "combat.json",
            "combat-template.bin",
            "categories.json",
            "profiles.json",
        ] {
            fs::copy(source.join(file), content.join(file))
                .unwrap_or_else(|error| panic!("fixture {file} copies: {error}"));
        }
        // 69 MB; link rather than copy, exactly like `tests/common/vendor_purchase.rs`.
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
            reference,
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

    /// The 100-slot bag `gameplay/npc.rs` applies to a turn-in: one occupied filler stack per
    /// slot, so adding the reward item makes 101.
    fn full_bag_state(&self) -> GameplayState {
        let mut state = self.state();
        for id in 100_000..100_100 {
            state.inventory.items.push(InventoryItem {
                definition_id: FILLER_ITEM,
                tint: 0,
                id,
                count: 1,
                consumed_count: 0,
                stored_count: 0,
                last_cast_time: 0,
                rental: None,
                ability_count: 0,
                activate_enabled: false,
            });
        }
        state.inventory.next_instance_id = Some(100_100);
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

/// 26/8 (client constructor `008b6bc0`): u64 target plus the trailing boolean byte.
fn interact(target: u64) -> Vec<u8> {
    let mut packet = vec![26, 0, 8, 0];
    packet.extend(target.to_le_bytes());
    packet.push(0);
    packet
}

/// 26/10 (client constructor `00a36f80`): u64 target plus i32 choice.
fn select(target: u64, choice: u32) -> Vec<u8> {
    let mut packet = vec![26, 0, 10, 0];
    packet.extend(target.to_le_bytes());
    packet.extend(choice.to_le_bytes());
    packet
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

/// The synthetic login body every non-account-store session sends: u16 count, three
/// length-prefixed strings, and the account guid after the ticket.
fn login(guid: u64) -> Vec<u8> {
    let mut bytes = 1u16.to_le_bytes().to_vec();
    for (index, text) in ["t258-synthetic-ticket", "1.910.1.530630", ""]
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

/// One session plus the shared containers `Session::dispatch_message` needs. `Server::new` is
/// not used here: these cases prove the transaction and its save, not the teardown.
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
            format!("127.0.0.1:{}", 35_000 + id).parse().unwrap(),
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
        .expect("quest session loads");
        Self {
            session,
            world: SharedWorld::new(),
            membership: World::runtime(),
            sessions: SessionMap::new(),
        }
    }

    fn dispatch(&mut self, options: &Options, inner: &[u8], now: u64) -> Result<(), String> {
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

    fn send(&mut self, options: &Options, inner: &[u8], now: u64) -> Vec<Vec<u8>> {
        self.dispatch(options, inner, now).unwrap_or_else(|error| {
            panic!(
                "packet {:?} was refused: {error}",
                &inner[..inner.len().min(4)]
            )
        });
        self.drain(now)
    }

    fn drain(&mut self, now: u64) -> Vec<Vec<u8>> {
        super::super::f120_fishing_hour_reconnect::drain_replies(&mut self.session, now)
    }

    /// Login, Ready and Loaded: the three messages every world session sends first.
    fn enter(&mut self, options: &Options, now: &mut u64) -> Vec<Vec<u8>> {
        let guid = self.session.gateway.state().guid;
        for message in [login(guid), wrap(&[13, 0]), wrap(&[10, 0])] {
            self.session
                .dispatch_message(
                    &message,
                    options,
                    *now,
                    &mut self.world,
                    &mut self.membership,
                    &mut self.sessions,
                    None,
                )
                .expect("initialization message handles");
            *now += 10;
        }
        self.drain(*now)
    }

    fn snapshot(&self) -> Snapshot {
        self.session.gateway.snapshot()
    }

    fn gameplay(&self) -> GameplayState {
        self.snapshot()
            .gameplay
            .expect("the session has gameplay state")
    }

    /// Break only the next save: the good save file is a component of the target path, so the
    /// save cannot create its temporary file beside it. The good bytes stay put.
    fn break_next_save(&mut self, good_save: &Path) {
        self.session.state_path = good_save.join("blocked.json");
    }

    fn repair_save(&mut self, good_save: &Path) {
        self.session.state_path = good_save.to_owned();
    }
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

fn count_definition(state: &GameplayState, definition_id: i32) -> u32 {
    state
        .inventory
        .items
        .iter()
        .filter(|item| item.definition_id == definition_id)
        .map(|item| item.count)
        .sum()
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

/// Offer and accept one synthetic quest through the real 26/8 -> 26/10 -> 26/6 -> 49/2
/// interaction path.
fn accept_quest(scene: &mut Scene, options: &Options, quest_id: u32, now: &mut u64) {
    *now += 10;
    let menu = scene.send(options, &interact(GIVER), *now);
    assert!(
        has_family(&menu, 26, 9),
        "the giver did not return its 26/9 menu: {menu:?}"
    );
    *now += 10;
    let dialogue = scene.send(options, &select(GIVER, TALK_OPTION), *now);
    assert!(
        has_family(&dialogue, 26, 3),
        "Talk did not open the 26/3 dialogue: {dialogue:?}"
    );
    *now += 10;
    let offered = scene.send(options, &dialogue_choice(QUEST_CHOICE), *now);
    assert!(
        has_quest_packet(&offered, 1, quest_id),
        "quest {quest_id} was not offered: {offered:?}"
    );
    *now += 10;
    let accepted = scene.send(options, &quest_reply(quest_id, true), *now);
    assert!(
        has_quest_packet(&accepted, 3, quest_id),
        "quest {quest_id} did not send QuestAdd: {accepted:?}"
    );
    assert_eq!(
        quest_status(&scene.gameplay(), quest_id),
        QuestStatus::Active
    );
}

/// Complete the quest's only Talk objective at the giver and release the held 49/13 end screen
/// with the client's 26/6 row.
fn complete_quest(scene: &mut Scene, options: &Options, quest_id: u32, now: &mut u64) {
    *now += 10;
    let menu = scene.send(options, &interact(GIVER), *now);
    assert!(
        has_family(&menu, 26, 9),
        "the giver did not return its 26/9 menu: {menu:?}"
    );
    *now += 10;
    let dialogue = scene.send(options, &select(GIVER, TALK_OPTION), *now);
    assert!(
        has_family(&dialogue, 26, 3),
        "the turn-in Talk did not open the 26/3 dialogue: {dialogue:?}"
    );
    let state = scene.gameplay();
    assert_eq!(pending_turnin(&state), Some(quest_id));
    *now += 10;
    let released = scene.send(options, &dialogue_choice(QUEST_CHOICE), *now);
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

fn count_items(state: &GameplayState) -> usize {
    state.inventory.items.len()
}

#[test]
fn a_working_turn_in_pays_coins_items_and_xp_once() {
    let fixture = Fixture::new("t258-control");
    let options = fixture.options(fixture.scratch.path().join("character.json"));
    let character = fixture.character();
    fixture.save(&options.state, &character, &fixture.state());
    let mut scene = Scene::new(&options, 91);
    let mut now = 1_000;
    scene.enter(&options, &mut now);
    let before = scene.gameplay();

    accept_quest(&mut scene, &options, CONTROL_QUEST, &mut now);
    complete_quest(&mut scene, &options, CONTROL_QUEST, &mut now);
    let pending_save = fs::read(&options.state).expect("the pending save is readable");
    now += 10;
    scene
        .dispatch(&options, &quest_end_reply(CONTROL_QUEST), now)
        .expect("a turn-in with a working save commits");

    let after = scene.gameplay();
    assert_eq!(
        coins(&after),
        coins(&before) + CONTROL_COINS,
        "the turn-in did not pay its configured coins"
    );
    assert_eq!(
        count_definition(&after, REWARD_ITEM),
        count_definition(&before, REWARD_ITEM) + 1,
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
        .dispatch(&options, &quest_end_reply(CONTROL_QUEST), now)
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
    let fixture = Fixture::new("t258-failed-save");
    let options = fixture.options(fixture.scratch.path().join("character.json"));
    let character = fixture.character();
    fixture.save(&options.state, &character, &fixture.state());
    let mut scene = Scene::new(&options, 92);
    let mut now = 1_000;
    scene.enter(&options, &mut now);

    // A good save first: the control quest is paid and committed.
    accept_quest(&mut scene, &options, CONTROL_QUEST, &mut now);
    complete_quest(&mut scene, &options, CONTROL_QUEST, &mut now);
    now += 10;
    scene
        .dispatch(&options, &quest_end_reply(CONTROL_QUEST), now)
        .expect("the control turn-in commits");
    assert_eq!(
        quest_status(&scene.gameplay(), CONTROL_QUEST),
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
        .dispatch(&options, &quest_end_reply(SECOND_QUEST), now)
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
    let after_state = scene.gameplay();
    assert_eq!(
        coins(&after_state),
        coins(&before_state),
        "a failed turn-in spent or paid coins"
    );
    assert_eq!(
        count_definition(&after_state, REWARD_ITEM),
        count_definition(&before_state, REWARD_ITEM),
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

/// The full server path: a real UDP 49/14 whose save fails ends A through `contain`, while B
/// stays in the world and later turns in its own quest with a working save.
#[test]
fn a_failed_save_ends_only_that_session() {
    let fixture = Fixture::new("t258-server");
    let (mut server, keys, server_scratch) = super::super::t020_group_routing::server_rig("t258");
    let a_guid = server.sessions[&keys[0]].character_guid;
    let b_guid = server.sessions[&keys[1]].character_guid;

    let a_save = fixture.scratch.path().join("a-character.json");
    let b_save = fixture.scratch.path().join("b-character.json");
    let a_gateway = pending_turnin_gateway(&fixture, a_guid, &a_save);
    let b_gateway = pending_turnin_gateway(&fixture, b_guid, &b_save);
    let a_good_save = fs::read(&a_save).expect("A's good save is readable");
    let b_good_save = fs::read(&b_save).expect("B's good save is readable");

    // A real client socket at the victim session's peer address; the 49/14 arrives over UDP.
    let client = UdpSocket::bind("127.0.0.1:0").expect("client socket binds");
    let mut victim = server
        .sessions
        .remove(&keys[0])
        .expect("the seeded victim session exists");
    victim.peer = client.local_addr().expect("client address");
    victim.gateway = a_gateway;
    victim.state_path = a_save.clone();
    victim.character_guid = a_guid;
    let a_key = SessionKey {
        peer: victim.peer,
        connection_id: victim.request.id,
    };
    server.world.session_keys.insert(victim.id, a_key);
    server.sessions.insert(a_key, victim);
    server
        .sessions
        .get_mut(&a_key)
        .expect("the victim session is present")
        .state_path = a_save.join("blocked.json");

    // B keeps its seeded session identity and joins with its own pending turn-in.
    let bystander = server
        .sessions
        .get_mut(&keys[1])
        .expect("the bystander session exists");
    assert_eq!(bystander.character_guid, b_guid);
    bystander.gateway = b_gateway;
    bystander.state_path = b_save.clone();

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
    let victim_id = server.sessions[&a_key].id;
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
    assert_eq!(
        fs::read(&a_save).expect("A's good save is still readable"),
        a_good_save,
        "the ended session wrote a partial save"
    );
    assert!(
        survivors
            .iter()
            .all(|guid| server.world.active_sessions.contains_key(guid)),
        "the ended session removed a survivor's world row"
    );
    assert_eq!(
        fs::read(&b_save).expect("B's good save is readable"),
        b_good_save,
        "A's failed save touched B's character file"
    );

    // B is untouched by A's failure and its own turn-in still commits and saves.
    let before_b = server.sessions[&keys[1]]
        .gateway
        .snapshot()
        .gameplay
        .expect("B has gameplay state");
    let mut bystander = server
        .sessions
        .remove(&keys[1])
        .expect("the bystander session exists");
    bystander
        .dispatch_message(
            &wrap(&quest_end_reply(CONTROL_QUEST)),
            &server.options,
            2_000,
            &mut server.world,
            &mut server.membership,
            &mut server.sessions,
            None,
        )
        .expect("B's own turn-in keeps working after A's failed save");
    server.sessions.insert(keys[1], bystander);
    let after_b = server.sessions[&keys[1]]
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
        count_definition(&after_b, REWARD_ITEM),
        count_definition(&before_b, REWARD_ITEM) + 1
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
    assert!(server.sessions.contains_key(&keys[1]));
    assert!(server.sessions.contains_key(&keys[2]));
    server.next_tick_ms = 0;
    server.step().expect("the surviving sessions keep stepping");
    assert_eq!(server.sessions.len(), 2);

    drop(server);
    let _ = fs::remove_dir_all(server_scratch);
}

/// A `Gateway` driven into the pending-turn-in state through the same packet bodies, saved so a
/// session can load it; used only for the two-session server case.
fn pending_turnin_gateway(fixture: &Fixture, guid: u64, save: &Path) -> Gateway {
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
    let mut now = 1_000;
    for message in [login(guid), wrap(&[13, 0]), wrap(&[10, 0])] {
        gateway
            .handle_at(&message, now)
            .expect("initialization message handles");
        now += 10;
    }
    for packet in [
        interact(GIVER),
        select(GIVER, TALK_OPTION),
        dialogue_choice(QUEST_CHOICE),
        quest_reply(CONTROL_QUEST, true),
    ] {
        gateway
            .handle_at(&wrap(&packet), now)
            .expect("quest acceptance handles");
        now += 10;
    }
    for packet in [
        interact(GIVER),
        select(GIVER, TALK_OPTION),
        dialogue_choice(QUEST_CHOICE),
    ] {
        gateway
            .handle_at(&wrap(&packet), now)
            .expect("quest completion handles");
        now += 10;
    }
    let state = gateway
        .snapshot()
        .gameplay
        .expect("the pending gateway has gameplay state");
    assert_eq!(pending_turnin(&state), Some(CONTROL_QUEST));
    osfr_tools::game_store::save(save, &gateway.snapshot()).expect("the good save writes");
    gateway
}

#[test]
fn a_failed_save_does_not_consume_the_turn_in() {
    let fixture = Fixture::new("t258-reconnect");
    let options = fixture.options(fixture.scratch.path().join("character.json"));
    let character = fixture.character();
    fixture.save(&options.state, &character, &fixture.state());
    let mut scene = Scene::new(&options, 94);
    let mut now = 1_000;
    scene.enter(&options, &mut now);
    accept_quest(&mut scene, &options, CONTROL_QUEST, &mut now);
    complete_quest(&mut scene, &options, CONTROL_QUEST, &mut now);
    let before = scene.gameplay();
    let good_save = fs::read(&options.state).expect("the good save is readable");
    scene.break_next_save(&options.state);
    now += 10;
    let error = scene
        .dispatch(&options, &quest_end_reply(CONTROL_QUEST), now)
        .expect_err("a turn-in whose save fails must not commit");
    assert!(
        error.contains("os error"),
        "expected the save error: {error}"
    );
    scene.repair_save(&options.state);

    // A fresh session loads the good save through the production reconnect path.
    let mut reconnected = Scene::new(&options, 95);
    let mut later = now + 100;
    reconnected.enter(&options, &mut later);
    let loaded = reconnected.gameplay();
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
        count_definition(&loaded, REWARD_ITEM),
        count_definition(&before, REWARD_ITEM)
    );
    assert_eq!(profile_xp(&loaded), profile_xp(&before));

    // The working reconnect pays it exactly once.
    later += 10;
    reconnected
        .dispatch(&options, &quest_end_reply(CONTROL_QUEST), later)
        .expect("the reconnected turn-in commits");
    let paid = reconnected.gameplay();
    assert_eq!(coins(&paid), coins(&before) + CONTROL_COINS);
    assert_eq!(
        count_definition(&paid, REWARD_ITEM),
        count_definition(&before, REWARD_ITEM) + 1
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
        .dispatch(&options, &quest_end_reply(CONTROL_QUEST), later)
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
    let fixture = Fixture::new("t258-replay");
    let options = fixture.options(fixture.scratch.path().join("character.json"));
    let character = fixture.character();
    fixture.save(&options.state, &character, &fixture.state());
    let mut scene = Scene::new(&options, 96);
    let mut now = 1_000;
    scene.enter(&options, &mut now);
    accept_quest(&mut scene, &options, CONTROL_QUEST, &mut now);
    complete_quest(&mut scene, &options, CONTROL_QUEST, &mut now);
    now += 10;
    scene
        .dispatch(&options, &quest_end_reply(CONTROL_QUEST), now)
        .expect("the first turn-in commits");
    let paid = scene.snapshot();
    let paid_save = fs::read(&options.state).expect("the paid save is readable");

    for round in 0..2 {
        now += 10;
        scene
            .dispatch(&options, &quest_end_reply(CONTROL_QUEST), now)
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
    reconnected.enter(&options, &mut later);
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
        .dispatch(&options, &quest_end_reply(CONTROL_QUEST), later)
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
    let fixture = Fixture::new("t258-full-bag");
    let options = fixture.options(fixture.scratch.path().join("character.json"));
    let character = fixture.character();
    let full = fixture.full_bag_state();
    fixture.save(&options.state, &character, &full);
    let mut scene = Scene::new(&options, 98);
    let mut now = 1_000;
    scene.enter(&options, &mut now);
    accept_quest(&mut scene, &options, CONTROL_QUEST, &mut now);
    complete_quest(&mut scene, &options, CONTROL_QUEST, &mut now);

    let before = scene.snapshot();
    let before_save = fs::read(&options.state).expect("the good save is readable");
    assert_eq!(
        count_items(before.gameplay.as_ref().expect("full-bag gameplay")),
        100,
        "fixture setup: the bag is full"
    );
    now += 10;
    scene
        .dispatch(&options, &quest_end_reply(CONTROL_QUEST), now)
        .expect("a full bag is a refusal, not a save failure");
    let refused_replies = scene.drain(now);
    assert!(
        !has_quest_packet(&refused_replies, 4, CONTROL_QUEST),
        "a full-bag refusal still completed the quest: {refused_replies:?}"
    );

    // The reward never became a quest completion, coin, item, XP or save change.
    assert_eq!(
        scene.snapshot(),
        before,
        "a full-bag refusal changed the character"
    );
    let after = scene.gameplay();
    assert_eq!(quest_status(&after, CONTROL_QUEST), QuestStatus::Active);
    assert_eq!(pending_turnin(&after), Some(CONTROL_QUEST));
    assert_eq!(count_items(&after), 100);
    assert_eq!(
        fs::read(&options.state).expect("the good save is readable"),
        before_save,
        "a full-bag refusal wrote the character save"
    );

    // The session is still healthy: the same giver reopens its menu.
    now += 10;
    let menu = scene.send(&options, &interact(GIVER), now);
    assert!(
        has_family(&menu, 26, 9),
        "the full-bag refusal ended the NPC session: {menu:?}"
    );
}

/// Gate-only proof: the save under test is `commit_outcome`'s save, which runs before the staged
/// gateway is installed. The failure cases drive `Session::dispatch_message`; the canary run
/// restored the production save path and made them red.
#[test]
fn the_save_failure_harness_is_the_real_commit_path() {
    let production = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/bin/osfr-gateway.rs"
    ));
    let save = production
        .find("save(&self.state_path, &staged_gateway.snapshot())?;")
        .expect("commit_outcome must save the staged character");
    let install = production
        .find("self.gateway = staged_gateway;")
        .expect("commit_outcome must install the staged gateway");
    assert!(
        save < install,
        "the staged gateway is installed before its save: the transaction is not all-or-nothing"
    );
    assert!(
        production.contains("fn commit_outcome("),
        "the save-ordering pin is aimed at the wrong function"
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
