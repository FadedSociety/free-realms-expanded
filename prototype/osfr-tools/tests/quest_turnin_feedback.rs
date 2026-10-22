//! T382: a quest reward the server cannot grant is a per-player chat refusal that keeps the
//! pending end screen for a retry, never a session-ending `io::Error`; a successful split-path
//! turn-in speaks no server chat line (the client writes its own completion text from 49/4).
//!
//! Every request is driven through the real `Gateway::handle_at`: 26/8 -> 26/10 Talk -> 26/6
//! dialogue choice -> 49/1 offer -> 49/2 accept -> the completion Talk (its 49/13 waits for
//! 26/6) -> 49/14 turn-in, plus the ordinary 26/23 abandon. The turn-in travels as a combined
//! `[0, 25]` bundle, the form whose child failure the gateway re-raises to the session (a single
//! tunneled request is contained by `handle_at`'s per-application arm); only the session-ending
//! form can prove the refusal stays an `Ok` outcome.
//!
//! Client shapes (2014 client, `lab/autopilot/ida_query.py`):
//! - 26/8 `008b6bc0`: u64 target + one flag byte (13-byte body).
//! - 26/10 `00a36f80`: u64 target + i32 choice (16-byte body).
//! - 26/6 `009f5890`: one i32 response row (default -1).
//! - 26/23 ctor `00c09f60`, writer `00c0a350`: preamble `0090ac70` writes u16 26 + u16 23, then
//!   the u32 quest id (8 bytes, the shape the server decodes since T408).
//! - 49/2 `00a86400`: u32 quest id + accept byte (11 bytes).
//! - 49/14 ctor `00a864c0`, writer `00a916d0`: preamble `00a8f860` writes u16 49 + u32 14, then
//!   the u32 quest id (10 bytes).
//! - 49/13 `00c7b990`: u64 actor 6..14, quest 14..18, end text 18..22, animation 22..26, reward
//!   bundle at 26, trailing dword.
//! - 15/3 system chat: the existing `chat::system` encoder (length-prefixed text + one byte).
//! The production decoders/encoders in `npc_interaction` and `chat` build and read every body;
//! the tests only use the shared `tests/common/gateway_packets.rs` request builders.
//!
//! Synthetic content: the private round70 content base (`tests/common/round70_content.rs`) plus
//! one giver NPC, two NPCs total, two jobs and this file's talk-objective quests. Quest ids,
//! reward values, end-text ids and NPC keys are test content, not retail values. The round70 base
//! needs the private lab package, the same fixture the vendor/quest gateway suites use.

#[path = "common/gateway_packets.rs"]
mod gateway_packets;
#[path = "common/round70_content.rs"]
mod round70_content;

use osfr_tools::{
    character_store::CharacterState,
    game_store::Snapshot,
    gameplay::{Gameplay, GameplayState},
    gateway::{Gateway, GatewayOutcome},
    inventory::{Inventory, InventoryItem, InventoryState, MAX_ITEMS},
    profiles::ProfileState,
    quest_engine::{QuestDenial, QuestStatus},
    rewards::{
        GrantFailure, ItemGrant, RewardBundleRow, RewardDisplay, RewardGrant, RewardState,
        apply_grant,
    },
};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs, io,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(0);

/// Character A (and the single-player cases).
const CHARACTER: u64 = 382_001;
/// Character B in the two-player case.
const SECOND_CHARACTER: u64 = 382_002;
const GIVER: u64 = 700;
const OTHER_NPC: u64 = 701;
const QUEST: u32 = 7_382_001;
const NEXT_QUEST: u32 = 7_382_002;
/// `bowl_basic.adr` in the round70 inventory catalogue: `max_stack` 1, so a grant adds one stack.
const REWARD_ITEM: i32 = 8;
/// A real catalogue definition used only to fill the bag.
const FILLER_ITEM: i32 = 75_060;
/// Absent from the catalogue: the content-error case.
const UNKNOWN_ITEM: i32 = 8_888_888;
/// A content profile the fixture's saved progression does not unlock.
const LOCKED_PROFILE: i32 = 48;
const QUEST_COINS: u32 = 41;
const QUEST_XP: u32 = 25;
const STARTING_COINS: u32 = 1_000;
const END_TEXT: u32 = 7_382_901;
/// `(R)` The production turn-in floor in `gameplay/npc.rs`: `items.len().max(100)`.
const BAG_SLOTS: usize = 100;
/// `build_options` lists Talk first with id 1 (`npc_interaction.rs`).
const TALK_OPTION: u32 = 1;
/// Quest dialogue row that opens the offer or releases the held end screen (`009f5890`).
const QUEST_CHOICE: i32 = 301;
const START_POSITION: [f32; 3] = [0.0; 3];

/// One private content copy plus the shared gateway reference. Dropped after the test.
struct Fixture {
    root: PathBuf,
    content: PathBuf,
    reference: PathBuf,
    version: String,
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

impl Fixture {
    fn new(quests: &[Value]) -> Self {
        let root = std::env::temp_dir().join(format!(
            "osfr-t382-{}-{}",
            std::process::id(),
            NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&root).expect("the fixture root creates");
        let reference = round70_content::reference();
        let content = round70_content::build(&root, "content", &["profiles.json"]);
        fs::create_dir_all(content.join("quests")).expect("the quest directory creates");
        let quest_ids: Vec<u32> = quests
            .iter()
            .filter_map(|quest| quest["id"].as_u64())
            .map(|id| id as u32)
            .collect();
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
                "items": []
            }),
        );
        write_json(
            content.join("npcs/t382.json"),
            json!({"schema_version": 1, "records": [
                npc(GIVER, "T382 Giver", &quest_ids),
                npc(OTHER_NPC, "T382 Other", &[])
            ]}),
        );
        write_json(
            content.join("quests/t382.json"),
            json!({"schema_version": 1, "records": quests}),
        );
        write_json(
            content.join("jobs.json"),
            json!({"schema_version": 1, "records": [
                job("synthetic", 1, &quest_ids),
                job("locked", LOCKED_PROFILE, &[])
            ]}),
        );
        let version = reference_version(&reference);
        Self {
            root,
            content,
            reference,
            version,
        }
    }

    /// The deterministic opening state: an empty bag, known coins and the profile content's
    /// default active progression (profile 1, profiles 1 and 43 unlocked; 48 locked).
    fn state(&self) -> GameplayState {
        let mut state = Gameplay::load(&self.content, None)
            .expect("the fixture gameplay content loads")
            .state();
        state.inventory.items.clear();
        for entry in &mut state.inventory.equipment {
            entry.item_id = 0;
        }
        state.inventory.coins = STARTING_COINS;
        state.inventory.next_instance_id = Some(1);
        state
    }

    fn character(guid: u64) -> CharacterState {
        CharacterState {
            guid,
            zone_id: 0,
            position: START_POSITION,
            rotation: [0.0; 3],
        }
    }

    /// Load one session (the production `Gameplay::load` + split quest bind) and drive the real
    /// login, 13/0 Ready and 10/0 Loaded sequence so world entry (and the saved end-screen
    /// recovery) runs exactly as a socket session runs it.
    fn entered(&self, guid: u64, saved: Option<GameplayState>, now: &mut u64) -> Gateway {
        let mut gateway =
            Gateway::load_gameplay(&self.reference, Self::character(guid), &self.content, saved)
                .expect("the gateway session loads");
        let login = gateway_packets::login_with_version(guid, &self.version);
        gateway.handle_at(&login, *now).expect("the login commits");
        *now += 10;
        gateway
            .handle_at(&wrap(&[13, 0]), *now)
            .expect("the Ready message commits");
        *now += 10;
        let loaded = gateway
            .handle_at(&wrap(&[10, 0]), *now)
            .expect("the Loaded message commits");
        *now += 10;
        assert!(loaded.entered_world, "the session entered its world");
        gateway
    }

    /// The live session state, for a saved-state reload (the T301/T258 pattern).
    fn state_after(&self, gateway: &Gateway) -> GameplayState {
        gateway
            .snapshot()
            .gameplay
            .expect("the session has gameplay state")
    }
}

fn reference_version(reference: &Path) -> String {
    let manifest: Value = serde_json::from_slice(
        &fs::read(reference.join("manifest.json")).expect("the reference manifest reads"),
    )
    .expect("the reference manifest parses");
    manifest["client_version"]
        .as_str()
        .expect("the reference manifest names its client version")
        .to_owned()
}

fn source() -> Value {
    json!([{
        "kind": "reconstructed",
        "evidence": "(R) T382 synthetic turn-in feedback content: talk-objective quests, local ids, 41 coins, one item 8 and 25 Adventurer XP. Test policy, not recovered retail values."
    }])
}

fn npc(npc_id: u64, name: &str, quests: &[u32]) -> Value {
    json!({
        "npc_id": npc_id,
        "name": name,
        "zone_id": 0,
        "position": START_POSITION,
        "placement": "legacy_data",
        "model_id": 1,
        "roles": ["quest_giver"],
        "quest_ids_given": quests
    })
}

fn job(id: &str, profile_id: i32, quests: &[u32]) -> Value {
    json!({
        "id": id,
        "name": id,
        "profile_id": profile_id,
        "category": "freestyle",
        "max_level": 20,
        "members_only": false,
        "trainer_npc_ids": [],
        "quest_ids": quests,
        "abilities": [],
        "source": source()
    })
}

/// One talk-to-npc quest at the giver. Its `talk_to_npc` target is the giver itself, so the
/// completion Talk both advances the objective and opens the pending end screen.
fn quest(
    id: u32,
    prerequisites: &[u32],
    coins: u32,
    items: &[(i32, u32)],
    job_xp: &[(i32, u32)],
) -> Value {
    let items: Vec<Value> = items
        .iter()
        .map(|(definition_id, count)| {
            json!({"definition_id": definition_id, "count": count, "source": source()})
        })
        .collect();
    let job_xp: BTreeMap<String, u32> = job_xp
        .iter()
        .map(|(profile, amount)| (profile.to_string(), *amount))
        .collect();
    json!({
        "id": id,
        "title_string_id": 1,
        "end_text_string_id": END_TEXT,
        "giver_npc_id": GIVER,
        "prerequisite_quest_ids": prerequisites,
        "objectives": [{
            "kind": "talk_to_npc",
            "id": 1,
            "npc_id": GIVER,
            "source": source()
        }],
        "rewards": [{
            "coins": coins,
            "items": items,
            "job_xp": job_xp,
            "source": source()
        }],
        "source": source(),
        "enabled": true
    })
}

fn write_json(path: PathBuf, value: Value) {
    fs::write(
        &path,
        serde_json::to_vec(&value).expect("the fixture JSON serializes"),
    )
    .unwrap_or_else(|error| panic!("fixture {} writes: {error}", path.display()));
}

// ---------------------------------------------------------------------------------------------
// Session packet helpers (all requests run through `Gateway::handle_at`).
// ---------------------------------------------------------------------------------------------

/// The client application wrapper (`0, 5, 0, 1` tunnel in the frontend): three-byte outer
/// header plus the payload length.
fn wrap(inner: &[u8]) -> Vec<u8> {
    let mut bytes = vec![5, 0, 1];
    bytes.extend((inner.len() as u32).to_le_bytes());
    bytes.extend(inner);
    bytes
}

/// A combined `[0, 25]` application bundle: a child error here is re-raised to the session.
fn bundle(children: &[Vec<u8>]) -> Vec<u8> {
    let mut bytes = vec![0, 25];
    for child in children {
        assert!(child.len() < 255, "one bundle child fits the short length");
        bytes.push(child.len() as u8);
        bytes.extend(child);
    }
    bytes
}

fn inner(packet: &[u8]) -> &[u8] {
    if packet.len() >= 7 && packet[..3] == [5, 0, 1] {
        &packet[7..]
    } else {
        packet
    }
}

fn has_family(packets: &[Vec<u8>], family: u16, subtype: u16, id: Option<u32>) -> bool {
    packets.iter().any(|packet| {
        let packet = inner(packet);
        if packet.len() < 6
            || packet[..2] != family.to_le_bytes()
            || packet[2..4] != subtype.to_le_bytes()
        {
            return false;
        }
        let Some(id) = id else { return true };
        // 49/13 carries the turn-in actor at 6..14 and the quest id at 14..18; every other
        // family/subtype pair used here carries its id at 6..10.
        let offset = if (family, subtype) == (49, 13) { 14 } else { 6 };
        packet
            .get(offset..offset + 4)
            .is_some_and(|bytes| u32::from_le_bytes(bytes.try_into().unwrap()) == id)
    })
}

/// `chat::system` (15/3) lines in reply order. The encoders escape `&`, `<` and `>`.
fn chat_texts(packets: &[Vec<u8>]) -> Vec<String> {
    packets
        .iter()
        .filter_map(|packet| {
            let packet = inner(packet);
            if packet.len() < 8 || packet[..4] != [15, 0, 3, 0] {
                return None;
            }
            let length = usize::try_from(u32::from_le_bytes(packet[4..8].try_into().unwrap()))
                .expect("the chat length fits");
            let text = std::str::from_utf8(packet.get(8..8 + length)?).ok()?;
            Some(
                text.replace("&lt;", "<")
                    .replace("&gt;", ">")
                    .replace("&amp;", "&"),
            )
        })
        .collect()
}

fn send(gateway: &mut Gateway, request: &[u8], now: u64) -> Vec<Vec<u8>> {
    gateway
        .handle_at(&wrap(request), now)
        .unwrap_or_else(|error| {
            panic!(
                "request {:?} was refused: {error}",
                &request[..request.len().min(4)]
            )
        })
        .replies
}

/// One 49/14 turn-in as the session-ending [`bundle`] form.
fn turn_in(gateway: &mut Gateway, quest_id: u32, now: u64) -> GatewayOutcome {
    let request = gateway_packets::quest_end_reply(quest_id);
    gateway
        .handle_at(&bundle(&[wrap(&request)]), now)
        .expect("a quest reward refusal must not end the session")
}

fn talk(gateway: &mut Gateway, target: u64, now: u64) -> Vec<Vec<u8>> {
    let menu = send(gateway, &gateway_packets::interact(target), now);
    assert!(
        has_family(&menu, 26, 9, None),
        "NPC {target} did not open its 26/9 menu"
    );
    send(
        gateway,
        &gateway_packets::select(target, TALK_OPTION),
        now + 1,
    )
}

/// Offer and accept one quest through the ordinary Talk -> 26/6 -> 49/1 -> 49/2 path.
fn offer_and_accept(gateway: &mut Gateway, quest_id: u32, now: &mut u64) {
    let dialogue = talk(gateway, GIVER, *now);
    *now += 10;
    assert!(
        has_family(&dialogue, 26, 3, None),
        "Talk did not open the 26/3 dialogue"
    );
    let offered = send(
        gateway,
        &gateway_packets::dialogue_choice(QUEST_CHOICE),
        *now,
    );
    *now += 10;
    assert!(
        has_family(&offered, 49, 1, Some(quest_id)),
        "quest {quest_id} was not offered"
    );
    let accepted = send(gateway, &gateway_packets::quest_reply(quest_id, true), *now);
    *now += 10;
    assert!(
        has_family(&accepted, 49, 3, Some(quest_id)),
        "quest {quest_id} did not send its 49/3"
    );
    assert_eq!(quest_status(gateway, quest_id), QuestStatus::Active);
}

/// The completion Talk advances the only objective, holds the 49/13, and the client's 26/6
/// releases it. Returns the end-screen packet.
fn open_end_screen(gateway: &mut Gateway, quest_id: u32, now: &mut u64) -> Vec<u8> {
    let dialogue = talk(gateway, GIVER, *now);
    *now += 10;
    assert!(
        has_family(&dialogue, 26, 3, None),
        "the completion Talk did not open its 26/3 dialogue"
    );
    assert!(
        !has_family(&dialogue, 49, 13, Some(quest_id)),
        "the 49/13 waits for the client's 26/6 choice"
    );
    let released = send(
        gateway,
        &gateway_packets::dialogue_choice(QUEST_CHOICE),
        *now,
    );
    *now += 10;
    let end = released
        .iter()
        .find(|packet| {
            let packet = inner(packet);
            packet.len() >= 18
                && packet[..4] == [49, 0, 13, 0]
                && u32::from_le_bytes(packet[14..18].try_into().unwrap()) == quest_id
        })
        .cloned()
        .unwrap_or_else(|| panic!("quest {quest_id} did not release its 49/13 end screen"));
    assert_eq!(pending_turnin(gateway), Some(quest_id));
    end
}

fn snapshot_state(gateway: &Gateway) -> GameplayState {
    gateway
        .snapshot()
        .gameplay
        .expect("the session has gameplay state")
}

fn saved_bytes(gateway: &Gateway) -> Vec<u8> {
    serde_json::to_vec(&gateway.snapshot()).expect("the session snapshot serializes")
}

fn coins(gateway: &Gateway) -> u32 {
    snapshot_state(gateway).inventory.coins
}

fn profile_xp(gateway: &Gateway, profile: i32) -> u32 {
    snapshot_state(gateway)
        .profiles
        .expect("the fixture has profile content")
        .experience
        .get(&profile)
        .copied()
        .unwrap_or(0)
}

fn active_profile(gateway: &Gateway) -> i32 {
    snapshot_state(gateway)
        .profiles
        .expect("the fixture has profile content")
        .active
}

fn count_definition(gateway: &Gateway, definition_id: i32) -> usize {
    snapshot_state(gateway)
        .inventory
        .items
        .iter()
        .filter(|item| item.definition_id == definition_id && item.rental.is_none())
        .map(|item| usize::try_from(item.count).unwrap_or(usize::MAX))
        .sum()
}

fn quest_status(gateway: &Gateway, quest_id: u32) -> QuestStatus {
    snapshot_state(gateway)
        .npc_quests
        .expect("the session has NPC quest state")
        .quests[&quest_id]
        .status
        .clone()
}

fn pending_turnin(gateway: &Gateway) -> Option<u32> {
    snapshot_state(gateway)
        .npc_quests
        .expect("the session has NPC quest state")
        .pending_turnin
}

fn pending_turnin_actor(gateway: &Gateway) -> Option<u64> {
    snapshot_state(gateway)
        .npc_quests
        .expect("the session has NPC quest state")
        .pending_turnin_actor
}

fn pending_offer(gateway: &Gateway) -> Option<u32> {
    snapshot_state(gateway)
        .npc_quests
        .expect("the session has NPC quest state")
        .pending_offer
}

/// Fill the saved bag to `slots` stacks (the turn-in floor is `BAG_SLOTS`), one filler per slot.
fn fill_bag(inventory: &mut InventoryState, slots: usize) {
    let base = 200_000;
    for offset in inventory.items.len()..slots {
        inventory.items.push(InventoryItem {
            definition_id: FILLER_ITEM,
            tint: 0,
            id: base + offset as i32,
            count: 1,
            consumed_count: 0,
            stored_count: 0,
            last_cast_time: 0,
            rental: None,
            ability_count: 0,
            activate_enabled: false,
        });
    }
    inventory.next_instance_id = Some(i64::from(base) + slots as i64);
}

/// 26/23 abandon as the client writes it: `[26,0,23,0]` + u32 id, 8 bytes (writer `00c0a350`,
/// preamble `0090ac70`).
fn abandon_packet(quest_id: u32) -> Vec<u8> {
    let mut packet = vec![26, 0, 23, 0];
    packet.extend(quest_id.to_le_bytes());
    packet
}

fn end_text(end_screen: &[u8]) -> u32 {
    let end_screen = inner(end_screen);
    u32::from_le_bytes(end_screen[18..22].try_into().unwrap())
}

/// A 49/14 reply that changes nothing but still closes the dialog camera (the arm's
/// unconditional 26/4).
fn closes_dialog_only(replies: &[Vec<u8>]) -> bool {
    replies.iter().all(|packet| inner(packet) == [26, 0, 4, 0])
}

// ---------------------------------------------------------------------------------------------
// Tests.
// ---------------------------------------------------------------------------------------------

/// A full bag refuses the reward as one chat line, keeps the pending end screen, and changes no
/// state or save byte. Releasing one slot and retrying through Talk/49/13/49/14 pays once.
#[test]
fn a_full_bag_turn_in_is_a_chat_refusal_that_keeps_the_end_screen() {
    let fixture = Fixture::new(&[quest(
        QUEST,
        &[],
        QUEST_COINS,
        &[(REWARD_ITEM, 1)],
        &[(1, QUEST_XP)],
    )]);
    let mut now = 10_000;
    let mut gateway = fixture.entered(CHARACTER, Some(fixture.state()), &mut now);
    offer_and_accept(&mut gateway, QUEST, &mut now);
    let end = open_end_screen(&mut gateway, QUEST, &mut now);
    assert_eq!(end_text(&end), END_TEXT);
    assert_eq!(pending_turnin(&gateway), Some(QUEST));

    let mut full = fixture.state_after(&gateway);
    fill_bag(&mut full.inventory, BAG_SLOTS);
    let mut gateway = fixture.entered(CHARACTER, Some(full), &mut now);
    let coins_before = coins(&gateway);
    let xp_before = profile_xp(&gateway, 1);
    let item_before = count_definition(&gateway, REWARD_ITEM);
    let before = gateway.snapshot();
    let before_save = saved_bytes(&gateway);

    let refused = turn_in(&mut gateway, QUEST, now);
    now += 10;
    let lines = chat_texts(&refused.replies);
    assert_eq!(
        lines,
        vec![QuestDenial::NeedSpace.message().unwrap().to_owned()],
        "a full bag is one player-facing refusal line"
    );
    assert!(
        !refused.replies.iter().any(|packet| has_family(
            std::slice::from_ref(packet),
            49,
            4,
            Some(QUEST)
        ) || has_family(
            std::slice::from_ref(packet),
            50,
            1,
            None
        )),
        "a refused turn-in sent a completion or reward packet"
    );
    assert_eq!(gateway.snapshot(), before, "the refusal changed state");
    assert_eq!(
        saved_bytes(&gateway),
        before_save,
        "the refusal changed the serialized save"
    );
    assert_eq!(coins(&gateway), coins_before);
    assert_eq!(profile_xp(&gateway, 1), xp_before);
    assert_eq!(count_definition(&gateway, REWARD_ITEM), item_before);
    assert_eq!(quest_status(&gateway, QUEST), QuestStatus::Active);
    assert_eq!(pending_turnin(&gateway), Some(QUEST));
    assert_eq!(pending_turnin_actor(&gateway), Some(GIVER));

    // Reconnect from the serialized save written after the refusal (the pending screen and its
    // actor survive it), make one slot, and retry through the normal Talk path.
    let reloaded: Snapshot =
        serde_json::from_slice(&saved_bytes(&gateway)).expect("the refused save deserializes");
    let mut fixed = reloaded
        .gameplay
        .expect("the refusal kept the gameplay save");
    let quests = fixed.npc_quests.as_ref().expect("the save has quest state");
    assert_eq!(quests.pending_turnin, Some(QUEST));
    assert_eq!(quests.pending_turnin_actor, Some(GIVER));
    fixed.inventory.items.pop();
    let mut gateway = fixture.entered(CHARACTER, Some(fixed), &mut now);
    let screen = open_end_screen(&mut gateway, QUEST, &mut now);
    assert!(
        has_family(std::slice::from_ref(&screen), 49, 13, Some(QUEST)),
        "the retry Talk reopens the held 49/13"
    );
    let paid = turn_in(&mut gateway, QUEST, now);
    now += 10;
    assert!(
        has_family(&paid.replies, 49, 4, Some(QUEST)),
        "the retried turn-in completes the quest"
    );
    assert_eq!(coins(&gateway), coins_before + QUEST_COINS);
    assert_eq!(profile_xp(&gateway, 1), xp_before + QUEST_XP);
    assert_eq!(count_definition(&gateway, REWARD_ITEM), item_before + 1);
    assert_eq!(quest_status(&gateway, QUEST), QuestStatus::Completed);
    assert_eq!(pending_turnin(&gateway), None);

    // A replay pays nothing and changes no byte.
    let paid_state = gateway.snapshot();
    let paid_bytes = saved_bytes(&gateway);
    let replay = turn_in(&mut gateway, QUEST, now);
    assert!(
        closes_dialog_only(&replay.replies),
        "a replay answers only its 26/4: {:?}",
        replay.replies
    );
    assert_eq!(gateway.snapshot(), paid_state);
    assert_eq!(saved_bytes(&gateway), paid_bytes);
    assert_eq!(coins(&gateway), coins_before + QUEST_COINS);
}

/// An unknown reward item is a typed content refusal with a bounded log line, not a disconnect.
/// The log line goes to the process's stderr, so the test re-runs itself in a child process
/// (the `character_store` restart pattern) and reads that child's stderr.
#[test]
fn an_unknown_reward_item_is_a_logged_refusal_not_a_disconnect() {
    const CHILD: &str = "OSFR_T382_LOG_CHILD";
    if std::env::var_os(CHILD).is_none() {
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "an_unknown_reward_item_is_a_logged_refusal_not_a_disconnect",
                "--nocapture",
            ])
            .env(CHILD, "1")
            .output()
            .expect("the child test process runs");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            output.status.success(),
            "the child refusal run failed: {}{stderr}",
            String::from_utf8_lossy(&output.stdout)
        );
        // One bounded line per refused turn-in (the refusal and its replay): the quest id and
        // the failure class, no identity, chat or packet bytes.
        let logged: Vec<&str> = stderr
            .lines()
            .filter(|line| line.contains("quest_turnin_reward_refused"))
            .collect();
        let expected = format!("quest_turnin_reward_refused quest={QUEST} reason=UnknownItem");
        assert_eq!(logged, [expected.as_str(), expected.as_str()], "{stderr}");
        return;
    }
    let fixture = Fixture::new(&[quest(QUEST, &[], 0, &[(UNKNOWN_ITEM, 1)], &[])]);
    let mut now = 20_000;
    let mut gateway = fixture.entered(CHARACTER, Some(fixture.state()), &mut now);
    offer_and_accept(&mut gateway, QUEST, &mut now);
    open_end_screen(&mut gateway, QUEST, &mut now);
    let before = gateway.snapshot();
    let before_save = saved_bytes(&gateway);

    let refused = turn_in(&mut gateway, QUEST, now);
    let lines = chat_texts(&refused.replies);
    assert_eq!(
        lines,
        vec![QuestDenial::RewardData.message().unwrap().to_owned()],
        "a content reward error is one player-facing refusal line"
    );
    assert_eq!(gateway.snapshot(), before, "the refusal changed state");
    assert_eq!(saved_bytes(&gateway), before_save);
    assert_eq!(quest_status(&gateway, QUEST), QuestStatus::Active);
    assert_eq!(pending_turnin(&gateway), Some(QUEST));
    assert_eq!(pending_turnin_actor(&gateway), Some(GIVER));

    // The end screen survives a replayed refused turn-in: the same denial line and no byte
    // changes.
    let again = turn_in(&mut gateway, QUEST, now);
    assert_eq!(
        chat_texts(&again.replies),
        lines,
        "a replay at the held screen is the same refusal"
    );
    assert_eq!(gateway.snapshot(), before);
}

/// The existing F249 locked-profile denial still releases the end screen silently.
#[test]
fn a_locked_reward_profile_still_releases_the_end_screen() {
    let fixture = Fixture::new(&[quest(QUEST, &[], 0, &[], &[(LOCKED_PROFILE, QUEST_XP)])]);
    let mut now = 30_000;
    let mut gateway = fixture.entered(CHARACTER, Some(fixture.state()), &mut now);
    offer_and_accept(&mut gateway, QUEST, &mut now);
    open_end_screen(&mut gateway, QUEST, &mut now);
    let coins_before = coins(&gateway);

    let refused = turn_in(&mut gateway, QUEST, now);
    assert!(
        refused.gameplay_changed,
        "releasing the screen is a saved change"
    );
    assert!(
        chat_texts(&refused.replies).is_empty(),
        "the existing job denial adds no player-facing line: {:?}",
        refused.replies
    );
    assert_eq!(pending_turnin(&gateway), None, "the screen is released");
    assert_eq!(pending_turnin_actor(&gateway), None);
    assert_eq!(quest_status(&gateway, QUEST), QuestStatus::Active);
    assert_eq!(coins(&gateway), coins_before);
    assert_eq!(profile_xp(&gateway, LOCKED_PROFILE), 0);

    // The released handshake stays released for an immediate 49/14: only the dialog close.
    let before = gateway.snapshot();
    let replay = turn_in(&mut gateway, QUEST, now);
    assert!(closes_dialog_only(&replay.replies));
    assert_eq!(gateway.snapshot(), before);

    // A later Talk can reopen the completed step, which is the F249 retry after unlocking.
    let dialogue = talk(&mut gateway, GIVER, now);
    now += 10;
    assert!(has_family(&dialogue, 26, 3, None));
    let reopened = send(
        &mut gateway,
        &gateway_packets::dialogue_choice(QUEST_CHOICE),
        now,
    );
    assert!(
        has_family(&reopened, 49, 13, Some(QUEST)),
        "the released end screen reopens for the retry"
    );
    assert_eq!(pending_turnin(&gateway), Some(QUEST));
}

/// A content-error denial keeps the screen, the matching actor is still required to reopen it,
/// and the client's ordinary 26/23 abandon remains the way out.
#[test]
fn a_content_error_can_still_be_abandoned() {
    let fixture = Fixture::new(&[quest(QUEST, &[], 0, &[(UNKNOWN_ITEM, 1)], &[])]);
    let mut now = 40_000;
    let mut gateway = fixture.entered(CHARACTER, Some(fixture.state()), &mut now);
    offer_and_accept(&mut gateway, QUEST, &mut now);
    open_end_screen(&mut gateway, QUEST, &mut now);
    turn_in(&mut gateway, QUEST, now);
    assert_eq!(
        quest_status(&gateway, QUEST),
        QuestStatus::Active,
        "the content-error denial keeps the quest"
    );
    assert_eq!(pending_turnin(&gateway), Some(QUEST));

    // A different NPC cannot reopen the saved end screen; the pending actor still matches.
    let other = talk(&mut gateway, OTHER_NPC, now);
    assert!(
        !other
            .iter()
            .any(|packet| has_family(std::slice::from_ref(packet), 49, 13, None)),
        "a non-matching actor reopened the pending end screen"
    );
    assert_eq!(pending_turnin(&gateway), Some(QUEST));
    assert_eq!(pending_turnin_actor(&gateway), Some(GIVER));

    now += 10;
    let abandoned = send(&mut gateway, &abandon_packet(QUEST), now);
    assert!(
        has_family(&abandoned, 49, 6, Some(QUEST)),
        "26/23 answers with its 49/6 abandonment"
    );
    assert_eq!(pending_turnin(&gateway), None);
    assert_eq!(pending_turnin_actor(&gateway), None);
    assert_eq!(quest_status(&gateway, QUEST), QuestStatus::Abandoned);

    // The character is free: the same guide offers the quest again through the ordinary path.
    let dialogue = talk(&mut gateway, GIVER, now);
    now += 10;
    assert!(has_family(&dialogue, 26, 3, None));
    let offered = send(
        &mut gateway,
        &gateway_packets::dialogue_choice(QUEST_CHOICE),
        now,
    );
    now += 10;
    assert!(
        has_family(&offered, 49, 1, Some(QUEST)),
        "the abandoned quest is offerable again"
    );

    // A replayed 49/14 has no pending screen and pays nothing.
    let before = gateway.snapshot();
    let replay = turn_in(&mut gateway, QUEST, now);
    assert!(closes_dialog_only(&replay.replies));
    assert_eq!(gateway.snapshot(), before);
}

/// A tiny reward fixture over the real inventory catalogue: no bag room and a chosen wallet, so
/// the classification is exercised through the production grant path, not a copy of its rules.
struct GrantFixture {
    fixture: Fixture,
    inventory: InventoryState,
}

impl GrantFixture {
    fn new() -> Self {
        let fixture = Fixture::new(&[]);
        let mut inventory = fixture.state().inventory;
        inventory.items.clear();
        Self { fixture, inventory }
    }

    fn state(&self, edit: impl FnOnce(&mut InventoryState)) -> RewardState {
        let mut saved = self.inventory.clone();
        edit(&mut saved);
        let inventory = Inventory::load(&self.fixture.content, Some(saved))
            .expect("the fixture inventory loads");
        let profiles = ProfileState {
            version: 1,
            active: 1,
            unlocked: BTreeSet::from([1]),
            experience: BTreeMap::new(),
        };
        RewardState::new(inventory, profiles, 0)
    }
}

fn grant() -> RewardGrant {
    RewardGrant {
        coins: 0,
        items: Vec::new(),
        job_xp: BTreeMap::new(),
        display: RewardDisplay {
            source_guid: GIVER,
            player_guid: CHARACTER,
            reward_kind: 0,
            icon_id: 0,
            name_id: 0,
            rows: Vec::new(),
        },
    }
}

fn items(definition_id: i32, count: u32) -> RewardGrant {
    RewardGrant {
        items: vec![ItemGrant {
            definition_id,
            count,
        }],
        ..grant()
    }
}

/// The grant classification is a type: each refusal is its variant with one fixed message (the
/// old wording where one existed), and converts to the same `InvalidData` `io::Error` every `?`
/// caller already handled.
#[test]
fn grant_failures_are_typed_and_keep_their_wording() {
    let fixture = GrantFixture::new();
    let state = fixture.state(|_| {});
    let shape = RewardGrant {
        display: RewardDisplay {
            rows: vec![RewardBundleRow {
                type_id: 2,
                body: vec![0; 34],
            }],
            ..grant().display
        },
        ..grant()
    };
    // A bag at `Inventory::grant`'s `MAX_ITEMS` stack ceiling, with the quest path's capacity
    // (`items.len().max(100)`) letting the pre-grant room check pass.
    let mut at_ceiling = fixture.state(|inventory| fill_bag(inventory, MAX_ITEMS));
    at_ceiling.bag_capacity = MAX_ITEMS;
    let cases = [
        (
            GrantFailure::InvalidKey,
            "invalid reward grant key",
            apply_grant(&state, &grant(), ""),
        ),
        (
            GrantFailure::BagFull,
            "reward exceeds configured bag capacity",
            apply_grant(&state, &items(REWARD_ITEM, 1), "bag-full"),
        ),
        (
            GrantFailure::UnknownItem,
            "unknown reward item definition",
            apply_grant(&state, &items(UNKNOWN_ITEM, 1), "unknown-item"),
        ),
        (
            GrantFailure::BadItem,
            "invalid reward item",
            apply_grant(&state, &items(0, 1), "bad-item"),
        ),
        // `REWARD_ITEM` stacks to one, so two in one row is a content error, not a full bag.
        (
            GrantFailure::BadItem,
            "invalid reward item",
            apply_grant(&state, &items(REWARD_ITEM, 2), "over-stack"),
        ),
        (
            GrantFailure::WireRange,
            "reward coins or XP exceed the wire field",
            apply_grant(
                &state,
                &RewardGrant {
                    coins: u32::MAX,
                    ..grant()
                },
                "wire-range",
            ),
        ),
        // A near-cap wallet: the award itself fits, the balance would not.
        (
            GrantFailure::WireRange,
            "reward coins or XP exceed the wire field",
            apply_grant(
                &fixture.state(|inventory| inventory.coins = i32::MAX as u32),
                &RewardGrant {
                    coins: 1,
                    ..grant()
                },
                "wallet-cap",
            ),
        ),
        // No item id is left to hand out: a server fault that bag room cannot fix.
        (
            GrantFailure::BadItem,
            "invalid reward item",
            apply_grant(
                &fixture.state(|inventory| {
                    inventory.next_instance_id = Some(i64::from(i32::MAX) + 1);
                }),
                &items(REWARD_ITEM, 1),
                "exhausted-ids",
            ),
        ),
        // A new stack past the ceiling is a full bag: making room fixes it.
        (
            GrantFailure::BagFull,
            "reward exceeds configured bag capacity",
            apply_grant(&at_ceiling, &items(REWARD_ITEM, 1), "item-ceiling"),
        ),
        (
            GrantFailure::Shape,
            "unsupported or truncated reward bundle entry",
            apply_grant(&state, &shape, "shape"),
        ),
        (
            GrantFailure::LockedProfile,
            "reward XP references a locked profile",
            apply_grant(
                &state,
                &RewardGrant {
                    job_xp: BTreeMap::from([(LOCKED_PROFILE, 5)]),
                    ..grant()
                },
                "locked-profile",
            ),
        ),
    ];
    for (expected, message, result) in cases {
        let Err(failure) = result else {
            panic!("the {expected:?} grant must be refused");
        };
        assert_eq!(failure, expected, "the refusal is the typed {expected:?}");
        assert_eq!(failure.to_string(), message);
        let error = io::Error::from(failure);
        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
        assert_eq!(error.to_string(), message);
    }
}

/// A successful split turn-in completes through 49/4 with no server chat line, pays the configured
/// profile even after the active job changed, and the chained rung is offered through the ordinary
/// Talk path. The client writes its own localized completion text from 49/4 (`0x00a928b0`
/// formats the quest name with its completed text), so a server `Completed: Quest <id>.` line
/// would only duplicate it with a placeholder title.
#[test]
fn a_successful_turn_in_still_shows_completion_and_next_step() {
    let fixture = Fixture::new(&[
        quest(
            QUEST,
            &[],
            QUEST_COINS,
            &[(REWARD_ITEM, 1)],
            &[(1, QUEST_XP)],
        ),
        quest(NEXT_QUEST, &[QUEST], 1, &[], &[]),
    ]);
    let mut now = 50_000;
    let mut gateway = fixture.entered(CHARACTER, Some(fixture.state()), &mut now);
    offer_and_accept(&mut gateway, QUEST, &mut now);

    // The active job changes after the accept; the reward map still names profile 1.
    let mut switched = fixture.state_after(&gateway);
    {
        let profiles = switched.profiles.as_mut().expect("the fixture profiles");
        profiles.active = LOCKED_PROFILE;
        profiles.unlocked.insert(LOCKED_PROFILE);
    }
    let mut gateway = fixture.entered(CHARACTER, Some(switched), &mut now);
    let end = open_end_screen(&mut gateway, QUEST, &mut now);
    assert_eq!(end_text(&end), END_TEXT, "49/13 carries the end text id");
    let switch_xp = profile_xp(&gateway, LOCKED_PROFILE);
    let active_xp = profile_xp(&gateway, 1);

    let paid = turn_in(&mut gateway, QUEST, now);
    now += 10;
    assert!(
        has_family(&paid.replies, 49, 4, Some(QUEST)),
        "the client's completion notice is the 49/4"
    );
    assert!(
        !paid
            .replies
            .iter()
            .any(|packet| inner(packet).starts_with(&[15, 0])),
        "the turn-in duplicated the client's completion as a chat/system line"
    );
    assert_eq!(coins(&gateway), STARTING_COINS + QUEST_COINS);
    assert_eq!(
        profile_xp(&gateway, 1),
        active_xp + QUEST_XP,
        "the configured profile is paid, not the active job"
    );
    assert_eq!(
        profile_xp(&gateway, LOCKED_PROFILE),
        switch_xp,
        "the active job receives no quest XP"
    );
    assert_eq!(active_profile(&gateway), LOCKED_PROFILE);
    assert_eq!(quest_status(&gateway, QUEST), QuestStatus::Completed);
    assert_eq!(pending_turnin(&gateway), None);

    // The next rung is offered through the ordinary Talk path.
    let dialogue = talk(&mut gateway, GIVER, now);
    now += 10;
    assert!(has_family(&dialogue, 26, 3, None));
    let offered = send(
        &mut gateway,
        &gateway_packets::dialogue_choice(QUEST_CHOICE),
        now,
    );
    assert!(has_family(&offered, 49, 1, Some(NEXT_QUEST)));
}

/// A wallet at the i32 wire cap refuses the coins as the one `RewardLimit` line and keeps the
/// screen. Content cannot carry a row past the wire field (`validate_definition` refuses it when
/// the engine loads), so on the turn-in path this refusal is always the player's balance.
#[test]
fn a_near_cap_wallet_is_a_limit_refusal_that_keeps_the_end_screen() {
    let fixture = Fixture::new(&[quest(QUEST, &[], QUEST_COINS, &[], &[])]);
    let mut now = 80_000;
    let mut capped = fixture.state();
    capped.inventory.coins = i32::MAX as u32;
    let mut gateway = fixture.entered(CHARACTER, Some(capped), &mut now);
    offer_and_accept(&mut gateway, QUEST, &mut now);
    open_end_screen(&mut gateway, QUEST, &mut now);
    let before = gateway.snapshot();
    let before_save = saved_bytes(&gateway);

    let refused = turn_in(&mut gateway, QUEST, now);
    assert_eq!(
        chat_texts(&refused.replies),
        [QuestDenial::RewardLimit.message().unwrap()],
        "a near-cap wallet is one player-facing refusal line"
    );
    assert_eq!(gateway.snapshot(), before, "the refusal changed state");
    assert_eq!(saved_bytes(&gateway), before_save);
    assert_eq!(pending_turnin(&gateway), Some(QUEST));
}

/// A refused turn-in for A cannot change B: each session owns its pending screen, payout and
/// serialized save, and the server keeps handling B while A is refused.
#[test]
fn two_players_each_get_their_own_denial_and_payout() {
    let fixture = Fixture::new(&[quest(
        QUEST,
        &[],
        QUEST_COINS,
        &[(REWARD_ITEM, 1)],
        &[(1, QUEST_XP)],
    )]);
    let mut now = 60_000;
    let mut a = fixture.entered(CHARACTER, Some(fixture.state()), &mut now);
    let mut b = fixture.entered(SECOND_CHARACTER, Some(fixture.state()), &mut now);
    offer_and_accept(&mut a, QUEST, &mut now);
    offer_and_accept(&mut b, QUEST, &mut now);
    open_end_screen(&mut a, QUEST, &mut now);
    open_end_screen(&mut b, QUEST, &mut now);

    // A's bag is full; B has room.
    let mut a_state = fixture.state_after(&a);
    fill_bag(&mut a_state.inventory, BAG_SLOTS);
    let mut a = fixture.entered(CHARACTER, Some(a_state), &mut now);
    let b_before = b.snapshot();
    let b_save_before = saved_bytes(&b);

    let refused = turn_in(&mut a, QUEST, now);
    assert_eq!(
        chat_texts(&refused.replies),
        vec![QuestDenial::NeedSpace.message().unwrap().to_owned()]
    );
    assert_eq!(pending_turnin(&a), Some(QUEST));
    assert_eq!(quest_status(&a, QUEST), QuestStatus::Active);
    assert_eq!(b.snapshot(), b_before, "A's denial changed B");
    assert_eq!(saved_bytes(&b), b_save_before, "A's denial wrote B's save");

    // B pays in the same run; then A makes room, retries and pays too.
    let paid_b = turn_in(&mut b, QUEST, now);
    assert!(has_family(&paid_b.replies, 49, 4, Some(QUEST)));
    assert_eq!(quest_status(&b, QUEST), QuestStatus::Completed);
    assert_eq!(coins(&b), STARTING_COINS + QUEST_COINS);
    assert_eq!(profile_xp(&b, 1), QUEST_XP);
    assert_eq!(
        pending_turnin(&a),
        Some(QUEST),
        "B's payout changed A's pending screen"
    );

    let mut fixed = fixture.state_after(&a);
    fixed.inventory.items.pop();
    let mut a = fixture.entered(CHARACTER, Some(fixed), &mut now);
    open_end_screen(&mut a, QUEST, &mut now);
    let paid_a = turn_in(&mut a, QUEST, now);
    assert!(has_family(&paid_a.replies, 49, 4, Some(QUEST)));
    assert_eq!(quest_status(&a, QUEST), QuestStatus::Completed);
    assert_eq!(coins(&a), STARTING_COINS + QUEST_COINS);
    assert_eq!(profile_xp(&a, 1), QUEST_XP);

    // Each save carries its own character and paid state.
    let a_state = fixture.state_after(&a);
    let b_state = fixture.state_after(&b);
    assert_eq!(a.state().guid, CHARACTER);
    assert_eq!(b.state().guid, SECOND_CHARACTER);
    assert_eq!(a_state.npc_quests.as_ref().unwrap().character_id, CHARACTER);
    assert_eq!(
        b_state.npc_quests.as_ref().unwrap().character_id,
        SECOND_CHARACTER
    );
    // B's replay of the quest pays nothing, and A's save is untouched by it.
    let a_save = saved_bytes(&a);
    let replayed = turn_in(&mut b, QUEST, now);
    assert!(closes_dialog_only(&replayed.replies));
    assert_eq!(saved_bytes(&a), a_save);
    assert_eq!(coins(&b), STARTING_COINS + QUEST_COINS);
}

/// The turn-in path uses no developer command, no teleport and leaves the character in a valid,
/// free state with every pending control flag clear.
#[test]
fn the_turn_in_uses_no_dev_commands_and_leaves_the_character_free() {
    let fixture = Fixture::new(&[
        quest(
            QUEST,
            &[],
            QUEST_COINS,
            &[(REWARD_ITEM, 1)],
            &[(1, QUEST_XP)],
        ),
        quest(NEXT_QUEST, &[QUEST], 1, &[], &[]),
    ]);
    let mut now = 70_000;
    let mut gateway = fixture.entered(CHARACTER, Some(fixture.state()), &mut now);
    for command in [
        "offer",
        "teleport",
        "here",
        "broadcast",
        "cheat",
        "give",
        "npx",
    ] {
        assert!(
            !gateway
                .gameplay()
                .expect("the session has gameplay")
                .may_run(command),
            "the character must not run the {command:?} developer command"
        );
    }
    offer_and_accept(&mut gateway, QUEST, &mut now);
    open_end_screen(&mut gateway, QUEST, &mut now);

    let requests = [
        gateway_packets::interact(GIVER),
        gateway_packets::select(GIVER, TALK_OPTION),
        gateway_packets::dialogue_choice(QUEST_CHOICE),
        gateway_packets::quest_reply(QUEST, true),
        gateway_packets::quest_end_reply(QUEST),
    ];
    for request in &requests {
        assert!(
            !request.starts_with(&[15, 0]),
            "the turn-in loop must send no chat/dev command"
        );
    }

    let paid = turn_in(&mut gateway, QUEST, now);
    for reply in &paid.replies {
        let reply = inner(reply);
        assert!(
            !reply.starts_with(&[15, 0, 1, 0]),
            "the turn-in reply contains a client chat request"
        );
        assert!(
            !reply.starts_with(&[38, 0, 12, 0]),
            "the turn-in reply contains a developer teleport"
        );
        assert!(
            !reply.windows(6).any(|window| window == b"realm "),
            "the turn-in reply contains a developer command line"
        );
    }

    let state = snapshot_state(&gateway);
    assert_eq!(pending_offer(&gateway), None);
    assert_eq!(pending_turnin(&gateway), None);
    let game = gateway.gameplay().expect("the session has gameplay");
    assert!(!game.in_encounter(), "an encounter is still active");
    assert!(!game.is_mounted(), "the character is still mounted");
    assert!(!game.is_soccer_active());
    assert!(!game.derby_active());

    // Nothing moved the character, and the free session still accepts the next rung through
    // the ordinary menu path (the shared busy gate would refuse a busy character).
    assert_eq!(gateway.state().zone_id, 0);
    assert_eq!(gateway.state().position, START_POSITION);
    assert_eq!(gateway.state().guid, CHARACTER);
    assert_eq!(state.inventory.coins, STARTING_COINS + QUEST_COINS);
    let dialogue = talk(&mut gateway, GIVER, now);
    now += 10;
    assert!(has_family(&dialogue, 26, 3, None));
    let offered = send(
        &mut gateway,
        &gateway_packets::dialogue_choice(QUEST_CHOICE),
        now,
    );
    assert!(has_family(&offered, 49, 1, Some(NEXT_QUEST)));
}
