//! T254 / restructure S15 step 0: one 26/10 NPC menu choice is one transaction.
//!
//! Default-run and Gameplay-level: every request below reaches the real
//! `Gameplay::handle_in_world` router (26/8 `008b6bc0`, 26/10 `00a36f80`,
//! 26/23, 26/4). The fixture is the checked-in opening-journal reference
//! content plus the bundled NPC/service content, so no private environment
//! variable is needed.
//!
//! The pins are the authority for the S15 move. A refused choice must leave
//! `GameplayState` byte-identical and keep its menu. A busy refusal answers one
//! chat line for every choice; at arm level a learn/fish refusal answers 26/4
//! before the chat line, a race/derby refusal answers chat only, and an admitted
//! learn, derby or fish entry starts with 26/4.

use super::*;
#[path = "../../tests/common/gateway_packets.rs"]
mod gateway_packets;
use crate::{
    account_store::AccountRole,
    derby::{DerbyConfig, DerbyRules, DerbyVehicleTemplate},
    derby_gateway::{DerbyGates, DerbyGatewayContent},
};
use gateway_packets::{chat, interact, select};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

/// 26/4 CommandPacketEndDialog (`009f5760`, handler `00aa0870`).
const END_DIALOG: [u8; 4] = [26, 0, 4, 0];
/// A real regional NPC on the planted-race gate list.
const KART_GATE: u64 = 9_000_000_273;
/// `kart_gateway::DEFAULT_MENU_STRING_ID`.
const KART_OPTION: u32 = 3_052;
/// A real regional fishing guide (also the Fisherman trainer).
const FISH_GATE: u64 = 9_000_000_181;
/// `fishing_spots.json` menu label.
const FISH_OPTION: u32 = 73_552;
/// The Ninja trainer requires a level-5 Archer, so its Learn choice refuses.
const NINJA_TRAINER: u64 = 9_000_000_223;
const NINJA_OPTION: u32 = 412_437;
/// The Miner trainer teaches an unlocked-on-success job with no prerequisite.
const MINER_TRAINER: u64 = 9_000_000_405;
const MINER_OPTION: u32 = 139_748;
/// A real regional NPC used here as the synthetic derby gate.
const DERBY_GATE: u64 = 9_000_000_391;
/// `derby_gates.json` menu label.
const DERBY_OPTION: u32 = 3_053;
const NOW: u64 = 5_000;
const CHARACTER_GUID: u64 = 0x1000_0000_0000_0001;
const INTRODUCE_YOURSELF: u32 = 3_000_021;
const SAMANTHA: u64 = 9_000_000_189;
const BUSY_REFUSAL: &str = "Finish the active activity before interacting.";
const LEARN_REFUSAL: &str = "level 5 Archer";
const FISH_REFUSAL: &str = "Fishing isn't open yet.";
static NEXT: AtomicU64 = AtomicU64::new(0);

/// One per-run content directory: the reference campaign plus the bundled NPC service files.
struct Fixture(PathBuf);

impl Fixture {
    /// Reference campaign + Mining, trainers, kart, fishing and a synthetic derby package.
    fn full() -> Self {
        Self::build(true)
    }

    /// The same NPC/service content with no class definitions, so fishing refuses after
    /// the menu choice (the fish service needs profiles to build its reward state).
    fn without_classes() -> Self {
        Self::build(false)
    }

    fn build(classes: bool) -> Self {
        let crate_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let source = crate_dir.join("../../../lab/references/opening-journal-text-v2/content");
        let data = crate_dir.join("data/content");
        assert!(
            source.join("opening.json").is_file() && source.join("profiles.json").is_file(),
            "the default-run S15 fixture needs the reference campaign: {}",
            source.display()
        );
        let path = std::env::temp_dir().join(format!(
            "osfr-s15-npc-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();
        let fixture = Self(path);
        if classes {
            for entry in fs::read_dir(&source).unwrap() {
                let file = entry.unwrap().path();
                if file.is_file() {
                    fs::copy(&file, fixture.0.join(file.file_name().unwrap())).unwrap();
                }
            }
            mining::write_content(&fixture.0, mining::MINING_GAME_ID).unwrap();
            for file in [
                "job_trainers.json",
                "kart_entry.json",
                "kart_gates.json",
                "fishing_spots.json",
            ] {
                fs::copy(data.join(file), fixture.0.join(file)).unwrap();
            }
            fs::write(
                fixture.0.join("derby_entry.json"),
                serde_json::to_vec(&derby_content()).unwrap(),
            )
            .unwrap();
            fs::write(
                fixture.0.join("derby_gates.json"),
                serde_json::to_vec(&DerbyGates {
                    schema_version: 1,
                    menu_string_id: DERBY_OPTION,
                    npcs: vec![DERBY_GATE],
                    reward_cooldown_s: None,
                })
                .unwrap(),
            )
            .unwrap();
        } else {
            for file in [
                "inventory.json",
                "categories.json",
                "combat.json",
                "combat-template.bin",
                "quests.json",
                "quest-journal.bin",
            ] {
                fs::copy(source.join(file), fixture.0.join(file)).unwrap();
            }
            fs::copy(
                data.join("fishing_spots.json"),
                fixture.0.join("fishing_spots.json"),
            )
            .unwrap();
        }
        fixture
    }

    fn game(&self) -> Gameplay {
        Gameplay::load(&self.0, None).expect("the S15 fixture must load")
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// A synthetic, valid derby package; S15 only needs the NPC entry offer.
fn derby_content() -> DerbyGatewayContent {
    DerbyGatewayContent {
        schema_version: 1,
        game_id: 39,
        vehicle: DerbyVehicleTemplate {
            name: b"synthetic-derby".to_vec(),
            definition: vec![1, 2, 3, 4],
            start_transform_bits: [0; 16],
            unknown_flag: false,
            unknown_value_bits: 0,
        },
        rules: DerbyRules {
            max_drivers: 8,
            start_health: 20,
            contact_damage_per_unit: 1.0,
            contact_cooldown_ms: 100,
            contact_radius_m: 5.0,
            max_impact: 50.0,
            max_pose_age_ms: 5_000,
            max_damage_per_contact: 100,
            lobby_timeout_ms: 120_000,
            pickup_radius: 3.0,
            health_pickup: 5,
            boost_pickup: 5.0,
            shield_duration_ms: 1_000,
            attack_duration_ms: 1_000,
            ultimate_duration_ms: 1_000,
            attack_multiplier: 2.0,
            ultimate_multiplier: 2.0,
            respawn_delay_ms: 1_000,
            results_duration_ms: 1_000,
            knockout_points: 2,
            leader_knockout_points: 3,
            knockout_point_loss: 1,
            score_target: Some(2),
            time_limit_ms: None,
            config: DerbyConfig {
                value_a_bits: 0,
                value_b: 0,
            },
        },
        pickups: Vec::new(),
        completion_reward_coins: 0,
        completion_reward_xp: 0,
        reward: None,
    }
}

fn character(game: &Gameplay, npc: u64) -> CharacterState {
    let (zone_id, position) = game
        .npc_interaction
        .as_ref()
        .unwrap()
        .location(npc)
        .unwrap_or_else(|| panic!("NPC {npc} has a location"));
    CharacterState {
        guid: CHARACTER_GUID,
        zone_id,
        position,
        rotation: [0.0, 0.0, 1.0],
    }
}

fn state_bytes(game: &Gameplay) -> Vec<u8> {
    serde_json::to_vec(&game.state()).unwrap()
}

fn said(replies: &[Vec<u8>], needle: &str) -> bool {
    replies
        .iter()
        .any(|reply| String::from_utf8_lossy(reply).contains(needle))
}

/// The option id of the 26/9 menu row whose label is `label` (client reader `00aa2360`).
fn option_id(menu: &[Vec<u8>], label: u32) -> i32 {
    let packet = menu
        .iter()
        .find(|packet| packet.starts_with(&[26, 0, 9, 0]))
        .expect("26/8 opens the 26/9 menu");
    let count = i32::from_le_bytes(packet[13..17].try_into().unwrap()) as usize;
    (0..count)
        .find_map(|index| {
            let at = 17 + index * 28;
            let id = i32::from_le_bytes(packet[at..at + 4].try_into().ok()?);
            let string_id = u32::from_le_bytes(packet[at + 8..at + 12].try_into().ok()?);
            (string_id == label).then_some(id)
        })
        .unwrap_or_else(|| panic!("the menu offers label {label}"))
}

fn open_menu(game: &mut Gameplay, character: &CharacterState, npc: u64, label: u32) -> i32 {
    let menu = game
        .handle_in_world(&interact(npc), character, NOW)
        .unwrap();
    option_id(&menu.replies, label)
}

fn start_mining(game: &mut Gameplay, character: &CharacterState) {
    {
        let profiles = game.profiles.as_mut().unwrap();
        if !profiles
            .state()
            .unlocked
            .contains(&mining::MINER_PROFILE_ID)
        {
            profiles.unlock(mining::MINER_PROFILE_ID).unwrap();
        }
        profiles
            .select(mining::MINER_PROFILE_ID, &game.inventory)
            .unwrap();
    }
    game.set_dev_tools(true);
    game.set_role(AccountRole::Admin);
    let started = game
        .handle_in_world(&chat("realm mine"), character, NOW)
        .unwrap();
    assert!(
        game.mining.as_ref().is_some_and(Type8Engine::active),
        "the live Mining run must start: {:?}",
        started.replies
    );
}

fn end_mining(game: &mut Gameplay, character: &CharacterState) {
    let ended = game
        .handle_in_world(&chat("realm exit"), character, NOW + 1)
        .unwrap();
    assert!(
        game.mining.as_ref().is_none_or(|mining| !mining.active()),
        "the Mining run must end: {:?}",
        ended.replies
    );
}

/// With a live Mining run every offered choice is refused at the shared busy gate,
/// before admission. It must answer one chat line only, change no state, and leave
/// the menu open (learn and fish refusals keep their 26/4 in their own arms).
#[test]
fn a_live_mining_run_refuses_every_choice_without_consuming_its_menu() {
    let fixture = Fixture::full();
    for (npc, option) in [
        (KART_GATE, KART_OPTION),
        (DERBY_GATE, DERBY_OPTION),
        (FISH_GATE, FISH_OPTION),
        (NINJA_TRAINER, NINJA_OPTION),
    ] {
        let mut game = fixture.game();
        let character = character(&game, npc);
        let choice = open_menu(&mut game, &character, npc, option);
        start_mining(&mut game, &character);
        let before = state_bytes(&game);
        let refused = game
            .handle_in_world(&select(npc, choice as u32), &character, NOW + 2)
            .unwrap();
        assert!(
            !refused.changed,
            "{npc}: a refusal must not report a change"
        );
        assert_eq!(
            refused.replies.len(),
            1,
            "{npc}: a busy refusal answers one chat line: {:?}",
            refused.replies
        );
        assert!(said(&refused.replies, BUSY_REFUSAL), "{npc}");
        assert_eq!(
            state_bytes(&game),
            before,
            "{npc}: a refused choice changed GameplayState"
        );
        end_mining(&mut game, &character);
        let talked = game
            .handle_in_world(&select(npc, 1), &character, NOW + 3)
            .unwrap();
        assert!(
            !talked.replies.is_empty(),
            "{npc}: the refused choice consumed the menu"
        );
    }
}

/// Learn refuses at its own gate (unmet prerequisite) and still answers 26/4 first.
#[test]
fn a_refused_learn_answers_end_dialog_then_chat_and_commits_nothing() {
    let fixture = Fixture::full();
    let mut game = fixture.game();
    let character = character(&game, NINJA_TRAINER);
    let choice = open_menu(&mut game, &character, NINJA_TRAINER, NINJA_OPTION);
    let before = state_bytes(&game);
    let refused = game
        .handle_in_world(&select(NINJA_TRAINER, choice as u32), &character, NOW + 1)
        .unwrap();
    assert!(!refused.changed);
    assert_eq!(
        refused.replies.first(),
        Some(&END_DIALOG.to_vec()),
        "a learn refusal answers 26/4 first"
    );
    assert_eq!(refused.replies.len(), 2, "{:?}", refused.replies);
    assert!(said(&refused.replies[1..], LEARN_REFUSAL));
    assert_eq!(state_bytes(&game), before, "a refused learn changed state");
    let talked = game
        .handle_in_world(&select(NINJA_TRAINER, 1), &character, NOW + 2)
        .unwrap();
    assert!(
        !talked.replies.is_empty(),
        "the refused learn consumed the menu"
    );
}

/// Fish refuses in its own arm (no class definitions) and still answers 26/4 first.
#[test]
fn a_refused_fish_answers_end_dialog_then_chat_and_commits_nothing() {
    let fixture = Fixture::without_classes();
    let mut game = fixture.game();
    let character = character(&game, FISH_GATE);
    let choice = open_menu(&mut game, &character, FISH_GATE, FISH_OPTION);
    let before = state_bytes(&game);
    let refused = game
        .handle_in_world(&select(FISH_GATE, choice as u32), &character, NOW + 1)
        .unwrap();
    assert!(!refused.changed);
    assert_eq!(
        refused.replies.first(),
        Some(&END_DIALOG.to_vec()),
        "a fish refusal answers 26/4 first"
    );
    assert_eq!(refused.replies.len(), 2, "{:?}", refused.replies);
    assert!(said(&refused.replies[1..], FISH_REFUSAL));
    assert_eq!(state_bytes(&game), before, "a refused fish changed state");
    let talked = game
        .handle_in_world(&select(FISH_GATE, 1), &character, NOW + 2)
        .unwrap();
    assert!(
        !talked.replies.is_empty(),
        "the refused fish consumed the menu"
    );
}

#[test]
fn an_admitted_learn_starts_with_end_dialog_and_commits() {
    let fixture = Fixture::full();
    let mut game = fixture.game();
    let character = character(&game, MINER_TRAINER);
    let choice = open_menu(&mut game, &character, MINER_TRAINER, MINER_OPTION);
    assert!(
        !game
            .profiles
            .as_ref()
            .unwrap()
            .state()
            .unlocked
            .contains(&mining::MINER_PROFILE_ID)
    );
    let learned = game
        .handle_in_world(&select(MINER_TRAINER, choice as u32), &character, NOW + 1)
        .unwrap();
    assert!(learned.changed);
    assert_eq!(
        learned.replies.first(),
        Some(&END_DIALOG.to_vec()),
        "an admitted learn starts with 26/4"
    );
    assert_eq!(
        learned
            .replies
            .iter()
            .filter(|packet| *packet == &END_DIALOG)
            .count(),
        1,
        "the learn end-dialog is inserted once, at the choice"
    );
    assert!(said(&learned.replies, "You learned the Miner job"));
    assert!(
        game.profiles
            .as_ref()
            .unwrap()
            .state()
            .unlocked
            .contains(&mining::MINER_PROFILE_ID)
    );
    assert!(
        game.handle_in_world(&select(MINER_TRAINER, choice as u32), &character, NOW + 2)
            .is_ok_and(|out| out.replies.is_empty() && !out.changed),
        "an admitted learn must consume its menu"
    );
}

#[test]
fn an_admitted_derby_starts_with_end_dialog_and_commits() {
    let fixture = Fixture::full();
    let mut game = fixture.game();
    let character = character(&game, DERBY_GATE);
    let choice = open_menu(&mut game, &character, DERBY_GATE, DERBY_OPTION);
    let started = game
        .handle_in_world(&select(DERBY_GATE, choice as u32), &character, NOW + 1)
        .unwrap();
    assert!(started.changed);
    assert_eq!(
        started.replies.first(),
        Some(&END_DIALOG.to_vec()),
        "an admitted derby starts with 26/4"
    );
    assert_eq!(
        started
            .replies
            .iter()
            .filter(|packet| *packet == &END_DIALOG)
            .count(),
        1,
        "the derby end-dialog is inserted once, at the choice"
    );
    assert!(
        started
            .replies
            .iter()
            .any(|packet| packet[..3] == [39, 0, 16]),
        "the derby entry offers the native match: {:?}",
        started.replies
    );
    assert!(game.derby_active());
}

#[test]
fn an_admitted_fish_entry_starts_with_end_dialog_and_opens_the_round() {
    let fixture = Fixture::full();
    let mut game = fixture.game();
    let character = character(&game, FISH_GATE);
    let choice = open_menu(&mut game, &character, FISH_GATE, FISH_OPTION);
    assert!(
        !game
            .profiles
            .as_ref()
            .unwrap()
            .state()
            .unlocked
            .contains(&137)
    );
    let entered = game
        .handle_in_world(&select(FISH_GATE, choice as u32), &character, NOW + 1)
        .unwrap();
    assert!(entered.changed);
    assert_eq!(
        entered.replies.first(),
        Some(&END_DIALOG.to_vec()),
        "an admitted fish entry starts with 26/4"
    );
    assert_eq!(
        entered
            .replies
            .iter()
            .filter(|packet| *packet == &END_DIALOG)
            .count(),
        1,
        "the fish end-dialog is inserted once, at the choice"
    );
    assert!(
        entered
            .replies
            .iter()
            .any(|packet| packet[..3] == [39, 0, 16]),
        "the fish entry offers the native round: {:?}",
        entered.replies
    );
    assert!(!game.fishing.is_empty());
    assert!(
        game.profiles
            .as_ref()
            .unwrap()
            .state()
            .unlocked
            .contains(&137),
        "the first trip unlocks Fisherman"
    );
}

/// T393: Go fish! answers 26/4, the 39/16 offer and its 41/107 ready, in order. The ready
/// (`00aa36c0` case 0x6b -> `009b0cc0` setReady) turns the start screen's "Loading..." into
/// Go. The client's Go (39/5, `009be7d0`) gets 41/2, the 39/17 that starts the 39/16 state
/// (`009b9550`, fishing HUD) and the fishing flag; the round then registers and casts.
#[test]
fn go_fish_sends_offer_then_ready_and_go_starts_the_round() {
    let fixture = Fixture::full();
    let mut game = fixture.game();
    let character = character(&game, FISH_GATE);
    let choice = open_menu(&mut game, &character, FISH_GATE, FISH_OPTION);
    let entered = game
        .handle_in_world(&select(FISH_GATE, choice as u32), &character, NOW + 1)
        .unwrap();
    let heads: Vec<_> = entered.replies.iter().map(|p| p[..3].to_vec()).collect();
    assert_eq!(
        heads,
        vec![vec![26, 0, 4], vec![39, 0, 16], vec![41, 0, 107]]
    );
    let offer = &entered.replies[1];
    let instance = i32::from_le_bytes(offer[3..7].try_into().unwrap());
    let game_id = i32::from_le_bytes(offer[11..15].try_into().unwrap());
    // (R) ordinary entry uses the spot's game id as its encounter id (`FishingService::enter`).
    let mut ready = vec![41, 0, 107, 0];
    ready.extend(game_id.to_le_bytes());
    ready.extend(instance.to_le_bytes());
    assert_eq!(entered.replies[2], ready, "the bare header naming this run");

    let mut go = vec![39, 0, 5];
    let mut start = vec![39, 0, 17];
    for (go_word, start_word) in [(instance, instance), (-1, -1), (-1, game_id)] {
        go.extend(go_word.to_le_bytes());
        start.extend(start_word.to_le_bytes());
    }
    let started = game.handle_in_world(&go, &character, NOW + 2).unwrap();
    let mut flag = vec![35, 0, 20, 0];
    flag.extend(CHARACTER_GUID.to_le_bytes());
    flag.extend(0x40_0001_u32.to_le_bytes());
    assert_eq!(started.replies.len(), 3, "{:?}", started.replies);
    assert!(started.replies[0].starts_with(&[41, 0, 2, 0]));
    assert_eq!(started.replies[1], start);
    assert_eq!(started.replies[2], flag);

    // The existing round runs: registration, then a cast 10 m from the spot (138/6 -> 138/8).
    let handle = [0x44_u8, 0x33, 0x22, 0x11, 0x88, 0x77, 0x66, 0x55];
    let mut register = vec![138, 0, 2, 0];
    register.extend(handle);
    register.push(0);
    let registered = game
        .handle_in_world(&register, &character, NOW + 3)
        .unwrap();
    assert!(registered.replies[0].starts_with(&[138, 0, 3, 0]));
    let mut cast = vec![138, 0, 6, 0];
    cast.extend(handle);
    let [x, y, z] = character.position;
    for coordinate in [x + 10.0, y, z, 1.0] {
        cast.extend(coordinate.to_le_bytes());
    }
    cast.push(0);
    let cast = game.handle_in_world(&cast, &character, NOW + 4).unwrap();
    assert!(
        cast.replies[0].starts_with(&[138, 0, 8, 0]),
        "{:?}",
        cast.replies
    );
}

/// T393: a refused Go fish! (busy, or chosen from out of range) sends none of the start
/// sequence, opens no round and changes nothing.
#[test]
fn a_refused_go_fish_sends_no_offer_or_ready() {
    let fixture = Fixture::full();
    let starts = |replies: &[Vec<u8>]| {
        replies
            .iter()
            .any(|p| p.starts_with(&[39, 0, 16]) || p.starts_with(&[41, 0, 107]))
    };
    let mut game = fixture.game();
    let character = character(&game, FISH_GATE);
    let choice = open_menu(&mut game, &character, FISH_GATE, FISH_OPTION);
    start_mining(&mut game, &character);
    let before = state_bytes(&game);
    let busy = game
        .handle_in_world(&select(FISH_GATE, choice as u32), &character, NOW + 2)
        .unwrap();
    assert!(
        !busy.changed && !starts(&busy.replies),
        "{:?}",
        busy.replies
    );
    assert!(game.fishing.is_empty());
    assert_eq!(state_bytes(&game), before, "busy");

    let mut game = fixture.game();
    let choice = open_menu(&mut game, &character, FISH_GATE, FISH_OPTION);
    let far = CharacterState {
        position: [
            character.position[0] + 100.0,
            character.position[1],
            character.position[2],
        ],
        ..character
    };
    let before = state_bytes(&game);
    assert!(
        game.handle_in_world(&select(FISH_GATE, choice as u32), &far, NOW + 1)
            .is_ok_and(|out| out.replies.is_empty() && !out.changed)
    );
    assert!(game.fishing.is_empty());
    assert_eq!(state_bytes(&game), before, "too far");
    // The refusal consumed nothing: back in range the same choice opens the round.
    let entered = game
        .handle_in_world(&select(FISH_GATE, choice as u32), &character, NOW + 2)
        .unwrap();
    assert!(starts(&entered.replies));
}

/// Introduce Yourself accepted, plus its 26/23 abandon. Client writer `0x00c0a350`: u16 26,
/// u16 23, u32 quest id (8 bytes).
fn accepted_quest_and_abandon(fixture: &Fixture) -> (Gameplay, Vec<u8>) {
    let mut game = fixture.game();
    let content = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content");
    game.bind_npc_quest_catalogue(&content, CHARACTER_GUID)
        .unwrap();
    let engine = game.npc_quests.as_mut().unwrap();
    assert!(engine.offer(INTRODUCE_YOURSELF, SAMANTHA).changed);
    assert!(engine.accept(INTRODUCE_YOURSELF).changed);
    let mut packet = vec![26, 0, 23, 0];
    packet.extend(INTRODUCE_YOURSELF.to_le_bytes());
    (game, packet)
}

#[test]
fn a_positive_26_23_abandon_keeps_its_current_packet() {
    let fixture = Fixture::full();
    let (mut game, packet) = accepted_quest_and_abandon(&fixture);
    let character = CharacterState {
        guid: CHARACTER_GUID,
        zone_id: 0,
        position: [0.0; 3],
        rotation: [0.0, 0.0, 1.0],
    };
    let abandoned = game.handle_in_world(&packet, &character, NOW).unwrap();
    assert!(abandoned.changed);
    let mut expected = vec![49, 0, 6, 0, 0, 0];
    expected.extend(INTRODUCE_YOURSELF.to_le_bytes());
    expected.push(0); // 49/6 reader `0x00c7ae40`: trailing bool.
    assert_eq!(abandoned.replies, vec![expected]);
    assert_eq!(
        game.npc_quests
            .as_ref()
            .unwrap()
            .progress(INTRODUCE_YOURSELF)
            .unwrap()
            .status,
        crate::quest_engine::QuestStatus::Abandoned
    );
}

/// T408 adds no busy gate: with a live Mining run the real 26/23 still abandons (as the old
/// router order did) and the run stays live.
#[test]
fn a_live_activity_neither_blocks_nor_is_ended_by_an_abandon() {
    let fixture = Fixture::full();
    let (mut game, packet) = accepted_quest_and_abandon(&fixture);
    let character = character(&game, KART_GATE);
    start_mining(&mut game, &character);
    let abandoned = game.handle_in_world(&packet, &character, NOW + 1).unwrap();
    assert!(abandoned.changed, "{:?}", abandoned.replies);
    assert!(game.mining.as_ref().is_some_and(Type8Engine::active));
}

#[test]
fn a_standalone_26_4_close_clears_the_menu_without_a_reply() {
    let fixture = Fixture::full();
    let mut game = fixture.game();
    let character = character(&game, KART_GATE);
    let _choice = open_menu(&mut game, &character, KART_GATE, KART_OPTION);
    let before = state_bytes(&game);
    let closed = game
        .handle_in_world(&[26, 0, 4, 0], &character, NOW + 1)
        .unwrap();
    assert!(closed.replies.is_empty());
    assert!(!closed.changed);
    assert_eq!(state_bytes(&game), before);
    assert!(
        game.handle_in_world(&select(KART_GATE, 1), &character, NOW + 2)
            .is_ok_and(|out| out.replies.is_empty() && !out.changed),
        "the standalone close must clear the open menu"
    );
}

/// The fishing entry unlocks Fisherman on the staged clone before the service decides.
/// A service refusal must discard that unlock; this is the commit-if-changed canary
/// (a zero character GUID is refused by the fishing service after the unlock).
#[test]
fn a_refused_fish_entry_discards_its_staged_fisherman_unlock() {
    let fixture = Fixture::full();
    let mut game = fixture.game();
    let character = character(&game, FISH_GATE);
    let choice = open_menu(&mut game, &character, FISH_GATE, FISH_OPTION);
    assert!(!unlocked_fisherman(&game), "the fixture starts locked");
    let before = state_bytes(&game);
    let refused = game
        .handle_in_world(
            &select(FISH_GATE, choice as u32),
            &CharacterState {
                guid: 0,
                ..character
            },
            NOW + 1,
        )
        .unwrap();
    assert!(!refused.changed);
    assert_eq!(
        refused.replies.first(),
        Some(&END_DIALOG.to_vec()),
        "a fish refusal answers 26/4 first"
    );
    assert_eq!(refused.replies.len(), 2, "{:?}", refused.replies);
    assert!(said(&refused.replies[1..], FISH_REFUSAL));
    assert_eq!(
        state_bytes(&game),
        before,
        "the staged Fisherman unlock reached the live state"
    );
    assert!(!unlocked_fisherman(&game));
}

fn unlocked_fisherman(game: &Gameplay) -> bool {
    game.profiles
        .as_ref()
        .unwrap()
        .state()
        .unlocked
        .contains(&crate::fishing_service::FISHERMAN_PROFILE)
}

/// Menus live on the per-session `Gameplay`: one player opening the same NPC's menu
/// does not give another player a choice, and it does not consume theirs.
#[test]
fn two_players_do_not_share_one_npc_menu() {
    let fixture = Fixture::full();
    let mut first = fixture.game();
    let mut second = fixture.game();
    let first_character = character(&first, KART_GATE);
    let second_character = character(&second, KART_GATE);
    let _choice = open_menu(&mut first, &first_character, KART_GATE, KART_OPTION);
    assert!(
        second
            .handle_in_world(&select(KART_GATE, 1), &second_character, NOW + 1)
            .is_ok_and(|out| out.replies.is_empty() && !out.changed),
        "a second session must not read the first session's menu"
    );
    let talked = first
        .handle_in_world(&select(KART_GATE, 1), &first_character, NOW + 2)
        .unwrap();
    assert!(
        !talked.replies.is_empty(),
        "the other session's failed choice consumed the menu"
    );
}

/// S16: one router family owns one parser. An unlisted 66/x subtype belongs to none of
/// them, so `handle_npc_packet` must answer `Ok(None)` — not an empty `Some` — and the rest
/// of the request path still sees the packet. `handle_in_world` cannot tell those two
/// apart, so this pin calls the router directly and then drives the real path for the same
/// bytes.
#[test]
fn unlisted_66_x_falls_through_to_ok_none() {
    let fixture = Fixture::full();
    let mut game = fixture.game();
    let character = character(&game, KART_GATE);
    let packet = [66u8, 0, 7, 0];
    assert!(
        game.handle_npc_packet(&packet, &character, NOW)
            .unwrap()
            .is_none(),
        "an unlisted 66/x subtype must fall through as Ok(None)"
    );
    let before = state_bytes(&game);
    let out = game.handle_in_world(&packet, &character, NOW).unwrap();
    assert!(!out.changed, "an unlisted 66/x changed gameplay");
    assert!(
        out.replies.is_empty(),
        "an unlisted 66/x answered {:?}",
        out.replies
    );
    assert_eq!(state_bytes(&game), before);
}
