//! F287: Reed Stillwater's Fisherman entry chapter ("Learn to Fish!" 3070003 and
//! "Testing the Waters" 3070004) driven through the real NPC -> quest -> fishing path.
//!
//! T244 gated both records on the *active* Fisherman profile 137. F270 removed the gate
//! on a false premise: the active play package
//! (`lab/bundles/playtest-20261016T115811Z/content/profiles.json`) grants
//! `starter_items.137 = [68430, 68360, 68465, 68395]`. Learning 137 at Reed therefore
//! equips the Chest (68360) and Legs (68465), the 26/13 switch succeeds, and the chapter
//! is offered only while Fisherman is active. F287 restores the gate; the fixture starts
//! from the reviewed opening `initial_state` (active Adventurer 1, unlocked {1,43}) and
//! replaces only the stale opening `profiles.json` with the active package's starter
//! tables, so no test seeds `active` or `unlocked`.
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
//! - Family 39 base `008a4420`: family, subtype, then id, `-1`, game id.
//! - 39/5 start `009be7d0` and 39/6 quit `009be8e0` both call `009bdc60(sub, -1, -1)`.
//! - 39/16 offer constructor `009b9c70`; body reader `009bfc50` / record `009bdd70`.
//! - 39/18 game-over reader `009b7ec0`, handler `009ad480`: won byte + three i32.
//! - 39/14 payload reader `009b89e0`: a length-prefixed NUL-terminated string passed to
//!   `MiniGameFlash:OnMessage`. The server's `OnGameWonMsg` payload is `(R)`: no client
//!   `OnGameWonMsg` literal exists, and the test only proves the server ignores it.
//!
//! The forged 39/18 and 39/14 forms are built byte-for-byte from the server's own
//! writers (`mining::game_over`, `mining::payload_packet_for`): a client cannot reach
//! `npc_minigame_win`, which only inspects server-generated replies after a paid catch.
//!
//! The catch event is the server's own `QuestEvent::FinishMinigame { minigame_id }`
//! from `fishing_service::settle_catch`, where `minigame_id` is the spot's
//! `game_id` (599 for Reed's Sacred Grove Shallows), never the framework type 21.

#[path = "support/fishing_packets.rs"]
mod fishing_packets;
#[path = "support/npc_quest_chain.rs"]
mod npc_quest_chain;

use fishing_packets::{
    add_fish_definitions, cast_at, fish_item_ids, handle_bytes, lifecycle, reel, register, request,
};
use npc_quest_chain::*;
use osfr_tools::{
    character_store::CharacterState,
    fishing::Handle,
    gameplay::Outcome,
    npc_interaction::{decode_dialogue_26_3, decode_quest_offer_49_1},
    quest_engine::{QuestProgress, QuestStatus},
};
use serde_json::{Value, json};
use std::{
    collections::BTreeSet,
    fs, io,
    path::{Path, PathBuf},
};

const REED: u64 = 9_000_000_181;
/// An ordinary spawned Stillwater Crossing vendor near Reed: a real in-range
/// NPC that must not advance Reed's quest.
const NEWMAN: u64 = 9_500_000_793;
const DEFAULT_PROFILE: i32 = 1;
const BRAWLER_PROFILE: i32 = 43;
const FISHERMAN_PROFILE: i32 = 137;
const LEARN_TO_FISH: u32 = 3_070_003;
const TESTING_THE_WATERS: u32 = 3_070_004;
const SACRED_GROVE_GAME_ID: i32 = 599;
const STILLWATER_ZONE: u32 = 14;
const ACTOR: Handle = Handle {
    id: 0x1122_3344,
    seq: 0x5566_7788,
};
/// The fishing service's timer base; bite 1,500 ms, nibble 900 ms, reel window 7 s.
const FISH_T0: u64 = 1_000_000;

// ---- content lookups ----------------------------------------------------------------------

/// Reed's trainer row in `job_trainers.json` (profile 137): the production label the
/// test selects, never a hand-copied string id.
fn fisherman_trainer_label() -> u32 {
    read_json(&content_dir().join("job_trainers.json"))["trainers"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["profile"] == json!(FISHERMAN_PROFILE))
        .expect("Reed teaches Fisherman")["label_string_id"]
        .as_u64()
        .unwrap() as u32
}

/// The fishing menu label is content (`fishing_spots.json`), as the service reads it.
fn fish_menu_label() -> u32 {
    read_json(&content_dir().join("fishing_spots.json"))["menu_string_id"]
        .as_u64()
        .unwrap() as u32
}

fn reed_position() -> [f32; 3] {
    content_position("stillwater_crossing.json", REED)
}

fn newman_position() -> [f32; 3] {
    content_position("stillwater_crossing.json", NEWMAN)
}

/// The active play package named by the F270 review. Only its `profiles.json` is used:
/// the reviewed opening fixture supplies the inventory, opening and campaign content.
fn active_play_profiles() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../lab/bundles/playtest-20261016T115811Z/content/profiles.json")
}

/// Replace the stale opening starter table with the active package's. The stale fixture
/// has only six starter tables and no 137, which is what made F270 remove the gate.
fn use_active_starter_tables(fixture: &Fixture) {
    let source = active_play_profiles();
    let profiles = read_json(&source);
    assert_eq!(
        profiles["starter_items"]["137"],
        json!([68_430, 68_360, 68_465, 68_395]),
        "the active package dresses Fisherman (137)"
    );
    fs::copy(&source, fixture.path().join("profiles.json")).unwrap();
}

// ---- forged / server-shaped completion frames ---------------------------------------------

/// A forged framework game-over, the full server writer shape (39/18): instance, -1,
/// the real 599 game id, won byte, then `0`, elapsed, score. Client reader `009b7ec0`
/// (shared record reader) and handler `009ad480` read a won byte plus three i32.
fn forged_game_over(instance: i32) -> Vec<u8> {
    let mut packet = vec![39, 0, 18];
    for value in [instance, -1, SACRED_GROVE_GAME_ID] {
        packet.extend(value.to_le_bytes());
    }
    packet.push(1);
    for value in [0_i32, 0, 0] {
        packet.extend(value.to_le_bytes());
    }
    packet
}

/// A forged 39/14 `OnGameWonMsg`, the full server writer shape
/// (`mining::payload_packet_for`): instance, -1, the real 599 game id, length, text,
/// terminator. Client reader `009b89e0` passes the string to
/// `MiniGameFlash:OnMessage`; `OnGameWonMsg` is the movie's own event name, so this
/// server payload form is `(R)`. The test only proves the server ignores the frame.
fn forged_game_won(instance: i32) -> Vec<u8> {
    let text = b"OnGameWonMsg";
    let mut packet = vec![39, 0, 14];
    for value in [instance, -1, SACRED_GROVE_GAME_ID] {
        packet.extend(value.to_le_bytes());
    }
    packet.extend(((text.len() + 1) as i32).to_le_bytes());
    packet.extend(text);
    packet.push(0);
    packet
}

// ---- assertions on the server's own packets ----------------------------------------------

fn find<'a>(packets: &'a [Vec<u8>], prefix: &[u8]) -> Option<&'a Vec<u8>> {
    packets.iter().find(|packet| packet.starts_with(prefix))
}

fn offer_fields(packet: &[u8]) -> (u32, u32, u32, u32, u32) {
    // 49/1 body reader 0x00c7b7a0, decoded by the production decoder. Wire dword 2
    // (`field_2`) is not shown by the client; the third is the card name.
    let offer = decode_quest_offer_49_1(packet).expect("49/1 decodes");
    (
        offer.quest_id,
        offer.intro_text_string_id,
        offer.name_string_id,
        offer.description_string_id,
        offer.field_2,
    )
}

fn quest_choice_title(rows: &[[u32; 5]]) -> Option<u32> {
    rows.iter()
        .find(|row| row[0] == QUEST_CHOICE as u32)
        .map(|row| row[2])
}

fn dialogue_rows(packet: &[u8]) -> Vec<[u32; 5]> {
    decode_dialogue_26_3(packet).expect("26/3 decodes").choices
}

// ---- the rig ------------------------------------------------------------------------------

struct Rig {
    game: osfr_tools::gameplay::Gameplay,
    character: CharacterState,
    fixture: Fixture,
    now: u64,
}

impl Rig {
    fn new() -> Self {
        Self::for_character(CHARACTER)
    }

    /// The reviewed opening fixture plus the active package's starter tables, the real
    /// fishing spot file and fish definitions; the profile/inventory `initial_state` is
    /// the reviewed opening snapshot.
    fn for_character(guid: u64) -> Self {
        let fixture = Fixture::for_character(guid);
        use_active_starter_tables(&fixture);
        fs::copy(
            content_dir().join("fishing_spots.json"),
            fixture.path().join("fishing_spots.json"),
        )
        .unwrap();
        add_fish_definitions(fixture.path());
        let game = fixture.game_as(guid, None);
        Self {
            game,
            character: character_at_for(guid, STILLWATER_ZONE, reed_position()),
            fixture,
            now: 10_000,
        }
    }

    fn stamp(&mut self) -> u64 {
        self.now += 1_000;
        self.now
    }

    fn send_at(&mut self, packet: &[u8], now: u64) -> io::Result<Outcome> {
        self.game.handle_in_world(packet, &self.character, now)
    }

    fn send(&mut self, packet: &[u8]) -> Outcome {
        let now = self.stamp();
        self.send_at(packet, now).expect("in-world request")
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

    fn interact_at(&mut self, target: u64) -> Outcome {
        let now = self.stamp();
        interact(&mut self.game, &self.character, target, now)
    }

    /// Select the generic Talk option (option id 1) and return the 26/3 dialogue rows.
    fn talk_at(&mut self, target: u64) -> Vec<[u32; 5]> {
        let now = self.stamp();
        let outcome = talk(&mut self.game, &self.character, target, now);
        let dialogue =
            find(&outcome.replies, &[26, 0, 3, 0]).expect("26/10 Talk opens the 26/3 dialogue");
        dialogue_rows(dialogue)
    }

    /// Learn Fisherman through Reed's production trainer menu, then activate it with the
    /// real 26/13 switch. The active package dresses 137, so the switch succeeds; F270's
    /// "outfit is unavailable" refusal was a stale-fixture artefact.
    fn learn_fisherman(&mut self) {
        assert_eq!(
            active_job(&self.game),
            DEFAULT_PROFILE,
            "the real initial_state is active Adventurer"
        );
        let menu = self.interact_at(REED);
        let learn = menu_option(&menu, fisherman_trainer_label());
        let learned = self.send(&npc_select(REED, learn));
        assert!(learned.changed, "Reed's trainer choice must commit");
        let text = String::from_utf8_lossy(&learned.replies.concat()).into_owned();
        assert!(
            text.contains("You learned the Fisherman job"),
            "Reed must confirm the trainer unlock: {text}"
        );
        assert!(
            !text.contains("cannot be selected yet"),
            "the active package dresses 137, so the trainer must not report a missing outfit: {text}"
        );
        let state = self.game.state();
        let profiles = state.profiles.as_ref().unwrap();
        assert!(
            profiles.unlocked.contains(&FISHERMAN_PROFILE),
            "Reed must unlock Fisherman"
        );
        assert_eq!(
            profiles.active, DEFAULT_PROFILE,
            "learning a job must not activate it"
        );
        for slot in [3, 4] {
            assert!(
                state
                    .inventory
                    .equipment
                    .iter()
                    .any(|entry| entry.profile_id == FISHERMAN_PROFILE
                        && entry.slot == slot
                        && entry.item_id != 0),
                "the learned job must equip starter slot {slot}"
            );
        }

        // The 26/13 switch now succeeds and sends the 38/21 activation.
        let switched = self.send(&select_job(FISHERMAN_PROFILE));
        assert!(
            switched.changed,
            "a dressed, unlocked Fisherman must activate"
        );
        assert!(
            switched
                .replies
                .iter()
                .any(|packet| packet.starts_with(&[38, 0, 21, 0])),
            "the switch must send the 38/21 activation"
        );
        assert_eq!(active_job(&self.game), FISHERMAN_PROFILE);
    }

    /// A real 26/13 switch to the Brawler (its starter outfit is complete).
    fn switch_to_brawler(&mut self) {
        let switched = self.send(&select_job(BRAWLER_PROFILE));
        assert!(switched.changed, "Brawler must activate");
        assert_eq!(active_job(&self.game), BRAWLER_PROFILE);
    }

    fn offer(&mut self, quest_id: u32) -> Outcome {
        let now = self.stamp();
        offer(&mut self.game, &self.character, REED, quest_id, now)
    }

    fn accept_offer(&mut self, quest_id: u32) -> Outcome {
        let now = self.stamp();
        accept(&mut self.game, &self.character, quest_id, now)
    }

    fn take_end(&mut self, quest_id: u32, end_text: u32) -> Vec<u8> {
        let _ = self.talk_at(REED);
        assert_eq!(
            pending_turnin(&self.game),
            Some(quest_id),
            "the final Talk must hold the 49/13 end screen"
        );
        let now = self.stamp();
        release_end_screen(
            &mut self.game,
            &self.character,
            quest_id,
            REED,
            end_text,
            now,
        )
    }

    fn turn_in(&mut self, quest_id: u32, coins_award: u32, xp_award: u32) {
        let now = self.stamp();
        finish_quest(
            &mut self.game,
            &self.character,
            quest_id,
            FISHERMAN_PROFILE,
            coins_award,
            xp_award,
            now,
        );
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

    fn coins(&self) -> u32 {
        coins(&self.game)
    }

    fn fisherman_xp(&self) -> u32 {
        profile_xp(&self.game, FISHERMAN_PROFILE)
    }

    fn save(&self) -> Vec<u8> {
        saved(&self.game)
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
        let game = fixture.game_as(character.guid, Some(save));
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
    let offer = rig.offer(LEARN_TO_FISH);
    assert_eq!(
        offer_fields(find(&offer.replies, &[49, 0, 1, 0]).unwrap()).0,
        LEARN_TO_FISH
    );
    rig.accept_offer(LEARN_TO_FISH);
    rig.take_end(LEARN_TO_FISH, 0);
    rig.turn_in(LEARN_TO_FISH, 10, 25);
    assert_eq!(rig.status(LEARN_TO_FISH), QuestStatus::Completed);
}

/// Accept 3070004 (after 3070003) so its catch objective is active.
fn accept_testing_the_waters(rig: &mut Rig) {
    let offer = rig.offer(TESTING_THE_WATERS);
    assert_eq!(
        offer_fields(find(&offer.replies, &[49, 0, 1, 0]).unwrap()),
        (TESTING_THE_WATERS, 408_258, 408_259, 408_260, 408_259)
    );
    rig.accept_offer(TESTING_THE_WATERS);
    assert_eq!(rig.status(TESTING_THE_WATERS), QuestStatus::Active);
    assert_eq!(counts(&rig.game, TESTING_THE_WATERS), vec![0, 0]);
}

/// Open Reed's menu, take the fishing trip, and return (39/16 offer, instance id).
fn enter_fishing(rig: &mut Rig) -> (Vec<u8>, i32) {
    let menu = rig.interact_at(REED);
    let fish = menu_option(&menu, fish_menu_label());
    let replies = rig.send(&npc_select(REED, fish)).replies;
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
    let go = rig.send(&lifecycle(5, instance));
    assert!(
        find(&go.replies, &[39, 0, 17]).is_some(),
        "39/5 starts the round (39/17), not an empty run: {:?}",
        go.replies
    );
    let registered = rig.send(&register(ACTOR));
    assert!(
        find(&registered.replies, &[138, 0, 3, 0]).is_some(),
        "138/2 registration is acknowledged"
    );
    instance
}

fn cast_and_hook(rig: &mut Rig) {
    let now = FISH_T0 + 10;
    rig.send_at(&cast_at(ACTOR, reed_position(), 10.0), now)
        .expect("cast accepted");
    rig.tick(FISH_T0 + 1_510);
    rig.tick(FISH_T0 + 2_410);
}

fn leave_round(rig: &mut Rig, instance: i32) -> Outcome {
    rig.send(&lifecycle(6, instance))
}

// ---- gates --------------------------------------------------------------------------------

#[test]
fn a_normal_character_learns_fisherman_and_reaches_the_chapter() {
    let mut rig = Rig::new();
    {
        let state = rig.game.state();
        let profiles = state.profiles.as_ref().unwrap();
        assert_eq!(
            profiles.active, DEFAULT_PROFILE,
            "the real initial_state is active Adventurer"
        );
        assert_eq!(
            profiles.unlocked,
            BTreeSet::from([DEFAULT_PROFILE, BRAWLER_PROFILE]),
            "the fixture must not seed Fisherman"
        );
    }

    // The active-job gate is the only way in: before Reed's trainer, a Talk has
    // nothing eligible and changes nothing.
    let mut now = rig.stamp();
    assert_no_offer(
        &mut rig.game,
        &rig.character,
        REED,
        &mut now,
        "before learning Fisherman",
    );
    rig.now = now;

    rig.learn_fisherman();

    let coins = rig.coins();
    let xp = rig.fisherman_xp();
    let rows = rig.talk_at(REED);
    assert_eq!(
        quest_choice_title(&rows),
        Some(417_055),
        "Learn to Fish! is offered after the real trainer and job switch"
    );
    let offer = rig.offer(LEARN_TO_FISH);
    assert_eq!(
        offer_fields(find(&offer.replies, &[49, 0, 1, 0]).unwrap()),
        (LEARN_TO_FISH, 417_054, 417_055, 417_056, 417_055),
        "49/1 quest id, intro, card title, description"
    );
    rig.accept_offer(LEARN_TO_FISH);
    assert_eq!(rig.status(LEARN_TO_FISH), QuestStatus::Active);
    rig.take_end(LEARN_TO_FISH, 0);
    rig.turn_in(LEARN_TO_FISH, 10, 25);
    assert_eq!(rig.coins(), coins + 10);
    assert_eq!(rig.fisherman_xp(), xp + 25);
    assert_eq!(rig.status(LEARN_TO_FISH), QuestStatus::Completed);
}

#[test]
fn the_chapter_is_gated_on_the_active_fisherman() {
    let mut rig = Rig::new();
    rig.learn_fisherman();
    rig.switch_to_brawler();
    let xp = rig.fisherman_xp();
    let brawler_xp = profile_xp(&rig.game, BRAWLER_PROFILE);

    // Fisherman is unlocked but no longer active: the content job gate hides the chapter,
    // and the refusal keeps the open menu usable and the save bytes unchanged.
    let mut now = rig.stamp();
    assert_no_offer(
        &mut rig.game,
        &rig.character,
        REED,
        &mut now,
        "Fisherman unlocked, Brawler active",
    );
    rig.now = now;
    assert!(
        rig.progress(LEARN_TO_FISH).is_none(),
        "a gated quest is not even known"
    );

    // Switching back to the dressed Fisherman opens the chapter again, and the reward
    // pays the explicit 137 profile, never the active Brawler.
    let switched = rig.send(&select_job(FISHERMAN_PROFILE));
    assert!(switched.changed, "the dressed Fisherman must re-activate");
    let rows = rig.talk_at(REED);
    assert_eq!(quest_choice_title(&rows), Some(417_055));
    rig.offer(LEARN_TO_FISH);
    rig.accept_offer(LEARN_TO_FISH);
    rig.take_end(LEARN_TO_FISH, 0);
    rig.turn_in(LEARN_TO_FISH, 10, 25);
    assert_eq!(rig.fisherman_xp(), xp + 25);
    assert_eq!(
        profile_xp(&rig.game, BRAWLER_PROFILE),
        brawler_xp,
        "the Brawler must not gain Fisherman quest XP"
    );
}

#[test]
fn reed_offers_the_two_fisherman_quests_in_order() {
    let mut rig = Rig::new();
    rig.learn_fisherman();

    // The guide menu already exposes the fishing trip; the quest itself opens from Talk.
    let menu = rig.interact_at(REED);
    assert_eq!(
        menu_option(&menu, TALK_LABEL),
        1,
        "Talk is the first option"
    );
    let _fish = menu_option(&menu, fish_menu_label());

    let rows = rig.talk_at(REED);
    assert_eq!(quest_choice_title(&rows), Some(417_055));
    rig.offer(LEARN_TO_FISH);
    rig.accept_offer(LEARN_TO_FISH);
    assert_eq!(rig.status(LEARN_TO_FISH), QuestStatus::Active);
    assert_eq!(rig.progress(LEARN_TO_FISH).unwrap().counts, [0]);
    assert!(
        rig.progress(TESTING_THE_WATERS).is_none(),
        "3070004 is not even offered before 3070003 completes"
    );

    // A forged 49/2 for the follow-up is refused without changing the save.
    let now = rig.stamp();
    assert_packet_changes_nothing(
        &mut rig.game,
        &rig.character,
        &quest_reply(TESTING_THE_WATERS, true),
        now,
        "forged 49/2 before 3070003 completes",
    );

    // 3070003 has no end-text id, so its 49/13 end text is 0.
    rig.take_end(LEARN_TO_FISH, 0);
    rig.turn_in(LEARN_TO_FISH, 10, 25);
    assert_eq!(rig.status(LEARN_TO_FISH), QuestStatus::Completed);

    let rows = rig.talk_at(REED);
    assert_eq!(quest_choice_title(&rows), Some(408_259));
    let offer = rig.offer(TESTING_THE_WATERS);
    assert_eq!(
        offer_fields(find(&offer.replies, &[49, 0, 1, 0]).unwrap()),
        (TESTING_THE_WATERS, 408_258, 408_259, 408_260, 408_259)
    );
    rig.accept_offer(TESTING_THE_WATERS);
    assert_eq!(rig.status(TESTING_THE_WATERS), QuestStatus::Active);
}

#[test]
fn learning_then_catching_and_turning_in_pays_once() {
    let mut rig = Rig::new();
    rig.learn_fisherman();
    let coins = rig.coins();
    let xp = rig.fisherman_xp();
    finish_learn_to_fish(&mut rig);
    assert_eq!(rig.coins(), coins + 10);
    assert_eq!(rig.fisherman_xp(), xp + 25);
    assert_eq!(rig.progress(LEARN_TO_FISH).unwrap().counts, [1]);
    accept_testing_the_waters(&mut rig);

    let instance = start_round(&mut rig);
    cast_and_hook(&mut rig);
    let caught = rig
        .send_at(&reel(ACTOR), FISH_T0 + 2_420)
        .expect("reel accepted");
    assert!(
        find(&caught.replies, &[138, 0, 14, 0]).is_some(),
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

    rig.take_end(TESTING_THE_WATERS, 408_261);
    rig.turn_in(TESTING_THE_WATERS, 10, 25);
    assert_eq!(rig.coins(), coins + 20);
    assert_eq!(rig.fisherman_xp(), xp + 60);
    assert_eq!(rig.status(TESTING_THE_WATERS), QuestStatus::Completed);
    assert_eq!(rig.game.state().npc_quests.unwrap().pending_turnin, None);

    // Replays pay nothing and the completed chapter is not offered again.
    let before = rig.save();
    let replayed = rig.send(&quest_end_reply(TESTING_THE_WATERS));
    assert!(find(&replayed.replies, &[49, 0, 4, 0]).is_none());
    assert_eq!(rig.save(), before);
    let rows = rig.talk_at(REED);
    assert!(
        rows.iter().all(|row| row[0] != QUEST_CHOICE as u32),
        "a completed chapter is not re-offered: {rows:?}"
    );
    assert_eq!(rig.coins(), coins + 20);
    assert_eq!(rig.fisherman_xp(), xp + 60);
    assert_eq!(rig.fishing_items(), 1);
}

#[test]
fn a_quit_escape_or_disconnect_does_not_count_the_catch() {
    let mut rig = Rig::new();
    rig.learn_fisherman();
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
    let caught = rig
        .send_at(&reel(ACTOR), FISH_T0 + 2_420)
        .expect("reel accepted");
    assert!(find(&caught.replies, &[138, 0, 14, 0]).is_some());
    assert_eq!(rig.progress(TESTING_THE_WATERS).unwrap().counts, [1, 0]);
    leave_round(&mut rig, instance);
}

#[test]
fn a_forged_or_missing_completion_changes_nothing() {
    let mut rig = Rig::new();
    rig.learn_fisherman();
    finish_learn_to_fish(&mut rig);
    accept_testing_the_waters(&mut rig);
    let coins = rig.coins();
    let xp = rig.fisherman_xp();

    // A real, in-range NPC that is not Reed cannot advance the return step: the refusal
    // keeps the menu usable and the save bytes equal.
    rig.character.position = newman_position();
    let mut now = rig.stamp();
    assert_no_offer(
        &mut rig.game,
        &rig.character,
        NEWMAN,
        &mut now,
        "wrong in-range NPC",
    );
    rig.character.position = reed_position();
    rig.now = now;

    // A replayed 49/14 with no open end screen is refused.
    let now = rig.stamp();
    assert_packet_changes_nothing(
        &mut rig.game,
        &rig.character,
        &quest_end_reply(TESTING_THE_WATERS),
        now,
        "replayed 49/14",
    );

    // A character who never learned Fisherman has no pending end screen to settle.
    let mut stranger = Rig::new();
    let stranger_coins = stranger.coins();
    let now = stranger.stamp();
    assert_packet_changes_nothing(
        &mut stranger.game,
        &stranger.character,
        &quest_end_reply(LEARN_TO_FISH),
        now,
        "a stranger's 49/14",
    );
    assert_eq!(stranger.coins(), stranger_coins);
    assert_eq!(stranger.fisherman_xp(), 0);

    // A live round: forged completion frames are refused and leave it intact.
    let instance = start_round(&mut rig);
    cast_and_hook(&mut rig);
    // The shared busy table: an active fishing round closes ordinary NPC interaction,
    // so the quest cannot be advanced while the minigame owns the player.
    let before_busy = rig.save();
    let busy = rig.interact_at(REED);
    assert!(
        find(&busy.replies, &[26, 0, 9, 0]).is_none(),
        "a fishing round refuses the Reed menu"
    );
    assert_eq!(
        rig.save(),
        before_busy,
        "a refused Reed menu changes nothing"
    );

    // A client-sent fishing result is server-to-client only; 138/14 must be refused.
    let mut body = handle_bytes(ACTOR);
    body.extend(5u32.to_le_bytes());
    let now = rig.stamp();
    assert_packet_changes_nothing(
        &mut rig.game,
        &rig.character,
        &request(14, &body),
        now,
        "forged 138/14",
    );

    // The forged 39 frames carry the real 599 game id in both server-written forms, so
    // they would match a 599 objective if any client frame reached `npc_minigame_win`.
    let now = rig.stamp();
    assert_packet_changes_nothing(
        &mut rig.game,
        &rig.character,
        &forged_game_over(1),
        now,
        "forged 39/18 (game 599)",
    );
    assert_eq!(rig.progress(TESTING_THE_WATERS).unwrap().counts, [0, 0]);
    let now = rig.stamp();
    assert_packet_changes_nothing(
        &mut rig.game,
        &rig.character,
        &forged_game_won(1),
        now,
        "forged 39/14 OnGameWonMsg (game 599)",
    );
    assert_eq!(rig.progress(TESTING_THE_WATERS).unwrap().counts, [0, 0]);

    // The forged frames did not damage the live round: the real catch still counts.
    let caught = rig
        .send_at(&reel(ACTOR), FISH_T0 + 2_440)
        .expect("reel accepted");
    assert!(find(&caught.replies, &[138, 0, 14, 0]).is_some());
    assert_eq!(rig.progress(TESTING_THE_WATERS).unwrap().counts, [1, 0]);
    assert_eq!(rig.coins(), coins, "the catch pays no coins");
    assert_eq!(rig.fisherman_xp(), xp + 10);
    leave_round(&mut rig, instance);
}

#[test]
fn the_two_records_load_with_the_real_game_id_and_giver() {
    let document = read_json(&content_dir().join("quests/fisherman.json"));
    let records = document["records"].as_array().unwrap();
    let find_record = |id: u32| {
        records
            .iter()
            .find(|record| record["id"] == id)
            .unwrap_or_else(|| panic!("fisherman record {id} exists"))
    };

    let learn = find_record(LEARN_TO_FISH);
    assert_eq!(learn["enabled"], json!(true));
    assert_eq!(
        learn["job"],
        json!(FISHERMAN_PROFILE),
        "the chapter is gated on the active Fisherman profile"
    );
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
    assert_eq!(
        testing["job"],
        json!(FISHERMAN_PROFILE),
        "the follow-up carries the same active-job gate"
    );
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
    let spots = read_json(&content_dir().join("fishing_spots.json"));
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
    let trainers = read_json(&content_dir().join("job_trainers.json"));
    assert!(
        trainers["trainers"]
            .as_array()
            .unwrap()
            .iter()
            .any(|row| { row["npc"] == json!(REED) && row["profile"] == json!(FISHERMAN_PROFILE) })
    );

    let npcs = read_json(&content_dir().join("npcs/stillwater_crossing.json"));
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
fn two_independent_sessions_fish_the_same_spot() {
    let mut first = Rig::for_character(CHARACTER);
    let mut second = Rig::for_character(CHARACTER + 1);
    for rig in [&mut first, &mut second] {
        rig.learn_fisherman();
    }
    let first_coins = first.coins();
    let second_coins = second.coins();
    let first_xp = first.fisherman_xp();
    let second_xp = second.fisherman_xp();
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
    let caught = second
        .send_at(&reel(ACTOR), FISH_T0 + 2_420)
        .expect("reel accepted");
    assert!(find(&caught.replies, &[138, 0, 14, 0]).is_some());
    assert_eq!(second.fishing_items(), 1);
    leave_round(&mut second, second_instance);
    assert_eq!(first.progress(TESTING_THE_WATERS).unwrap().counts, [0, 0]);
    assert_eq!(second.progress(TESTING_THE_WATERS).unwrap().counts, [1, 0]);
    assert_eq!(first.coins(), first_coins + 10);
    assert_eq!(second.coins(), second_coins + 10);
    assert_eq!(first.fisherman_xp(), first_xp + 25);
    assert_eq!(second.fisherman_xp(), second_xp + 35);
}
