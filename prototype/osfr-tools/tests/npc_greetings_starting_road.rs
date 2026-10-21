//! T246: the starting-road NPC rows carry recovered locale greetings.
//!
//! The service path (`NpcInteraction::interact` -> `select`) is the same one
//! `gameplay/npc.rs` uses; the dispatch path (`Gameplay::handle_in_world`) drives
//! the real router and `dialogue_packet_with`. The 26/3 body is the first dword
//! after the header: client handler `00aa12e0` calls reader `00a9ef10` (stores
//! that dword at object `+16`), and display routine `009f5ae0` passes it to the
//! locale lookup before `:SetDialogText`. The byte layout is pinned by
//! `tests/npc_interaction.rs::dialogue_26_3_synthetic_fixture_round_trips_exact_native_field_order`.
//! Locale ids, hashes, dat lines and speaker proofs are in
//! `notes/Research/NPC Greetings Starting Road.md`.
//!
//! The packet builders, the locale line convention and the 26/3 body offset
//! live in the shared `tests/common/npc_greetings.rs` harness.

#[path = "common/npc_greetings.rs"]
mod npc_greetings;

use npc_greetings::{
    content_dir, gameplay_dir, gameplay_talk_body, npc_record, select, service, service_talk_body,
};
use osfr_tools::{
    gameplay::Gameplay,
    npc_interaction::{NpcAction, NpcInteractionError, NpcSelection},
};
use std::{collections::BTreeSet, fs};

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
        assert!(
            record["model_id"].as_u64().is_some_and(|id| id > 0),
            "NPC {npc_id}: model id (NpcVisibility spawns model_id.unwrap_or(0))"
        );
        assert!(
            record["name_string_id"].as_u64().is_some_and(|id| id > 0),
            "NPC {npc_id}: name string id"
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

    // A refused choice must leave the menu usable. Select Talk directly on the
    // still-open menu (no second `interact`), so this fails if `select` consumes
    // `active` on `InvalidOption`.
    let (zone, position) = service.location(BLANK).unwrap();
    let menu = service.interact(BLANK, zone, position, None).unwrap();
    let talk = menu
        .options
        .iter()
        .find(|option| option.action == NpcAction::Talk)
        .expect("the blank NPC offers Talk");
    assert!(matches!(
        service.select(&select(BLANK, i32::MAX as u32), zone, position, None),
        Err(NpcInteractionError::InvalidOption)
    ));
    match service.select(&select(BLANK, talk.id as u32), zone, position, None) {
        Ok(NpcSelection::Talk(context)) => assert_eq!(
            context.body_string_id, None,
            "the refused choice must not consume the menu"
        ),
        _ => panic!("the refused choice must leave the menu usable"),
    }

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
