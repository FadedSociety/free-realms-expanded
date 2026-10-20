//! T244: Reed Stillwater's Fisherman entry chapter ("Learn to Fish!" 3070003 and
//! "Testing the Waters" 3070004) driven through the real NPC -> quest -> fishing path.
//!
//! Client evidence (2014 client, IDA database `lab/working/ida/FreeRealms.exe.i64`):
//! - 26/8 interact constructor `008b6bc0`: u64 actor + one flag byte.
//! - 26/10 selection writer `00a36f80`: u64 actor + i32 option id.
//! - 26/6 dialogue reply stores one i32 (constructor `009f5890`).
//! - 49/1 QuestInfo reader `00c7bb60` -> `00c7b7a0`: qid, five screen dwords (the
//!   third is the card title, the fourth the description), flag, reward bundle.
//! - 49/2 accept constructor `00a86400`: quest id + accept byte.
//! - 49/3 QuestAdd is emitted by the existing engine; 49/14 end reply constructor
//!   `00a864c0`: one quest id.
//! - 49/13 QuestEnd reader `00c7b990`: 8-byte actor, qid, end text, then bundle.
//! - Opcode 138 fishing requests: cast writer `00b6a460` (handle + vector + flag),
//!   reel writer `00b6a4d0` (handle + flag); result reader `00b66be0` carries no
//!   coin or XP field, so the quest count cannot come from a client result.
//!
//! The catch event is the server's own `QuestEvent::FinishMinigame { minigame_id }`
//! from `fishing_service::settle_catch`, where `minigame_id` is the spot's
//! `game_id` (599 for Reed's Sacred Grove Shallows), never the framework type 21.

use osfr_tools::{
    character_store::CharacterState,
    fishing::Handle,
    gameplay::Gameplay,
    quest_engine::{QuestProgress, QuestStatus},
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

const REED: u64 = 9_000_000_181;
/// An ordinary spawned Stillwater Crossing vendor near Reed: a real in-range
/// NPC that must not advance Reed's quest.
const NEWMAN: u64 = 9_500_000_793;
const FISHERMAN_PROFILE: i32 = 137;
const LEARN_TO_FISH: u32 = 3_070_003;
const TESTING_THE_WATERS: u32 = 3_070_004;
const SACRED_GROVE_GAME_ID: i32 = 599;
const REED_POSITION: [f32; 3] = [-1319.765, -65.62, 898.9357];
const NEWMAN_POSITION: [f32; 3] = [-1283.267, -57.52569, 852.1632];
const STILLWATER_ZONE: u32 = 14;
const FISH_MENU_STRING_ID: u32 = 73_552;
const TALK_CHOICE: i32 = 301;
const GOODBYE_CHOICE: u32 = 300;
const ACTOR: Handle = Handle {
    id: 0x1122_3344,
    seq: 0x5566_7788,
};
/// The fishing service's timer base; bite 1,500 ms, nibble 900 ms, reel window 7 s.
const FISH_T0: u64 = 1_000_000;
const CHARACTER: u64 = 0x0000_00a1_b2c3_d4e5;
static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(0);

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn bundled() -> PathBuf {
    crate_dir().join("data/content")
}

fn opening_reference() -> PathBuf {
    crate_dir().join("../../../lab/references/rust-opening-cooking-original-assets")
}

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn copy_tree(source: &Path, destination: &Path) {
    fs::create_dir_all(destination).unwrap();
    for entry in fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        let target = destination.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_tree(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), target).unwrap();
        }
    }
}

/// Every item id `fishing_spots.json` binds to a fish size.
fn fish_item_ids() -> std::collections::BTreeSet<i32> {
    let value: Value =
        serde_json::from_slice(&fs::read(bundled().join("fishing_spots.json")).unwrap()).unwrap();
    value["items"]
        .as_object()
        .unwrap()
        .values()
        .flat_map(|sizes| sizes.as_array().unwrap().iter())
        .map(|id| id.as_i64().unwrap() as i32)
        .filter(|id| *id > 0)
        .collect()
}

/// A private content directory: the reviewed opening reference (profiles, inventory,
/// combat, opening) plus the repository's real fishing spots and trainer catalogue.
/// The inventory is widened with a stub definition for every fish item so a catch can
/// be stored; the spot file itself is the shipped one, so 599 is the tested value.
struct Fixture(PathBuf);

impl Fixture {
    fn new(active_profile: i32) -> Self {
        let reference = opening_reference();
        assert!(
            reference.join("profiles.json").is_file() && reference.join("opening.json").is_file(),
            "the opening reference is required at {}",
            reference.display()
        );
        let dir = std::env::temp_dir().join(format!(
            "osfr-t244-{}-{}",
            std::process::id(),
            NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed)
        ));
        let _ = fs::remove_dir_all(&dir);
        copy_tree(&reference, &dir);
        fs::copy(
            bundled().join("fishing_spots.json"),
            dir.join("fishing_spots.json"),
        )
        .unwrap();
        fs::copy(
            bundled().join("job_trainers.json"),
            dir.join("job_trainers.json"),
        )
        .unwrap();

        // Fisherman active/unlocked, or Brawler active with Fisherman unlocked: the
        // second case proves the job gate follows the active job, not the unlock.
        let mut profiles: Value =
            serde_json::from_slice(&fs::read(dir.join("profiles.json")).unwrap()).unwrap();
        profiles["default_state"]["active"] = json!(active_profile);
        profiles["default_state"]["unlocked"] = json!([1, 43, FISHERMAN_PROFILE]);
        fs::write(
            dir.join("profiles.json"),
            serde_json::to_vec(&profiles).unwrap(),
        )
        .unwrap();

        let mut inventory: Value =
            serde_json::from_slice(&fs::read(dir.join("inventory.json")).unwrap()).unwrap();
        let definitions = inventory["definitions"].as_array_mut().unwrap();
        let known: std::collections::BTreeSet<i32> = definitions
            .iter()
            .filter_map(|row| row["id"].as_i64().map(|id| id as i32))
            .collect();
        for id in fish_item_ids() {
            if known.contains(&id) {
                continue;
            }
            let bytes = [id.to_le_bytes(), 8i32.to_le_bytes()].concat();
            definitions.push(json!({
                "id": id,
                "slot": 3,
                "class_id": 1,
                "default_tint": 5,
                "max_stack": 10,
                "model": "fixture.adr",
                "texture": "fixture",
                "tint_alias": "fixture",
                "effect": 0,
                "wield_type": 0,
                "serialized_hex": hex(&bytes),
                "sha256": digest(&bytes)
            }));
        }
        fs::write(
            dir.join("inventory.json"),
            serde_json::to_vec(&inventory).unwrap(),
        )
        .unwrap();
        Self(dir)
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

// ---- packets ------------------------------------------------------------------------------

fn npc_interact(target: u64) -> Vec<u8> {
    // Client 26/8 constructor 0x008b6bc0: u64 target plus one flag byte.
    let mut packet = vec![26, 0, 8, 0];
    packet.extend(target.to_le_bytes());
    packet.push(0);
    packet
}

fn npc_select(target: u64, option: i32) -> Vec<u8> {
    // Client 26/10 constructor 0x00a36f80: u64 target plus i32 choice.
    let mut packet = vec![26, 0, 10, 0];
    packet.extend(target.to_le_bytes());
    packet.extend(option.to_le_bytes());
    packet
}

fn dialogue_choice(choice: i32) -> Vec<u8> {
    // Client 26/6: one i32 response row (reader 0x009f5890).
    let mut packet = vec![26, 0, 6, 0];
    packet.extend(choice.to_le_bytes());
    packet
}

fn quest_reply(quest_id: u32, accept: bool) -> Vec<u8> {
    // Captured 49/2 wire: 6-byte header, u32 quest id, u8 accept (0x00a86400).
    let mut packet = vec![49, 0, 2, 0, 0, 0];
    packet.extend(quest_id.to_le_bytes());
    packet.push(u8::from(accept));
    packet
}

fn quest_end_reply(quest_id: u32) -> Vec<u8> {
    // 49/14 QuestEndReplyPacket constructor 0x00a864c0: one quest id.
    let mut packet = vec![49, 0, 14, 0, 0, 0];
    packet.extend(quest_id.to_le_bytes());
    packet
}

fn fishing_request(sub: u16, body: &[u8]) -> Vec<u8> {
    let mut packet = vec![138, 0];
    packet.extend(sub.to_le_bytes());
    packet.extend(body);
    packet
}

fn handle_bytes() -> Vec<u8> {
    [ACTOR.id.to_le_bytes(), ACTOR.seq.to_le_bytes()].concat()
}

fn register() -> Vec<u8> {
    let mut body = handle_bytes();
    body.push(0);
    fishing_request(2, &body)
}

fn cast_at(distance: f32) -> Vec<u8> {
    let mut body = handle_bytes();
    for coordinate in [
        REED_POSITION[0] + distance,
        REED_POSITION[1],
        REED_POSITION[2],
        1.0f32,
    ] {
        body.extend(coordinate.to_le_bytes());
    }
    body.push(0);
    fishing_request(6, &body)
}

fn reel() -> Vec<u8> {
    let mut body = handle_bytes();
    body.push(1);
    fishing_request(7, &body)
}

fn lifecycle(sub: u8, instance: i32) -> Vec<u8> {
    // Family 39 is u16 opcode + u8 subtype + i32 instance + -1 + game id.
    let mut packet = vec![39, 0, sub];
    packet.extend(instance.to_le_bytes());
    packet.extend((-1i32).to_le_bytes());
    packet.extend((-1i32).to_le_bytes());
    packet
}

// ---- assertions on the server's own packets ----------------------------------------------

fn find<'a>(packets: &'a [Vec<u8>], prefix: &[u8]) -> Option<&'a Vec<u8>> {
    packets.iter().find(|packet| packet.starts_with(prefix))
}

fn menu_options(packets: &[Vec<u8>]) -> Vec<(i32, u32)> {
    let menu = find(packets, &[26, 0, 9, 0]).expect("26/8 returns the 26/9 menu");
    let count = i32::from_le_bytes(menu[13..17].try_into().unwrap());
    (0..count)
        .map(|index| {
            let at = 17 + index as usize * 28;
            (
                i32::from_le_bytes(menu[at..at + 4].try_into().unwrap()),
                u32::from_le_bytes(menu[at + 8..at + 12].try_into().unwrap()),
            )
        })
        .collect()
}

fn menu_option(options: &[(i32, u32)], label: u32) -> i32 {
    options
        .iter()
        .find_map(|(id, option_label)| (*option_label == label).then_some(*id))
        .unwrap_or_else(|| panic!("menu has no option labelled {label}: {options:?}"))
}

fn dialogue_rows(packet: &[u8]) -> Vec<[u32; 5]> {
    assert!(packet.starts_with(&[26, 0, 3, 0]), "not a 26/3 dialogue");
    let count = u32::from_le_bytes(packet[25..29].try_into().unwrap()) as usize;
    (0..count)
        .map(|index| {
            let at = 29 + index * 20;
            let mut row = [0u32; 5];
            for (slot, value) in row.iter_mut().enumerate() {
                let start = at + slot * 4;
                *value = u32::from_le_bytes(packet[start..start + 4].try_into().unwrap());
            }
            row
        })
        .collect()
}

fn quest_choice_title(rows: &[[u32; 5]]) -> Option<u32> {
    rows.iter()
        .find(|row| row[0] == TALK_CHOICE as u32)
        .map(|row| row[2])
}

fn offer_fields(packet: &[u8]) -> (u32, u32, u32, u32, u32) {
    // 49/1 body reader 0x00c7b7a0: qid, intro, then five screen dwords; the
    // third of those (wire name) is the client card title (probe run 143124).
    assert_eq!(&packet[..6], &[49, 0, 1, 0, 0, 0]);
    let word = |at: usize| u32::from_le_bytes(packet[at..at + 4].try_into().unwrap());
    (word(6), word(10), word(18), word(22), word(14))
}

fn end_screen_fields(packet: &[u8]) -> (u32, u32) {
    // 49/13 reader 0x00c7b990: 8-byte actor at 6, quest id at 14, end text at 18.
    assert_eq!(&packet[..6], &[49, 0, 13, 0, 0, 0]);
    (
        u32::from_le_bytes(packet[14..18].try_into().unwrap()),
        u32::from_le_bytes(packet[18..22].try_into().unwrap()),
    )
}

// ---- the rig ------------------------------------------------------------------------------

struct Rig {
    game: Gameplay,
    character: CharacterState,
    fixture: Fixture,
    now: u64,
}

impl Rig {
    fn new(active_profile: i32) -> Self {
        let fixture = Fixture::new(active_profile);
        let mut game = Gameplay::load(&fixture.0, None).expect("fixture gameplay loads");
        game.bind_npc_quest_catalogue(&fixture.0, CHARACTER)
            .expect("split quest catalogue binds");
        Self {
            game,
            character: CharacterState {
                guid: CHARACTER,
                zone_id: STILLWATER_ZONE,
                position: REED_POSITION,
                rotation: [0.0, 0.0, 1.0],
            },
            fixture,
            now: 10_000,
        }
    }

    fn stamp(&mut self) -> u64 {
        self.now += 1_000;
        self.now
    }

    fn send(&mut self, packet: &[u8], now: u64) -> Vec<Vec<u8>> {
        self.send_at(packet, now).expect("in-world request")
    }

    fn send_at(&mut self, packet: &[u8], now: u64) -> std::io::Result<Vec<Vec<u8>>> {
        self.game
            .handle_in_world(packet, &self.character, now)
            .map(|outcome| outcome.replies)
    }

    fn tick(&mut self, now: u64) -> Vec<Vec<u8>> {
        self.game
            .tick(&self.character, now)
            .expect("world tick")
            .replies
    }

    fn tick_as(&mut self, character: &CharacterState, now: u64) {
        self.game.tick(character, now).expect("world tick");
    }

    fn interact(&mut self, npc: u64) -> Vec<Vec<u8>> {
        let now = self.stamp();
        self.send(&npc_interact(npc), now)
    }

    /// Select the generic Talk option (option id 1) and return the 26/3 dialogue rows.
    fn talk(&mut self, npc: u64) -> Vec<[u32; 5]> {
        let menu = self.interact(npc);
        let options = menu_options(&menu);
        assert_eq!(options[0], (1, 3300), "Talk is always the first option");
        let now = self.stamp();
        let replies = self.send(&npc_select(npc, 1), now);
        let dialogue = find(&replies, &[26, 0, 3, 0]).expect("26/10 Talk opens the 26/3 dialogue");
        dialogue_rows(dialogue)
    }

    fn choose(&mut self) -> Vec<Vec<u8>> {
        let now = self.stamp();
        self.send(&dialogue_choice(TALK_CHOICE), now)
    }

    fn take_offer(&mut self, npc: u64) -> (Vec<[u32; 5]>, Vec<u8>) {
        let rows = self.talk(npc);
        assert!(
            rows.iter().any(|row| row[0] == TALK_CHOICE as u32),
            "Talk must carry the quest choice row"
        );
        let replies = self.choose();
        let offer = find(&replies, &[49, 0, 1, 0])
            .expect("26/6 releases the 49/1 offer")
            .clone();
        (rows, offer)
    }

    fn take_end_screen(&mut self, npc: u64) -> Vec<u8> {
        let rows = self.talk(npc);
        assert!(
            rows.iter().any(|row| row[0] == TALK_CHOICE as u32),
            "Talk must carry the held end-screen choice"
        );
        let replies = self.choose();
        find(&replies, &[49, 0, 13, 0])
            .expect("26/6 releases the 49/13 end screen")
            .clone()
    }

    fn accept(&mut self, quest_id: u32) -> Vec<Vec<u8>> {
        let now = self.stamp();
        let replies = self.send(&quest_reply(quest_id, true), now);
        assert!(
            find(&replies, &[49, 0, 3, 0]).is_some(),
            "49/2 accept returns the 49/3 quest record"
        );
        replies
    }

    fn turn_in(&mut self, quest_id: u32) -> Vec<Vec<u8>> {
        let now = self.stamp();
        let replies = self.send(&quest_end_reply(quest_id), now);
        assert!(
            find(&replies, &[49, 0, 4, 0]).is_some(),
            "49/14 returns the 49/4 completion"
        );
        replies
    }

    fn coins(&self) -> u32 {
        self.game.state().inventory.coins
    }

    fn fisherman_xp(&self) -> u32 {
        self.game
            .state()
            .profiles
            .as_ref()
            .unwrap()
            .experience
            .get(&FISHERMAN_PROFILE)
            .copied()
            .unwrap_or(0)
    }

    fn progress(&self, quest_id: u32) -> Option<QuestProgress> {
        self.game
            .state()
            .npc_quests
            .and_then(|state| state.quests.get(&quest_id).cloned())
    }

    fn status(&self, quest_id: u32) -> QuestStatus {
        self.progress(quest_id)
            .unwrap_or_else(|| panic!("quest {quest_id} is known"))
            .status
    }

    /// The persisted character save, as the gateway would write it.
    fn save(&self) -> Vec<u8> {
        serde_json::to_vec(&self.game.state()).unwrap()
    }

    /// The saved state a refused fishing frame must not touch.
    fn subject_state(&self) -> (Option<QuestProgress>, u32, u32, u32) {
        (
            self.progress(TESTING_THE_WATERS),
            self.coins(),
            self.fisherman_xp(),
            self.fishing_items(),
        )
    }

    fn fishing_items(&self) -> u32 {
        let ids = fish_item_ids();
        self.game
            .state()
            .inventory
            .items
            .iter()
            .filter(|item| ids.contains(&item.definition_id) && item.rental.is_none())
            .map(|item| item.count)
            .sum()
    }

    /// Reconnect: reload the typed save into a fresh Gameplay and rebind quests.
    fn reconnect(self) -> Self {
        let save = self.game.state();
        let Rig {
            game,
            character,
            fixture,
            now,
        } = self;
        drop(game);
        let mut game = Gameplay::load(&fixture.0, Some(save)).expect("saved gameplay reloads");
        game.bind_npc_quest_catalogue(&fixture.0, character.guid)
            .expect("split quest catalogue rebinds");
        Self {
            game,
            character,
            fixture,
            now,
        }
    }
}

/// Accept 3070003 and complete it with Reed's own Talk step.
fn finish_learn_to_fish(rig: &mut Rig) {
    let (_, offer) = rig.take_offer(REED);
    assert_eq!(offer_fields(&offer).0, LEARN_TO_FISH);
    rig.accept(LEARN_TO_FISH);
    let end = rig.take_end_screen(REED);
    assert_eq!(end_screen_fields(&end).0, LEARN_TO_FISH);
    rig.turn_in(LEARN_TO_FISH);
    assert_eq!(rig.status(LEARN_TO_FISH), QuestStatus::Completed);
}

/// Accept 3070004 (after 3070003) so its catch objective is active.
fn accept_testing_the_waters(rig: &mut Rig) {
    let (_, offer) = rig.take_offer(REED);
    assert_eq!(offer_fields(&offer).0, TESTING_THE_WATERS);
    rig.accept(TESTING_THE_WATERS);
    assert_eq!(rig.status(TESTING_THE_WATERS), QuestStatus::Active);
    assert_eq!(rig.progress(TESTING_THE_WATERS).unwrap().counts, [0, 0]);
}

/// Open Reed's menu, take the fishing trip, and return (39/16 offer, instance id).
fn enter_fishing(rig: &mut Rig) -> (Vec<u8>, i32) {
    let menu = rig.interact(REED);
    let fish = menu_option(&menu_options(&menu), FISH_MENU_STRING_ID);
    let now = rig.stamp();
    let replies = rig.send(&npc_select(REED, fish), now);
    let offer = find(&replies, &[39, 0, 16])
        .expect("the guide's Fish option returns the 39/16 offer")
        .clone();
    let instance = i32::from_le_bytes(offer[3..7].try_into().unwrap());
    assert_eq!(
        i32::from_le_bytes(offer[11..15].try_into().unwrap()),
        SACRED_GROVE_GAME_ID,
        "the offer names Reed's real spot, not framework type 21"
    );
    (offer, instance)
}

/// Start the round and register the actor; ready to cast.
fn start_round(rig: &mut Rig) -> i32 {
    let (_, instance) = enter_fishing(rig);
    let now = rig.stamp();
    let go = rig.send(&lifecycle(5, instance), now);
    assert!(
        find(&go, &[41, 0, 2]).is_some(),
        "39/5 starts the encounter (41/2), not an empty run"
    );
    let now = rig.stamp();
    let registered = rig.send(&register(), now);
    assert!(
        find(&registered, &[138, 0, 3, 0]).is_some(),
        "138/2 registration is acknowledged"
    );
    instance
}

fn cast_and_hook(rig: &mut Rig) {
    let now = FISH_T0 + 10;
    rig.send(&cast_at(10.0), now);
    rig.tick(FISH_T0 + 1_510);
    rig.tick(FISH_T0 + 2_410);
}

fn leave_round(rig: &mut Rig, instance: i32) -> Vec<Vec<u8>> {
    let now = rig.stamp();
    rig.send(&lifecycle(6, instance), now)
}

// ---- gates --------------------------------------------------------------------------------

#[test]
fn reed_offers_the_two_fisherman_quests_in_order() {
    let mut rig = Rig::new(FISHERMAN_PROFILE);

    // The guide menu already exposes the fishing trip; the quest itself opens from Talk.
    let options = menu_options(&rig.interact(REED));
    assert_eq!(options[0], (1, 3300));
    assert!(
        options
            .iter()
            .any(|(_, label)| *label == FISH_MENU_STRING_ID)
    );

    let (rows, offer) = rig.take_offer(REED);
    assert_eq!(
        quest_choice_title(&rows),
        Some(417_055),
        "Learn to Fish! title"
    );
    assert_eq!(
        offer_fields(&offer),
        (LEARN_TO_FISH, 417_054, 417_055, 417_056, 417_055),
        "49/1 quest id, intro, card title, description"
    );
    rig.accept(LEARN_TO_FISH);
    assert_eq!(rig.status(LEARN_TO_FISH), QuestStatus::Active);
    let after_accept = rig.progress(LEARN_TO_FISH).unwrap();
    assert_eq!(after_accept.counts, [0]);
    assert!(
        rig.progress(TESTING_THE_WATERS).is_none(),
        "3070004 is not even offered before 3070003 completes"
    );

    // A forged 49/2 for the follow-up is refused with a byte-identical save.
    let before = rig.save();
    let now = rig.stamp();
    let forged = rig.send(&quest_reply(TESTING_THE_WATERS, true), now);
    assert!(find(&forged, &[49, 0, 3, 0]).is_none());
    assert_eq!(rig.save(), before);

    let end = rig.take_end_screen(REED);
    assert_eq!(
        end_screen_fields(&end),
        (LEARN_TO_FISH, 0),
        "no E3 end text id"
    );
    rig.turn_in(LEARN_TO_FISH);
    assert_eq!(rig.status(LEARN_TO_FISH), QuestStatus::Completed);

    let rows = rig.talk(REED);
    assert_eq!(
        quest_choice_title(&rows),
        Some(408_259),
        "Testing the Waters title"
    );
    let (_, offer) = rig.take_offer(REED);
    assert_eq!(
        offer_fields(&offer),
        (TESTING_THE_WATERS, 408_258, 408_259, 408_260, 408_259),
        "49/1 quest id, intro, card title, description"
    );
    rig.accept(TESTING_THE_WATERS);
    assert_eq!(rig.status(TESTING_THE_WATERS), QuestStatus::Active);

    // A character on another active job sees neither chapter, even with Fisherman unlocked.
    let mut brawler = Rig::new(43);
    let before = brawler.save();
    let rows = brawler.talk(REED);
    assert_eq!(rows, vec![[GOODBYE_CHOICE, 17, 8_075, 0, 18]]);
    let closed = brawler.choose();
    assert!(find(&closed, &[49, 0, 1, 0]).is_none());
    assert_eq!(brawler.save(), before, "a refused Talk changes nothing");
    let now = brawler.stamp();
    let forged = brawler.send(&quest_reply(LEARN_TO_FISH, true), now);
    assert!(find(&forged, &[49, 0, 3, 0]).is_none());
    assert_eq!(brawler.save(), before);
}

#[test]
fn learning_then_catching_and_turning_in_pays_once() {
    let mut rig = Rig::new(FISHERMAN_PROFILE);
    let coins = rig.coins();
    let xp = rig.fisherman_xp();
    finish_learn_to_fish(&mut rig);
    assert_eq!(rig.coins(), coins + 10);
    assert_eq!(rig.fisherman_xp(), xp + 25);
    assert_eq!(rig.progress(LEARN_TO_FISH).unwrap().counts, [1]);
    accept_testing_the_waters(&mut rig);

    let instance = start_round(&mut rig);
    cast_and_hook(&mut rig);
    let caught = rig.send(&reel(), FISH_T0 + 2_420);
    assert!(
        find(&caught, &[138, 0, 14, 0]).is_some(),
        "the paid catch sends the 138/14 result"
    );
    assert_eq!(rig.fishing_items(), 1, "exactly one fish is paid");
    assert_eq!(
        rig.fisherman_xp(),
        xp + 25 + 10,
        "the catch pays its own 10 Fisherman XP"
    );
    assert_eq!(rig.coins(), coins + 10, "the catch pays no coins");
    let progress = rig.progress(TESTING_THE_WATERS).unwrap();
    assert_eq!(progress.counts, [1, 0], "catch step complete, return open");
    assert_eq!(progress.status, QuestStatus::Active);
    leave_round(&mut rig, instance);

    // Disconnect after the paid catch but before the turn-in: the counter is saved.
    let mut rig = rig.reconnect();
    assert_eq!(rig.progress(TESTING_THE_WATERS).unwrap().counts, [1, 0]);
    assert_eq!(rig.fishing_items(), 1);
    assert_eq!(rig.coins(), coins + 10);

    let end = rig.take_end_screen(REED);
    assert_eq!(
        end_screen_fields(&end),
        (TESTING_THE_WATERS, 408_261),
        "49/13 end text and quest id"
    );
    rig.turn_in(TESTING_THE_WATERS);
    assert_eq!(rig.coins(), coins + 20);
    assert_eq!(rig.fisherman_xp(), xp + 60);
    assert_eq!(rig.status(TESTING_THE_WATERS), QuestStatus::Completed);
    assert_eq!(rig.game.state().npc_quests.unwrap().pending_turnin, None);

    // Replays pay nothing and the completed chapter is not offered again.
    let before = rig.save();
    let now = rig.stamp();
    let replayed = rig.send(&quest_end_reply(TESTING_THE_WATERS), now);
    assert!(find(&replayed, &[49, 0, 4, 0]).is_none());
    assert_eq!(rig.save(), before);
    let rows = rig.talk(REED);
    assert!(
        rows.iter().all(|row| row[0] != TALK_CHOICE as u32),
        "a completed chapter is not re-offered: {rows:?}"
    );
    assert_eq!(rig.coins(), coins + 20);
    assert_eq!(rig.fisherman_xp(), xp + 60);
    assert_eq!(rig.fishing_items(), 1);
}

#[test]
fn a_quit_escape_or_disconnect_does_not_count_the_catch() {
    let mut rig = Rig::new(FISHERMAN_PROFILE);
    finish_learn_to_fish(&mut rig);
    accept_testing_the_waters(&mut rig);
    let before = rig.subject_state();

    // Quit: the client leaves the run (39/6) after the hook but before the reel.
    let instance = start_round(&mut rig);
    cast_and_hook(&mut rig);
    leave_round(&mut rig, instance);
    assert_eq!(
        rig.subject_state(),
        before,
        "quit changes no saved subject state"
    );

    // Expiry: the hooked fish waits past the 7 s reel window. The round stays open
    // until the client leaves, so prove the leave path too.
    let instance = start_round(&mut rig);
    cast_and_hook(&mut rig);
    rig.tick(FISH_T0 + 10_000);
    assert_eq!(
        rig.subject_state(),
        before,
        "expiry changes no saved subject state"
    );
    assert_eq!(rig.status(TESTING_THE_WATERS), QuestStatus::Active);
    leave_round(&mut rig, instance);
    assert_eq!(rig.subject_state(), before);

    // Zone change: the spot's leash supervision ends the run for free.
    let _ = start_round(&mut rig);
    cast_and_hook(&mut rig);
    let away = CharacterState {
        zone_id: 0,
        ..rig.character.clone()
    };
    rig.tick_as(&away, FISH_T0 + 5_000);
    assert_eq!(
        rig.subject_state(),
        before,
        "leaving the zone changes no saved subject state"
    );

    // Disconnect: the round is dropped with no award; the active quest survives.
    let _ = start_round(&mut rig);
    cast_and_hook(&mut rig);
    rig.game.reset_session();
    let mut rig = rig.reconnect();
    assert_eq!(
        rig.subject_state(),
        before,
        "disconnect changes no saved subject state"
    );
    assert_eq!(rig.status(TESTING_THE_WATERS), QuestStatus::Active);

    // The chapter is still playable after every failure.
    let instance = start_round(&mut rig);
    cast_and_hook(&mut rig);
    let caught = rig.send(&reel(), FISH_T0 + 2_420);
    assert!(find(&caught, &[138, 0, 14, 0]).is_some());
    assert_eq!(rig.progress(TESTING_THE_WATERS).unwrap().counts, [1, 0]);
    leave_round(&mut rig, instance);
}

#[test]
fn a_forged_or_missing_completion_changes_nothing() {
    let mut rig = Rig::new(FISHERMAN_PROFILE);
    finish_learn_to_fish(&mut rig);
    accept_testing_the_waters(&mut rig);
    let coins = rig.coins();
    let xp = rig.fisherman_xp();

    // A real, in-range NPC that is not Reed cannot advance the return step.
    let before = rig.save();
    rig.character.position = NEWMAN_POSITION;
    let menu = rig.interact(NEWMAN);
    assert!(find(&menu, &[26, 0, 9, 0]).is_some(), "Newman is reachable");
    let rows = rig.talk(NEWMAN);
    assert_eq!(rows, vec![[GOODBYE_CHOICE, 17, 8_075, 0, 18]]);
    let closed = rig.choose();
    assert!(find(&closed, &[49, 0, 13, 0]).is_none());
    rig.character.position = REED_POSITION;
    assert_eq!(rig.save(), before, "a wrong NPC changes nothing");

    // A replayed 49/14 with no open end screen is refused.
    let now = rig.stamp();
    let replayed = rig.send(&quest_end_reply(TESTING_THE_WATERS), now);
    assert!(find(&replayed, &[49, 0, 4, 0]).is_none());
    assert_eq!(rig.save(), before, "replayed 49/14 changes nothing");

    // A non-Fisherman cannot turn in Reed's chapter (no pending end screen exists).
    let mut brawler = Rig::new(43);
    let brawler_before = brawler.save();
    let brawler_coins = brawler.coins();
    let now = brawler.stamp();
    let refused = brawler.send(&quest_end_reply(LEARN_TO_FISH), now);
    assert!(find(&refused, &[49, 0, 4, 0]).is_none());
    assert_eq!(brawler.save(), brawler_before);
    assert_eq!(brawler.coins(), brawler_coins);
    assert_eq!(brawler.fisherman_xp(), 0);

    // A live round: forged completion frames are refused and leave it intact.
    let instance = start_round(&mut rig);
    cast_and_hook(&mut rig);
    // The shared busy table: an active fishing round closes ordinary NPC interaction,
    // so the quest cannot be advanced while the minigame owns the player.
    let busy = rig.interact(REED);
    assert!(
        find(&busy, &[26, 0, 9, 0]).is_none(),
        "a fishing round refuses the Reed menu"
    );
    let live = rig.save();

    // A client-sent fishing result is server-to-client only; 138/14 must be refused.
    let mut body = handle_bytes();
    body.extend(5u32.to_le_bytes());
    let forged_result = fishing_request(14, &body);
    let _ = rig.send_at(&forged_result, FISH_T0 + 2_420);
    assert_eq!(rig.save(), live, "forged 138/14 changes nothing");

    // A forged framework game-over with type 21 must not complete a 599 objective.
    let mut forged_over = vec![39, 0, 18];
    forged_over.extend(1i32.to_le_bytes());
    forged_over.extend((-1i32).to_le_bytes());
    forged_over.extend(21i32.to_le_bytes());
    forged_over.push(1);
    let _ = rig.send_at(&forged_over, FISH_T0 + 2_430);
    assert_eq!(rig.save(), live, "forged 39/18 (type 21) changes nothing");
    assert_eq!(rig.progress(TESTING_THE_WATERS).unwrap().counts, [0, 0]);

    // The forged frames did not damage the live round: the real catch still counts.
    let caught = rig.send(&reel(), FISH_T0 + 2_440);
    assert!(find(&caught, &[138, 0, 14, 0]).is_some());
    assert_eq!(rig.progress(TESTING_THE_WATERS).unwrap().counts, [1, 0]);
    assert_eq!(rig.coins(), coins, "the catch pays no coins");
    assert_eq!(rig.fisherman_xp(), xp + 10);
    leave_round(&mut rig, instance);
}

#[test]
fn the_two_records_load_with_the_real_game_id_and_giver() {
    let content = bundled();
    let document: Value =
        serde_json::from_slice(&fs::read(content.join("quests/fisherman.json")).unwrap()).unwrap();
    let records = document["records"].as_array().unwrap();
    let find_record = |id: u32| {
        records
            .iter()
            .find(|record| record["id"] == id)
            .unwrap_or_else(|| panic!("fisherman record {id} exists"))
    };

    let learn = find_record(LEARN_TO_FISH);
    assert_eq!(learn["enabled"], json!(true));
    assert_eq!(learn["job"], json!(FISHERMAN_PROFILE));
    assert_eq!(learn["giver_npc_id"], json!(REED));
    assert_eq!(learn["prerequisite_quest_ids"], json!([]));
    let objectives = learn["objectives"].as_array().unwrap();
    assert_eq!(objectives.len(), 1);
    assert_eq!(objectives[0]["kind"], json!("talk_to_npc"));
    assert_eq!(objectives[0]["id"], json!(1));
    assert_eq!(objectives[0]["npc_id"], json!(REED));
    assert_objective_sources(objectives);
    assert_reward_shape(learn);

    let testing = find_record(TESTING_THE_WATERS);
    assert_eq!(testing["enabled"], json!(true));
    assert_eq!(testing["job"], json!(FISHERMAN_PROFILE));
    assert_eq!(testing["giver_npc_id"], json!(REED));
    assert_eq!(testing["prerequisite_quest_ids"], json!([LEARN_TO_FISH]));
    let objectives = testing["objectives"].as_array().unwrap();
    assert_eq!(objectives.len(), 2);
    assert_eq!(objectives[0]["kind"], json!("finish_minigame"));
    assert_eq!(objectives[0]["id"], json!(1));
    assert_eq!(objectives[0]["count"], json!(1));
    assert_objective_sources(objectives);
    assert_eq!(objectives[1]["kind"], json!("talk_to_npc"));
    assert_eq!(objectives[1]["id"], json!(2));
    assert_eq!(objectives[1]["npc_id"], json!(REED));
    assert_reward_shape(testing);

    // The two alternatives stay unpublished; this task only enables E3/E4.
    assert_eq!(find_record(3_070_001)["enabled"], json!(false));
    assert_eq!(find_record(3_070_002)["enabled"], json!(false));

    // Reed's fishing spot is the real game id 599, not the framework type 21.
    let spots: Value =
        serde_json::from_slice(&fs::read(content.join("fishing_spots.json")).unwrap()).unwrap();
    let reed_spot = spots["spots"]
        .as_array()
        .unwrap()
        .iter()
        .find(|spot| spot["npc"] == json!(REED))
        .expect("Reed is a fishing-spot guide");
    assert_eq!(reed_spot["area"], json!("sacred_grove_shallows"));
    assert_eq!(reed_spot["game_id"], json!(SACRED_GROVE_GAME_ID));
    assert_ne!(reed_spot["game_id"], json!(21));
    assert_eq!(objectives[0]["minigame_id"], reed_spot["game_id"]);

    // The same NPC is the Fisherman trainer and gives both records.
    let trainers: Value =
        serde_json::from_slice(&fs::read(content.join("job_trainers.json")).unwrap()).unwrap();
    assert!(
        trainers["trainers"]
            .as_array()
            .unwrap()
            .iter()
            .any(|row| { row["npc"] == json!(REED) && row["profile"] == json!(FISHERMAN_PROFILE) })
    );

    let npcs: Value =
        serde_json::from_slice(&fs::read(content.join("npcs/stillwater_crossing.json")).unwrap())
            .unwrap();
    let reed = npcs["records"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["npc_id"] == json!(REED))
        .expect("Reed is NPC content");
    assert_eq!(reed["name"], json!("Reed Stillwater"));
    assert_eq!(reed["zone_id"], json!(STILLWATER_ZONE));
    assert_eq!(reed["placement"], json!("legacy_data"), "a drawable spawn");
    assert!(reed["position"].as_array().is_some_and(|p| p.len() == 3));
    assert!(
        reed["roles"]
            .as_array()
            .unwrap()
            .contains(&json!("quest_giver"))
    );
    assert!(
        reed["quest_ids_given"]
            .as_array()
            .unwrap()
            .contains(&json!(LEARN_TO_FISH))
    );
    assert!(
        reed["quest_ids_given"]
            .as_array()
            .unwrap()
            .contains(&json!(TESTING_THE_WATERS))
    );
}

fn assert_objective_sources(objectives: &[Value]) {
    for objective in objectives {
        assert!(
            objective["source"]
                .as_array()
                .is_some_and(|source| !source.is_empty()),
            "objective {objective} has provenance"
        );
    }
}

fn assert_reward_shape(record: &Value) {
    let rewards = record["rewards"].as_array().unwrap();
    assert_eq!(rewards.len(), 1);
    assert_eq!(rewards[0]["coins"], json!(10));
    assert_eq!(rewards[0]["items"], json!([]));
    assert_eq!(rewards[0]["job_xp"], json!({"137": 25}));
    assert!(
        rewards[0]["source"]
            .as_array()
            .is_some_and(|source| !source.is_empty())
    );
}

#[test]
fn two_fishermen_fish_the_same_spot_independently() {
    let mut first = Rig::new(FISHERMAN_PROFILE);
    let mut second = Rig::new(FISHERMAN_PROFILE);
    let first_coins = first.coins();
    let second_coins = second.coins();
    for rig in [&mut first, &mut second] {
        finish_learn_to_fish(rig);
        accept_testing_the_waters(rig);
    }

    // Both rounds are live at Reed's spot at the same time.
    let first_instance = start_round(&mut first);
    let second_instance = start_round(&mut second);
    cast_and_hook(&mut first);
    cast_and_hook(&mut second);
    assert!(
        first.fishing_items() == 0 && second.fishing_items() == 0,
        "no catch yet"
    );

    // The first player quits; the second still lands the same round.
    leave_round(&mut first, first_instance);
    assert_eq!(first.fishing_items(), 0);
    let caught = second.send(&reel(), FISH_T0 + 2_420);
    assert!(find(&caught, &[138, 0, 14, 0]).is_some());
    assert_eq!(second.fishing_items(), 1);
    leave_round(&mut second, second_instance);
    assert_eq!(first.progress(TESTING_THE_WATERS).unwrap().counts, [0, 0]);
    assert_eq!(second.progress(TESTING_THE_WATERS).unwrap().counts, [1, 0]);
    assert_eq!(first.coins(), first_coins + 10);
    assert_eq!(second.coins(), second_coins + 10);
    assert_eq!(first.fisherman_xp(), 25);
    assert_eq!(second.fisherman_xp(), 35);
}
