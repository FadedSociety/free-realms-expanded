//! T315: the remaining Highroad Junction and Farnum Farmstead NPC rows carry
//! recovered locale greetings.
//!
//! Content only: `NpcInteraction::dialogue_context` already copies
//! `greeting_string_id` into `DialogueContext.body_string_id`, and
//! `gameplay/npc.rs::dialogue_packet_with` writes that body into 26/3, so this
//! slice adds no packet, handler, table or flag. Client evidence re-checked in
//! IDA for T315: 26/3 handler `00aa12e0` calls payload reader `00a9ef10`, which
//! stores the first payload dword in the message object at `+16`; display
//! routine `009f5ae0` passes `*(a2 + 16)` to the `Global.Text.<id>` lookup
//! immediately before `:SetDialogText`. The 26/3 byte layout is pinned by
//! `tests/npc_interaction.rs::dialogue_26_3_synthetic_fixture_round_trips_exact_native_field_order`.
//! Locale ids, hashes, `en_us_data.dat` lines and speaker proofs are in
//! `notes/Autopilot/T315.md`.
//!
//! The service path (`NpcInteraction::interact` -> `select`) is the same one
//! `gameplay/npc.rs` uses; the dispatch path (`Gameplay::handle_in_world`)
//! drives the real router and `dialogue_packet_with`.

use osfr_tools::{
    character_store::CharacterState,
    gameplay::Gameplay,
    npc_interaction::{NpcAction, NpcInteraction, NpcInteractionError, NpcSelection},
    world::NpcVisibility,
    world_text::locale_hash,
};
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

const GREETING_HEADER: [u8; 4] = [26, 0, 3, 0];
const MENU_HEADER: [u8; 4] = [26, 0, 9, 0];

/// One edited row. `hash` is `JenkinsLookup2("Global.Text.<greeting>")`, the
/// key the stock client locale stores on `dat_line`; `guide` is the repository
/// notes path whose `guide_line` names the speaker. `quest` names the server
/// quest record when the line is that quest's intro/offer speech.
struct Target {
    npc_id: u64,
    name: &'static str,
    file: &'static str,
    greeting: u32,
    hash: u32,
    dat_line: u32,
    guide: &'static str,
    guide_line: u32,
    quest: Option<(&'static str, u32)>,
}

const TARGETS: [Target; 7] = [
    Target {
        npc_id: 9000000178,
        name: "Ranger Ricardo",
        file: "highroad_junction.json",
        greeting: 104137,
        hash: 1310252094,
        dat_line: 34689,
        guide: "Build Guides/Content/NPCs/NPCs - Highroad Junction.md",
        guide_line: 118,
        quest: Some(("chef.json", 3050012)),
    },
    Target {
        npc_id: 9500000270,
        name: "Hartley Harrison",
        file: "farnum_farmstead.json",
        greeting: 37921,
        hash: 1412193614,
        dat_line: 37461,
        guide: "Build Guides/Content/NPCs/NPCs - Farnum Farmstead.md",
        guide_line: 152,
        quest: None,
    },
    Target {
        npc_id: 9000000141,
        name: "Maple Sugarleaf",
        file: "farnum_farmstead.json",
        greeting: 104344,
        hash: 3677735990,
        dat_line: 97388,
        guide: "Build Guides/Content/Jobs/Card Duelist.md",
        guide_line: 113,
        quest: Some(("card_duelist.json", 3040028)),
    },
    Target {
        npc_id: 9500000268,
        name: "Bo",
        file: "farnum_farmstead.json",
        greeting: 41654,
        hash: 1903646073,
        dat_line: 50162,
        guide: "Build Guides/Content/NPCs/NPCs - Farnum Farmstead.md",
        guide_line: 151,
        quest: None,
    },
    Target {
        npc_id: 9500000273,
        name: "Papa Salt",
        file: "farnum_farmstead.json",
        greeting: 384841,
        hash: 3802931860,
        dat_line: 100591,
        guide: "Build Guides/Content/NPCs/NPCs - Farnum Farmstead.md",
        guide_line: 160,
        quest: None,
    },
    Target {
        npc_id: 9500000280,
        name: "Tamara",
        file: "farnum_farmstead.json",
        greeting: 390174,
        hash: 4119559481,
        dat_line: 108808,
        guide: "Build Guides/Content/NPCs/NPCs - Farnum Farmstead.md",
        guide_line: 165,
        quest: None,
    },
    Target {
        npc_id: 9000000225,
        name: "Uncle Irving",
        file: "farnum_farmstead.json",
        greeting: 37701,
        hash: 1511702253,
        dat_line: 40071,
        guide: "Build Guides/Content/Jobs/Adventurer.md",
        guide_line: 92,
        quest: Some(("adventurer.json", 3000018)),
    },
];

/// T246's eight starting-road bindings in these two files: five Farnum and
/// three Highroad rows. This slice must not overwrite them.
const T246_GREETINGS: [(&str, u64, u32); 8] = [
    ("farnum_farmstead.json", 9000000104, 391041),
    ("farnum_farmstead.json", 9500000271, 391391),
    ("farnum_farmstead.json", 9000000396, 5100509),
    ("farnum_farmstead.json", 9500000266, 101014),
    ("farnum_farmstead.json", 9000000050, 5100125),
    ("highroad_junction.json", 9000000015, 436533),
    ("highroad_junction.json", 9000000102, 140509),
    ("highroad_junction.json", 9000000019, 431140),
];

/// The 19 W126 bindings across the NPC content; this slice must not touch one.
const W126_GREETINGS: [(&str, u64, u32); 19] = [
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

/// Every other drawable `legacy_data` row in the two edited files. None has an
/// exact speaker proof: the client locale holds only the name for the vendors,
/// and the Trading Card Game Riley is named only in third-person objectives
/// (the first-person "Riley" line is the lazy cat pet, a different actor).
/// The note lists this set and why each row stays blank.
const UNBOUND: [(&str, u64); 7] = [
    ("highroad_junction.json", 9500000328), // Bowawow, vendor
    ("highroad_junction.json", 9500000331), // Cillian, vendor
    ("highroad_junction.json", 9500000333), // Gaston, vendor
    ("farnum_farmstead.json", 9500000269),  // Estus, vendor
    ("farnum_farmstead.json", 9500000276),  // Riley, Trading Card Game prodigy
    ("farnum_farmstead.json", 9500000277),  // Rocky, vendor
    ("farnum_farmstead.json", 9500000281),  // Vela, vendor
];

/// Every NPC file not edited by this slice: schema version, region page name
/// and region zone id. The top-level values must stay put.
const OTHER_REGIONS: [(&str, u32, &str, u64); 15] = [
    ("blackspore_swamp.json", 1, "Blackspore Swamp", 1),
    ("briarwood.json", 1, "Briarwood", 2),
    ("cobblestone_village.json", 1, "Cobblestone Village", 3),
    ("durango_zoo.json", 1, "Durango Zoo", 4),
    ("greenwood_forest.json", 1, "Greenwood Forest", 6),
    ("lakeshore.json", 1, "Lakeshore", 8),
    ("merry_vale.json", 1, "Merry Vale", 9),
    ("sanctuary.json", 1, "Sanctuary", 10),
    ("seaside.json", 1, "Seaside", 11),
    ("shrouded_glade.json", 1, "Shrouded Glade", 12),
    ("snowhill.json", 1, "Snowhill", 13),
    ("stillwater_crossing.json", 1, "Stillwater Crossing", 14),
    ("sunstone_valley.json", 1, "Sunstone Valley", 15),
    ("wilds.json", 1, "Wilds", 16),
    ("wugachug.json", 1, "Wugachug", 17),
];

const HIGHROAD_RECORDS: usize = 23;
const FARNUM_RECORDS: usize = 24;

fn content_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content")
}

/// Full server content used by the existing in-world router tests. It has no
/// `npcs/` directory, so `Gameplay::load` takes the NPC rows from the bundled
/// `data/content` (the same rows this test edits).
fn gameplay_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../lab/references/opening-anvil-placement-v1/content")
}

/// Stock 2014 client locale. `hash_to_ids` in `lab/reports/locale-id-map.json`
/// was built from it; this test re-derives each key instead of trusting it.
fn locale_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../lab/working/OSFR.Client/OSFR Client/locale/en_us_data.dat")
}

/// A generated lab report used by the T246/T280 resolution method (outside the
/// repository, like the client locale read above).
fn lab_report(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../lab/reports")
        .join(name)
}

fn read_json(path: &Path) -> Value {
    serde_json::from_slice(
        &fs::read(path).unwrap_or_else(|error| panic!("{}: {error}", path.display())),
    )
    .unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

fn npc_records(file: &str) -> Vec<Value> {
    let path = content_dir().join("npcs").join(file);
    read_json(&path)["records"]
        .as_array()
        .unwrap_or_else(|| panic!("{}: records array", path.display()))
        .clone()
}

fn npc_record(file: &str, npc_id: u64) -> Value {
    npc_records(file)
        .into_iter()
        .find(|record| record["npc_id"].as_u64() == Some(npc_id))
        .unwrap_or_else(|| panic!("{file}: NPC {npc_id}"))
}

fn npc_position(record: &Value, npc_id: u64) -> [f32; 3] {
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
    let character = character_at(npc_position(&npc_record(file, npc_id), npc_id));
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
    // Body dword follows the four-byte family/subtype header; client reader
    // 0x00a9ef10 stores it at object +16, and patch_npc_actor writes the runtime
    // actor GUID only into bytes 12..20.
    u32::from_le_bytes(dialogue[4..8].try_into().unwrap())
}

/// The loader spawns a row only when it is `legacy_data`, positioned with a
/// finite transform, and carries a positive model and name.
fn assert_drawable(record: &Value, npc_id: u64) {
    assert!(
        is_drawable(record),
        "NPC {npc_id}: must be a spawned legacy_data row with a finite position, model and name"
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
}

fn is_drawable(record: &Value) -> bool {
    record["placement"].as_str() == Some("legacy_data")
        && record["position"].as_array().is_some_and(|position| {
            position.len() == 3
                && position
                    .iter()
                    .all(|value| value.as_f64().is_some_and(f64::is_finite))
        })
        && record["model_id"].as_u64().is_some_and(|id| id > 0)
        && record["name_string_id"].as_u64().is_some_and(|id| id > 0)
}

fn drawable_rows() -> BTreeSet<(String, u64)> {
    let mut rows = BTreeSet::new();
    for file in ["farnum_farmstead.json", "highroad_junction.json"] {
        for record in npc_records(file) {
            if is_drawable(&record) {
                rows.insert((file.to_owned(), record["npc_id"].as_u64().expect("npc id")));
            }
        }
    }
    rows
}

/// Physical stock-locale rows for the wanted key hashes: one-based line and text.
fn locale_rows(wanted: &BTreeSet<u32>) -> BTreeMap<u32, (usize, String)> {
    let path = locale_path();
    let text =
        fs::read_to_string(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    let mut rows = BTreeMap::new();
    for (index, line) in text.lines().enumerate() {
        let mut fields = line.splitn(3, '\t');
        let (Some(key), Some(_table), Some(body)) = (fields.next(), fields.next(), fields.next())
        else {
            continue;
        };
        let Ok(hash) = key.trim_start_matches('\u{feff}').parse::<u32>() else {
            continue;
        };
        if wanted.contains(&hash) {
            rows.insert(hash, (index + 1, body.to_owned()));
        }
    }
    rows
}

fn locale_id_to_text() -> BTreeMap<u64, String> {
    read_json(&lab_report("locale-id-to-text.json"))
        .as_array()
        .expect("locale-id-to-text rows")
        .iter()
        .filter_map(|row| Some((row["id"].as_u64()?, row["text"].as_str()?.to_owned())))
        .collect()
}

/// The cited speaker-proof line in the repository notes.
fn guide_line(guide: &str, line: u32) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../notes")
        .join(guide);
    let text =
        fs::read_to_string(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    text.lines()
        .nth(line as usize - 1)
        .unwrap_or_else(|| panic!("{}: no line {line}", path.display()))
        .to_owned()
}

#[test]
fn every_edited_row_has_resolved_client_provenance() {
    assert!(!TARGETS.is_empty(), "this slice must bind at least one row");
    assert!(
        TARGETS.len() <= 16,
        "the slice may bind at most 16 rows, not {}",
        TARGETS.len()
    );

    let wanted = TARGETS.iter().map(|target| target.hash).collect();
    let locale = locale_rows(&wanted);
    let hash_to_ids = read_json(&lab_report("locale-id-map.json"))["hash_to_ids"].clone();
    let id_to_text = locale_id_to_text();
    let visibility = NpcVisibility::load(&content_dir()).expect("the visibility loader accepts it");

    let mut greeting_ids = BTreeSet::new();
    let mut edited = BTreeSet::new();
    for target in &TARGETS {
        assert!(target.greeting > 0, "NPC {}: zero greeting", target.npc_id);
        assert!(
            greeting_ids.insert(target.greeting),
            "NPC {}: greeting {} reused",
            target.npc_id,
            target.greeting
        );
        assert!(
            edited.insert((target.file.to_owned(), target.npc_id)),
            "NPC {}: duplicate target row",
            target.npc_id
        );

        let record = npc_record(target.file, target.npc_id);
        assert_eq!(
            record["name"].as_str(),
            Some(target.name),
            "NPC {}: npc_id resolves the expected row",
            target.npc_id
        );
        assert_eq!(
            record["greeting_string_id"].as_u64(),
            Some(u64::from(target.greeting)),
            "NPC {}: greeting binding",
            target.npc_id
        );
        assert_ne!(
            u64::from(target.greeting),
            record["name_string_id"].as_u64().expect("name id"),
            "NPC {}: a name id is not a body id",
            target.npc_id
        );
        assert_drawable(&record, target.npc_id);
        assert_eq!(
            visibility.content_key(target.npc_id),
            Some(target.npc_id),
            "NPC {} ({}) is not spawned by NpcVisibility",
            target.npc_id,
            target.name
        );

        let sources = record["field_sources"]["greeting_string_id"]
            .as_array()
            .unwrap_or_else(|| panic!("NPC {}: greeting sources", target.npc_id));
        assert_eq!(
            sources.len(),
            2,
            "NPC {}: exactly one client and one reconstructed source",
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
        let evidence = client["evidence"].as_str().unwrap_or_default();
        for needle in [
            "T315".to_owned(),
            format!("Global.Text.{}", target.greeting),
            format!("JenkinsLookup2 hash {}", target.hash),
            format!("line {}", target.dat_line),
            target.guide.to_owned(),
            format!("line {}", target.guide_line),
        ] {
            assert!(
                evidence.contains(&needle),
                "NPC {}: client source must name {needle:?}",
                target.npc_id
            );
        }
        if let Some((quest_file, quest_id)) = target.quest {
            assert!(
                evidence.contains(&format!("data/content/quests/{quest_file}"))
                    && evidence.contains(&format!("quest {quest_id}")),
                "NPC {}: client source must name its quest record",
                target.npc_id
            );
        }
        let reconstruction = reconstructed["evidence"].as_str().unwrap_or_default();
        assert!(
            reconstruction.starts_with("(R)"),
            "NPC {}: reconstructed evidence must start with (R)",
            target.npc_id
        );
        assert!(
            reconstruction.contains("fixed default Talk greeting"),
            "NPC {}: reconstructed evidence must state the fixed-default policy",
            target.npc_id
        );
        assert!(
            guide_line(target.guide, target.guide_line).contains(target.name),
            "NPC {}: {} line {} must name {}",
            target.npc_id,
            target.guide,
            target.guide_line,
            target.name
        );

        // Resolve the id the way the client does: hash Global.Text.<id>, then
        // find that hash on the cited physical locale line.
        assert_eq!(
            locale_hash(i32::try_from(target.greeting).expect("stock locale id fits i32")),
            target.hash,
            "NPC {}: greeting id must be a Global.Text key, not a raw hash",
            target.npc_id
        );
        let bound = hash_to_ids
            .get(&target.hash.to_string())
            .and_then(Value::as_array)
            .unwrap_or_else(|| {
                panic!(
                    "NPC {}: hash {} is not in locale-id-map.json",
                    target.npc_id, target.hash
                )
            });
        assert!(
            bound
                .iter()
                .any(|id| id.as_u64() == Some(u64::from(target.greeting))),
            "NPC {}: hash {} must resolve to {}",
            target.npc_id,
            target.hash,
            target.greeting
        );
        let reported = id_to_text
            .get(&u64::from(target.greeting))
            .unwrap_or_else(|| {
                panic!(
                    "NPC {}: id {} is not in locale-id-to-text.json",
                    target.npc_id, target.greeting
                )
            });
        let (line, text) = locale.get(&target.hash).unwrap_or_else(|| {
            panic!(
                "NPC {}: hash {} is not a stock client locale key",
                target.npc_id, target.hash
            )
        });
        assert_eq!(
            *line, target.dat_line as usize,
            "NPC {}: en_us_data.dat line",
            target.npc_id
        );
        assert_eq!(
            text, reported,
            "NPC {}: stock locale text must equal the id-to-text report",
            target.npc_id
        );
        assert!(
            !text.trim().is_empty(),
            "NPC {}: recovered locale text is empty",
            target.npc_id
        );
    }
    let pre_existing: BTreeSet<u32> = T246_GREETINGS
        .iter()
        .chain(W126_GREETINGS.iter())
        .map(|(_, _, greeting)| *greeting)
        .collect();
    assert!(
        greeting_ids.is_disjoint(&pre_existing),
        "a new greeting reuses an existing binding"
    );

    // Every drawable row in the two files is a pre-existing greeted row, a row
    // bound here, or an unbound row listed in notes/Autopilot/T315.md.
    let drawable = drawable_rows();
    let mut declared: BTreeSet<(String, u64)> = TARGETS
        .iter()
        .map(|target| (target.file.to_owned(), target.npc_id))
        .chain(
            UNBOUND
                .iter()
                .map(|(file, npc_id)| ((*file).to_owned(), *npc_id)),
        )
        .collect();
    for (file, npc_id, _) in T246_GREETINGS.iter().chain(W126_GREETINGS.iter()) {
        if *file == "farnum_farmstead.json" || *file == "highroad_junction.json" {
            assert!(
                declared.insert(((*file).to_owned(), *npc_id)),
                "NPC {npc_id}: a pre-existing greeted row is also declared unbound"
            );
        }
    }
    assert_eq!(
        drawable, declared,
        "list every drawable row that is not bound in notes/Autopilot/T315.md"
    );
    for (file, npc_id) in UNBOUND {
        let record = npc_record(file, npc_id);
        assert_drawable(&record, npc_id);
        assert!(
            record.get("greeting_string_id").is_none(),
            "NPC {npc_id}: an unbound row must stay blank"
        );
        assert!(
            record["field_sources"].get("greeting_string_id").is_none(),
            "NPC {npc_id}: an unbound row must carry no greeting source"
        );
    }
}

#[test]
fn highroad_and_farnum_rows_send_their_greeting_in_26_3() {
    // Service path: the context the router copies into the 26/3 body.
    let mut interaction = service();
    let mut greetings = BTreeSet::new();
    for target in &TARGETS {
        assert!(
            greetings.insert(target.greeting),
            "NPC {}: greeting {} is reused",
            target.npc_id,
            target.greeting
        );
        assert_eq!(
            service_talk_body(&mut interaction, target.npc_id),
            Some(target.greeting),
            "NPC {} ({}) service Talk body",
            target.npc_id,
            target.name
        );
    }
    // Dispatch path: gateway -> Gameplay -> gameplay/npc.rs -> dialogue_packet_with.
    let mut game = Gameplay::load(&gameplay_dir(), None).expect("gameplay content loads");
    for target in &TARGETS {
        assert_eq!(
            gameplay_talk_body(&mut game, target.file, target.npc_id),
            target.greeting,
            "NPC {} ({}) 26/3 body dword",
            target.npc_id,
            target.name
        );
    }

    // A reconnect rebuilds the same static content; the body is not session state.
    let first = &TARGETS[0];
    assert_eq!(
        service_talk_body(&mut service(), first.npc_id),
        Some(first.greeting),
        "NPC {}: a fresh loader sends the same body",
        first.npc_id
    );
}

#[test]
fn a_zero_or_missing_greeting_falls_back_to_no_body() {
    let mut service = service();
    for (file, npc_id) in [
        ("highroad_junction.json", 9500000333),
        ("farnum_farmstead.json", 9500000269),
    ] {
        let record = npc_record(file, npc_id);
        assert!(
            record.get("greeting_string_id").is_none(),
            "NPC {npc_id}: the blank control must stay blank"
        );
        assert_drawable(&record, npc_id);
        assert_eq!(
            service_talk_body(&mut service, npc_id),
            None,
            "NPC {npc_id}: an NPC without a greeting has no body"
        );
    }
    assert!(
        TARGETS.iter().all(|target| target.greeting != 0),
        "zero is not a recovered greeting id"
    );

    // A refused choice must leave the menu usable (F271): select Talk directly
    // on the still-open menu, with no second `interact`, so this fails if
    // `select` consumes `active` on `InvalidOption`.
    let blank = 9500000333;
    let (zone, position) = service.location(blank).unwrap();
    let menu = service.interact(blank, zone, position, None).unwrap();
    let talk = menu
        .options
        .iter()
        .find(|option| option.action == NpcAction::Talk)
        .expect("the blank NPC offers Talk")
        .id;
    assert!(matches!(
        service.select(&select_request(blank, i32::MAX), zone, position, None),
        Err(NpcInteractionError::InvalidOption)
    ));
    match service.select(&select_request(blank, talk), zone, position, None) {
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

    // The real router still encodes 26/3 with a zero body and no invented id.
    let mut game = Gameplay::load(&gameplay_dir(), None).expect("gameplay content loads");
    assert_eq!(
        gameplay_talk_body(&mut game, "highroad_junction.json", blank),
        0,
        "no invented locale id"
    );

    // An out-of-list 26/10 on a bound row's open menu is refused, and Talk
    // straight afterwards still returns that row's own recovered body.
    let bound = &TARGETS[0];
    let character = character_at(npc_position(
        &npc_record(bound.file, bound.npc_id),
        bound.npc_id,
    ));
    let menu = game
        .handle_in_world(&interact_request(bound.npc_id), &character, 2_000)
        .expect("bound NPC menu");
    assert!(
        menu.replies
            .iter()
            .any(|packet| packet.starts_with(&MENU_HEADER))
    );
    assert!(
        game.handle_in_world(&select_request(bound.npc_id, i32::MAX), &character, 2_100)
            .is_err(),
        "an out-of-list choice is refused"
    );
    let after = game
        .handle_in_world(&select_request(bound.npc_id, 1), &character, 2_200)
        .expect("the menu must survive the refusal");
    let dialogue = after
        .replies
        .into_iter()
        .find(|packet| packet.starts_with(&GREETING_HEADER))
        .expect("the refused choice must not consume the menu");
    assert_eq!(
        u32::from_le_bytes(dialogue[4..8].try_into().unwrap()),
        bound.greeting,
        "the refusal changed nothing"
    );
}

#[test]
fn the_t246_rows_and_all_other_regions_are_unchanged() {
    for (file, npc_id, greeting) in T246_GREETINGS {
        assert_eq!(
            npc_record(file, npc_id)["greeting_string_id"].as_u64(),
            Some(u64::from(greeting)),
            "T246 binding {npc_id} in {file} changed"
        );
    }
    for (file, npc_id, greeting) in W126_GREETINGS {
        assert_eq!(
            npc_record(file, npc_id)["greeting_string_id"].as_u64(),
            Some(u64::from(greeting)),
            "W126 binding {npc_id} in {file} changed"
        );
    }

    // Every non-target row in the two edited files keeps its exact greeting
    // state (absent, or one of the pinned pre-existing bindings), and no row
    // outside the target set carries this slice's evidence.
    let targets: BTreeSet<(String, u64)> = TARGETS
        .iter()
        .map(|target| (target.file.to_owned(), target.npc_id))
        .collect();
    let pinned: BTreeMap<(String, u64), u32> = T246_GREETINGS
        .iter()
        .chain(W126_GREETINGS.iter())
        .map(|(file, npc_id, greeting)| ((file.to_string(), *npc_id), *greeting))
        .collect();
    for (file, expected_rows) in [
        ("farnum_farmstead.json", FARNUM_RECORDS),
        ("highroad_junction.json", HIGHROAD_RECORDS),
    ] {
        let records = npc_records(file);
        assert_eq!(records.len(), expected_rows, "{file}: row count");
        for record in &records {
            let npc_id = record["npc_id"].as_u64().expect("npc id");
            if targets.contains(&(file.to_owned(), npc_id)) {
                continue;
            }
            let expected = pinned
                .get(&(file.to_owned(), npc_id))
                .map(|greeting| u64::from(*greeting));
            assert_eq!(
                record["greeting_string_id"].as_u64(),
                expected,
                "{file} NPC {npc_id}: a non-target greeting changed"
            );
            assert_eq!(
                record["field_sources"].get("greeting_string_id").is_some(),
                expected.is_some(),
                "{file} NPC {npc_id}: non-target greeting sources changed"
            );
            assert!(
                !record.to_string().contains("T315"),
                "{file} NPC {npc_id}: T315 evidence outside the target rows"
            );
        }
    }

    // All 15 other region files keep their top-level values and gain no row
    // from this slice.
    for (file, schema_version, region_page, region_zone_id) in OTHER_REGIONS {
        let document = read_json(&content_dir().join("npcs").join(file));
        assert_eq!(
            document["schema_version"].as_u64(),
            Some(u64::from(schema_version)),
            "{file}: schema version"
        );
        assert_eq!(
            document["region_page"].as_str(),
            Some(region_page),
            "{file}: region page"
        );
        assert_eq!(
            document["region_zone_id"].as_u64(),
            Some(region_zone_id),
            "{file}: region zone id"
        );
        assert!(
            !document.to_string().contains("T315"),
            "{file}: T315 edited a region file outside this slice"
        );
    }
}

#[test]
fn the_content_stays_within_the_loader_bounds() {
    let data = content_dir().join("npcs");
    let mut edited_bytes = 0u64;
    for file in ["farnum_farmstead.json", "highroad_junction.json"] {
        let path = data.join(file);
        let bytes = fs::read(&path).unwrap();
        let size = bytes.len() as u64;
        assert!(
            size <= 8 * 1024 * 1024,
            "{file}: exceeds the 8 MiB per-file loader bound"
        );
        edited_bytes += size;
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
        edited_bytes <= 32 * 1024 * 1024,
        "the two edited files exceed the 32 MiB loader bound"
    );
    // The loader's 32 MiB total counts every `npcs/*.json` file.
    let mut all_bytes = 0u64;
    for entry in fs::read_dir(&data).expect("NPC content directory") {
        let path = entry.expect("NPC content entry").path();
        if path
            .extension()
            .is_some_and(|extension| extension == "json")
        {
            all_bytes += fs::metadata(&path).expect("NPC content metadata").len();
        }
    }
    assert!(
        all_bytes <= 32 * 1024 * 1024,
        "the NPC content directory exceeds the 32 MiB loader bound"
    );
    // The loader itself is the byte, row-count and shape gate.
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
        let sources = record["field_sources"]["greeting_string_id"]
            .as_array()
            .unwrap_or_else(|| panic!("NPC {}: greeting sources", target.npc_id));
        for source in sources {
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
            if source["kind"] == "reconstructed" {
                assert!(
                    evidence.starts_with("(R)"),
                    "NPC {}: (R) sources must start with (R)",
                    target.npc_id
                );
            }
        }
    }
}
