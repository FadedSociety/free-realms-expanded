//! T246: the starting-road NPC rows carry recovered locale greetings.
//!
//! The service path (`NpcInteraction::interact` -> `select`) is the same one
//! `gameplay/npc.rs` uses; the dispatch path (`Gameplay::handle_in_world`) drives
//! the real router and `dialogue_packet_with`. The 26/3 body is the first dword
//! after the header (client reader `009f5ae0`); the byte layout is pinned by
//! `tests/npc_interaction.rs::dialogue_26_3_synthetic_fixture_round_trips_exact_native_field_order`.
//! Locale ids, hashes, dat lines and speaker proofs are in
//! `notes/Research/NPC Greetings Starting Road.md`.

use osfr_tools::{
    character_store::CharacterState,
    gameplay::Gameplay,
    npc_interaction::{NpcAction, NpcInteraction, NpcInteractionError, NpcSelection},
};
use serde_json::Value;
use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

const GREETING_HEADER: [u8; 4] = [26, 0, 3, 0];
const MENU_HEADER: [u8; 4] = [26, 0, 9, 0];

/// (npc id, content file, greeting id, region guide file, guide row line)
const NEW_GREETINGS: [(u64, &str, u32, &str, u32); 16] = [
    (
        9000000104,
        "farnum_farmstead.json",
        391041,
        "NPCs - Farnum Farmstead.md",
        40,
    ),
    (
        9500000271,
        "farnum_farmstead.json",
        391391,
        "NPCs - Farnum Farmstead.md",
        59,
    ),
    (
        9000000396,
        "farnum_farmstead.json",
        5100509,
        "NPCs - Farnum Farmstead.md",
        38,
    ),
    (
        9500000266,
        "farnum_farmstead.json",
        101014,
        "NPCs - Farnum Farmstead.md",
        50,
    ),
    (
        9000000050,
        "farnum_farmstead.json",
        5100125,
        "NPCs - Farnum Farmstead.md",
        39,
    ),
    (
        9500000245,
        "cobblestone_village.json",
        388245,
        "NPCs - Cobblestone Village.md",
        62,
    ),
    (
        9500000254,
        "cobblestone_village.json",
        94388,
        "NPCs - Cobblestone Village.md",
        40,
    ),
    (
        9000000015,
        "highroad_junction.json",
        436533,
        "NPCs - Highroad Junction.md",
        46,
    ),
    (
        9000000102,
        "highroad_junction.json",
        140509,
        "NPCs - Highroad Junction.md",
        45,
    ),
    (
        9000000019,
        "highroad_junction.json",
        431140,
        "NPCs - Highroad Junction.md",
        47,
    ),
    (
        9000000028,
        "sanctuary.json",
        387246,
        "NPCs - Sanctuary.md",
        46,
    ),
    (
        9000000052,
        "sanctuary.json",
        73782,
        "NPCs - Sanctuary.md",
        43,
    ),
    (
        9000000073,
        "sanctuary.json",
        76747,
        "NPCs - Sanctuary.md",
        42,
    ),
    (
        9000000161,
        "sanctuary.json",
        73360,
        "NPCs - Sanctuary.md",
        41,
    ),
    (
        9000000205,
        "sanctuary.json",
        71435,
        "NPCs - Sanctuary.md",
        33,
    ),
    (
        9000000126,
        "sanctuary.json",
        76438,
        "NPCs - Sanctuary.md",
        62,
    ),
];

/// The 19 W126 bindings that this slice must not touch.
const EXISTING_GREETINGS: [(&str, u64, u32); 19] = [
    ("cobblestone_village.json", 9000000021, 4420092),
    ("cobblestone_village.json", 9000000120, 94293),
    ("cobblestone_village.json", 9000000121, 94297),
    ("cobblestone_village.json", 9000000189, 24940),
    ("farnum_farmstead.json", 9000000165, 4420036),
    ("highroad_junction.json", 9000000402, 80),
    ("highroad_junction.json", 9000000177, 104100),
    ("sanctuary.json", 9500000450, 441420),
    ("sanctuary.json", 9000000091, 24842),
    ("sanctuary.json", 9500000487, 37872),
    ("sanctuary.json", 9000000207, 46166),
    ("sanctuary.json", 9500000541, 21602),
    ("seaside.json", 9500000563, 441432),
    ("seaside.json", 9500000639, 441037),
    ("snowhill.json", 9000000093, 430860),
    ("snowhill.json", 9000000158, 5696),
    ("stillwater_crossing.json", 9500000786, 116463),
    ("sunstone_valley.json", 9500000824, 413082),
    ("merry_vale.json", 9500000430, 41268),
];

/// Cleetus Farnum, a legacy-data farm NPC that is spawned by `NpcVisibility`.
const CHANGED: u64 = 9000000050;
const CHANGED_GREETING: u32 = 5100125;
/// Merigold's W126 binding, used as the unchanged control on the same path.
const CONTROL: u64 = 9000000402;
const CONTROL_GREETING: u32 = 80;
/// A legacy-data starting NPC that has no greeting.
const BLANK: u64 = 9500000269;

fn content_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content")
}

/// Full server content used by the existing in-world router tests. It has no
/// `npcs/` or `npc_spawns_extra.json`, so `Gameplay::load` takes the NPC rows
/// from the bundled `data/content` (the same rows this test edits).
fn gameplay_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../lab/references/opening-anvil-placement-v1/content")
}

fn read_json(path: &Path) -> Value {
    serde_json::from_slice(
        &fs::read(path).unwrap_or_else(|error| panic!("{}: {error}", path.display())),
    )
    .unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

fn npc_record(file: &str, npc_id: u64) -> Value {
    let path = content_dir().join("npcs").join(file);
    let document = read_json(&path);
    document["records"]
        .as_array()
        .unwrap_or_else(|| panic!("{}: records array", path.display()))
        .iter()
        .find(|record| record["npc_id"].as_u64() == Some(npc_id))
        .unwrap_or_else(|| panic!("{}: NPC {npc_id}", path.display()))
        .clone()
}

fn npc_location(file: &str, npc_id: u64) -> [f32; 3] {
    let record = npc_record(file, npc_id);
    let position = record["position"]
        .as_array()
        .unwrap_or_else(|| panic!("NPC {npc_id} has no position"));
    assert_eq!(position.len(), 3, "NPC {npc_id} position shape");
    [
        position[0].as_f64().expect("x") as f32,
        position[1].as_f64().expect("y") as f32,
        position[2].as_f64().expect("z") as f32,
    ]
}

fn service() -> NpcInteraction {
    let data = content_dir();
    NpcInteraction::load(&data.join("npcs"), &data.join("stores.json"))
        .expect("NPC content loads through the real loader")
}

fn interact_request(npc_id: u64) -> Vec<u8> {
    // 26/8 CommandPacketInteractRequest (constructor 0x008b6bc0).
    let mut packet = vec![26, 0, 8, 0];
    packet.extend(npc_id.to_le_bytes());
    packet.push(0);
    packet
}

fn select_request(npc_id: u64, choice: i32) -> Vec<u8> {
    // 26/10 CommandPacketInteractionSelect (constructor 0x00a36f80).
    let mut packet = vec![26, 0, 10, 0];
    packet.extend(npc_id.to_le_bytes());
    packet.extend(choice.to_le_bytes());
    packet
}

/// Real Talk selection through the service the router calls.
fn service_talk_body(service: &mut NpcInteraction, npc_id: u64) -> Option<u32> {
    let (zone, position) = service
        .location(npc_id)
        .unwrap_or_else(|| panic!("NPC {npc_id} has a location"));
    let menu = service
        .interact(npc_id, zone, position, None)
        .unwrap_or_else(|error| panic!("NPC {npc_id} menu: {error:?}"));
    let talk = menu
        .options
        .iter()
        .find(|option| option.action == NpcAction::Talk)
        .unwrap_or_else(|| panic!("NPC {npc_id} offers Talk"));
    let selected = service
        .select(&select_request(npc_id, talk.id), zone, position, None)
        .unwrap_or_else(|error| panic!("NPC {npc_id} Talk: {error:?}"));
    match selected {
        NpcSelection::Talk(context) => context.body_string_id,
        _ => panic!("NPC {npc_id} Talk must return dialogue"),
    }
}

fn character_at(position: [f32; 3]) -> CharacterState {
    CharacterState {
        guid: 101,
        zone_id: 0,
        position,
        rotation: [1.0, 0.0, 0.0],
    }
}

/// Real in-world Talk: 26/8 opens the menu, 26/10 selects Talk, and the 26/3
/// reply is decoded from the production packet.
fn gameplay_talk_body(game: &mut Gameplay, file: &str, npc_id: u64) -> u32 {
    let character = character_at(npc_location(file, npc_id));
    let menu = game
        .handle_in_world(&interact_request(npc_id), &character, 1_000)
        .unwrap_or_else(|error| panic!("NPC {npc_id} interact: {error}"));
    assert!(
        menu.replies
            .iter()
            .any(|packet| packet.starts_with(&MENU_HEADER)),
        "NPC {npc_id} must answer 26/8 with a 26/9 menu"
    );
    let outcome = game
        .handle_in_world(&select_request(npc_id, 1), &character, 1_100)
        .unwrap_or_else(|error| panic!("NPC {npc_id} Talk: {error}"));
    let dialogue = outcome
        .replies
        .into_iter()
        .find(|packet| packet.starts_with(&GREETING_HEADER))
        .unwrap_or_else(|| panic!("NPC {npc_id} Talk must answer 26/3"));
    // Body dword follows the four-byte family/subtype header; patch_npc_actor
    // writes the runtime actor GUID only into bytes 12..20.
    u32::from_le_bytes(dialogue[4..8].try_into().unwrap())
}

#[test]
fn the_starting_road_npcs_have_recovered_greetings() {
    let mut service = service();
    let mut greeting_ids = BTreeSet::new();
    for (npc_id, file, greeting, guide, guide_line) in NEW_GREETINGS {
        assert!(greeting > 0, "NPC {npc_id}: greeting must be positive");
        assert!(
            greeting_ids.insert(greeting),
            "NPC {npc_id}: greeting {greeting} reused"
        );

        let record = npc_record(file, npc_id);
        assert_eq!(
            record["greeting_string_id"].as_u64(),
            Some(u64::from(greeting)),
            "NPC {npc_id}: greeting binding"
        );
        assert!(
            record["position"]
                .as_array()
                .is_some_and(|position| position.len() == 3),
            "NPC {npc_id}: position"
        );
        assert!(
            record["name"].as_str().is_some_and(|name| !name.is_empty()),
            "NPC {npc_id}: name"
        );
        assert!(
            record["roles"]
                .as_array()
                .is_some_and(|roles| !roles.is_empty()),
            "NPC {npc_id}: roles"
        );
        assert_eq!(
            record["placement"].as_str(),
            Some("legacy_data"),
            "NPC {npc_id}: must stay a spawned placement"
        );

        let sources = record["field_sources"]["greeting_string_id"]
            .as_array()
            .unwrap_or_else(|| panic!("NPC {npc_id}: greeting sources"));
        assert!(!sources.is_empty(), "NPC {npc_id}: empty greeting sources");
        let evidence = sources
            .iter()
            .map(|source| source["evidence"].as_str().unwrap_or_default())
            .collect::<Vec<_>>()
            .join(" ");
        assert!(
            evidence.contains(&greeting.to_string()),
            "NPC {npc_id}: source must name the greeting id"
        );
        assert!(
            evidence.contains(guide) && evidence.contains(&format!("line {guide_line}")),
            "NPC {npc_id}: source must name {guide} line {guide_line}"
        );

        assert_eq!(
            service_talk_body(&mut service, npc_id),
            Some(greeting),
            "NPC {npc_id}: real Talk body"
        );
    }

    for (file, npc_id, greeting) in EXISTING_GREETINGS {
        assert_eq!(
            npc_record(file, npc_id)["greeting_string_id"].as_u64(),
            Some(u64::from(greeting)),
            "existing W126 binding {npc_id} changed"
        );
    }
}

#[test]
fn a_starting_road_npc_sends_its_greeting_in_26_3() {
    // Service path: the context the router copies into the 26/3 body.
    let mut service = service();
    assert_eq!(
        service_talk_body(&mut service, CHANGED),
        Some(CHANGED_GREETING)
    );
    assert_eq!(
        service_talk_body(&mut service, CONTROL),
        Some(CONTROL_GREETING)
    );

    // Dispatch path: gateway -> Gameplay -> gameplay/npc.rs -> dialogue_packet_with.
    let mut game = Gameplay::load(&gameplay_dir(), None).expect("gameplay content loads");
    assert_eq!(
        gameplay_talk_body(&mut game, "farnum_farmstead.json", CHANGED),
        CHANGED_GREETING
    );
    assert_eq!(
        gameplay_talk_body(&mut game, "highroad_junction.json", CONTROL),
        CONTROL_GREETING
    );
}

#[test]
fn missing_or_zero_greeting_still_falls_back_to_an_empty_body() {
    let mut service = service();
    assert_eq!(
        service_talk_body(&mut service, BLANK),
        None,
        "an NPC without a greeting has no body"
    );

    // A refused choice must leave the menu usable.
    let (zone, position) = service.location(BLANK).unwrap();
    service.interact(BLANK, zone, position, None).unwrap();
    assert!(matches!(
        service.select(&select_request(BLANK, i32::MAX), zone, position, None),
        Err(NpcInteractionError::InvalidOption)
    ));
    assert_eq!(
        service_talk_body(&mut service, BLANK),
        None,
        "the refused choice must not consume the menu"
    );

    // The real router still encodes 26/3 with a zero body and the one goodbye row.
    let mut game = Gameplay::load(&gameplay_dir(), None).expect("gameplay content loads");
    assert_eq!(
        gameplay_talk_body(&mut game, "farnum_farmstead.json", BLANK),
        0,
        "no invented locale id"
    );
}

#[test]
fn the_content_rows_stay_bounded_and_sourced() {
    let data = content_dir().join("npcs");
    let mut total = 0u64;
    for file in [
        "farnum_farmstead.json",
        "cobblestone_village.json",
        "highroad_junction.json",
        "sanctuary.json",
    ] {
        let path = data.join(file);
        let bytes = fs::read(&path).unwrap();
        let size = bytes.len() as u64;
        assert!(
            size <= 8 * 1024 * 1024,
            "{file}: exceeds the 8 MiB per-file loader bound"
        );
        total += size;
        std::str::from_utf8(&bytes).unwrap_or_else(|error| panic!("{file}: {error}"));
    }
    assert!(
        total <= 32 * 1024 * 1024,
        "the four edited files exceed the 32 MiB loader bound"
    );
    // The loader itself is the row-count/shape gate.
    let _ = service();

    for (npc_id, file, _, _, _) in NEW_GREETINGS {
        let record = npc_record(file, npc_id);
        let sources = record["field_sources"]["greeting_string_id"]
            .as_array()
            .unwrap_or_else(|| panic!("NPC {npc_id}: greeting sources"));
        for source in sources {
            let evidence = source["evidence"].as_str().unwrap_or_default();
            assert!(
                !evidence.trim().is_empty(),
                "NPC {npc_id}: empty source evidence"
            );
            assert!(
                !evidence.chars().any(char::is_control),
                "NPC {npc_id}: control character in source"
            );
            assert!(
                evidence.len() <= 4096,
                "NPC {npc_id}: source evidence too long"
            );
        }
    }
}
