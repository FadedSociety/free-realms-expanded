//! T280: the remaining Sanctuary NPCs carry recovered locale greetings.
//!
//! Content only: `NpcInteraction::dialogue_context` already copies `greeting_string_id` into
//! `DialogueContext.body_string_id`, and `gameplay/npc.rs::dialogue_packet_with` writes that body
//! into 26/3, so this slice adds no packet, handler, table or flag. Client evidence re-checked in
//! IDA for T280: 26/3 handler `00aa12e0` calls payload reader `00a9ef10`, which stores the first
//! payload dword in the message object at `+16`; display routine `009f5ae0` passes `*(a2 + 16)` to
//! the `Global.Text.<id>` lookup immediately before `:SetDialogText`. 26/8 reader `008b6bc0`,
//! 26/10 writer `00a36f80`, 26/6 `009f5890`, 26/4 close. The 26/3 byte layout is pinned by
//! `tests/npc_interaction.rs::dialogue_26_3_synthetic_fixture_round_trips_exact_native_field_order`.
//! Locale ids, hashes, `en_us_data.dat` lines and speaker proofs are in `notes/Autopilot/T280.md`.
//!
//! The service path (`NpcInteraction::interact` -> `select`) is the same one `gameplay/npc.rs`
//! uses; the dispatch path (`Gameplay::handle_in_world`) drives the real router and
//! `dialogue_packet_with`. The packet builders, the locale row convention and the 26/3 body
//! offset live in the shared `tests/common/npc_greetings.rs` harness.

#[path = "common/npc_greetings.rs"]
mod npc_greetings;

use npc_greetings::{
    GREETING_HEADER, character_at, content_dir, gameplay_dir, gameplay_talk_body, interact,
    lab_report, locale_id_to_text, locale_rows, npc_position, npc_record, npc_records, read_json,
    select, service, service_talk_body,
};
use osfr_tools::{
    gameplay::Gameplay,
    npc_interaction::{NpcAction, NpcInteractionError, NpcSelection},
    world::NpcVisibility,
};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

const SANCTUARY_GUIDE: &str = "notes/Build Guides/Content/NPCs/NPCs - Sanctuary.md";
const FARNUM_GUIDE: &str = "notes/Build Guides/Content/NPCs/NPCs - Farnum Farmstead.md";
const CAVES_GUIDE: &str = "notes/Build Guides/Content/Dungeons/Forgotten Caves.md";
const CHEF_GUIDE: &str = "notes/Build Guides/Content/Jobs/Chef.md";
const WARRIOR_GUIDE: &str = "notes/Build Guides/Content/Jobs/Warrior.md";
const SMITH_GUIDE: &str = "notes/Build Guides/Content/Jobs/Blacksmith.md";
const DUELIST_GUIDE: &str = "notes/Build Guides/Content/Jobs/Card Duelist.md";

/// One edited row: NPC content key, bound `Global.Text` id, its JenkinsLookup2 key hash, the
/// physical stock-locale line, and the guide row that proves the speaker.
struct Greeting {
    npc_id: u64,
    greeting: u32,
    hash: u32,
    dat_line: u32,
    guide: &'static str,
    guide_line: u32,
}

const NEW_GREETINGS: [Greeting; 20] = [
    Greeting {
        npc_id: 9500000465,
        greeting: 74526,
        hash: 1379065920,
        dat_line: 36572,
        guide: FARNUM_GUIDE,
        guide_line: 156,
    },
    Greeting {
        npc_id: 9500000528,
        greeting: 21534,
        hash: 3995278468,
        dat_line: 105649,
        guide: CAVES_GUIDE,
        guide_line: 95,
    },
    Greeting {
        npc_id: 9500000520,
        greeting: 428076,
        hash: 3087743397,
        dat_line: 81536,
        guide: SANCTUARY_GUIDE,
        guide_line: 34,
    },
    Greeting {
        npc_id: 9500000476,
        greeting: 390786,
        hash: 3255903404,
        dat_line: 85999,
        guide: SANCTUARY_GUIDE,
        guide_line: 75,
    },
    Greeting {
        npc_id: 9000000467,
        greeting: 94274,
        hash: 2496072465,
        dat_line: 65655,
        guide: CHEF_GUIDE,
        guide_line: 78,
    },
    Greeting {
        npc_id: 9500000515,
        greeting: 415266,
        hash: 1547845466,
        dat_line: 40984,
        guide: SANCTUARY_GUIDE,
        guide_line: 93,
    },
    Greeting {
        npc_id: 9500000493,
        greeting: 415302,
        hash: 2703417406,
        dat_line: 71158,
        guide: SANCTUARY_GUIDE,
        guide_line: 134,
    },
    Greeting {
        npc_id: 9500000505,
        greeting: 70827,
        hash: 3481579165,
        dat_line: 92199,
        guide: SANCTUARY_GUIDE,
        guide_line: 113,
    },
    Greeting {
        npc_id: 9500000509,
        greeting: 46279,
        hash: 1390791907,
        dat_line: 36924,
        guide: SANCTUARY_GUIDE,
        guide_line: 168,
    },
    Greeting {
        npc_id: 9500000522,
        greeting: 37041,
        hash: 81168942,
        dat_line: 2089,
        guide: SANCTUARY_GUIDE,
        guide_line: 182,
    },
    Greeting {
        npc_id: 9500000474,
        greeting: 21785,
        hash: 3806996357,
        dat_line: 100688,
        guide: SANCTUARY_GUIDE,
        guide_line: 110,
    },
    Greeting {
        npc_id: 9500000477,
        greeting: 21557,
        hash: 570833766,
        dat_line: 15144,
        guide: SANCTUARY_GUIDE,
        guide_line: 107,
    },
    Greeting {
        npc_id: 9500000529,
        greeting: 74933,
        hash: 1469153882,
        dat_line: 38886,
        guide: SANCTUARY_GUIDE,
        guide_line: 95,
    },
    Greeting {
        npc_id: 9500000545,
        greeting: 76061,
        hash: 2852799536,
        dat_line: 75206,
        guide: SANCTUARY_GUIDE,
        guide_line: 108,
    },
    Greeting {
        npc_id: 9000000097,
        greeting: 91895,
        hash: 2241336804,
        dat_line: 58993,
        guide: WARRIOR_GUIDE,
        guide_line: 105,
    },
    Greeting {
        npc_id: 9000000160,
        greeting: 101215,
        hash: 2980092336,
        dat_line: 78667,
        guide: SMITH_GUIDE,
        guide_line: 134,
    },
    Greeting {
        npc_id: 9500000497,
        greeting: 15500,
        hash: 752124044,
        dat_line: 19804,
        guide: SANCTUARY_GUIDE,
        guide_line: 51,
    },
    Greeting {
        npc_id: 9500000500,
        greeting: 103560,
        hash: 635925311,
        dat_line: 16842,
        guide: DUELIST_GUIDE,
        guide_line: 67,
    },
    Greeting {
        npc_id: 9500000438,
        greeting: 37452,
        hash: 501203270,
        dat_line: 13326,
        guide: SANCTUARY_GUIDE,
        guide_line: 167,
    },
    Greeting {
        npc_id: 9500000440,
        greeting: 22622,
        hash: 113727630,
        dat_line: 2952,
        guide: SANCTUARY_GUIDE,
        guide_line: 109,
    },
];

/// The bindings already in `sanctuary.json` before T280 (T246 and W126). Every other row must
/// still carry no greeting.
const PRE_EXISTING: [(u64, u32); 11] = [
    (9000000028, 387246),
    (9000000052, 73782),
    (9000000073, 76747),
    (9000000161, 73360),
    (9000000205, 71435),
    (9000000126, 76438),
    (9500000450, 441420),
    (9000000091, 24842),
    (9500000487, 37872),
    (9000000207, 46166),
    (9500000541, 21602),
];

/// T378 bindings recovered after T280. They are expected in `sanctuary.json`,
/// not a T280 regression. T546 bound Kellin Warpwatcher's Snowhill Warp Tests
/// offer line (`tests/npc_greetings_warpwatchers.rs`).
const LATER_GREETINGS: [(u64, u32); 4] = [
    (9500000435, 5102533), // T894 Abacus Coinmiser
    (9000000330, 90621),
    (9500000488, 415317),
    (9500000502, 385689),
];

/// A legacy-data Sanctuary trainer, merchant and non-quest greeter in the edited set.
const TRAINER: u64 = 9000000097; // General Glowlight
const MERCHANT: u64 = 9500000515; // Pucca De'Flor
const GREETER: u64 = 9500000438; // Anura, ambient
/// An untouched legacy-data Sanctuary row with no recovered greeting.
const BLANK: u64 = 9500000443; // Barin Battlerock

fn roles(record: &Value) -> BTreeSet<&str> {
    record["roles"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .collect()
}

#[test]
fn the_new_rows_have_nonzero_greetings_and_real_sources() {
    let records = npc_records("sanctuary.json");
    let by_id: BTreeMap<u64, &Value> = records
        .iter()
        .map(|record| (record["npc_id"].as_u64().expect("npc id"), record))
        .collect();
    let wanted: BTreeSet<u32> = NEW_GREETINGS.iter().map(|row| row.hash).collect();
    let locale = locale_rows(&wanted);
    let hash_to_ids = read_json(&lab_report("locale-id-map.json"))["hash_to_ids"].clone();
    let id_to_text = locale_id_to_text();
    let mut greeting_ids = BTreeSet::new();
    for row in &NEW_GREETINGS {
        assert!(
            row.greeting > 0,
            "NPC {}: greeting must be positive",
            row.npc_id
        );
        assert!(
            greeting_ids.insert(row.greeting),
            "NPC {}: greeting {} reused by another speaker",
            row.npc_id,
            row.greeting
        );
        let record = by_id
            .get(&row.npc_id)
            .unwrap_or_else(|| panic!("NPC {} is in sanctuary.json", row.npc_id));
        assert_eq!(
            record["greeting_string_id"].as_u64(),
            Some(u64::from(row.greeting)),
            "NPC {}: greeting binding",
            row.npc_id
        );
        let sources = record["field_sources"]["greeting_string_id"]
            .as_array()
            .unwrap_or_else(|| panic!("NPC {}: greeting sources", row.npc_id));
        let client = sources
            .iter()
            .find(|source| source["kind"] == "client")
            .unwrap_or_else(|| panic!("NPC {}: client greeting source", row.npc_id));
        let evidence = client["evidence"].as_str().unwrap_or_default();
        for needle in [
            format!("Global.Text.{}", row.greeting),
            format!("hash {}", row.hash),
            format!("line {}", row.dat_line),
            row.guide.to_string(),
            format!("line {}", row.guide_line),
            "T280".to_string(),
        ] {
            assert!(
                evidence.contains(&needle),
                "NPC {}: client source must name {needle:?}",
                row.npc_id
            );
        }
        assert!(
            sources.iter().any(|source| {
                source["kind"] == "reconstructed"
                    && source["evidence"]
                        .as_str()
                        .is_some_and(|text| text.contains("fixed default Talk greeting"))
            }),
            "NPC {}: fixed-default policy source",
            row.npc_id
        );

        // Resolve three ways: hash_to_ids, the id-to-text report, and the physical stock line.
        let bound = hash_to_ids
            .get(&row.hash.to_string())
            .and_then(Value::as_array)
            .unwrap_or_else(|| {
                panic!(
                    "NPC {}: hash {} is not in locale-id-map.json hash_to_ids",
                    row.npc_id, row.hash
                )
            });
        assert!(
            bound
                .iter()
                .any(|id| id.as_u64() == Some(u64::from(row.greeting))),
            "NPC {}: hash {} must resolve to {}",
            row.npc_id,
            row.hash,
            row.greeting
        );
        let reported = id_to_text.get(&u64::from(row.greeting)).unwrap_or_else(|| {
            panic!(
                "NPC {}: id {} is not in locale-id-to-text.json",
                row.npc_id, row.greeting
            )
        });
        let (line, text) = locale.get(&row.hash).unwrap_or_else(|| {
            panic!(
                "NPC {}: hash {} is not a key in the stock client locale",
                row.npc_id, row.hash
            )
        });
        assert_eq!(
            *line, row.dat_line as usize,
            "NPC {}: en_us_data.dat line",
            row.npc_id
        );
        assert_eq!(
            text, reported,
            "NPC {}: stock locale text must equal the id-to-text report",
            row.npc_id
        );
        assert!(
            !text.trim().is_empty(),
            "NPC {}: recovered locale text is empty",
            row.npc_id
        );
    }
}

#[test]
fn a_bound_npc_sends_its_greeting_in_26_3() {
    // Service path: the context the router copies into the 26/3 body.
    let mut service = service();
    for row in &NEW_GREETINGS {
        assert_eq!(
            service_talk_body(&mut service, row.npc_id),
            Some(row.greeting),
            "NPC {}: real Talk body",
            row.npc_id
        );
    }

    // Dispatch path: gateway -> Gameplay -> gameplay/npc.rs -> dialogue_packet_with.
    let mut game = Gameplay::load(&gameplay_dir(), None).expect("gameplay content loads");
    for row in &NEW_GREETINGS {
        assert_eq!(
            gameplay_talk_body(&mut game, "sanctuary.json", row.npc_id),
            row.greeting,
            "NPC {}: 26/3 body dword",
            row.npc_id
        );
    }

    // The edited set covers a trainer, a merchant and a non-quest greeter.
    let trainer = npc_record("sanctuary.json", TRAINER);
    assert!(
        roles(&trainer).contains("trainer"),
        "the edited set must include a trainer"
    );
    let merchant = npc_record("sanctuary.json", MERCHANT);
    assert!(
        roles(&merchant).contains("vendor"),
        "the edited set must include a merchant"
    );
    // T774: the vendor role alone no longer opens a window. Every merchant in this
    // edited set has no priced `vendors.json` stock, so none may list Shop; the client
    // must never open an empty merchant window.
    let (zone, position) = service.location(MERCHANT).expect("merchant location");
    let menu = service
        .interact(MERCHANT, zone, position, None)
        .expect("merchant menu");
    assert!(
        menu.options
            .iter()
            .all(|option| option.action != NpcAction::Shop),
        "a merchant without priced stock must not list Shop"
    );
    let greeter = npc_record("sanctuary.json", GREETER);
    let greeter_roles = roles(&greeter);
    assert!(
        greeter_roles.contains("ambient") && !greeter_roles.contains("quest_giver"),
        "the edited set must include a non-quest greeter"
    );
}

#[test]
fn a_refused_choice_leaves_the_menu_usable() {
    let mut service = service();
    let npc_id = 9500000520; // Quills
    let (zone, position) = service.location(npc_id).expect("Quills location");
    let menu = service
        .interact(npc_id, zone, position, None)
        .expect("Quills menu");
    let talk = menu
        .options
        .iter()
        .find(|option| option.action == NpcAction::Talk)
        .expect("Quills offers Talk")
        .id;
    let greeting = NEW_GREETINGS
        .iter()
        .find(|row| row.npc_id == npc_id)
        .expect("Quills is edited")
        .greeting;

    // Invalid choice id: refused, and the still-open menu must select Talk directly afterwards
    // with no second `interact` (the T246 review blocker).
    assert!(matches!(
        service.select(&select(npc_id, i32::MAX as u32), zone, position, None),
        Err(NpcInteractionError::InvalidOption)
    ));
    match service.select(&select(npc_id, talk as u32), zone, position, None) {
        Ok(NpcSelection::Talk(context)) => assert_eq!(
            context.body_string_id,
            Some(greeting),
            "the refused choice must not consume the menu"
        ),
        _ => panic!("the refused choice must leave the menu usable"),
    }
    // A malformed 26/10 body and an unknown target are refused too; the menu stays usable.
    assert!(matches!(
        service.select(&[26, 0, 10, 0, 1, 2, 3], zone, position, None),
        Err(NpcInteractionError::InvalidPacket)
    ));
    assert!(matches!(
        service.select(&select(9_999_999_999, talk as u32), zone, position, None),
        Err(NpcInteractionError::UnknownNpc)
    ));
    match service.select(&select(npc_id, talk as u32), zone, position, None) {
        Ok(NpcSelection::Talk(context)) => assert_eq!(context.body_string_id, Some(greeting)),
        _ => panic!("refusals must change nothing"),
    }

    // The real dispatch path behaves the same: an invalid choice is an error, not a menu reset.
    let mut game = Gameplay::load(&gameplay_dir(), None).expect("gameplay content loads");
    let character = character_at(npc_position(&npc_record("sanctuary.json", npc_id), npc_id));
    game.handle_in_world(&interact(npc_id), &character, 2_000)
        .expect("in-world interact");
    assert!(
        game.handle_in_world(&select(npc_id, i32::MAX as u32), &character, 2_100)
            .is_ok_and(|out| out.replies.is_empty() && !out.changed),
        "an invalid 26/10 choice must be refused"
    );
    let outcome = game
        .handle_in_world(&select(npc_id, 1), &character, 2_200)
        .expect("the menu must survive the refusal");
    assert_eq!(
        u32::from_le_bytes(
            outcome
                .replies
                .iter()
                .find(|packet| packet.starts_with(&GREETING_HEADER))
                .expect("Talk answers 26/3")[4..8]
                .try_into()
                .unwrap()
        ),
        greeting
    );
}

#[test]
fn a_missing_or_zero_greeting_still_falls_back() {
    let mut service = service();
    let blank = npc_record("sanctuary.json", BLANK);
    assert!(
        blank.get("greeting_string_id").is_none(),
        "the fallback control row must stay unbound"
    );
    assert_eq!(
        service_talk_body(&mut service, BLANK),
        None,
        "an NPC without a greeting has no body"
    );

    // A malformed select target is refused and leaves the open menu usable.
    let (zone, position) = service.location(BLANK).expect("blank location");
    let menu = service
        .interact(BLANK, zone, position, None)
        .expect("blank menu");
    let talk = menu
        .options
        .iter()
        .find(|option| option.action == NpcAction::Talk)
        .expect("the blank NPC offers Talk")
        .id;
    assert!(matches!(
        service.select(&select(BLANK, i32::MAX as u32), zone, position, None),
        Err(NpcInteractionError::InvalidOption)
    ));
    match service.select(&select(BLANK, talk as u32), zone, position, None) {
        Ok(NpcSelection::Talk(context)) => assert_eq!(
            context.body_string_id, None,
            "a zero/missing greeting is absent, not a fabricated id"
        ),
        _ => panic!("the blank menu stays usable"),
    }

    // The real router still encodes 26/3 with a zero body, and no bound body is a name id.
    let records: BTreeMap<u64, Value> = npc_records("sanctuary.json")
        .into_iter()
        .map(|record| (record["npc_id"].as_u64().expect("npc id"), record))
        .collect();
    let mut game = Gameplay::load(&gameplay_dir(), None).expect("gameplay content loads");
    assert_eq!(
        gameplay_talk_body(&mut game, "sanctuary.json", BLANK),
        0,
        "no invented locale id"
    );
    for row in &NEW_GREETINGS {
        let name_id = records[&row.npc_id]["name_string_id"]
            .as_u64()
            .expect("name id");
        assert_ne!(u64::from(row.greeting), name_id, "name id is not a body id");
        assert_ne!(row.greeting, 0);
    }
}

#[test]
fn every_edited_row_is_drawable_and_no_other_row_changed() {
    let content = content_dir();
    let visibility = NpcVisibility::load(&content).expect("NPC visibility loads");
    let spawned: BTreeMap<u64, [f32; 3]> = visibility
        .teleport_targets(0)
        .into_iter()
        .map(|target| (target.guid, target.position))
        .collect();
    let records = npc_records("sanctuary.json");
    let by_id: BTreeMap<u64, &Value> = records
        .iter()
        .map(|record| (record["npc_id"].as_u64().expect("npc id"), record))
        .collect();
    for row in &NEW_GREETINGS {
        let record = by_id[&row.npc_id];
        assert_eq!(
            record["placement"].as_str(),
            Some("legacy_data"),
            "NPC {}: only spawned legacy_data rows are bindable",
            row.npc_id
        );
        assert!(
            record["model_id"].as_u64().is_some_and(|id| id > 0),
            "NPC {}: positive model id",
            row.npc_id
        );
        assert!(
            record["name_string_id"].as_u64().is_some_and(|id| id > 0),
            "NPC {}: positive name string id",
            row.npc_id
        );
        let position = npc_position(record, row.npc_id);
        assert!(
            position.iter().all(|value| value.is_finite()),
            "NPC {}: finite position",
            row.npc_id
        );
        assert_eq!(
            spawned.get(&row.npc_id),
            Some(&position),
            "NPC {}: must be in the NpcVisibility spawn list",
            row.npc_id
        );
    }

    // Only the target rows gained a greeting: every other row keeps its pre-T280 state.
    let edited: BTreeSet<u64> = NEW_GREETINGS.iter().map(|row| row.npc_id).collect();
    let existing: BTreeMap<u64, u32> = PRE_EXISTING.into_iter().collect();
    for record in &records {
        let npc_id = record["npc_id"].as_u64().expect("npc id");
        if edited.contains(&npc_id) {
            continue;
        }
        if let Some((_, greeting)) = LATER_GREETINGS.iter().find(|(known, _)| *known == npc_id) {
            assert_eq!(
                record["greeting_string_id"].as_u64(),
                Some(u64::from(*greeting)),
                "NPC {npc_id}: later recovered binding changed"
            );
            continue;
        }
        match existing.get(&npc_id) {
            Some(greeting) => assert_eq!(
                record["greeting_string_id"].as_u64(),
                Some(u64::from(*greeting)),
                "NPC {npc_id}: pre-existing binding changed"
            ),
            None => {
                assert!(
                    record.get("greeting_string_id").is_none(),
                    "NPC {npc_id}: gained a greeting outside this slice"
                );
                assert!(
                    record["field_sources"].get("greeting_string_id").is_none(),
                    "NPC {npc_id}: gained greeting sources outside this slice"
                );
            }
        }
        assert!(
            !record.to_string().contains("T280"),
            "NPC {npc_id}: T280 evidence outside the target rows"
        );
    }

    // The three Cobblestone vendor rows stay blank: no recovered first-person line.
    for npc_id in [9500000246, 9500000247, 9500000255] {
        let record = npc_record("cobblestone_village.json", npc_id);
        assert_eq!(record["placement"].as_str(), Some("legacy_data"));
        assert!(
            record.get("greeting_string_id").is_none(),
            "NPC {npc_id}: no recovered speaker line, so no greeting"
        );
    }
}
