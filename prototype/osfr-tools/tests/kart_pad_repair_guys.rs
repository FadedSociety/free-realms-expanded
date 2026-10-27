//! T379: the two kart-pad racer repair guys answer a click with one repair-man greeting
//! instead of logging `unknown_runtime_guid`.
//!
//! Content only: `data/content/npc_spawns_extra.json` binds the existing actors
//! `100000001989`/`100000002048` to content keys `9000001989`/`9000002048`, and one
//! `placement: unanchored` row per key in `data/content/npcs/seaside.json` and
//! `data/content/npcs/cobblestone_village.json` carries the greeting. No production Rust changed.
//!
//! Client shapes, re-checked in IDA for T379:
//! * 26/8 constructor `0x008b6bc0` (u64 target + bool), shared builder `gateway_packets::interact`.
//! * 26/9 entry `0x00aa2360` -> `0x00aa2220` -> `0x00aa1600` -> `0x00aa04d0` (i32 option count, then
//!   seven dwords per row); row reader `0x00a9dca0` reads id at +0, label at +8.
//! * 26/10 constructor `0x00a36f80` (u64 target + i32 choice), shared `gateway_packets::select`.
//! * 26/3 header `0x00a9cb50` (two u16), body `0x00a9ef10` (first payload dword at object +16);
//!   display `0x009f5ae0` passes `*(a2 + 16)` to the `Global.Text.<id>` lookup before
//!   `:SetDialogText`.
//! * 35/2 AddNpc reader `0x00921a10`; the test parses the real packet with the production
//!   `startup_world::parse_add_npc`.
//!
//! The tests drive the real `NpcVisibility`, `NpcInteraction` (through `Gameplay` and directly
//! through the service) and the 26/8 -> 26/9 -> 26/10 route. The shared
//! `common/npc_greetings.rs` harness supplies the `gateway_packets` builders and the content
//! helpers. The fixture copies a lab gameplay package and adds only the shipped
//! `kart_entry.json`/`kart_gates.json`; because it has no `npcs/` directory, `Gameplay` takes the
//! NPC rows from the bundled `data/content` through the production fallback.

#[path = "common/content_copy.rs"]
mod content_copy;
#[allow(dead_code)]
#[path = "common/npc_greetings.rs"]
mod npc_greetings;

use content_copy::copy_tree;
use npc_greetings::{
    GREETING_HEADER, character_at, content_dir, gameplay_dir, interact, npc_position, npc_record,
    read_json, select, service, service_talk_body,
};
use osfr_tools::{
    character_store::CharacterState,
    gameplay::{Gameplay, GameplayState, Outcome},
    kart_gateway::DEFAULT_MENU_STRING_ID,
    mining,
    startup_world::{NpcSpawn, parse_add_npc},
    world::NpcVisibility,
};
use serde_json::Value;
use std::{
    collections::BTreeSet,
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

/// `(spawn_guid, content_key, NPC file, Legacy Npc id)`.
const ACTORS: [(u64, u64, &str, u64); 2] = [
    (1_000_000_01989, 9_000_001_989, "seaside.json", 1989),
    (
        1_000_000_02048,
        9_000_002_048,
        "cobblestone_village.json",
        2048,
    ),
];
const REPAIR_GREETING: u32 = 401_156;
const REPAIR_MODEL: i32 = 968;
/// Spud Edgins, a shipped Sandstorm kart driver, used as the positive control that the fixture
/// really publishes the kart gate menu.
const KART_GATE: u64 = 9_500_000_630;
const NOW: u64 = 5_000;
static NEXT: AtomicU64 = AtomicU64::new(0);

fn extra_records() -> Vec<Value> {
    read_json(&content_dir().join("npc_spawns_extra.json"))["records"]
        .as_array()
        .expect("extra spawn records")
        .clone()
}

fn extra_record(spawn_guid: u64) -> Value {
    extra_records()
        .into_iter()
        .find(|record| record["spawn_guid"].as_u64() == Some(spawn_guid))
        .unwrap_or_else(|| panic!("npc_spawns_extra.json has no spawn {spawn_guid}"))
}

fn state_bytes(game: &Gameplay) -> Vec<u8> {
    serde_json::to_vec(&game.state()).expect("gameplay state serializes")
}

/// The real 26/9 option rows: i32 count at 13, then seven dwords per row, id first, label at +8.
fn menu_options(menu: &[u8]) -> Vec<(i32, u32)> {
    assert!(
        menu.starts_with(&[26, 0, 9, 0]),
        "a 26/9 menu packet was expected"
    );
    let count = i32::from_le_bytes(menu[13..17].try_into().expect("option count"));
    assert!(count >= 0, "negative option count");
    (0..count as usize)
        .map(|index| {
            let at = 17 + index * 28;
            (
                i32::from_le_bytes(menu[at..at + 4].try_into().unwrap()),
                u32::from_le_bytes(menu[at + 8..at + 12].try_into().unwrap()),
            )
        })
        .collect()
}

fn menu(outcome: &Outcome) -> Vec<u8> {
    outcome
        .replies
        .iter()
        .find(|packet| packet.starts_with(&[26, 0, 9, 0]))
        .cloned()
        .unwrap_or_default()
}

fn talk_body(outcome: &Outcome) -> Option<u32> {
    outcome
        .replies
        .iter()
        .find(|packet| packet.starts_with(&GREETING_HEADER))
        .map(|packet| u32::from_le_bytes(packet[4..8].try_into().unwrap()))
}

fn click(game: &mut Gameplay, target: u64, character: &CharacterState) -> Outcome {
    game.handle_in_world(&interact(target), character, NOW)
        .unwrap_or_else(|error| panic!("26/8 for {target}: {error}"))
}

fn choose(game: &mut Gameplay, target: u64, option: u32, character: &CharacterState) -> Outcome {
    game.handle_in_world(&select(target, option), character, NOW + 100)
        .unwrap_or_else(|error| panic!("26/10 for {target}: {error}"))
}

/// Opens the real menu and selects its Talk row (always the first option the NPC service builds).
fn click_and_talk(game: &mut Gameplay, target: u64, character: &CharacterState) -> u32 {
    let opened = click(game, target, character);
    let menu_packet = menu(&opened);
    assert!(
        !menu_packet.is_empty(),
        "26/8 for actor {target} must open a 26/9 menu; no menu means the click hit the \
         unknown_runtime_guid branch (content key missing or not bound to the runtime actor)"
    );
    let options = menu_options(&menu_packet);
    assert!(!options.is_empty(), "actor {target}: empty 26/9 menu");
    let chosen = choose(game, target, options[0].0 as u32, character);
    talk_body(&chosen).unwrap_or_else(|| {
        panic!(
            "actor {target}: Talk must answer 26/3 with its greeting body (outcome replies: {})",
            chosen.replies.len()
        )
    })
}

/// A lab gameplay package plus the shipped kart entry/gates. It has no `npcs/` or
/// `npc_spawns_extra.json`, so `Gameplay` uses the bundled `data/content` NPC rows.
struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "kart-pad-repair-guys-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&path).unwrap();
        copy_tree(&gameplay_dir(), &path);
        for file in ["kart_entry.json", "kart_gates.json"] {
            fs::copy(content_dir().join(file), path.join(file)).unwrap();
        }
        Self(path)
    }

    fn load(&self) -> Gameplay {
        Gameplay::load(&self.0, None).expect("gameplay content loads")
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn adds(packets: &[Vec<u8>]) -> Vec<NpcSpawn> {
    packets
        .iter()
        .filter(|packet| packet.starts_with(&[35, 0, 2, 0]))
        .map(|packet| parse_add_npc(packet).expect("a real 35/2 AddNpc packet parses"))
        .collect()
}

#[test]
fn both_repair_guys_are_drawable_at_their_legacy_positions() {
    let extras = extra_records();
    let mut spawn_guids = BTreeSet::new();
    let mut extra_keys = BTreeSet::new();
    for record in &extras {
        assert!(
            spawn_guids.insert(record["spawn_guid"].as_u64().expect("extra spawn GUID")),
            "the extra spawn table repeats a GUID"
        );
        if let Some(key) = record["content_key"].as_u64() {
            assert!(extra_keys.insert(key), "content key {key} reused");
        }
    }
    let mut visibility = NpcVisibility::load(&content_dir()).expect("NPC visibility loads");

    for (spawn_guid, content_key, file, legacy_id) in ACTORS {
        let extra = extra_record(spawn_guid);
        let record = npc_record(file, content_key);
        assert_eq!(
            extra["legacy_npc_id"].as_u64(),
            Some(legacy_id),
            "spawn {spawn_guid}: Legacy Npc id"
        );
        assert_eq!(
            extra["content_key"].as_u64(),
            Some(content_key),
            "spawn {spawn_guid}: content_key must be the NPC row's npc_id"
        );
        assert_ne!(content_key, 0);
        assert_eq!(record["npc_id"].as_u64(), Some(content_key));
        assert_eq!(
            record["placement"].as_str(),
            Some("unanchored"),
            "spawn {spawn_guid}: the NPC row must add no second actor"
        );
        assert_eq!(
            record["model_id"].as_u64(),
            Some(REPAIR_MODEL as u64),
            "spawn {spawn_guid}: model"
        );
        assert_eq!(
            record["model_file_name"], extra["model_file_name"],
            "spawn {spawn_guid}: model file copied from the extra row"
        );
        assert_eq!(
            record["position"], extra["position"],
            "spawn {spawn_guid}: position copied from the extra row"
        );
        assert_eq!(
            record["heading"], extra["heading"],
            "spawn {spawn_guid}: heading copied from the extra row"
        );
        assert_eq!(
            record["greeting_string_id"].as_u64(),
            Some(u64::from(REPAIR_GREETING)),
            "spawn {spawn_guid}: one-line greeting"
        );
        assert_eq!(
            visibility.content_key(spawn_guid),
            Some(content_key),
            "spawn {spawn_guid}: the production loader must bind runtime GUID -> content key"
        );
        assert!(
            !visibility
                .content_key(spawn_guid + 1)
                .is_some_and(|key| key == content_key),
            "spawn {spawn_guid}: the neighbouring GUID must not receive this click"
        );

        let at = npc_position(&record, content_key);
        let packets = visibility.update(0, at).expect("NPC visibility update");
        let visible = adds(&packets);
        let matching: Vec<&NpcSpawn> = visible
            .iter()
            .filter(|npc| npc.actor_id == spawn_guid)
            .collect();
        assert_eq!(
            matching.len(),
            1,
            "spawn {spawn_guid}: exactly one drawable 35/2 actor at {at:?}"
        );
        assert_eq!(matching[0].model_id, REPAIR_MODEL);
        for (axis, expected) in at.iter().enumerate() {
            assert!(
                (matching[0].position[axis] - expected).abs() < 0.0001,
                "spawn {spawn_guid}: 35/2 position axis {axis}"
            );
        }
        // The extra row is the only drawable actor at this exact position: the NPC row is
        // unanchored, so the loader must not emit a second copy from the regional rows.
        assert_eq!(
            visibility
                .model_actors()
                .filter(|(_, model, model_at)| {
                    *model == REPAIR_MODEL as u32
                        && model_at
                            .iter()
                            .zip(at)
                            .all(|(left, right)| (*left - right).abs() < 0.0001)
                })
                .count(),
            1,
            "spawn {spawn_guid}: no duplicate actor at the Legacy position"
        );
    }

    // No second NPC row may claim the new ids, and no existing content key may be reused.
    let mut all_npc_ids = BTreeSet::new();
    for entry in fs::read_dir(content_dir().join("npcs")).expect("npcs directory") {
        let path = entry.expect("NPC content entry").path();
        if path.extension().and_then(|value| value.to_str()) != Some("json") {
            continue;
        }
        for record in read_json(&path)["records"].as_array().expect("NPC records") {
            let npc_id = record["npc_id"].as_u64().expect("npc id");
            if ACTORS.iter().any(|(_, key, _, _)| *key == npc_id) {
                assert!(
                    all_npc_ids.insert(npc_id),
                    "NPC id {npc_id} is defined twice"
                );
            }
        }
    }
    for (_, content_key, _, _) in ACTORS {
        assert!(
            all_npc_ids.contains(&content_key),
            "NPC row {content_key} exists"
        );
    }
}

#[test]
fn clicking_a_repair_guy_opens_a_greeting_not_unknown() {
    let fixture = Fixture::new();
    let mut game = fixture.load();
    let before = state_bytes(&game);
    let mut service = service();
    for (spawn_guid, content_key, file, _) in ACTORS {
        let record = npc_record(file, content_key);
        let character = character_at(npc_position(&record, content_key));
        assert_eq!(
            service_talk_body(&mut service, content_key),
            Some(REPAIR_GREETING),
            "actor {content_key}: the real NpcInteraction service path"
        );
        assert_eq!(
            click_and_talk(&mut game, spawn_guid, &character),
            REPAIR_GREETING,
            "actor {spawn_guid}: 26/3 body dword must equal the row's greeting id"
        );
    }
    assert_eq!(
        state_bytes(&game),
        before,
        "a greeting click must not grant an item, coin or XP, change the job or move the player"
    );
}

#[test]
fn a_far_or_wrong_target_refusal_changes_nothing() {
    let fixture = Fixture::new();
    let mut game = fixture.load();
    let (spawn_guid, content_key, file, _) = ACTORS[0];
    let record = npc_record(file, content_key);
    let character = character_at(npc_position(&record, content_key));
    let before: GameplayState = game.state();
    let before_bytes = state_bytes(&game);

    // A GUID that exists in no content-key table takes the unknown_runtime_guid branch.
    let wrong = game
        .handle_in_world(&interact(spawn_guid + 1), &character, NOW)
        .expect("an unknown target is a per-session refusal, not an error");
    assert!(
        menu(&wrong).is_empty(),
        "an unknown runtime GUID must not open a menu"
    );
    assert!(
        talk_body(&wrong).is_none(),
        "an unknown runtime GUID must not answer 26/3"
    );
    assert_eq!(game.state(), before, "wrong GUID: whole GameplayState");
    assert_eq!(state_bytes(&game), before_bytes, "wrong GUID: save bytes");

    // The right GUID from far away is refused by the existing 12 m interaction reach.
    let at = npc_position(&record, content_key);
    let far = character_at([at[0] + 1_000.0, at[1], at[2]]);
    assert!(
        game.handle_in_world(&interact(spawn_guid), &far, NOW + 100)
            .is_ok_and(|out| out.replies.is_empty() && !out.changed),
        "a far click is refused"
    );
    assert_eq!(game.state(), before, "far click: whole GameplayState");
    assert_eq!(state_bytes(&game), before_bytes, "far click: save bytes");
}

#[test]
fn two_players_see_the_same_actor() {
    let fixture = Fixture::new();
    let mut first = fixture.load();
    let mut second = fixture.load();
    for (spawn_guid, content_key, file, _) in ACTORS {
        let record = npc_record(file, content_key);
        let character = character_at(npc_position(&record, content_key));

        // Both sessions get the same 35/2 body for the same runtime actor.
        let first_packets = first
            .update_npc_visibility(&character)
            .expect("first NPC visibility");
        let second_packets = second
            .update_npc_visibility(&character)
            .expect("second NPC visibility");
        let first_add = first_packets
            .iter()
            .filter(|packet| packet.starts_with(&[35, 0, 2, 0]))
            .find(|packet| parse_add_npc(packet).is_ok_and(|npc| npc.actor_id == spawn_guid))
            .expect("first session sees the actor");
        let second_add = second_packets
            .iter()
            .filter(|packet| packet.starts_with(&[35, 0, 2, 0]))
            .find(|packet| parse_add_npc(packet).is_ok_and(|npc| npc.actor_id == spawn_guid))
            .expect("second session sees the actor");
        assert_eq!(first_add, second_add, "actor {spawn_guid}: 35/2 bytes");

        // Each session owns its dialogue state: opening one menu, then the other, then
        // closing the first must leave the second's menu selectable.
        let first_menu = click(&mut first, spawn_guid, &character);
        assert!(!menu(&first_menu).is_empty(), "first menu");
        let first_after_menu = state_bytes(&first);
        let second_menu = click(&mut second, spawn_guid, &character);
        assert!(!menu(&second_menu).is_empty(), "second menu");
        assert_eq!(
            state_bytes(&first),
            first_after_menu,
            "the second session's click must not touch the first session"
        );
        assert_eq!(
            click_and_talk(&mut first, spawn_guid, &character),
            REPAIR_GREETING
        );
        assert_eq!(
            click_and_talk(&mut second, spawn_guid, &character),
            REPAIR_GREETING
        );

        // A close in the first session must not blank the second's actor or menu.
        first
            .handle_in_world(&[26, 0, 4, 0], &character, NOW + 200)
            .expect("dialogue close");
        let second_menu = click(&mut second, spawn_guid, &character);
        let options = menu_options(&menu(&second_menu));
        assert!(!options.is_empty(), "second session after first close");
        assert_eq!(
            talk_body(&choose(
                &mut second,
                spawn_guid,
                options[0].0 as u32,
                &character
            )),
            Some(REPAIR_GREETING)
        );
    }
}

#[test]
fn the_repair_guy_is_not_a_kart_gate() {
    let fixture = Fixture::new();
    let gates = read_json(&content_dir().join("kart_gates.json"));
    let gate_ids: Vec<u64> = gates["npcs"]
        .as_array()
        .expect("kart gate ids")
        .iter()
        .map(|id| id.as_u64().expect("gate id"))
        .collect();
    for (_, content_key, _, _) in ACTORS {
        assert!(
            !gate_ids.contains(&content_key),
            "NPC {content_key} must not be a kart gate"
        );
    }

    let mut game = fixture.load();
    // Positive control: the same fixture publishes the race option for a real shipped gate.
    let gate = npc_record("seaside.json", KART_GATE);
    let gate_menu = menu(&click(
        &mut game,
        KART_GATE,
        &character_at(npc_position(&gate, KART_GATE)),
    ));
    assert!(
        menu_options(&gate_menu)
            .iter()
            .any(|(_, label)| *label == DEFAULT_MENU_STRING_ID),
        "the fixture must publish the kart gate menu (Kart Race Lobby {DEFAULT_MENU_STRING_ID})"
    );

    let before_gate = state_bytes(&game);
    for (spawn_guid, content_key, file, _) in ACTORS {
        let record = npc_record(file, content_key);
        let character = character_at(npc_position(&record, content_key));
        let before = state_bytes(&game);
        let opened = click(&mut game, spawn_guid, &character);
        let options = menu_options(&menu(&opened));
        assert_eq!(
            options.len(),
            1,
            "actor {spawn_guid}: Talk only, no race option"
        );
        assert_ne!(
            options[0].1, DEFAULT_MENU_STRING_ID,
            "actor {spawn_guid}: the only option must not be the kart lobby"
        );
        // A forged second choice (the race slot on a real gate) is refused and changes nothing.
        assert!(
            game.handle_in_world(&select(spawn_guid, 2), &character, NOW + 150)
                .is_ok_and(|out| out.replies.is_empty() && !out.changed),
            "actor {spawn_guid}: selecting an unoffered race option is refused"
        );
        assert_eq!(
            state_bytes(&game),
            before,
            "actor {spawn_guid}: a forged race choice changes no saved state"
        );
    }
    assert_eq!(
        state_bytes(&game),
        before_gate,
        "opening the repair guys' menus and refusing forged choices changes no saved state"
    );
}

#[test]
fn a_busy_player_is_refused_and_the_menu_is_not_opened() {
    let fixture = Fixture::new();
    mining::write_content(&fixture.0, mining::MINING_GAME_ID).expect("mining test content");
    let base = fixture.load();
    let mut state = base.state();
    let mut engine = mining::Type8Engine::load(
        &fixture.0,
        mining::CONTENT_FILE,
        mining::MINING_GAME_ID,
        mining::FIRST_MINING_INSTANCE,
        None,
    )
    .expect("mining engine");
    engine
        .begin(mining::MINER_PROFILE_ID)
        .expect("start the busy Mining run");
    state.mining = Some(engine.state().clone());
    let mut game = Gameplay::load(&fixture.0, Some(state)).expect("busy gameplay content");
    assert!(
        game.state().mining.is_some(),
        "the busy Mining run must load"
    );

    let (spawn_guid, content_key, file, _) = ACTORS[1];
    let record = npc_record(file, content_key);
    let character = character_at(npc_position(&record, content_key));
    let before = state_bytes(&game);
    let refused = click(&mut game, spawn_guid, &character);
    assert!(
        menu(&refused).is_empty(),
        "a busy player must not get the repair guy's menu"
    );
    assert!(
        talk_body(&refused).is_none(),
        "a busy refusal must not answer 26/3"
    );
    assert_eq!(
        state_bytes(&game),
        before,
        "a busy refusal changes no save state"
    );
    assert!(
        game.state().mining.is_some(),
        "the activity is still active"
    );
}
