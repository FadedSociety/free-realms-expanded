//! Shared one-goal Mining18 fixture and production round helpers for the NPC gather
//! quest suites.
//!
//! `farnum_starting_quests.rs` (T241) and `miner_first_ore.rs` (T300) both include
//! `#[path = "support/npc_quest_chain.rs"] mod npc_quest_chain;` and then
//! `#[path = "support/mining_quest_chain.rs"] mod mining_quest_chain;`. The generic
//! opening fixture and packet builders stay in `npc_quest_chain`; this file adds the
//! established synthetic W140/W171 one-goal board and the node/round helpers shared by
//! both suites. The shipped `mining.json` board pays a 12-ore batch per win, so it cannot
//! prove a one-ore gather goal; node entry, the win transaction, the gather event and the
//! quest payout remain the production path. Test-only: production never copies content.
#![allow(dead_code)]

use crate::npc_quest_chain::{
    CHARACTER, Fixture as ChainFixture, character_at_for, content_dir, has_family, npc_interact,
    release_end_screen, send, talk,
};
use osfr_tools::{
    character_store::CharacterState,
    game_store,
    gameplay::{Gameplay, GameplayState, Outcome},
    harvest_nodes::GatheringNodes,
    inventory::{InventoryItem, InventoryState, MAX_ITEMS},
    mining::{self, Type8BoardSpec, Type8Goal, Type8Kind, Type8Output},
    profiles::ProfileState,
};
use serde_json::{Value, json};
use std::{collections::BTreeMap, fs, path::Path};

/// (R) The live copper-vein node emits the mined item definition, Copper Ore 2739.
pub const COPPER_ORE: u32 = 2_739;
/// 26/42 closes the mining results window; the paid node hides on the next tick.
const CLOSE_GAME: [u8; 4] = [26, 0, 42, 0];
pub const MINING_XP_PER_WIN: u32 = mining::lifecycle::RECONSTRUCTED_MINER_XP_PER_WIN;

fn profile_state(active: i32, unlocked: &[i32]) -> ProfileState {
    ProfileState {
        version: 1,
        active,
        unlocked: unlocked.iter().copied().collect(),
        experience: BTreeMap::new(),
    }
}

/// One-goal Mining18 stand-in so a win is exactly one Copper Ore. Same shape as the
/// established T241 fixture; only the production seams around it matter here.
fn synthetic_mining() -> Value {
    const LEFT: usize = 3;
    const TOP: usize = 3;
    const PLAYABLE: usize = 8;
    const BOTTOM: usize = TOP + PLAYABLE - 1;
    let side = mining::SIDE;
    let mut shape = vec![0_i32; side * side];
    for dy in 0..PLAYABLE {
        for dx in 0..PLAYABLE {
            shape[(TOP + dy) * side + LEFT + dx] = 1;
        }
    }
    shape[BOTTOM * side + LEFT] = -601;
    for x in LEFT..LEFT + PLAYABLE {
        shape[(TOP + PLAYABLE) * side + x] = 4;
    }
    json!({
        "schema_version": 1,
        "game_id": mining::MINING_GAME_ID,
        "profile_id": mining::MINER_PROFILE_ID,
        "name_id": 7910,
        "description_id": 7911,
        "icon_id": 909,
        "spec": Type8BoardSpec {
            width: side,
            height: side,
            shape,
            selectable: vec![401, 402, 403, 404],
            collectable: vec![601],
            tiles: [401, 402, 403, 404]
                .iter()
                .map(|value| Type8Kind {
                    value: *value,
                    category: 400,
                    icon_id: i32::from(*value) - 400,
                })
                .chain([Type8Kind {
                    value: 601,
                    category: 600,
                    icon_id: 1301,
                }])
                .collect(),
            goals: vec![Type8Goal {
                index: 0,
                value: 601,
                quota: 1,
                per_reward: 1,
                icon_id: 1301,
            }],
            outputs: vec![Type8Output {
                value: 601,
                definition_id: COPPER_ORE as i32,
                cluster_quantity: 1,
            }],
            duration_seconds: 180,
            seed: 241,
            wildcard_percent: 0,
            collectable_percent: 0,
            blocker_percent: 0,
            powerup_percent: 0,
            meter_id: 0,
            reconstructed_rules: true,
            rule_notice: "Synthetic T241/T300 fixture; original Mining18 level tables unrecovered.".into(),
        },
    })
}

/// A bag the production payout gate cannot add Copper Ore to: one non-ore filler stack
/// per slot and no existing Copper Ore stack to merge into, so the node must start a new
/// stack. The node test asserts the no-ore precondition; the turn-in test fills this same
/// shape after the player already holds one mined ore.
pub fn fill_bag(inventory: &mut InventoryState) {
    let definition = inventory
        .items
        .iter()
        .map(|item| item.definition_id)
        .find(|definition| *definition != COPPER_ORE as i32)
        .expect("the opening fixture has a non-Copper-Ore starter item");
    let mut next = inventory
        .items
        .iter()
        .map(|item| item.id)
        .max()
        .unwrap_or(0)
        + 1;
    while inventory.items.len() < MAX_ITEMS {
        inventory.items.push(InventoryItem {
            definition_id: definition,
            tint: 1,
            id: next,
            count: 1,
            consumed_count: 0,
            stored_count: 0,
            last_cast_time: i64::from(i32::MAX),
            rental: None,
            ability_count: 0,
            activate_enabled: false,
        });
        next += 1;
    }
    inventory.next_instance_id = None;
    assert_eq!(inventory.items.len(), MAX_ITEMS);
}

/// The canonical opening fixture with the synthetic one-goal mining board on top.
pub struct Fixture(ChainFixture);

impl Fixture {
    pub fn new() -> Self {
        let fixture = ChainFixture::new();
        fs::write(
            fixture.path().join(mining::CONTENT_FILE),
            synthetic_mining().to_string(),
        )
        .expect("synthetic mining board writes");
        Self(fixture)
    }

    pub fn path(&self) -> &Path {
        self.0.path()
    }

    pub fn game(&self, active: i32, unlocked: &[i32]) -> Gameplay {
        self.game_as(CHARACTER, active, unlocked)
    }

    pub fn game_as(&self, character_id: u64, active: i32, unlocked: &[i32]) -> Gameplay {
        let mut state = Gameplay::load(self.path(), None)
            .expect("opening reference loads")
            .state();
        state.profiles = Some(profile_state(active, unlocked));
        self.0.game_as(character_id, Some(state))
    }

    pub fn game_with_state(&self, state: GameplayState) -> Gameplay {
        self.0.game_as(CHARACTER, Some(state))
    }

    /// Seed the character's gameplay payload with a full bag.
    pub fn game_with_full_bag(&self, active: i32, unlocked: &[i32]) -> Gameplay {
        let mut state = Gameplay::load(self.path(), None)
            .expect("opening reference loads")
            .state();
        state.profiles = Some(profile_state(active, unlocked));
        fill_bag(&mut state.inventory);
        self.game_with_state(state)
    }

    /// The production save envelope, not a `GameplayState` proxy.
    pub fn save_and_reload(&self, game: &Gameplay, character: &CharacterState) -> Gameplay {
        let path = self.path().join("character.json");
        let snapshot = game_store::Snapshot {
            character: character.clone(),
            gameplay: Some(game.state()),
        };
        game_store::save(&path, &snapshot).expect("the save envelope writes");
        let loaded =
            game_store::load_or_create(&path, character.clone()).expect("the envelope loads");
        assert_eq!(loaded.character, *character);
        self.0.game_as(CHARACTER, loaded.gameplay)
    }
}

pub fn word(packet: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(packet[offset..offset + 4].try_into().unwrap())
}

pub fn signed_word(packet: &[u8], offset: usize) -> i32 {
    i32::from_le_bytes(packet[offset..offset + 4].try_into().unwrap())
}

pub fn offer_packet(outcome: &Outcome, quest_id: u32) -> Vec<u8> {
    outcome
        .replies
        .iter()
        .find(|packet| has_family(packet, 49, 1, Some(quest_id)))
        .cloned()
        .expect("49/1 quest offer")
}

/// 49/1 quest at 6, intro at 10, title at 14/18, description at 22 (`00c7b7a0`).
pub fn assert_offer_ids(packet: &[u8], quest_id: u32, intro: u32, title: u32, description: u32) {
    assert_eq!(word(packet, 6), quest_id);
    assert_eq!(word(packet, 10), intro, "49/1 intro id");
    assert_eq!(word(packet, 14), title, "49/1 title id");
    assert_eq!(word(packet, 18), title, "49/1 title id (card)");
    assert_eq!(word(packet, 22), description, "49/1 description id");
}

/// A Talk holds the completed quest's 49/13; the 26/6 dialogue choice releases it. The
/// end screen names the clicked runtime actor and carries the quest's end text.
pub fn open_end_screen(
    game: &mut Gameplay,
    character: &CharacterState,
    target: u64,
    quest_id: u32,
    end_text: u32,
    now: u64,
) -> Vec<u8> {
    let dialogue = talk(game, character, target, now);
    assert!(
        !dialogue
            .replies
            .iter()
            .any(|packet| has_family(packet, 49, 13, None)),
        "the end screen opens only after the dialogue choice"
    );
    release_end_screen(game, character, quest_id, target, end_text, now + 2)
}

pub fn copper_nodes() -> Vec<(u64, [f32; 3])> {
    let nodes = GatheringNodes::load(&content_dir())
        .expect("node content loads")
        .expect("node content exists");
    let mut found: Vec<_> = nodes
        .actors()
        .filter(|actor| actor.name == "Copper Vein")
        .map(|actor| (actor.guid, actor.position))
        .collect();
    found.sort_by_key(|(guid, _)| *guid);
    found
}

pub fn copper_ore(game: &Gameplay) -> u32 {
    game.state()
        .inventory
        .items
        .iter()
        .filter(|item| item.definition_id == COPPER_ORE as i32 && item.rental.is_none())
        .map(|item| item.count)
        .sum()
}

// Client 39/5 RequestStartGame writes subtype, current instance and two -1 words;
// 39/14 SendMiniGameFlashPayload carries the movie's NUL-terminated text.
pub fn type8_frame(sub: u8, instance: i32) -> Vec<u8> {
    let mut packet = vec![39, 0, sub];
    packet.extend(instance.to_le_bytes());
    packet.extend((-1_i32).to_le_bytes());
    packet.extend((-1_i32).to_le_bytes());
    packet
}

pub fn type8_payload(instance: i32, text: &str) -> Vec<u8> {
    let mut packet = type8_frame(14, instance);
    packet.extend(((text.len() + 1) as i32).to_le_bytes());
    packet.extend(text.as_bytes());
    packet.push(0);
    packet
}

pub fn active_mining(game: &Gameplay) -> Option<i32> {
    game.state()
        .mining
        .as_ref()?
        .active
        .as_ref()
        .map(|run| run.instance_id)
}

pub fn play_mining_round(
    game: &mut Gameplay,
    character: &CharacterState,
    instance: i32,
    base: u64,
) {
    for (step, packet) in [
        type8_frame(5, instance),
        type8_payload(instance, "OnConnectMsg"),
        type8_payload(instance, "OnStartGameMsg"),
        type8_payload(instance, "OnStartLevelMsg"),
    ]
    .iter()
    .enumerate()
    {
        send(game, character, packet, base + 100 * step as u64);
    }
    send(
        game,
        character,
        &type8_payload(instance, "OnLevelEventMsg\t50\t3\t10"),
        base + 600,
    );
    send(
        game,
        character,
        &type8_payload(instance, "OnNextLevelMsg"),
        base + 700,
    );
}

/// One ordinary copper-vein gather: click the node where `character` stands, play the
/// Mining18 round through its real payloads, close the results and let the tick hide the
/// paid node.
pub fn gather_copper(
    game: &mut Gameplay,
    character: &CharacterState,
    guid: u64,
    position: [f32; 3],
    now: &mut u64,
) -> Option<i32> {
    let at_node = character_at_for(character.guid, 0, position);
    *now += 100;
    game.tick(&at_node, *now).expect("node tick");
    send(game, &at_node, &npc_interact(guid), *now);
    let instance = active_mining(game)?;
    *now += 100;
    play_mining_round(game, &at_node, instance, *now);
    *now += 1_000;
    send(game, &at_node, &CLOSE_GAME, *now);
    *now += 100;
    game.tick(&at_node, *now).expect("node sync tick");
    Some(instance)
}
