//! T381: the remaining drawable Merry Vale and Stillwater Crossing NPC rows
//! carry recovered first-person greetings.
//!
//! Content only: `NpcInteraction::dialogue_context` already copies
//! `greeting_string_id` into `DialogueContext.body_string_id`, and
//! `gameplay/npc.rs::dialogue_packet_with` writes that body into the first 26/3
//! payload dword. This slice adds no production code, spawn, quest or menu.
//!
//! Client evidence (re-read for T316/T381): 26/8 constructor `0x008b6bc0`,
//! 26/10 constructor `0x00a36f80`, 26/3 header `0x00a9cb50`, body reader
//! `0x00a9ef10` (`*(this + 16) = *v3`), display `0x009f5ae0` (passes
//! `*(a2 + 16)` to the `Global.Text` lookup before `SetDialogText`), 26/4
//! `0x00aa0870`. The body is the first dword after the four-byte family/subtype
//! header; no wire field changed. The packet helpers, the locale line
//! convention (`str::lines()`) and the drawability rule live in the shared
//! `tests/common/npc_greetings.rs` harness.
//!
//! The `BLANK_WITH_REASON` table is the recorded decision for every drawable
//! row this slice leaves blank. The `(R)` source notes say what speaker link is
//! inferred; no text is invented and no line is borrowed from a neighbour.

#[path = "common/npc_greetings.rs"]
mod npc_greetings;

use npc_greetings::{
    assert_drawable, content_dir, drawable, gameplay_dir, gameplay_talk_body, greeting_bindings,
    locale_rows, npc_record, npc_records, select, service, service_talk_body,
};
use osfr_tools::{
    gameplay::Gameplay,
    npc_interaction::{NpcAction, NpcInteractionError, NpcSelection},
    world::NpcVisibility,
    world_text::locale_hash,
};
use serde_json::Value;
use std::collections::BTreeSet;

const MERRY_VALE_GUIDE: &str = "notes/Build Guides/Content/NPCs/NPCs - Merry Vale.md";
const STILLWATER_GUIDE: &str = "notes/Build Guides/Content/NPCs/NPCs - Stillwater Crossing.md";

/// The two NPC files this slice owns.
const OWNED_FILES: [&str; 2] = ["merry_vale.json", "stillwater_crossing.json"];

/// One bound row: content key, the `Global.Text` id the client looks up, its
/// JenkinsLookup2 hash, the physical stock-locale line, the region guide row and
/// whether the speaker link is inferred (the source note then starts `(R)`).
struct Greeting {
    npc_id: u64,
    file: &'static str,
    greeting: u32,
    hash: u32,
    dat_line: u32,
    guide: &'static str,
    guide_line: u32,
    inferred: bool,
}

/// The 17 rows T381 binds. Every line is in the row's own recovered text; the
/// proof and the `(R)` inference (where any) are in the row's
/// `field_sources.greeting_string_id` client note.
const NEW_GREETINGS: [Greeting; 17] = [
    Greeting {
        npc_id: 9500000368,
        file: "merry_vale.json",
        greeting: 115532,
        hash: 2532711647,
        dat_line: 66723,
        guide: MERRY_VALE_GUIDE,
        guide_line: 70,
        inferred: false,
    },
    Greeting {
        npc_id: 9500000369,
        file: "merry_vale.json",
        greeting: 70932,
        hash: 2531687873,
        dat_line: 66687,
        guide: MERRY_VALE_GUIDE,
        guide_line: 80,
        inferred: true,
    },
    Greeting {
        npc_id: 9500000385,
        file: "merry_vale.json",
        greeting: 70762,
        hash: 1293215637,
        dat_line: 34252,
        guide: MERRY_VALE_GUIDE,
        guide_line: 33,
        inferred: false,
    },
    Greeting {
        npc_id: 9500000394,
        file: "merry_vale.json",
        greeting: 2885,
        hash: 2216243585,
        dat_line: 58327,
        guide: MERRY_VALE_GUIDE,
        guide_line: 57,
        inferred: true,
    },
    Greeting {
        npc_id: 9500000395,
        file: "merry_vale.json",
        greeting: 46673,
        hash: 3285115298,
        dat_line: 86747,
        guide: MERRY_VALE_GUIDE,
        guide_line: 56,
        inferred: false,
    },
    Greeting {
        npc_id: 9500000396,
        file: "merry_vale.json",
        greeting: 427168,
        hash: 2333594798,
        dat_line: 61367,
        guide: MERRY_VALE_GUIDE,
        guide_line: 119,
        inferred: false,
    },
    Greeting {
        npc_id: 9500000398,
        file: "merry_vale.json",
        greeting: 2898,
        hash: 1115395599,
        dat_line: 29544,
        guide: MERRY_VALE_GUIDE,
        guide_line: 58,
        inferred: true,
    },
    Greeting {
        npc_id: 9500000405,
        file: "merry_vale.json",
        greeting: 71075,
        hash: 1748567956,
        dat_line: 46160,
        guide: MERRY_VALE_GUIDE,
        guide_line: 31,
        inferred: true,
    },
    Greeting {
        npc_id: 9500000410,
        file: "merry_vale.json",
        greeting: 46252,
        hash: 2197412592,
        dat_line: 57859,
        guide: MERRY_VALE_GUIDE,
        guide_line: 91,
        inferred: false,
    },
    Greeting {
        npc_id: 9500000411,
        file: "merry_vale.json",
        greeting: 117186,
        hash: 1492140926,
        dat_line: 39507,
        guide: MERRY_VALE_GUIDE,
        guide_line: 86,
        inferred: true,
    },
    Greeting {
        npc_id: 9500000414,
        file: "merry_vale.json",
        greeting: 385493,
        hash: 3989348772,
        dat_line: 105481,
        guide: MERRY_VALE_GUIDE,
        guide_line: 81,
        inferred: true,
    },
    Greeting {
        npc_id: 9500000418,
        file: "merry_vale.json",
        greeting: 71363,
        hash: 1766237798,
        dat_line: 46622,
        guide: MERRY_VALE_GUIDE,
        guide_line: 73,
        inferred: false,
    },
    Greeting {
        npc_id: 9500000419,
        file: "merry_vale.json",
        greeting: 46570,
        hash: 3306096356,
        dat_line: 87359,
        guide: MERRY_VALE_GUIDE,
        guide_line: 74,
        inferred: false,
    },
    Greeting {
        npc_id: 9500000421,
        file: "merry_vale.json",
        greeting: 117031,
        hash: 379954746,
        dat_line: 10075,
        guide: MERRY_VALE_GUIDE,
        guide_line: 87,
        inferred: true,
    },
    Greeting {
        npc_id: 9500000431,
        file: "merry_vale.json",
        greeting: 387033,
        hash: 2896366823,
        dat_line: 76401,
        guide: MERRY_VALE_GUIDE,
        guide_line: 75,
        inferred: false,
    },
    Greeting {
        npc_id: 9500000432,
        file: "merry_vale.json",
        greeting: 74198,
        hash: 1472403084,
        dat_line: 38978,
        guide: MERRY_VALE_GUIDE,
        guide_line: 77,
        inferred: false,
    },
    Greeting {
        npc_id: 9000000124,
        file: "stillwater_crossing.json",
        greeting: 395718,
        hash: 186107631,
        dat_line: 4925,
        guide: STILLWATER_GUIDE,
        guide_line: 56,
        inferred: true,
    },
];

/// T316's 17 bindings; this slice must not disturb them.
const T316_BINDINGS: [(u64, &str, u32); 17] = [
    (9000000222, "merry_vale.json", 71035),
    (9000000030, "merry_vale.json", 71067),
    (9000000174, "merry_vale.json", 117035),
    (9500000371, "merry_vale.json", 427481),
    (9000000044, "merry_vale.json", 71783),
    (9500000417, "merry_vale.json", 387081),
    (9500000420, "merry_vale.json", 41743),
    (9500000424, "merry_vale.json", 441732),
    (9500000427, "merry_vale.json", 141684),
    (9500000400, "merry_vale.json", 73922),
    (9500000381, "merry_vale.json", 387043),
    (9000000038, "stillwater_crossing.json", 116793),
    (9500000785, "stillwater_crossing.json", 388502),
    (9000000196, "stillwater_crossing.json", 116508),
    (9000000103, "stillwater_crossing.json", 116777),
    (9000000096, "stillwater_crossing.json", 116496),
    (9000000181, "stillwater_crossing.json", 408272),
];

/// The two bindings that already existed in the owned files (W126).
const OWNED_PRE_EXISTING: [(u64, &str, u32); 2] = [
    (9500000430, "merry_vale.json", 41268),
    (9500000786, "stillwater_crossing.json", 116463),
];

/// A sample from other regions, pinned by their own suites; this slice must not
/// perturb them.
const OTHER_REGION_SAMPLES: [(&str, u64, u32); 2] = [
    ("cobblestone_village.json", 9000000189, 24940),
    ("farnum_farmstead.json", 9000000165, 4420036),
];

/// A drawable row this slice deliberately leaves blank, with the exact reason.
struct Blank {
    npc_id: u64,
    file: &'static str,
    reason: &'static str,
}

/// Every drawable row left blank. A new blank row fails
/// `every_recovered_blank_row_is_bound_or_recorded` until it is recorded here.
const BLANK_WITH_REASON: [Blank; 20] = [
    Blank {
        npc_id: 9500000370,
        file: "merry_vale.json",
        reason: "Panel-only 2013 Postman gear vendor: the locale block lists his name and gear-shop labels, and no first-person line is recovered for him.",
    },
    Blank {
        npc_id: 9500000374,
        file: "merry_vale.json",
        reason: "Enemy minion row: the only related speech is the Festival-of-Hearts combat-bark block (427126-427131), shared by the encounter and not proven to be this row.",
    },
    Blank {
        npc_id: 9500000375,
        file: "merry_vale.json",
        reason: "Panel-only 2013 Warrior gear vendor: no first-person line is recovered.",
    },
    Blank {
        npc_id: 9500000378,
        file: "merry_vale.json",
        reason: "Wiki-only fairy: the client locale has the name but no line for her.",
    },
    Blank {
        npc_id: 9500000388,
        file: "merry_vale.json",
        reason: "Vendor row: her locale block holds Snowhill blacksmith/Smitty lines, not a Giggles line.",
    },
    Blank {
        npc_id: 9500000399,
        file: "merry_vale.json",
        reason: "Rescue target: only \"Find Lessi!\" / \"Return to Meliee!\" objectives; no first-person line.",
    },
    Blank {
        npc_id: 9500000402,
        file: "merry_vale.json",
        reason: "Only Aida's description of Lynzi's balloons (\"she says they need to fly free\"); no line in Lynzi's voice.",
    },
    Blank {
        npc_id: 9500000404,
        file: "merry_vale.json",
        reason: "No speaker-proven line: the \"News Girl\" title is unconfirmed and the generic News Girl/Boy welcome (432459) is not tied to this row.",
    },
    Blank {
        npc_id: 9500000406,
        file: "merry_vale.json",
        reason: "Panel-only 2013 Archer gear vendor: no first-person line is recovered.",
    },
    Blank {
        npc_id: 9500000407,
        file: "merry_vale.json",
        reason: "No matched quest and no recovered speech; only name entries.",
    },
    Blank {
        npc_id: 9500000408,
        file: "merry_vale.json",
        reason: "Panel-only 2013 Brawler gear vendor: no first-person line is recovered.",
    },
    Blank {
        npc_id: 9500000409,
        file: "merry_vale.json",
        reason: "Script receiver only (\"Deliver the newly revised script to Morrie\"); no line in Morrie's voice.",
    },
    Blank {
        npc_id: 9500000412,
        file: "merry_vale.json",
        reason: "Panel-only 2013 Chef gear vendor: no first-person line is recovered.",
    },
    Blank {
        npc_id: 9500000426,
        file: "merry_vale.json",
        reason: "Only the Festival-of-Hearts combat barks (427126-427131); the block is shared by Stone Heart and his changelings, so the speaker is not proven for this row.",
    },
    Blank {
        npc_id: 9500000428,
        file: "merry_vale.json",
        reason: "Audition only: her in-role lines (2889-2891) are second-person/imperative and contain no first-person marker.",
    },
    Blank {
        npc_id: 9500000787,
        file: "stillwater_crossing.json",
        reason: "2013 Wizard gear vendor: the nearby wand-for-sale line (5102521) sits in a multi-vendor wizard-merchant block (Melninster/Falemicent/Alusru etc. also sell wands), so the speaker is not unique.",
    },
    Blank {
        npc_id: 9500000790,
        file: "stillwater_crossing.json",
        reason: "Panel-only 2013 Chef gear vendor: no first-person line is recovered.",
    },
    Blank {
        npc_id: 9500000791,
        file: "stillwater_crossing.json",
        reason: "Panel-only 2013 Medic gear vendor: no first-person line is recovered.",
    },
    Blank {
        npc_id: 9500000793,
        file: "stillwater_crossing.json",
        reason: "Panel-only 2013 Postman gear vendor: no first-person line is recovered.",
    },
    Blank {
        npc_id: 9000000227,
        file: "stillwater_crossing.json",
        reason: "Only the zone Welcome window (3453817544, explicitly not tied to a speaker per the guide) and third-person objectives; no first-person line.",
    },
];

fn is_blank(record: &Value) -> bool {
    record
        .get("greeting_string_id")
        .and_then(Value::as_u64)
        .is_none_or(|greeting| greeting == 0)
}

/// A first-person marker as a whole word. Case-insensitive so a line opening
/// with "My ..." counts; a third-person objective line has no such word.
fn is_first_person(text: &str) -> bool {
    text.split(|character: char| !(character.is_alphanumeric() || character == '\''))
        .map(|word| word.to_ascii_lowercase())
        .any(|word| {
            matches!(
                word.as_str(),
                "i" | "i'm"
                    | "i've"
                    | "i'd"
                    | "i'll"
                    | "me"
                    | "my"
                    | "mine"
                    | "we"
                    | "we're"
                    | "our"
                    | "ours"
                    | "us"
            )
        })
}

/// The exact locale line quoted at the start of a client source note.
fn quoted_locale_line(evidence: &str) -> &str {
    let marker = ": \"";
    let start = evidence
        .find(marker)
        .unwrap_or_else(|| panic!("source note has no quoted locale line: {evidence}"))
        + marker.len();
    let end = start
        + evidence[start..]
            .find('"')
            .unwrap_or_else(|| panic!("source note has no closing quote: {evidence}"));
    &evidence[start..end]
}

/// Bound after T316 (T894 Rylan, an undrawn row); expected in the owned files.
const LATER_BOUND: [(u64, &str); 1] = [(9500000422, "merry_vale.json")];

fn known_bound() -> BTreeSet<(String, u64)> {
    NEW_GREETINGS
        .iter()
        .map(|target| (target.file.to_owned(), target.npc_id))
        .chain(
            T316_BINDINGS
                .iter()
                .map(|(npc_id, file, _)| ((*file).to_owned(), *npc_id)),
        )
        .chain(
            OWNED_PRE_EXISTING
                .iter()
                .map(|(npc_id, file, _)| ((*file).to_owned(), *npc_id)),
        )
        .chain(
            LATER_BOUND
                .iter()
                .map(|(npc_id, file)| ((*file).to_owned(), *npc_id)),
        )
        .collect()
}

/// The drawable set is derived at the tip, not from the brief's old count: a
/// row that another task bound or un-spawned changes this test. Every drawable
/// row is either one of the known bindings or in `BLANK_WITH_REASON`.
#[test]
fn every_recovered_blank_row_is_bound_or_recorded() {
    let blanks: BTreeSet<(String, u64)> = BLANK_WITH_REASON
        .iter()
        .map(|blank| (blank.file.to_owned(), blank.npc_id))
        .collect();
    assert_eq!(
        blanks.len(),
        BLANK_WITH_REASON.len(),
        "a row is listed twice in BLANK_WITH_REASON"
    );
    let known = known_bound();
    assert!(
        known.is_disjoint(&blanks),
        "a row is both bound and recorded blank"
    );

    let visibility =
        NpcVisibility::load(&content_dir()).expect("the production visibility loader accepts it");
    let mut interaction = service();
    let mut known_seen = BTreeSet::new();
    let mut blank_seen = BTreeSet::new();
    for file in OWNED_FILES {
        for record in npc_records(file) {
            let npc_id = record["npc_id"].as_u64().expect("NPC row has an id");
            if !drawable(&record) {
                continue;
            }
            assert_eq!(
                visibility.content_key(npc_id),
                Some(npc_id),
                "NPC {npc_id} is not spawned by NpcVisibility"
            );
            let key = (file.to_owned(), npc_id);
            let name = record["name"].as_str().unwrap_or_default();
            if is_blank(&record) {
                assert!(
                    blanks.contains(&key),
                    "NPC {npc_id} ({name}) in {file} is a new blank; record it in BLANK_WITH_REASON"
                );
                let body = service_talk_body(&mut interaction, npc_id);
                let explicit_zero = record["greeting_string_id"].as_u64() == Some(0);
                assert_eq!(
                    body,
                    if explicit_zero { Some(0) } else { None },
                    "NPC {npc_id}: a blank row must not send a body"
                );
                blank_seen.insert(key);
            } else {
                assert!(
                    known.contains(&key),
                    "NPC {npc_id} in {file} has an unrecorded greeting"
                );
                known_seen.insert(key);
            }
        }
    }

    for target in &NEW_GREETINGS {
        let record = npc_record(target.file, target.npc_id);
        let name = record["name"].as_str().unwrap_or_default();
        assert_drawable(&record, target.npc_id);
        assert_eq!(
            record["greeting_string_id"].as_u64(),
            Some(u64::from(target.greeting)),
            "NPC {} ({name}): greeting binding",
            target.npc_id
        );
        assert!(
            known_seen.contains(&(target.file.to_owned(), target.npc_id)),
            "NPC {} ({name}): not reached as a drawable row",
            target.npc_id
        );
    }
    for (npc_id, file, _) in T316_BINDINGS {
        assert!(
            known_seen.contains(&(file.to_owned(), npc_id)),
            "T316 binding {npc_id} in {file} is no longer a drawable row"
        );
    }
    for blank in &BLANK_WITH_REASON {
        assert!(
            blank.reason.trim().len() > 40,
            "NPC {}: a blank row needs its exact reason",
            blank.npc_id
        );
        let record = npc_record(blank.file, blank.npc_id);
        assert_drawable(&record, blank.npc_id);
        assert!(
            is_blank(&record),
            "NPC {}: a BLANK_WITH_REASON row now carries a greeting",
            blank.npc_id
        );
        assert!(
            blank_seen.contains(&(blank.file.to_owned(), blank.npc_id)),
            "NPC {}: not reached as a blank drawable row",
            blank.npc_id
        );
    }
}

/// Each new greeting is the row's own recovered first-person line: the id
/// resolves in the stock locale, the source note quotes that exact line, the
/// line has a first-person marker, and no other row reuses the id. A
/// third-person objective line or a borrowed neighbour line fails.
#[test]
fn each_new_greeting_is_the_npcs_own_first_person_line() {
    let wanted: BTreeSet<u32> = NEW_GREETINGS.iter().map(|target| target.hash).collect();
    let locale = locale_rows(&wanted);
    let mut greeting_ids = BTreeSet::new();
    for target in &NEW_GREETINGS {
        assert!(
            target.greeting > 0,
            "NPC {}: greeting must be positive",
            target.npc_id
        );
        assert!(
            greeting_ids.insert(target.greeting),
            "NPC {}: greeting {} reused by another new binding",
            target.npc_id,
            target.greeting
        );
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
            *line, target.dat_line as usize,
            "NPC {}: en_us_data.dat line",
            target.npc_id
        );
        assert!(
            is_first_person(text),
            "NPC {}: the greeting line is not first-person: {text}",
            target.npc_id
        );

        let record = npc_record(target.file, target.npc_id);
        let name = record["name"].as_str().expect("NPC name");
        assert_eq!(
            record["greeting_string_id"].as_u64(),
            Some(u64::from(target.greeting)),
            "NPC {} ({name}): greeting binding",
            target.npc_id
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
        let evidence = client["evidence"].as_str().expect("client evidence");
        assert!(
            evidence.contains(name),
            "NPC {}: the source note must name the speaker",
            target.npc_id
        );
        for expected in [
            format!("Global.Text.{}", target.greeting),
            format!("en_us_data.dat line {}", target.dat_line),
            format!("JenkinsLookup2 hash {}", target.hash),
            format!("{} line {}", target.guide, target.guide_line),
        ] {
            assert!(
                evidence.contains(&expected),
                "NPC {}: source must name {expected:?}",
                target.npc_id
            );
        }
        if target.inferred {
            assert!(
                evidence.starts_with("(R) speaker inferred") && evidence.contains("Inferred from"),
                "NPC {}: an inferred speaker link must be labelled (R) and say what is inferred",
                target.npc_id
            );
        } else {
            assert!(
                evidence.contains("Speaker proof:"),
                "NPC {}: an exact speaker link must say so",
                target.npc_id
            );
        }
        assert_eq!(
            quoted_locale_line(evidence),
            text,
            "NPC {}: the source note must quote the exact locale line",
            target.npc_id
        );
        let reconstructed = sources
            .iter()
            .find(|source| source["kind"] == "reconstructed")
            .unwrap_or_else(|| panic!("NPC {}: reconstructed source", target.npc_id));
        let reconstructed = reconstructed["evidence"].as_str().unwrap_or_default();
        assert!(
            reconstructed.starts_with("(R)")
                && reconstructed.contains("fixed default Talk greeting"),
            "NPC {}: the reconstruction must be labelled (R) fixed-default policy",
            target.npc_id
        );
    }

    // No other row (in either owned file or any other region) reuses an id.
    let bindings = greeting_bindings();
    for target in &NEW_GREETINGS {
        let reused: Vec<_> = bindings
            .iter()
            .filter(|((file, npc_id), greeting)| {
                **greeting == target.greeting && !(file == target.file && *npc_id == target.npc_id)
            })
            .map(|((file, npc_id), greeting)| (file.clone(), *npc_id, *greeting))
            .collect();
        assert!(
            reused.is_empty(),
            "NPC {}: greeting {} is reused by {reused:?}",
            target.npc_id,
            target.greeting
        );
    }
}

/// Five representative rows drive the real 26/8 -> 26/9 -> Talk path and the
/// in-world router. The 26/3 body is the row's greeting; the menu still carries
/// only the existing options (Talk first, no greeting-derived option).
#[test]
fn the_new_greetings_are_26_3_bodies() {
    let representatives: [(u64, &str); 5] = [
        (9500000368, "merry_vale.json"),
        (9500000395, "merry_vale.json"),
        (9500000405, "merry_vale.json"),
        (9500000414, "merry_vale.json"),
        (9000000124, "stillwater_crossing.json"),
    ];
    let mut interaction = service();
    for (npc_id, file) in representatives {
        let target = NEW_GREETINGS
            .iter()
            .find(|target| target.npc_id == npc_id && target.file == file)
            .unwrap_or_else(|| panic!("NPC {npc_id} is not one of the new bindings"));
        let (zone, position) = interaction
            .location(npc_id)
            .unwrap_or_else(|| panic!("NPC {npc_id} has a location"));
        let menu = interaction
            .interact(npc_id, zone, position, None)
            .unwrap_or_else(|error| panic!("NPC {npc_id} menu: {error:?}"));
        assert_eq!(
            menu.options.len(),
            1,
            "NPC {npc_id}: the greeting adds no menu option"
        );
        assert_eq!(menu.options[0].id, 1, "NPC {npc_id}: Talk is choice 1");
        assert!(
            matches!(menu.options[0].action, NpcAction::Talk),
            "NPC {npc_id}: the only option is Talk"
        );
        assert!(
            menu.packet.starts_with(&[26, 0, 9, 0]),
            "NPC {npc_id}: the 26/8 interaction is answered with a 26/9 menu"
        );
        let talk = u32::try_from(menu.options[0].id).expect("Talk choice fits u32");
        match interaction
            .select(&select(npc_id, talk), zone, position, None)
            .unwrap_or_else(|error| panic!("NPC {npc_id} Talk: {error:?}"))
        {
            NpcSelection::Talk(context) => assert_eq!(
                context.body_string_id,
                Some(target.greeting),
                "NPC {npc_id}: service Talk body"
            ),
            _ => panic!("NPC {npc_id}: Talk must return dialogue"),
        }
    }

    let mut game = Gameplay::load(&gameplay_dir(), None).expect("gameplay content loads");
    for (npc_id, file) in representatives {
        let target = NEW_GREETINGS
            .iter()
            .find(|target| target.npc_id == npc_id && target.file == file)
            .unwrap_or_else(|| panic!("NPC {npc_id} is not one of the new bindings"));
        assert_eq!(
            gameplay_talk_body(&mut game, file, npc_id),
            target.greeting,
            "NPC {npc_id}: in-world 26/3 body"
        );
    }
}

/// T316's rows, the two pre-existing owned rows and a sample from other regions
/// keep their exact greeting ids.
#[test]
fn the_t316_bindings_and_other_regions_are_unchanged() {
    for (npc_id, file, greeting) in T316_BINDINGS {
        assert_eq!(
            npc_record(file, npc_id)["greeting_string_id"].as_u64(),
            Some(u64::from(greeting)),
            "T316 binding {npc_id} in {file} changed"
        );
    }
    for (npc_id, file, greeting) in OWNED_PRE_EXISTING {
        assert_eq!(
            npc_record(file, npc_id)["greeting_string_id"].as_u64(),
            Some(u64::from(greeting)),
            "pre-existing binding {npc_id} in {file} changed"
        );
    }
    for (file, npc_id, greeting) in OTHER_REGION_SAMPLES {
        assert_eq!(
            npc_record(file, npc_id)["greeting_string_id"].as_u64(),
            Some(u64::from(greeting)),
            "another region's binding {npc_id} in {file} changed"
        );
    }

    // The owned files carry exactly the known bindings, no more and no less.
    let bindings = greeting_bindings();
    let owned: BTreeSet<(String, u64)> = bindings
        .into_keys()
        .filter(|(file, _)| OWNED_FILES.contains(&file.as_str()))
        .collect();
    assert_eq!(
        owned,
        known_bound(),
        "the owned files gained or lost a greeting binding"
    );
}

/// Two sessions clicking the same static NPC get the same body; a blank control
/// sends body 0 and never inherits a neighbour's line. A refused choice leaves
/// the open menu usable, so a refusal changes nothing.
#[test]
fn two_players_see_the_same_line_and_a_blank_stays_blank() {
    let target = NEW_GREETINGS
        .iter()
        .find(|target| target.npc_id == 9500000431)
        .expect("the Verna binding is in the table");
    let mut player_a = service();
    let mut player_b = service();
    assert_eq!(
        service_talk_body(&mut player_a, target.npc_id),
        Some(target.greeting),
        "player A body"
    );
    assert_eq!(
        service_talk_body(&mut player_b, target.npc_id),
        Some(target.greeting),
        "player B body"
    );
    // One player's close cannot blank the other's static content.
    player_a.close_dialogue();
    assert_eq!(
        service_talk_body(&mut player_b, target.npc_id),
        Some(target.greeting),
        "player B body after player A closed"
    );

    let blank = BLANK_WITH_REASON
        .iter()
        .find(|blank| blank.npc_id == 9500000404)
        .expect("Madison is the recorded blank control");
    let (zone, position) = player_a.location(blank.npc_id).expect("blank location");
    let menu = player_a
        .interact(blank.npc_id, zone, position, None)
        .expect("blank menu");
    let talk = menu
        .options
        .iter()
        .find(|option| option.action == NpcAction::Talk)
        .expect("the blank NPC offers Talk")
        .id;
    assert!(matches!(
        player_a.select(&select(blank.npc_id, i32::MAX as u32), zone, position, None),
        Err(NpcInteractionError::InvalidOption)
    ));
    match player_a.select(&select(blank.npc_id, talk as u32), zone, position, None) {
        Ok(NpcSelection::Talk(context)) => assert_eq!(
            context.body_string_id, None,
            "the refused choice must not consume the menu or inject a body"
        ),
        _ => panic!("the refused choice must leave the menu usable"),
    }

    // The real router still encodes 26/3 with a zero body for the blank and the
    // row's own id for the bound row.
    let mut game = Gameplay::load(&gameplay_dir(), None).expect("gameplay content loads");
    assert_eq!(
        gameplay_talk_body(&mut game, blank.file, blank.npc_id),
        0,
        "a blank row must not inherit a neighbour's body"
    );
    assert_eq!(
        gameplay_talk_body(&mut game, target.file, target.npc_id),
        target.greeting,
        "the bound row keeps its own body"
    );
}
