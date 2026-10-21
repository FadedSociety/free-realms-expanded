//! T299: the Snowhill and Greenwood Forest job-hub NPC rows carry recovered
//! locale greetings.
//!
//! These tests drive the same production paths T246 used. The service path is
//! `NpcInteraction::interact` -> `select`; the dispatch path is
//! `Gameplay::handle_in_world` -> `gameplay/npc.rs` -> `dialogue_packet_with`.
//! The 26/3 body is the first dword after the four-byte header: client handler
//! `00aa12e0` calls reader `00a9ef10` (which stores that dword at object `+16`)
//! and display routine `009f5ae0` passes it to the Lua `Global.Text` lookup
//! before `:SetDialogText`. The byte layout is pinned by
//! `tests/npc_interaction.rs::dialogue_26_3_synthetic_fixture_round_trips_exact_native_field_order`.
//! Locale ids, JenkinsLookup2 hashes, physical `en_us_data.dat` lines and
//! speaker proofs are in `notes/Autopilot/T299.md`.
//!
//! The packet builders, the talk helpers and the 26/3 body offset live in the
//! shared `tests/common/npc_greetings.rs` harness. This suite keeps its own
//! locale row index: T299's `dat_line` pins are the CRLF (`split("\r\n")`)
//! physical lines, which differ from the harness `str::lines()` index after
//! the locale's 38 lone-LF records (see the note's T356 section).

#[path = "common/npc_greetings.rs"]
mod npc_greetings;

use npc_greetings::{
    GREETING_HEADER, MENU_HEADER, character_at, content_dir, gameplay_dir, gameplay_talk_body,
    interact, locale_path, npc_position, npc_record, read_json, select, service, service_talk_body,
};
use osfr_tools::{
    gameplay::Gameplay,
    npc_interaction::{NpcAction, NpcInteractionError, NpcSelection},
    world::NpcVisibility,
    world_text::locale_hash,
};
use serde_json::Value;
use std::{
    collections::{BTreeSet, HashMap},
    fs,
};

/// One edited row. `hash` is `JenkinsLookup2("Global.Text.<greeting>")`, the
/// key the stock client locale stores on `dat_line`; `guide_line` is the NPC
/// row in the region build guide.
struct Target {
    npc_id: u64,
    name: &'static str,
    file: &'static str,
    greeting: u32,
    hash: u32,
    dat_line: u32,
    guide_line: u32,
}

const TARGETS: [Target; 20] = [
    // Snowhill: Warrior, Blacksmith, Miner and other job trainers/quest givers.
    Target {
        npc_id: 9000000076,
        name: "Drill Sergeant Dewey",
        file: "snowhill.json",
        greeting: 46059,
        hash: 365276068,
        dat_line: 9699,
        guide_line: 35,
    },
    Target {
        npc_id: 9000000155,
        name: "Morninglory",
        file: "snowhill.json",
        greeting: 70892,
        hash: 682434084,
        dat_line: 17991,
        guide_line: 36,
    },
    Target {
        npc_id: 9000000208,
        name: "Steele",
        file: "snowhill.json",
        greeting: 387432,
        hash: 1250175886,
        dat_line: 33068,
        guide_line: 34,
    },
    Target {
        npc_id: 9000000206,
        name: "Smitty",
        file: "snowhill.json",
        greeting: 78478,
        hash: 2726091184,
        dat_line: 71745,
        guide_line: 46,
    },
    Target {
        npc_id: 9000000220,
        name: "Trixi",
        file: "snowhill.json",
        greeting: 76208,
        hash: 2576815658,
        dat_line: 67797,
        guide_line: 45,
    },
    Target {
        npc_id: 9000000033,
        name: "Brody Sparfist",
        file: "snowhill.json",
        greeting: 91267,
        hash: 4155254588,
        dat_line: 109750,
        guide_line: 50,
    },
    Target {
        npc_id: 9000000234,
        name: "Yaren Sunstare",
        file: "snowhill.json",
        greeting: 74937,
        hash: 2958328408,
        dat_line: 78056,
        guide_line: 80,
    },
    Target {
        npc_id: 9000000151,
        name: "Michi",
        file: "snowhill.json",
        greeting: 78904,
        hash: 979442635,
        dat_line: 25918,
        guide_line: 93,
    },
    // Greenwood Forest: the archer camp, Southvale and the Summer Camp.
    Target {
        npc_id: 9000000387,
        name: "Lugabow",
        file: "greenwood_forest.json",
        greeting: 22074,
        hash: 2967121729,
        dat_line: 78290,
        guide_line: 60,
    },
    Target {
        npc_id: 9000000106,
        name: "Hilforn",
        file: "greenwood_forest.json",
        greeting: 383044,
        hash: 4136546835,
        dat_line: 109263,
        guide_line: 66,
    },
    Target {
        npc_id: 9000000405,
        name: "Mili Ironcall",
        file: "greenwood_forest.json",
        greeting: 94431,
        hash: 3079919145,
        dat_line: 81310,
        guide_line: 112,
    },
    Target {
        npc_id: 9000000479,
        name: "Supply Master Leo",
        file: "greenwood_forest.json",
        greeting: 94417,
        hash: 2088544163,
        dat_line: 55027,
        guide_line: 113,
    },
    Target {
        npc_id: 9000000337,
        name: "Grumpore",
        file: "greenwood_forest.json",
        greeting: 77548,
        hash: 2366800564,
        dat_line: 62180,
        guide_line: 114,
    },
    Target {
        npc_id: 9000000005,
        name: "Ambrose Worcester",
        file: "greenwood_forest.json",
        greeting: 413790,
        hash: 1713126354,
        dat_line: 45199,
        guide_line: 89,
    },
    Target {
        npc_id: 9000000444,
        name: "Royal Courier",
        file: "greenwood_forest.json",
        greeting: 389186,
        hash: 2813630996,
        dat_line: 74126,
        guide_line: 91,
    },
    Target {
        npc_id: 9000000334,
        name: "Glugmort",
        file: "greenwood_forest.json",
        greeting: 47871,
        hash: 379736909,
        dat_line: 10056,
        guide_line: 59,
    },
    Target {
        npc_id: 9500000291,
        name: "Burnetta",
        file: "greenwood_forest.json",
        greeting: 24816,
        hash: 3672366051,
        dat_line: 97222,
        guide_line: 61,
    },
    Target {
        npc_id: 9500000297,
        name: "Counselor Edmond",
        file: "greenwood_forest.json",
        greeting: 429061,
        hash: 3466662338,
        dat_line: 91771,
        guide_line: 40,
    },
    Target {
        npc_id: 9500000301,
        name: "Counselor Kaela",
        file: "greenwood_forest.json",
        greeting: 429430,
        hash: 2531750111,
        dat_line: 66657,
        guide_line: 43,
    },
    Target {
        npc_id: 9500000298,
        name: "Counselor Jacen",
        file: "greenwood_forest.json",
        greeting: 429370,
        hash: 1148190599,
        dat_line: 30400,
        guide_line: 46,
    },
];

/// The W126/T246 bindings that this slice must not touch.
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

/// A legacy-data Snowhill NPC with no recovered greeting.
const BLANK_SNOWHILL: u64 = 9500000730;
/// A legacy-data Greenwood Forest NPC with no recovered greeting.
const BLANK_GREENWOOD: u64 = 9500000324;

fn guide_name(file: &str) -> &'static str {
    match file {
        "snowhill.json" => "NPCs - Snowhill.md",
        _ => "NPCs - Greenwood Forest.md",
    }
}

/// The stock client locale rows for the 20 target hashes, keyed by hash with
/// the one-based physical line and the client text. T299 pinned `dat_line` with
/// the CRLF (`split("\r\n")`) index; after the locale's lone-LF records every
/// line differs from the harness `str::lines()` index, so this suite keeps its
/// own lookup rather than `npc_greetings::locale_rows` (T356 note).
fn locale_rows() -> HashMap<u32, (u32, String)> {
    let path = locale_path();
    let text =
        fs::read_to_string(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    let mut rows = HashMap::new();
    for (index, line) in text.split("\r\n").enumerate() {
        let mut fields = line.splitn(3, '\t');
        let (Some(key), Some(_table), Some(body)) = (fields.next(), fields.next(), fields.next())
        else {
            continue;
        };
        let Ok(hash) = key.trim_start_matches('\u{feff}').parse::<u32>() else {
            continue;
        };
        if TARGETS.iter().any(|target| target.hash == hash) {
            rows.entry(hash)
                .or_insert((index as u32 + 1, body.to_owned()));
        }
    }
    rows
}

#[test]
fn snowhill_and_greenwood_rows_send_their_greeting_in_26_3() {
    // Service path: the context the router copies into the 26/3 body.
    let mut service = service();
    let mut greetings = BTreeSet::new();
    for target in &TARGETS {
        assert!(
            greetings.insert(target.greeting),
            "NPC {}: greeting {} is reused",
            target.npc_id,
            target.greeting
        );
        assert_eq!(
            service_talk_body(&mut service, target.npc_id),
            Some(target.greeting),
            "NPC {} ({}) service Talk body",
            target.npc_id,
            target.name
        );
    }
    // A body that resolved to a neighbouring NPC's line would still be a valid
    // locale id; pinning every row's own id and requiring 20 distinct ids
    // rejects that substitution.
    assert_eq!(greetings.len(), TARGETS.len());

    // Dispatch path: gateway -> Gameplay -> gameplay/npc.rs -> dialogue_packet_with.
    let mut game = Gameplay::load(&gameplay_dir(), None).expect("gameplay content loads");
    for target in &TARGETS {
        assert_eq!(
            gameplay_talk_body(&mut game, target.file, target.npc_id),
            target.greeting,
            "NPC {} ({}) 26/3 body",
            target.npc_id,
            target.name
        );
    }
}

#[test]
fn a_zero_or_missing_greeting_falls_back_to_no_body() {
    let mut service = service();
    for (file, npc_id) in [
        ("snowhill.json", BLANK_SNOWHILL),
        ("greenwood_forest.json", BLANK_GREENWOOD),
    ] {
        assert_eq!(
            npc_record(file, npc_id)["greeting_string_id"].as_u64(),
            None,
            "NPC {npc_id}: the blank control must stay blank"
        );
        assert_eq!(
            service_talk_body(&mut service, npc_id),
            None,
            "NPC {npc_id}: an NPC without a greeting has no body"
        );
    }

    // A refused choice must leave the menu usable (F271): select Talk directly
    // on the still-open menu, with no second `interact`, so this fails if
    // `select` consumes `active` on `InvalidOption`.
    let (zone, position) = service.location(BLANK_SNOWHILL).unwrap();
    let menu = service
        .interact(BLANK_SNOWHILL, zone, position, None)
        .unwrap();
    let talk = menu
        .options
        .iter()
        .find(|option| option.action == NpcAction::Talk)
        .expect("the blank NPC offers Talk")
        .id;
    assert!(matches!(
        service.select(
            &select(BLANK_SNOWHILL, i32::MAX as u32),
            zone,
            position,
            None
        ),
        Err(NpcInteractionError::InvalidOption)
    ));
    match service.select(&select(BLANK_SNOWHILL, talk as u32), zone, position, None) {
        Ok(NpcSelection::Talk(context)) => assert_eq!(
            context.body_string_id, None,
            "the refused choice must not consume the menu"
        ),
        _ => panic!("the refused choice must leave the menu usable"),
    }

    // The real router still encodes 26/3 with a zero body and no invented id.
    let mut game = Gameplay::load(&gameplay_dir(), None).expect("gameplay content loads");
    for (file, npc_id) in [
        ("snowhill.json", BLANK_SNOWHILL),
        ("greenwood_forest.json", BLANK_GREENWOOD),
    ] {
        assert_eq!(
            gameplay_talk_body(&mut game, file, npc_id),
            0,
            "NPC {npc_id}: no invented locale id"
        );
    }

    // Router refusal: an out-of-list 26/10 on Smitty's open dialogue is
    // rejected, and Talk straight afterwards still returns his recovered body.
    let smitty = npc_record("snowhill.json", 9000000206);
    let character = character_at(npc_position(&smitty, 9000000206));
    let menu = game
        .handle_in_world(&interact(9000000206), &character, 2_000)
        .expect("Smitty menu");
    assert!(
        menu.replies
            .iter()
            .any(|packet| packet.starts_with(&MENU_HEADER))
    );
    assert!(
        game.handle_in_world(&select(9000000206, i32::MAX as u32), &character, 2_100)
            .is_err(),
        "an out-of-list choice is refused"
    );
    let after = game
        .handle_in_world(&select(9000000206, 1), &character, 2_200)
        .expect("Talk after the refusal");
    let dialogue = after
        .replies
        .into_iter()
        .find(|packet| packet.starts_with(&GREETING_HEADER))
        .expect("the refused choice must not consume the menu");
    assert_eq!(
        u32::from_le_bytes(dialogue[4..8].try_into().unwrap()),
        78478,
        "the refusal changed nothing"
    );
}

#[test]
fn the_target_rows_are_spawned_and_non_target_rows_are_unchanged() {
    // The production visibility loader spawns both legacy_data rows and the
    // T201/T257 extra-spawn actors, and maps each runtime GUID to this row.
    let visibility =
        NpcVisibility::load(&content_dir()).expect("the production visibility loader accepts it");
    let locale = locale_rows();
    let existing: BTreeSet<u32> = EXISTING_GREETINGS.iter().map(|(_, _, id)| *id).collect();
    for target in &TARGETS {
        let record = npc_record(target.file, target.npc_id);
        assert_eq!(
            record["name"].as_str(),
            Some(target.name),
            "NPC {}: npc_id resolves the expected row",
            target.npc_id
        );
        assert_eq!(
            visibility.content_key(target.npc_id),
            Some(target.npc_id),
            "NPC {} ({}) is not spawned by NpcVisibility",
            target.npc_id,
            target.name
        );
        assert_eq!(
            record["greeting_string_id"].as_u64(),
            Some(u64::from(target.greeting)),
            "NPC {}: greeting binding",
            target.npc_id
        );
        assert!(
            !existing.contains(&target.greeting),
            "NPC {}: greeting {} duplicates an existing binding",
            target.npc_id,
            target.greeting
        );

        let sources = record["field_sources"]["greeting_string_id"]
            .as_array()
            .unwrap_or_else(|| panic!("NPC {}: greeting sources", target.npc_id));
        assert_eq!(
            sources.len(),
            2,
            "NPC {}: one client and one reconstructed source",
            target.npc_id
        );
        let client = sources
            .iter()
            .find(|source| source["kind"] == "client")
            .unwrap_or_else(|| panic!("NPC {}: client source", target.npc_id));
        let reconstructed = sources
            .iter()
            .find(|source| source["kind"] == "reconstructed")
            .unwrap_or_else(|| panic!("NPC {}: reconstructed source", target.npc_id));
        assert!(
            reconstructed["evidence"]
                .as_str()
                .is_some_and(|evidence| evidence.starts_with("(R)")),
            "NPC {}: the reconstruction must be labelled (R)",
            target.npc_id
        );
        let evidence = client["evidence"].as_str().unwrap_or_default();
        for expected in [
            format!("Global.Text.{}", target.greeting),
            format!("JenkinsLookup2 hash {}", target.hash),
            format!("line {}", target.dat_line),
            format!("{}:{}", guide_name(target.file), target.guide_line),
        ] {
            assert!(
                evidence.contains(&expected),
                "NPC {}: source must name {expected:?}",
                target.npc_id
            );
        }

        // Resolve the id the way the client does: hash Global.Text.<id>, then
        // find that hash on the cited physical locale line.
        assert_eq!(
            locale_hash(i32::try_from(target.greeting).expect("stock locale id fits i32")),
            target.hash,
            "NPC {}: greeting id must be a Global.Text key, not a raw hash",
            target.npc_id
        );
        let (line, text) = locale
            .get(&target.hash)
            .unwrap_or_else(|| panic!("NPC {}: locale hash missing", target.npc_id));
        assert_eq!(
            *line, target.dat_line,
            "NPC {}: en_us_data.dat line",
            target.npc_id
        );
        assert!(
            !text.trim().is_empty(),
            "NPC {}: empty locale text",
            target.npc_id
        );
    }

    // Every non-target row in the two edited files keeps its exact current
    // greeting value (absent, or one of the pinned pre-existing bindings).
    for file in ["snowhill.json", "greenwood_forest.json"] {
        let document = read_json(&content_dir().join("npcs").join(file));
        for record in document["records"].as_array().unwrap() {
            let npc_id = record["npc_id"].as_u64().unwrap();
            if TARGETS
                .iter()
                .any(|target| target.file == file && target.npc_id == npc_id)
            {
                continue;
            }
            let expected = EXISTING_GREETINGS
                .iter()
                .find(|(known_file, known_id, _)| *known_file == file && *known_id == npc_id)
                .map(|(_, _, greeting)| u64::from(*greeting));
            assert_eq!(
                record["greeting_string_id"].as_u64(),
                expected,
                "{file} NPC {npc_id}: a non-target greeting changed"
            );
        }
    }
}

#[test]
fn the_existing_greetings_and_other_regions_are_unchanged() {
    for (file, npc_id, greeting) in EXISTING_GREETINGS {
        assert_eq!(
            npc_record(file, npc_id)["greeting_string_id"].as_u64(),
            Some(u64::from(greeting)),
            "existing binding {npc_id} in {file} changed"
        );
    }
    // The T201/T257 extra spawns the three Snowhill greetings now travel with
    // keep the same runtime GUID -> content key binding.
    let extras = read_json(&content_dir().join("npc_spawns_extra.json"));
    for npc_id in [9000000076u64, 9000000155, 9000000208] {
        let spawn = extras["records"]
            .as_array()
            .unwrap()
            .iter()
            .find(|spawn| spawn["spawn_guid"].as_u64() == Some(npc_id))
            .unwrap_or_else(|| panic!("extra spawn {npc_id}"));
        assert_eq!(
            spawn["content_key"].as_u64(),
            Some(npc_id),
            "extra spawn {npc_id}: content key"
        );
        assert!(
            spawn["model_id"].as_u64().is_some_and(|model| model > 0),
            "extra spawn {npc_id}: drawable model"
        );
        assert!(
            spawn["name_string_id"]
                .as_u64()
                .is_some_and(|name| name > 0),
            "extra spawn {npc_id}: drawable name"
        );
    }
}

#[test]
fn the_content_stays_within_the_loader_bounds() {
    let data = content_dir().join("npcs");
    let mut total = 0u64;
    for file in ["snowhill.json", "greenwood_forest.json"] {
        let path = data.join(file);
        let bytes = fs::read(&path).unwrap();
        let size = bytes.len() as u64;
        assert!(
            size <= 8 * 1024 * 1024,
            "{file}: exceeds the 8 MiB per-file loader bound"
        );
        total += size;
        let text = std::str::from_utf8(&bytes).unwrap_or_else(|error| panic!("{file}: {error}"));
        let document: Value =
            serde_json::from_str(text).unwrap_or_else(|error| panic!("{file}: {error}"));
        let rows = document["records"].as_array().unwrap();
        assert!(
            rows.len() <= 20_000,
            "{file}: exceeds the 20,000-row loader bound"
        );
    }
    assert!(
        total <= 32 * 1024 * 1024,
        "the two edited files exceed the 32 MiB loader bound"
    );
    // The loader itself is the row-count/shape gate.
    let _ = service();

    for target in &TARGETS {
        let record = npc_record(target.file, target.npc_id);
        assert!(
            record["greeting_string_id"]
                .as_u64()
                .is_some_and(|greeting| greeting > 0 && greeting <= u64::from(u32::MAX)),
            "NPC {}: greeting id",
            target.npc_id
        );
        for source in record["field_sources"]["greeting_string_id"]
            .as_array()
            .unwrap()
        {
            let evidence = source["evidence"].as_str().unwrap_or_default();
            assert!(
                !evidence.trim().is_empty(),
                "NPC {}: empty source evidence",
                target.npc_id
            );
            assert!(
                !evidence.chars().any(char::is_control),
                "NPC {}: control character in source",
                target.npc_id
            );
            assert!(
                evidence.len() <= 4096,
                "NPC {}: source evidence too long",
                target.npc_id
            );
        }
    }
}
