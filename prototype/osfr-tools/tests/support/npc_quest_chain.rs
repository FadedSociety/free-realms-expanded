//! Shared fixture and packet helpers for the NPC quest-chain tests.
//!
//! Test files include this module with
//! `#[path = "support/npc_quest_chain.rs"] mod npc_quest_chain;` (grep `tests`
//! for the current consumers: the entry chains, `quest_job_xp.rs`, the
//! story-quest suites including `missing_sisters_quest.rs` and
//! `local_hero_quest.rs`, and `shared_field_kill_quest_credit.rs`). Test-only;
//! production never copies content. The fixture supplies profile/inventory
//! content; the repository's `job_trainers.json` and split quest/NPC content are
//! bound from `data/content` exactly as the bundled play package does.
#![allow(dead_code)]

#[path = "../common/content_copy.rs"]
mod content_copy;

pub use content_copy::{copy_missing, copy_tree};

use osfr_tools::{
    character_store::CharacterState,
    encounter::{FieldParticipant, SharedFieldAction, SharedFieldState},
    gameplay::{Gameplay, GameplayState, Outcome},
    gateway::Gateway,
    inventory::InventoryItem,
    npc_interaction::decode_quest_offer_49_1,
    quest_engine::QuestStatus,
};
use serde_json::Value;
use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

pub const CHARACTER: u64 = 194_017;
/// Generic Talk option label in the reconstructed 26/9 menu.
pub const TALK_LABEL: u32 = 3_300;
/// First response row of the quest dialogue (client 26/6).
pub const QUEST_CHOICE: i32 = 301;
/// (R) The production turn-in floor in `gameplay/npc.rs`: the reward
/// `bag_capacity` is `items.len().max(100)`.
pub const BAG_SLOTS: usize = 100;
/// A real inventory definition used to fill the bag; only the stack count matters.
pub const FILLER_ITEM: i32 = 75_060;

static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(0);

pub fn content_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content")
}

pub fn reference_content() -> PathBuf {
    std::env::var_os("OSFR_NPC_QUEST_CHAIN_CONTENT")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../../lab/references/opening-anvil-placement-v1/content")
        })
}

pub fn read_json(path: &Path) -> Value {
    serde_json::from_slice(
        &fs::read(path).unwrap_or_else(|error| panic!("{}: {error}", path.display())),
    )
    .unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

pub struct Fixture {
    path: PathBuf,
    character_id: u64,
}

impl Fixture {
    pub fn new() -> Self {
        Self::for_character(CHARACTER)
    }

    /// A private content copy whose `game` binds this character's quest and reward
    /// ledger; two sessions in the same fixture need separate character ids.
    pub fn for_character(character_id: u64) -> Self {
        let source = reference_content();
        assert!(
            source.join("profiles.json").is_file() && source.join("inventory.json").is_file(),
            "the quest chapter needs the opening content fixture at {}",
            source.display()
        );
        let path = std::env::temp_dir().join(format!(
            "osfr-npc-quest-chain-{}-{}",
            std::process::id(),
            NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed)
        ));
        copy_tree(&source, &path);
        fs::copy(
            content_dir().join("job_trainers.json"),
            path.join("job_trainers.json"),
        )
        .unwrap();
        Self { path, character_id }
    }

    /// Copy every repository content file this fixture does not already supply, so
    /// `Gameplay::load` reads NPCs, field actors and `jobs.json` from the fixture's own
    /// tree. Opening/profile/inventory files keep winning (`copy_missing`), and the
    /// caller may then patch the scratch copy.
    pub fn overlay_content(&self, source: &Path) {
        copy_missing(source, &self.path);
    }

    /// A copy without the opening scene and the campaign files that require it,
    /// so `Gameplay::on_location` cannot advance the opening story. Event-quest
    /// tests use it when their objective and wallet deltas must be exact.
    pub fn minimal(character_id: u64) -> Self {
        let fixture = Self::for_character(character_id);
        for name in [
            "opening.json",
            "opening-abilities.json",
            "smelting.json",
            "forging.json",
            "forging-campaign.json",
            "cooking.json",
            "cooking-campaign.json",
            "mining.json",
            "harvesting.json",
        ] {
            let path = fixture.path.join(name);
            if path.is_file() {
                fs::remove_file(&path).unwrap();
            }
        }
        fixture
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Replace the shared field population with `enemies`. The reference opening
    /// content has no `field_enemies.json`, so this also makes the private copy
    /// the session's NPC content root (the spawn catalogue, store and vendor
    /// files are copied from `data/content`) and copies T240's
    /// `field_kill_rewards.json`; `Gameplay::load` then reads the synthetic field
    /// through its production path instead of the bundled one. The split quest
    /// catalogue is still bound from `data/content` by [`Self::game_as`].
    pub fn with_field_enemies(self, enemies: &[Value]) -> Self {
        for name in [
            "zones.json",
            "npc_spawns_extra.json",
            "stores.json",
            "vendors.json",
            "field_kill_rewards.json",
        ] {
            fs::copy(content_dir().join(name), self.path.join(name)).unwrap();
        }
        copy_tree(&content_dir().join("npcs"), &self.path.join("npcs"));
        fs::write(
            self.path.join("field_enemies.json"),
            serde_json::to_vec(&serde_json::json!({
                "schema_version": 1,
                "enemies": enemies,
            }))
            .unwrap(),
        )
        .unwrap();
        self
    }

    pub fn game(&self, state: Option<GameplayState>) -> Gameplay {
        self.game_as(self.character_id, state)
    }

    /// Bind the split catalogue for a specific character, so a second session in
    /// the same fixture owns its own quest and reward ledger.
    pub fn game_as(&self, character_id: u64, state: Option<GameplayState>) -> Gameplay {
        let mut game = Gameplay::load(&self.path, state).expect("opening fixture loads");
        game.bind_npc_quest_catalogue(&content_dir(), character_id)
            .expect("split quest content loads");
        game
    }

    /// Reload the saved character through the same production bind the session uses.
    pub fn reload(&self, game: &Gameplay) -> Gameplay {
        self.game(Some(game.state()))
    }

    /// Reload through a JSON round trip, so decoding the saved state is exercised too.
    pub fn reload_serialized(&self, game: &Gameplay) -> Gameplay {
        self.reload_serialized_as(self.character_id, game)
    }

    /// As [`Self::reload_serialized`], but bind the split quest catalogue for
    /// `character_id`: a second session in the same fixture saves under its own id.
    pub fn reload_serialized_as(&self, character_id: u64, game: &Gameplay) -> Gameplay {
        let bytes = serde_json::to_vec(&game.state()).unwrap();
        let state: GameplayState = serde_json::from_slice(&bytes).unwrap();
        self.game_as(character_id, Some(state))
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

pub fn character_at_for(guid: u64, zone_id: u32, position: [f32; 3]) -> CharacterState {
    CharacterState {
        guid,
        zone_id,
        position,
        rotation: [1.0, 0.0, 0.0],
    }
}

pub fn character_at(zone_id: u32, position: [f32; 3]) -> CharacterState {
    character_at_for(CHARACTER, zone_id, position)
}

/// Fill the bag to the production capacity floor (exactly [`BAG_SLOTS`] stacks), so
/// the next new stack is refused. Existing opening items count toward the total; the
/// filler rows are real inventory instances.
pub fn fill_bag(state: &mut GameplayState) {
    let base = 100_000;
    for offset in state.inventory.items.len()..BAG_SLOTS {
        state.inventory.items.push(InventoryItem {
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
    state.inventory.next_instance_id = Some(i64::from(base) + BAG_SLOTS as i64);
}

pub fn content_position(file: &str, npc_id: u64) -> [f32; 3] {
    let document = read_json(&content_dir().join("npcs").join(file));
    let row = document["records"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["npc_id"].as_u64() == Some(npc_id))
        .unwrap_or_else(|| panic!("NPC {npc_id} is defined in {file}"));
    serde_json::from_value(row["position"].clone()).expect("NPC has a position")
}

pub fn npc_interact(target: u64) -> Vec<u8> {
    let mut packet = vec![26, 0, 8, 0];
    packet.extend(target.to_le_bytes());
    packet.push(0);
    packet
}

pub fn npc_select(target: u64, choice: i32) -> Vec<u8> {
    let mut packet = vec![26, 0, 10, 0];
    packet.extend(target.to_le_bytes());
    packet.extend(choice.to_le_bytes());
    packet
}

pub fn dialogue_choice(row: i32) -> Vec<u8> {
    let mut packet = vec![26, 0, 6, 0];
    packet.extend(row.to_le_bytes());
    packet
}

pub fn quest_reply(quest_id: u32, accept: bool) -> Vec<u8> {
    let mut packet = vec![49, 0, 2, 0, 0, 0];
    packet.extend(quest_id.to_le_bytes());
    packet.push(u8::from(accept));
    packet
}

pub fn quest_end_reply(quest_id: u32) -> Vec<u8> {
    let mut packet = vec![49, 0, 14, 0, 0, 0];
    packet.extend(quest_id.to_le_bytes());
    packet
}

pub fn select_job(profile: i32) -> Vec<u8> {
    let mut packet = vec![26, 0, 13, 0];
    packet.extend(profile.to_le_bytes());
    packet
}

pub fn send(game: &mut Gameplay, character: &CharacterState, packet: &[u8], now: u64) -> Outcome {
    game.handle_in_world(packet, character, now)
        .unwrap_or_else(|error| {
            panic!(
                "packet {:?} was refused: {error}",
                &packet[..packet.len().min(4)]
            )
        })
}

pub fn has_family(packet: &[u8], family: u16, subtype: u16, id: Option<u32>) -> bool {
    if packet.len() < 6
        || packet[..2] != family.to_le_bytes()
        || packet[2..4] != subtype.to_le_bytes()
    {
        return false;
    }
    let Some(id) = id else { return true };
    // 49/13 carries the turn-in speaker at 6..14 and the reply quest id at 14..18;
    // the other family/subtype pairs used here carry their id at 6..10.
    let offset = if (family, subtype) == (49, 13) { 14 } else { 6 };
    packet
        .get(offset..offset + 4)
        .is_some_and(|bytes| u32::from_le_bytes(bytes.try_into().unwrap()) == id)
}

/// 35/5 `PlayerUpdatePacketUpdateHitpoints` reader `0x008d6f10` (base header reader
/// `0x008d6830`): after the four-byte header it reads the actor guid as two dwords
/// (`4..12`), then three i32s; the first is the current hitpoints (`12..16`).
pub fn enemy_hp(replies: &[Vec<u8>], guid: u64) -> Option<u32> {
    replies.iter().find_map(|packet| {
        (has_family(packet, 35, 5, None)
            && packet.get(4..12) == Some(guid.to_le_bytes().as_slice())
            && packet.len() >= 16)
            .then(|| u32::from_le_bytes(packet[12..16].try_into().unwrap()))
    })
}

/// Full 49/8 `ObjectiveActivated` packet: the 6-byte family/subtype header
/// (`0x00c7acd0`), quest id, objective id, required count, and the reconstructed
/// (R) visible-notification bool. Body reader `0x00c7aeb0` (through `0x00c7b570`);
/// apply `0x00c7a910`.
pub fn objective_activated_packet(quest_id: u32, objective_id: u32, required: u32) -> Vec<u8> {
    let mut packet = vec![49, 0, 8, 0, 0, 0];
    for value in [quest_id, objective_id, required] {
        packet.extend(value.to_le_bytes());
    }
    packet.push(0);
    packet
}

/// Full 49/9 `QuestObjectiveUpdatePacket`: header, quest id, objective id,
/// current counter, progress float, (R) notification bool and one (R)
/// unresolved trailing dword. Reader `0x00c7af60` reads exactly that order.
pub fn objective_update_packet(
    quest_id: u32,
    objective_id: u32,
    current: u32,
    progress: f32,
) -> Vec<u8> {
    let mut packet = vec![49, 0, 9, 0, 0, 0];
    for value in [quest_id, objective_id, current] {
        packet.extend(value.to_le_bytes());
    }
    packet.extend(progress.to_le_bytes());
    packet.push(0);
    packet.extend(0_i32.to_le_bytes());
    packet
}

/// Full 49/10 `QuestObjectiveCompletePacket`: header, quest id, objective id,
/// progress float, one (R) unresolved dword and the (R) completion bool. Reader
/// `0x00c7b070` reads exactly that order.
pub fn objective_complete_packet(quest_id: u32, objective_id: u32, progress: f32) -> Vec<u8> {
    let mut packet = vec![49, 0, 10, 0, 0, 0];
    for value in [quest_id, objective_id] {
        packet.extend(value.to_le_bytes());
    }
    packet.extend(progress.to_le_bytes());
    packet.extend(0_i32.to_le_bytes());
    packet.push(0);
    packet
}

/// The selected option id for `label` in one 26/9 interaction menu body.
/// Dispatcher `0x00aa2360` -> `0x00aa2220` -> `0x00aa1600` (guid at 4, flag
/// at 12) -> list reader `0x00aa04d0` (count at 13) -> row reader `0x00a9dca0`
/// (seven dwords, 28-byte rows from 17; label at +8). `multiplayer.rs` shares it.
pub fn menu_option_bytes(packet: &[u8], label: u32) -> i32 {
    let count = i32::from_le_bytes(packet[13..17].try_into().unwrap());
    assert!(count > 0, "the interaction menu has no options");
    (0..count)
        .find_map(|index| {
            let offset = 17 + usize::try_from(index).unwrap() * 28;
            let id = i32::from_le_bytes(packet[offset..offset + 4].try_into().unwrap());
            let string_id = u32::from_le_bytes(packet[offset + 8..offset + 12].try_into().unwrap());
            (string_id == label).then_some(id)
        })
        .unwrap_or_else(|| panic!("menu label {label} is absent"))
}

pub fn menu_option(outcome: &Outcome, label: u32) -> i32 {
    let menu = outcome
        .replies
        .iter()
        .find(|packet| packet.starts_with(&[26, 0, 9, 0]))
        .expect("26/8 returns the 26/9 interaction menu");
    menu_option_bytes(menu, label)
}

pub fn interact(game: &mut Gameplay, character: &CharacterState, target: u64, now: u64) -> Outcome {
    send(game, character, &npc_interact(target), now)
}

pub fn talk(game: &mut Gameplay, character: &CharacterState, target: u64, now: u64) -> Outcome {
    let menu = interact(game, character, target, now);
    let choice = menu_option(&menu, TALK_LABEL);
    send(game, character, &npc_select(target, choice), now + 1)
}

/// Complete one ordered Talk objective. A final target opens the pending end screen,
/// which the client releases with its 26/6 dialogue choice.
pub fn talk_step(
    game: &mut Gameplay,
    character: &CharacterState,
    target: u64,
    now: u64,
    label: &str,
) {
    let talked = talk(game, character, target, now);
    assert!(talked.changed, "{label} did not advance its objective");
}

pub fn offer(
    game: &mut Gameplay,
    character: &CharacterState,
    target: u64,
    quest_id: u32,
    now: u64,
) -> Outcome {
    let dialogue = talk(game, character, target, now);
    assert!(
        dialogue
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[26, 0, 3, 0])),
        "Talk must open the 26/3 dialogue"
    );
    let offered = send(game, character, &dialogue_choice(QUEST_CHOICE), now + 2);
    let offers: Vec<&Vec<u8>> = offered
        .replies
        .iter()
        .filter(|packet| packet.starts_with(&[49, 0, 1, 0]))
        .collect();
    assert_eq!(offers.len(), 1, "one Talk offers exactly one quest");
    // Decode with the production reader so a wrong field order or a corrupt
    // packet fails here in every chapter, not only in the Wizard card test.
    let decoded = decode_quest_offer_49_1(offers[0].as_slice())
        .expect("the offered 49/1 decodes with the production reader");
    assert_eq!(
        decoded.quest_id, quest_id,
        "quest {quest_id} was not offered"
    );
    assert_eq!(quest_status(game, quest_id), QuestStatus::Offered);
    offered
}

pub fn accept(game: &mut Gameplay, character: &CharacterState, quest_id: u32, now: u64) -> Outcome {
    let accepted = send(game, character, &quest_reply(quest_id, true), now);
    assert!(
        accepted
            .replies
            .iter()
            .any(|packet| has_family(packet, 49, 3, Some(quest_id))),
        "quest {quest_id} did not send QuestAdd"
    );
    assert!(
        accepted
            .replies
            .iter()
            .any(|packet| has_family(packet, 49, 8, Some(quest_id))),
        "quest {quest_id} did not activate its first objective"
    );
    assert_eq!(quest_status(game, quest_id), QuestStatus::Active);
    accepted
}

/// 26/6 releases the 49/13 that the final Talk held. Its speaker must be the
/// runtime actor the client clicked, not the content key, and its end text must
/// be the quest's debrief. `QuestEndPacket::ReadFields` (`00c7b990`) reads the
/// common header, actor `6..14`, quest id `14..18`, end text `18..22`, one more
/// dword at `22..26`, the reward bundle (`008e7930`), then a trailing float;
/// `ShowQuestEndScreen` (`00a95420`) passes the end text dword as the message id.
pub fn release_end_screen(
    game: &mut Gameplay,
    character: &CharacterState,
    quest_id: u32,
    expected_actor: u64,
    expected_end_text: u32,
    now: u64,
) -> Vec<u8> {
    let released = send(game, character, &dialogue_choice(QUEST_CHOICE), now);
    let end = released
        .replies
        .iter()
        .find(|packet| has_family(packet, 49, 13, Some(quest_id)))
        .cloned()
        .unwrap_or_else(|| panic!("quest {quest_id} did not release its 49/13 end screen"));
    assert_eq!(
        u64::from_le_bytes(end[6..14].try_into().unwrap()),
        expected_actor,
        "the end screen must name the clicked runtime actor"
    );
    assert_eq!(
        u32::from_le_bytes(end[18..22].try_into().unwrap()),
        expected_end_text,
        "the end screen must carry the quest's debrief text"
    );
    end
}

/// The 49/13 reward bundle's leading end-screen words. `QuestEndPacket::ReadFields`
/// (`00c7b990`) reads the actor at `6..14`, quest id at `14..18`, end text at `18..22`, the
/// animation dword at `22..26`, then calls the reward bundle reader (`008e7930`): a bool at
/// `26`, coins at `27..31`, and the stars total at `31..35` (the sum of the quest's job XP,
/// not one profile's).
pub fn end_screen_reward(end: &[u8]) -> (u32, u32) {
    assert!(
        end.len() >= 35,
        "a 49/13 packet carries the reward bundle's coin and star words"
    );
    (
        u32::from_le_bytes(end[27..31].try_into().unwrap()),
        u32::from_le_bytes(end[31..35].try_into().unwrap()),
    )
}

/// The ending profile is explicit: a quest's reward map names its own job, which
/// need not be the profile the character has active at turn-in.
pub fn finish_quest(
    game: &mut Gameplay,
    character: &CharacterState,
    quest_id: u32,
    profile: i32,
    expected_coins: u32,
    expected_xp: u32,
    now: u64,
) -> Outcome {
    let coins_before = coins(game);
    let xp_before = profile_xp(game, profile);
    let completed = send(game, character, &quest_end_reply(quest_id), now);
    assert!(
        completed.changed,
        "quest {quest_id} turn-in did not change state"
    );
    assert!(
        completed
            .replies
            .iter()
            .any(|packet| has_family(packet, 49, 4, Some(quest_id))),
        "quest {quest_id} did not complete"
    );
    assert_eq!(coins(game), coins_before + expected_coins);
    assert_eq!(profile_xp(game, profile), xp_before + expected_xp);
    assert_eq!(quest_status(game, quest_id), QuestStatus::Completed);
    assert_eq!(pending_turnin(game), None);
    completed
}

pub fn counts(game: &Gameplay, quest_id: u32) -> Vec<u32> {
    game.state().npc_quests.as_ref().unwrap().quests[&quest_id]
        .counts
        .clone()
}

pub fn quest_status(game: &Gameplay, quest_id: u32) -> QuestStatus {
    game.state().npc_quests.as_ref().unwrap().quests[&quest_id]
        .status
        .clone()
}

pub fn pending_offer(game: &Gameplay) -> Option<u32> {
    game.state().npc_quests.as_ref().unwrap().pending_offer
}

pub fn pending_turnin(game: &Gameplay) -> Option<u32> {
    game.state().npc_quests.as_ref().unwrap().pending_turnin
}

pub fn active_job(game: &Gameplay) -> i32 {
    state_active_job(&game.state())
}

pub fn coins(game: &Gameplay) -> u32 {
    game.state().inventory.coins
}

pub fn profile_xp(game: &Gameplay, profile: i32) -> u32 {
    state_profile_xp(&game.state(), profile)
}

fn state_active_job(state: &GameplayState) -> i32 {
    state.profiles.as_ref().unwrap().active
}

fn state_profile_xp(state: &GameplayState, profile: i32) -> u32 {
    state
        .profiles
        .as_ref()
        .unwrap()
        .experience
        .get(&profile)
        .copied()
        .unwrap_or(0)
}

/// The same two reads through the gateway session snapshot battle suites assert.
pub fn gateway_active_job(gateway: &Gateway) -> i32 {
    state_active_job(
        gateway
            .snapshot()
            .gameplay
            .as_ref()
            .expect("the session has gameplay"),
    )
}

pub fn gateway_profile_xp(gateway: &Gateway, profile: i32) -> u32 {
    state_profile_xp(
        gateway
            .snapshot()
            .gameplay
            .as_ref()
            .expect("the session has gameplay"),
        profile,
    )
}

/// Seed the saved active profile when direct play cannot reach a job switch (the captured
/// opening inventory may lack that job's starter outfit). The quest gate reads this
/// production save, not the caller's expectation. Jobs the caller lists stay unlocked; the
/// caller's own fixture decides what was unlocked before.
pub fn seed_active_profile(state: &mut GameplayState, active: i32, unlocked: &[i32]) {
    let profiles = state
        .profiles
        .as_mut()
        .expect("the fixture has profile content");
    profiles.active = active;
    profiles.unlocked.insert(active);
    profiles.unlocked.extend(unlocked.iter().copied());
}

/// The serialized `GameplayState`, not the character-store save envelope. Use it to compare
/// two in-memory sessions; a save-file claim needs the real store round trip.
pub fn saved(game: &Gameplay) -> Vec<u8> {
    serde_json::to_vec(&game.state()).unwrap()
}

/// A Talk with nothing eligible must not offer, complete or mutate anything: the
/// open 26/9 menu stays usable, the 26/6 choice releases no offer or end screen,
/// and the serialized state stays equal.
pub fn assert_no_offer(
    game: &mut Gameplay,
    character: &CharacterState,
    target: u64,
    now: &mut u64,
    label: &str,
) {
    let before = saved(game);
    *now += 10;
    let menu = interact(game, character, target, *now);
    let choice = menu_option(&menu, TALK_LABEL);
    *now += 1;
    let dialogue = send(game, character, &npc_select(target, choice), *now);
    assert!(
        !dialogue.changed,
        "{label}: a Talk with nothing eligible changed gameplay"
    );
    assert!(
        !dialogue
            .replies
            .iter()
            .any(|packet| has_family(packet, 49, 1, None)),
        "{label}: a Talk with nothing eligible offered a quest"
    );
    // The refusal must also leave the open NPC menu intact: the same 26/9 option
    // still accepts a second Talk selection.
    *now += 1;
    let again = send(game, character, &npc_select(target, choice), *now);
    assert!(
        again
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[26, 0, 3, 0])),
        "{label}: a refused Talk consumed the open NPC menu"
    );
    *now += 8;
    let closed = send(game, character, &dialogue_choice(QUEST_CHOICE), *now);
    assert!(
        !closed
            .replies
            .iter()
            .any(|packet| has_family(packet, 49, 1, None) || has_family(packet, 49, 13, None)),
        "{label}: no quest may be offered or turned in"
    );
    assert_eq!(
        saved(game),
        before,
        "{label}: the refusal changed the serialized state"
    );
}

pub fn assert_packet_changes_nothing(
    game: &mut Gameplay,
    character: &CharacterState,
    packet: &[u8],
    now: u64,
    label: &str,
) {
    let before = saved(game);
    let outcome = send(game, character, packet, now);
    assert!(!outcome.changed, "{label} changed gameplay");
    assert!(
        !outcome
            .replies
            .iter()
            .any(|reply| has_family(reply, 49, 3, None) || has_family(reply, 49, 4, None)),
        "{label} completed or added a quest"
    );
    assert_eq!(saved(game), before, "{label} changed the serialized state");
}

/// One shared-field basic attack (`SharedFieldAction::Ability` slot 0): the
/// production request `Session::commit_outcome` runs. Panics on a refused
/// request; a test that proves a refusal calls `Gameplay::handle_shared_field`
/// directly so it can inspect the `io::Result`.
pub fn shared_attack(
    game: &mut Gameplay,
    field: &mut SharedFieldState,
    character: &CharacterState,
    known: &BTreeSet<u64>,
    target: u64,
    now: u64,
) -> Outcome {
    game.handle_shared_field(
        &SharedFieldAction::Ability { slot: 0, target },
        field,
        known,
        character,
        now,
    )
    .unwrap_or_else(|error| panic!("shared attack on {target} was refused: {error}"))
}

/// The basic-attack interval the field weapon table uses.
pub const SHARED_ATTACK_INTERVAL_MS: u64 = 700;

/// Attack `target` from its current field position until its health reaches
/// zero; returns the lethal outcome and the next free timestamp. Reading the
/// position before each swing covers wandering actors and stationary ones alike;
/// `character` supplies the attacker's guid and zone.
pub fn kill_shared_target(
    game: &mut Gameplay,
    field: &mut SharedFieldState,
    known: &BTreeSet<u64>,
    character: &CharacterState,
    target: u64,
    now: u64,
) -> (Outcome, u64) {
    let mut now = now;
    for _ in 0..16 {
        let position = field
            .position(target)
            .unwrap_or_else(|| panic!("field target {target} exists"));
        let attacker = character_at_for(character.guid, character.zone_id, position);
        let outcome = shared_attack(game, field, &attacker, known, target, now);
        now += SHARED_ATTACK_INTERVAL_MS;
        if field.health(target) == Some(0) {
            return (outcome, now);
        }
    }
    panic!("field target {target} did not die");
}

/// One shared-motion tick for a participant far from the field, so the tick only
/// settles and respawns the actors. `elapsed` is the elapsed time the session
/// reports for the same tick.
pub fn tick_shared_field(
    game: &mut Gameplay,
    field: &mut SharedFieldState,
    known: &BTreeSet<u64>,
    player: u64,
    now: u64,
    elapsed: u64,
) {
    let participant = FieldParticipant {
        guid: player,
        position: [0.0, 0.0, 0.0],
        health: 1_000,
        max_health: 1_000,
        mitigation: 1.0,
        grace_until: 0,
        combat: true,
        known_npcs: Some(known.clone()),
    };
    game.tick_shared_motion(field, &[participant], now, elapsed)
        .expect("the shared field ticks");
}

/// 49/9 `ObjectiveUpdate`: quest `6..10`, objective `10..14`, current `14..18`
/// (client reader `0x00c7af60`).
pub fn objective_update(outcome: &Outcome, quest: u32, objective: u32, current: u32) -> bool {
    outcome.replies.iter().any(|packet| {
        has_family(packet, 49, 9, Some(quest))
            && packet.len() >= 18
            && u32::from_le_bytes(packet[10..14].try_into().unwrap()) == objective
            && u32::from_le_bytes(packet[14..18].try_into().unwrap()) == current
    })
}

/// 49/10 `GoalComplete`: quest `6..10`, objective `10..14` (client reader
/// `0x00c7b070`).
pub fn objective_complete(outcome: &Outcome, quest: u32, objective: u32) -> bool {
    outcome.replies.iter().any(|packet| {
        has_family(packet, 49, 10, Some(quest))
            && packet.len() >= 14
            && u32::from_le_bytes(packet[10..14].try_into().unwrap()) == objective
    })
}

/// 49/8 `ObjectiveActivated`: quest `6..10`, objective `10..14` (client reader
/// `0x00c7aeb0`).
pub fn objective_activated(outcome: &Outcome, quest: u32, objective: u32) -> bool {
    outcome.replies.iter().any(|packet| {
        has_family(packet, 49, 8, Some(quest))
            && packet.len() >= 14
            && u32::from_le_bytes(packet[10..14].try_into().unwrap()) == objective
    })
}
