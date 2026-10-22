//! T298: Lakeshore and Wilds drawable NPC rows carry recovered locale greetings.
//!
//! The body is the clicked actor's content row `greeting_string_id`, not the nearest
//! locale line. The service path (`NpcInteraction::interact` -> `select`) is the one
//! `gameplay/npc.rs` calls; the dispatch path (`Gameplay::handle_in_world`) drives the
//! real router and `dialogue_packet_with`. The 26/3 body is the first dword after the
//! four-byte header: client handler `00aa12e0` calls reader `00a9ef10` (stores that
//! dword at object `+16`), and display routine `009f5ae0` passes `*(a2+16)` to the Lua
//! lookup before `:SetDialogText`. The byte layout is pinned by
//! `tests/npc_interaction.rs::dialogue_26_3_synthetic_fixture_round_trips_exact_native_field_order`.
//! Locale ids, hashes, dat lines and speaker proofs are in
//! `notes/Research/NPC Greetings Lakeshore and Wilds.md`.
//!
//! The packet builders, the locale line convention and the 26/3 body offset
//! live in the shared `tests/common/npc_greetings.rs` harness.

#[path = "common/npc_greetings.rs"]
mod npc_greetings;

use npc_greetings::{
    content_dir, gameplay_dir, gameplay_talk_body, npc_position, npc_record, read_json, select,
    service, service_talk_body,
};
use osfr_tools::{
    gameplay::Gameplay,
    npc_interaction::{NpcAction, NpcInteractionError, NpcSelection},
    world::NpcVisibility,
};
use serde_json::Value;
use std::{collections::BTreeSet, fs};

/// (npc id, content file, greeting id, locale hash, en_us_data.dat line,
///  proof guide, guide line, content quest (file, id) for the offer speech)
const NEW_GREETINGS: [(u64, &str, u32, u32, u32, &str, u32, Option<(&str, u32)>); 19] = [
    (
        9000000223,
        "lakeshore.json",
        24829,
        1824578395,
        48173,
        "Jobs/Ninja.md",
        69,
        Some(("ninja.json", 3110001)),
    ),
    (
        9000000233,
        "lakeshore.json",
        409412,
        1256773114,
        33294,
        "Jobs/Fisherman.md",
        124,
        Some(("fisherman.json", 3070033)),
    ),
    (
        9500000363,
        "lakeshore.json",
        41207,
        3015081219,
        79651,
        "NPCs/NPCs - Lakeshore.md",
        45,
        None,
    ),
    (
        9000000243,
        "wilds.json",
        387488,
        3163514493,
        83481,
        "Jobs/Warrior.md",
        101,
        Some(("warrior.json", 3150010)),
    ),
    (
        9000000273,
        "wilds.json",
        394709,
        1007521181,
        26689,
        "Jobs/Kart Driver.md",
        92,
        Some(("kart_driver.json", 3080029)),
    ),
    (
        9000000317,
        "wilds.json",
        130251,
        4010850362,
        106064,
        "Jobs/Postman.md",
        129,
        Some(("postman.json", 3130033)),
    ),
    (
        9000000324,
        "wilds.json",
        24947,
        698508245,
        18388,
        "Jobs/Wizard.md",
        76,
        Some(("wizard.json", 3160003)),
    ),
    (
        9000000332,
        "wilds.json",
        73077,
        3466351639,
        91803,
        "Jobs/Chef.md",
        136,
        Some(("chef.json", 3050030)),
    ),
    (
        9000000347,
        "wilds.json",
        104327,
        2193026908,
        57742,
        "Jobs/Card Duelist.md",
        112,
        Some(("card_duelist.json", 3040027)),
    ),
    (
        9000000415,
        "wilds.json",
        141009,
        2705176635,
        71216,
        "Jobs/Brawler.md",
        136,
        Some(("brawler.json", 3030028)),
    ),
    (
        9000000477,
        "wilds.json",
        391043,
        3886409354,
        102718,
        "Jobs/Adventurer.md",
        115,
        Some(("adventurer.json", 3000027)),
    ),
    (
        9000000488,
        "wilds.json",
        73492,
        1322558947,
        35003,
        "Jobs/Medic.md",
        87,
        Some(("medic.json", 3090010)),
    ),
    (
        9500000869,
        "wilds.json",
        115443,
        1892561356,
        49906,
        "Jobs/Card Duelist.md",
        110,
        Some(("card_duelist.json", 3040025)),
    ),
    (
        9000000335,
        "wilds.json",
        78924,
        2060862346,
        54301,
        "Jobs/Ninja.md",
        70,
        Some(("ninja.json", 3110002)),
    ),
    (
        9500000893,
        "wilds.json",
        25001,
        1008851474,
        26732,
        "NPCs/NPCs - Wilds.md",
        42,
        None,
    ),
    (
        9500000958,
        "wilds.json",
        21014,
        932035218,
        24650,
        "NPCs/NPCs - Wilds.md",
        116,
        None,
    ),
    (
        9500000954,
        "wilds.json",
        20984,
        217873432,
        5721,
        "NPCs/NPCs - Wilds.md",
        124,
        None,
    ),
    (
        9500000876,
        "wilds.json",
        73378,
        2001002164,
        52697,
        "NPCs/NPCs - Wilds.md",
        126,
        None,
    ),
    (
        9500000907,
        "wilds.json",
        9599,
        2276171514,
        59936,
        "NPCs/NPCs - Wilds.md",
        198,
        None,
    ),
];

/// The 19 W126 bindings T246 found; this slice must not touch them.
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

/// Bindings added after this suite was written (T378, T369). They are expected
/// in the two owned files, not a T298 regression.
const LATER_GREETINGS: [(&str, u64, u32); 3] = [
    ("wilds.json", 9500000941, 94519),
    ("wilds.json", 9000000289, 393376),
    ("wilds.json", 9000000295, 395495),
];

/// Sam Durango: a drawable Lakeshore trainer with no recovered speaker line.
/// He stays blank, and a blank row must not inherit another row's body.
const BLANK: u64 = 9500000364;

const LAKESHORE_RECORDS: usize = 39;
const WILDS_RECORDS: usize = 118;

fn assert_drawable(record: &Value, npc_id: u64) {
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
}

#[test]
fn the_new_rows_have_nonzero_greetings_and_real_sources() {
    let mut greeting_ids = BTreeSet::new();
    for (npc_id, file, greeting, hash, dat_line, guide, guide_line, quest) in NEW_GREETINGS {
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
        assert_drawable(&record, npc_id);

        let sources = record["field_sources"]["greeting_string_id"]
            .as_array()
            .unwrap_or_else(|| panic!("NPC {npc_id}: greeting sources"));
        let clients = sources
            .iter()
            .filter(|source| source["kind"] == "client")
            .count();
        assert_eq!(clients, 1, "NPC {npc_id}: exactly one client source");
        assert!(
            sources
                .iter()
                .any(|source| source["kind"] == "reconstructed"),
            "NPC {npc_id}: fixed-default policy source"
        );
        assert!(
            sources.iter().any(|source| {
                source["kind"] == "reconstructed"
                    && source["evidence"]
                        .as_str()
                        .is_some_and(|text| text.contains("fixed default Talk greeting"))
            }),
            "NPC {npc_id}: fixed-default Talk policy"
        );
        let evidence = sources
            .iter()
            .map(|source| source["evidence"].as_str().unwrap_or_default())
            .collect::<Vec<_>>()
            .join(" ");
        assert!(
            evidence.contains(&format!("Global.Text.{greeting}")),
            "NPC {npc_id}: source must name its Global.Text id"
        );
        assert!(
            evidence.contains(&format!("JenkinsLookup2 hash {hash}")),
            "NPC {npc_id}: source must name locale hash {hash}"
        );
        assert!(
            evidence.contains(&format!("line {dat_line}")),
            "NPC {npc_id}: source must name the en_us_data.dat line {dat_line}"
        );
        assert!(
            evidence.contains(guide) && evidence.contains(&format!("line {guide_line}")),
            "NPC {npc_id}: source must name {guide} line {guide_line}"
        );
        if let Some((quest_file, quest_id)) = quest {
            assert!(
                evidence.contains(&format!("data/content/quests/{quest_file}"))
                    && evidence.contains(&format!("quest {quest_id}")),
                "NPC {npc_id}: source must name its quest record"
            );
        }
        assert!(
            sources.iter().any(|source| source["evidence"]
                .as_str()
                .is_some_and(|text| text.starts_with("(R)"))),
            "NPC {npc_id}: reconstructed evidence must start with (R)"
        );
    }
}

#[test]
fn lakeshore_and_wilds_rows_send_their_greeting_in_26_3() {
    // Service path: the context the router copies into the 26/3 body.
    let mut interaction = service();
    for (npc_id, _, greeting, _, _, _, _, _) in NEW_GREETINGS {
        assert_eq!(
            service_talk_body(&mut interaction, npc_id),
            Some(greeting),
            "NPC {npc_id}: service Talk body"
        );
    }

    // Dispatch path: gateway -> Gameplay -> gameplay/npc.rs -> dialogue_packet_with.
    let mut game = Gameplay::load(&gameplay_dir(), None).expect("gameplay content loads");
    for (npc_id, file, greeting, _, _, _, _, _) in NEW_GREETINGS {
        assert_eq!(
            gameplay_talk_body(&mut game, file, npc_id),
            greeting,
            "NPC {npc_id}: in-world 26/3 body"
        );
    }

    // A reconnect rebuilds the same static content; the body is not session state.
    let (first_npc, _, first_greeting, ..) = NEW_GREETINGS[0];
    assert_eq!(
        service_talk_body(&mut service(), first_npc),
        Some(first_greeting),
        "NPC {first_npc}: a fresh loader sends the same body"
    );
}

#[test]
fn a_zero_or_missing_greeting_falls_back_to_no_body() {
    let mut service = service();
    assert_eq!(
        service_talk_body(&mut service, BLANK),
        None,
        "an NPC without a greeting has no body"
    );
    assert!(
        NEW_GREETINGS
            .iter()
            .all(|(_, _, greeting, _, _, _, _, _)| *greeting != 0),
        "zero is not a recovered greeting id"
    );

    // A refused choice must leave the menu usable. Select Talk directly on the
    // still-open menu (no second `interact`), so this fails if `select` consumes
    // the open menu on `InvalidOption`.
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
            "the refused choice must not consume the menu or inject a body"
        ),
        _ => panic!("the refused choice must leave the menu usable"),
    }
    // An unknown NPC id resolves to no row at all; a client cannot choose a body.
    assert!(matches!(
        service.interact(123, zone, position, None),
        Err(NpcInteractionError::UnknownNpc)
    ));

    // The real router still encodes 26/3 with a zero body and the one goodbye row.
    let mut game = Gameplay::load(&gameplay_dir(), None).expect("gameplay content loads");
    assert_eq!(
        gameplay_talk_body(&mut game, "lakeshore.json", BLANK),
        0,
        "no invented locale id"
    );
}

#[test]
fn the_target_rows_are_spawned_and_non_target_rows_are_unchanged() {
    let mut visibility = NpcVisibility::load(&content_dir()).expect("NPC visibility loads");
    for (npc_id, file, _, _, _, _, _, _) in NEW_GREETINGS {
        let position = npc_position(&npc_record(file, npc_id), npc_id);
        // Clear the previous interest set, then look from this NPC's own feet so
        // it is the nearest candidate (distance 0).
        visibility.update(999, [0.0; 3]).unwrap();
        let packets = visibility.update(0, position).unwrap();
        assert!(
            packets.iter().any(|packet| {
                packet.len() >= 12
                    && packet[..4] == [35, 0, 2, 0]
                    && u64::from_le_bytes(packet[4..12].try_into().unwrap()) == npc_id
            }),
            "NPC {npc_id}: NpcVisibility must spawn the row at its content position"
        );
    }

    // Non-target rows keep no greeting (the loader spawns legacy_data only).
    let targets = NEW_GREETINGS
        .iter()
        .map(|(npc_id, file, _, _, _, _, _, _)| (*file, *npc_id))
        .collect::<BTreeSet<_>>();
    for (file, expected) in [
        ("lakeshore.json", LAKESHORE_RECORDS),
        ("wilds.json", WILDS_RECORDS),
    ] {
        let path = content_dir().join("npcs").join(file);
        let document = read_json(&path);
        let records = document["records"].as_array().unwrap();
        assert_eq!(records.len(), expected, "{file}: row count");
        for record in records {
            let npc_id = record["npc_id"].as_u64().unwrap();
            if targets.contains(&(file, npc_id)) {
                continue;
            }
            if let Some((_, _, greeting)) = LATER_GREETINGS
                .iter()
                .find(|(known_file, known_id, _)| *known_file == file && *known_id == npc_id)
            {
                assert_eq!(
                    record["greeting_string_id"].as_u64(),
                    Some(u64::from(*greeting)),
                    "NPC {npc_id}: later recovered binding"
                );
                continue;
            }
            assert!(
                record["greeting_string_id"].as_u64().unwrap_or(0) == 0,
                "NPC {npc_id}: non-target row gained a greeting"
            );
            assert!(
                record["field_sources"].get("greeting_string_id").is_none(),
                "NPC {npc_id}: non-target row gained greeting sources"
            );
        }
    }

    // The blank control is still a drawable row, so its empty body is reachable.
    assert_drawable(&npc_record("lakeshore.json", BLANK), BLANK);
}

#[test]
fn the_existing_greetings_and_other_regions_are_unchanged() {
    for (file, npc_id, greeting) in EXISTING_GREETINGS {
        assert_eq!(
            npc_record(file, npc_id)["greeting_string_id"].as_u64(),
            Some(u64::from(greeting)),
            "existing W126 binding {npc_id} changed"
        );
    }

    // Exactly the 19 target rows plus the later recovered bindings are greeted in
    // the two owned files; no existing binding is overwritten and no duplicate
    // speaker gains the same body.
    let mut expected = NEW_GREETINGS
        .iter()
        .map(|(npc_id, _, greeting, _, _, _, _, _)| (*npc_id, *greeting))
        .collect::<BTreeSet<_>>();
    expected.extend(
        LATER_GREETINGS
            .iter()
            .map(|(_, npc_id, greeting)| (*npc_id, *greeting)),
    );
    let mut found = BTreeSet::new();
    for file in ["lakeshore.json", "wilds.json"] {
        for record in read_json(&content_dir().join("npcs").join(file))["records"]
            .as_array()
            .unwrap()
        {
            if let Some(greeting) = record["greeting_string_id"].as_u64().filter(|id| *id > 0) {
                found.insert((record["npc_id"].as_u64().unwrap(), greeting as u32));
            }
        }
    }
    assert_eq!(
        found, expected,
        "greeted rows changed outside the target set"
    );
}

#[test]
fn the_content_stays_within_the_loader_bounds() {
    let data = content_dir().join("npcs");
    let mut total = 0u64;
    for file in ["lakeshore.json", "wilds.json"] {
        let path = data.join(file);
        let bytes = fs::read(&path).unwrap();
        let size = bytes.len() as u64;
        assert!(
            size <= 8 * 1024 * 1024,
            "{file}: exceeds the 8 MiB per-file loader bound"
        );
        total += size;
        std::str::from_utf8(&bytes).unwrap_or_else(|error| panic!("{file}: {error}"));
        let rows = read_json(&path)["records"].as_array().unwrap().len();
        assert!(
            rows <= 20_000,
            "{file}: exceeds the 20,000-row loader bound"
        );
    }
    assert!(
        total <= 32 * 1024 * 1024,
        "the two edited files exceed the 32 MiB loader bound"
    );
    // The loader itself is the row-count/shape gate.
    let _ = service();

    for (npc_id, file, greeting, hash, dat_line, guide, guide_line, _) in NEW_GREETINGS {
        assert!(greeting > 0 && hash > 0 && dat_line > 0, "NPC {npc_id}");
        assert!(!guide.is_empty() && guide_line > 0, "NPC {npc_id}");
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
            if source["kind"] == "reconstructed" {
                assert!(
                    evidence.starts_with("(R)"),
                    "NPC {npc_id}: (R) sources must start with (R)"
                );
            }
        }
    }
}
