//! T317: Seaside and Briarwood drawable NPC rows carry recovered locale greetings.
//!
//! The body is the clicked actor's own content row `greeting_string_id`, not the nearest
//! locale line and not the last opened menu. The service path (`NpcInteraction::interact`
//! -> `select`) is the one `gameplay/npc.rs` calls; the dispatch path
//! (`Gameplay::handle_in_world`) drives the real router and `dialogue_packet_with`.
//!
//! Client evidence re-checked with IDA for T317: 26/3 handler `0x00aa12e0` calls reader
//! `0x00a9ef10`, which stores the first payload dword in the message object at `+16`
//! (`*(this + 16) = *v3`), and then calls display routine `0x009f5ae0`, which passes
//! `*(a2 + 16)` to the locale lookup `sub_97C1E0` immediately before `:SetDialogText`.
//! That dword is the first dword after the four-byte family/subtype header, matching
//! `encode_dialogue_26_3`'s `initial_dwords[0]`. The full 26/3 byte order is pinned by
//! `tests/npc_interaction.rs::dialogue_26_3_synthetic_fixture_round_trips_exact_native_field_order`.
//!
//! Locale ids, hashes, `en_us_data.dat` lines and speaker proofs are in
//! `notes/Autopilot/T317.md`; every unbound drawable row is listed there with its reason.
//! No production file changes: this is content plus this test. The packet builders, the
//! locale row convention and the 26/3 body offset live in the shared
//! `tests/common/npc_greetings.rs` harness.

#[path = "common/npc_greetings.rs"]
mod npc_greetings;

use npc_greetings::{
    content_dir, gameplay_dir, gameplay_talk_body, lab_report, locale_id_to_text, locale_rows,
    npc_position, npc_record, npc_records, read_json, select, service, service_talk_body,
};
use osfr_tools::{
    gameplay::Gameplay,
    npc_interaction::{NpcAction, NpcInteractionError, NpcSelection},
    world::NpcVisibility,
    world_text::locale_hash,
};
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

const SEASIDE: &str = "seaside.json";
const BRIARWOOD: &str = "briarwood.json";
const SEASIDE_GUIDE: &str = "notes/Build Guides/Content/NPCs/NPCs - Seaside.md";
const BRIARWOOD_GUIDE: &str = "notes/Build Guides/Content/NPCs/NPCs - Briarwood.md";
const MEDIC_GUIDE: &str = "notes/Build Guides/Content/Jobs/Medic.md";
const POSTMAN_GUIDE: &str = "notes/Build Guides/Content/Jobs/Postman.md";
const KART_GUIDE: &str = "notes/Build Guides/Content/Jobs/Kart Driver.md";
const BRAWLER_GUIDE: &str = "notes/Build Guides/Content/Jobs/Brawler.md";
const FISHERMAN_GUIDE: &str = "notes/Build Guides/Content/Jobs/Fisherman.md";
const ADVENTURER_GUIDE: &str = "notes/Build Guides/Content/Jobs/Adventurer.md";
const MINER_GUIDE: &str = "notes/Build Guides/Content/Jobs/Miner.md";

/// One edited row: the content key, bound `Global.Text` id, its JenkinsLookup2 key hash,
/// the physical stock-locale line, the guide row that proves the speaker (it must contain
/// the hash, or the line must self-identify), and the proof phrase the `client` source
/// must carry. `quest` is the server quest record whose 49/1 intro is this line, when the
/// line is a quest intro; the test asserts both the giver and the intro binding.
struct Greeting {
    npc_id: u64,
    file: &'static str,
    greeting: u32,
    hash: u32,
    dat_line: u32,
    guide: &'static str,
    guide_line: u32,
    quest: Option<(&'static str, u32)>,
    proof: &'static str,
}

const NEW_GREETINGS: [Greeting; 22] = [
    Greeting {
        npc_id: 9000000072,
        file: SEASIDE,
        greeting: 387576,
        hash: 4293886124,
        dat_line: 113302,
        guide: MEDIC_GUIDE,
        guide_line: 114,
        quest: Some(("medic.json", 3090027)),
        proof: "first-person Medic trainer quest offer",
    },
    Greeting {
        npc_id: 9000000080,
        file: SEASIDE,
        greeting: 387934,
        hash: 1553080327,
        dat_line: 41118,
        guide: POSTMAN_GUIDE,
        guide_line: 98,
        quest: Some(("postman.json", 3130017)),
        proof: "first-person Postman trainer quest offer",
    },
    Greeting {
        npc_id: 9500000598,
        file: SEASIDE,
        greeting: 5101019,
        hash: 250427778,
        dat_line: 6572,
        guide: SEASIDE_GUIDE,
        guide_line: 31,
        quest: None,
        proof: "first-person Seaside welcome",
    },
    Greeting {
        npc_id: 9500000629,
        file: SEASIDE,
        greeting: 93282,
        hash: 2185228639,
        dat_line: 57559,
        guide: SEASIDE_GUIDE,
        guide_line: 29,
        quest: None,
        proof: "first-person new-visitor greeting",
    },
    Greeting {
        npc_id: 9500000623,
        file: SEASIDE,
        greeting: 93409,
        hash: 1382851082,
        dat_line: 36681,
        guide: SEASIDE_GUIDE,
        guide_line: 32,
        quest: None,
        proof: "first-person innkeeper line",
    },
    Greeting {
        npc_id: 9500000591,
        file: SEASIDE,
        greeting: 427889,
        hash: 2642857805,
        dat_line: 69548,
        guide: SEASIDE_GUIDE,
        guide_line: 61,
        quest: None,
        proof: "first-person Pirate's Plunder registration offer",
    },
    Greeting {
        npc_id: 9500000627,
        file: SEASIDE,
        greeting: 101261,
        hash: 3493387114,
        dat_line: 92519,
        guide: SEASIDE_GUIDE,
        guide_line: 91,
        quest: None,
        proof: "first-person helmet request",
    },
    Greeting {
        npc_id: 9500000571,
        file: SEASIDE,
        greeting: 76268,
        hash: 3385240829,
        dat_line: 89537,
        guide: SEASIDE_GUIDE,
        guide_line: 107,
        quest: None,
        proof: "first-person pearl-collection offer",
    },
    Greeting {
        npc_id: 9500000550,
        file: SEASIDE,
        greeting: 101729,
        hash: 1666566655,
        dat_line: 44078,
        guide: SEASIDE_GUIDE,
        guide_line: 78,
        quest: None,
        proof: "first-person Beach Ball outfit request",
    },
    Greeting {
        npc_id: 9500000586,
        file: SEASIDE,
        greeting: 400812,
        hash: 1618656001,
        dat_line: 42849,
        guide: SEASIDE_GUIDE,
        guide_line: 56,
        quest: None,
        proof: "first-person Seaside Summer Splash secret-deck challenge",
    },
    Greeting {
        npc_id: 9500000559,
        file: SEASIDE,
        greeting: 406431,
        hash: 108726065,
        dat_line: 2823,
        guide: KART_GUIDE,
        guide_line: 142,
        quest: None,
        proof: "first-person Sandstorm Speedway welcome",
    },
    Greeting {
        npc_id: 9000000009,
        file: BRIARWOOD,
        greeting: 72532,
        hash: 1975092436,
        dat_line: 52030,
        guide: BRAWLER_GUIDE,
        guide_line: 133,
        quest: None,
        proof: "first-person wallet request quoted in the Brawler guide",
    },
    Greeting {
        npc_id: 9000000071,
        file: BRIARWOOD,
        greeting: 74072,
        hash: 1961466830,
        dat_line: 51685,
        guide: MEDIC_GUIDE,
        guide_line: 101,
        quest: Some(("medic.json", 3090019)),
        proof: "first-person Medic trainer quest offer",
    },
    Greeting {
        npc_id: 9000000308,
        file: BRIARWOOD,
        greeting: 412249,
        hash: 1500179458,
        dat_line: 39754,
        guide: FISHERMAN_GUIDE,
        guide_line: 111,
        quest: Some(("fisherman.json", 3070027)),
        proof: "first-person Fisherman quest offer",
    },
    Greeting {
        npc_id: 9000000268,
        file: BRIARWOOD,
        greeting: 93809,
        hash: 202408898,
        dat_line: 5344,
        guide: ADVENTURER_GUIDE,
        guide_line: 139,
        quest: None,
        proof: "first-person box hand-over line quoted in the Adventurer guide",
    },
    Greeting {
        npc_id: 9000000470,
        file: BRIARWOOD,
        greeting: 78312,
        hash: 2183725585,
        dat_line: 57531,
        guide: MINER_GUIDE,
        guide_line: 147,
        quest: Some(("miner.json", 3100028)),
        proof: "first-person Miner quest offer",
    },
    Greeting {
        npc_id: 9500000223,
        file: BRIARWOOD,
        greeting: 72039,
        hash: 1265014476,
        dat_line: 33509,
        guide: BRIARWOOD_GUIDE,
        guide_line: 61,
        quest: None,
        proof: "first-person fellow-Medic request",
    },
    Greeting {
        npc_id: 9500000089,
        file: BRIARWOOD,
        greeting: 72077,
        hash: 596647416,
        dat_line: 15796,
        guide: BRIARWOOD_GUIDE,
        guide_line: 60,
        quest: None,
        proof: "first-person brazier request",
    },
    Greeting {
        npc_id: 9500000094,
        file: BRIARWOOD,
        greeting: 72220,
        hash: 2622756725,
        dat_line: 69041,
        guide: BRIARWOOD_GUIDE,
        guide_line: 100,
        quest: None,
        proof: "first-person sapling-watering request",
    },
    Greeting {
        npc_id: 9500000111,
        file: BRIARWOOD,
        greeting: 72073,
        hash: 582349080,
        dat_line: 15443,
        guide: BRIARWOOD_GUIDE,
        guide_line: 79,
        quest: None,
        proof: "first-person courting-stone request",
    },
    Greeting {
        npc_id: 9500000133,
        file: BRIARWOOD,
        greeting: 139295,
        hash: 3857115367,
        dat_line: 101958,
        guide: BRIARWOOD_GUIDE,
        guide_line: 32,
        quest: None,
        proof: "first-person lost-friends line",
    },
    Greeting {
        npc_id: 9500000206,
        file: BRIARWOOD,
        greeting: 435562,
        hash: 647951782,
        dat_line: 17166,
        guide: BRIARWOOD_GUIDE,
        guide_line: 101,
        quest: None,
        proof: "self-identifying first-person line",
    },
];

/// The guide rows whose text proves the speaker without quoting the hash: the
/// bound line itself self-identifies (Reginald: "my name is Reginald").
const SELF_IDENTIFYING: [u64; 1] = [9500000206];

/// The bindings already in the two files before T317 (W126): Burglz and Troy Ridewrangler.
const PRE_EXISTING: [(u64, u32); 2] = [(9500000563, 441432), (9500000639, 441037)];

/// T379 (another slice) appended one binding-only repair-guy row to seaside.json;
/// its own suite pins that row's greeting and transform.
const APPENDED_BY_OTHER_SLICES: [u64; 1] = [9000001989];

/// Row counts: this slice edits rows, it never adds or removes one. T379
/// appended one binding-only row to seaside.json.
const ROW_COUNTS: [(&str, usize); 2] = [(SEASIDE, 109), (BRIARWOOD, 181)];

/// Alonzo: a drawable Seaside quest giver left blank because the guide only carries
/// third-person narration about him. A blank row must not inherit another row's body.
const BLANK: u64 = 9500000548;

/// Every NPC file this slice did not edit: region page name and region zone id.
/// The explicit list (not a directory count) fails if a check is lost when a
/// region is added, and it checks each file this slice must not have touched.
const OTHER_REGIONS: [(&str, &str, u64); 15] = [
    ("blackspore_swamp.json", "Blackspore Swamp", 1),
    ("cobblestone_village.json", "Cobblestone Village", 3),
    ("durango_zoo.json", "Durango Zoo", 4),
    ("farnum_farmstead.json", "Farnum Farmstead", 5),
    ("greenwood_forest.json", "Greenwood Forest", 6),
    ("highroad_junction.json", "Highroad Junction", 7),
    ("lakeshore.json", "Lakeshore", 8),
    ("merry_vale.json", "Merry Vale", 9),
    ("sanctuary.json", "Sanctuary", 10),
    ("shrouded_glade.json", "Shrouded Glade", 12),
    ("snowhill.json", "Snowhill", 13),
    ("stillwater_crossing.json", "Stillwater Crossing", 14),
    ("sunstone_valley.json", "Sunstone Valley", 15),
    ("wilds.json", "Wilds", 16),
    ("wugachug.json", "Wugachug", 17),
];

/// The brief's drawability rule: a spawned `legacy_data` row with a finite position,
/// positive model id, positive name id and a name. `NpcVisibility::load` spawns exactly
/// these rows; `poi_anchor` and `unanchored` rows are not bindable.
fn is_drawable(record: &Value) -> bool {
    record["placement"].as_str() == Some("legacy_data")
        && record["position"]
            .as_array()
            .is_some_and(|position| position.len() == 3)
        && record["model_id"].as_u64().is_some_and(|id| id > 0)
        && record["name_string_id"].as_u64().is_some_and(|id| id > 0)
        && record["name"].as_str().is_some_and(|name| !name.is_empty())
}

fn assert_drawable(record: &Value, npc_id: u64) {
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
    assert!(
        record["name"].as_str().is_some_and(|name| !name.is_empty()),
        "NPC {npc_id}: name"
    );
    let position = npc_position(record, npc_id);
    assert!(
        position.iter().all(|value| value.is_finite()),
        "NPC {npc_id}: finite position"
    );
}

/// The cited guide line. The client evidence pins its physical line; the speaker
/// check reads the same line, so a note-row edit that moves the proof fails here.
fn guide_line(guide: &str, line: u32) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(guide);
    let text =
        fs::read_to_string(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    text.lines()
        .nth(line as usize - 1)
        .unwrap_or_else(|| panic!("{}: no line {line}", path.display()))
        .to_string()
}

/// The server quest record a greeting is the 49/1 intro of.
fn quest_record(file: &str, quest_id: u32) -> Value {
    let path = content_dir().join("quests").join(file);
    read_json(&path)["records"]
        .as_array()
        .unwrap_or_else(|| panic!("{}: records array", path.display()))
        .iter()
        .find(|record| record["id"].as_u64() == Some(u64::from(quest_id)))
        .unwrap_or_else(|| panic!("{}: quest {quest_id}", path.display()))
        .clone()
}

/// The speaker check: the cited guide line must name the NPC and either quote the
/// bound hash or (for a self-identifying line) the locale text must say the name.
/// Binding another speaker's valid locale line fails both.
fn speaker_proof(row: &Greeting, name: &str, text: &str) -> Result<(), String> {
    let line = guide_line(row.guide, row.guide_line);
    if !line.contains(name) {
        return Err(format!(
            "{} line {} does not name {name}",
            row.guide, row.guide_line
        ));
    }
    if line.contains(&row.hash.to_string()) {
        return Ok(());
    }
    if SELF_IDENTIFYING.contains(&row.npc_id)
        && text
            .to_lowercase()
            .contains(&format!("my name is {}", name.to_lowercase()))
    {
        return Ok(());
    }
    Err(format!(
        "{} line {} does not quote hash {} and the resolved text does not self-identify {name}",
        row.guide, row.guide_line, row.hash
    ))
}

fn locale_text(hash: u32) -> String {
    locale_rows(&BTreeSet::from([hash]))
        .remove(&hash)
        .unwrap_or_else(|| panic!("hash {hash} is not a stock client locale key"))
        .1
}

/// Every unbound drawable row must be listed with its reason in the note.
fn listed_unbound_ids() -> BTreeSet<u64> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../notes/Autopilot/T317.md");
    let text =
        fs::read_to_string(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    let marker = "## Unbound drawable rows";
    let after = text
        .split_once(marker)
        .unwrap_or_else(|| panic!("{}: missing {marker:?}", path.display()))
        .1;
    let section = after
        .split_once("\n## ")
        .map_or(after, |(section, _)| section);
    let mut ids = BTreeSet::new();
    for line in section.lines() {
        let Some(rest) = line.strip_prefix("- ") else {
            continue;
        };
        let token = rest.split_whitespace().next().unwrap_or_default();
        let Ok(npc_id) = token.parse::<u64>() else {
            continue;
        };
        assert!(
            rest.contains(" — "),
            "NPC {npc_id}: unbound row is listed without a reason"
        );
        assert!(ids.insert(npc_id), "NPC {npc_id}: listed twice as unbound");
    }
    ids
}

#[test]
fn every_edited_row_has_resolved_client_provenance() {
    assert!(
        NEW_GREETINGS.len() <= 24,
        "the slice binds at most 24 rows, not {}",
        NEW_GREETINGS.len()
    );
    let hash_to_ids = read_json(&lab_report("locale-id-map.json"))["hash_to_ids"].clone();
    let id_to_text = locale_id_to_text();
    let wanted: BTreeSet<u32> = NEW_GREETINGS.iter().map(|row| row.hash).collect();
    let locale = locale_rows(&wanted);
    let mut greeting_ids = BTreeSet::new();
    let mut edited = BTreeSet::new();
    for row in &NEW_GREETINGS {
        assert!(row.greeting > 0, "NPC {}: greeting", row.npc_id);
        assert!(
            greeting_ids.insert(row.greeting),
            "NPC {}: greeting {} reused by another speaker",
            row.npc_id,
            row.greeting
        );
        assert!(
            edited.insert(row.npc_id),
            "NPC {}: edited twice",
            row.npc_id
        );
        let record = npc_record(row.file, row.npc_id);
        assert_eq!(
            record["greeting_string_id"].as_u64(),
            Some(u64::from(row.greeting)),
            "NPC {}: greeting binding",
            row.npc_id
        );
        assert_drawable(&record, row.npc_id);

        let sources = record["field_sources"]["greeting_string_id"]
            .as_array()
            .unwrap_or_else(|| panic!("NPC {}: greeting sources", row.npc_id));
        assert_eq!(
            sources.len(),
            2,
            "NPC {}: exactly one client and one (R) source",
            row.npc_id
        );
        assert_eq!(
            sources[0]["kind"].as_str(),
            Some("client"),
            "NPC {}: first source is the client locale",
            row.npc_id
        );
        assert_eq!(
            sources[1]["kind"].as_str(),
            Some("reconstructed"),
            "NPC {}: second source is the fixed-default policy",
            row.npc_id
        );
        let evidence = sources[0]["evidence"].as_str().unwrap_or_default();
        for needle in [
            format!("Global.Text.{}", row.greeting),
            format!("JenkinsLookup2 hash {}", row.hash),
            format!("en_us_data.dat line {}", row.dat_line),
            row.guide.to_string(),
            format!("line {}", row.guide_line),
            row.proof.to_string(),
        ] {
            assert!(
                evidence.contains(&needle),
                "NPC {}: client source must name {needle:?}",
                row.npc_id
            );
        }
        assert!(
            !evidence.contains("Global.Text.0"),
            "NPC {}: a raw hash is not a locale id",
            row.npc_id
        );
        if let Some((file, quest_id)) = row.quest {
            // A quest intro is reconstructed T-1 text, so the test also pins the
            // giver and intro binding and the job guide that quotes the same line.
            assert!(
                evidence.contains(&format!("data/content/quests/{file}"))
                    && evidence.contains(&format!("quest {quest_id}")),
                "NPC {}: client source must name its quest record",
                row.npc_id
            );
            let quest = quest_record(file, quest_id);
            assert_eq!(
                quest["giver_npc_id"].as_u64(),
                Some(row.npc_id),
                "NPC {}: quest {quest_id} in {file} must be giver-owned",
                row.npc_id
            );
            assert_eq!(
                quest["intro_text_string_id"].as_u64(),
                Some(u64::from(row.greeting)),
                "NPC {}: quest {quest_id} in {file} must use the greeting as its intro",
                row.npc_id
            );
        }
        let policy = sources[1]["evidence"].as_str().unwrap_or_default();
        assert!(
            policy.starts_with("(R)"),
            "NPC {}: reconstructed evidence must start with (R)",
            row.npc_id
        );
        assert!(
            policy.contains("fixed default Talk greeting"),
            "NPC {}: fixed-default policy",
            row.npc_id
        );

        // Resolve three ways: hash_to_ids, the id-to-text report, and the physical line.
        assert_eq!(
            locale_hash(i32::try_from(row.greeting).expect("stock locale id fits i32")),
            row.hash,
            "NPC {}: greeting id must hash to the cited Global.Text key",
            row.npc_id
        );
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
        let name = record["name"].as_str().expect("NPC name");
        if let Err(error) = speaker_proof(row, name, text) {
            panic!("NPC {}: {error}", row.npc_id);
        }
    }

    // Reachability: the production visibility loader spawns every edited row at its
    // content position (35/2 actors are keyed by npc id for regional legacy_data rows).
    let visibility = NpcVisibility::load(&content_dir()).expect("NPC visibility loads");
    let spawned: BTreeMap<u64, [f32; 3]> = visibility
        .teleport_targets(0)
        .into_iter()
        .map(|target| (target.guid, target.position))
        .collect();
    for row in &NEW_GREETINGS {
        let record = npc_record(row.file, row.npc_id);
        assert_eq!(
            spawned.get(&row.npc_id),
            Some(&npc_position(&record, row.npc_id)),
            "NPC {}: NpcVisibility must spawn the row at its content position",
            row.npc_id
        );
    }

    // Every unbound drawable row is listed with a reason, and no bound row is.
    let mut unbound = BTreeSet::new();
    for (file, _) in ROW_COUNTS {
        for record in npc_records(file) {
            let npc_id = record["npc_id"].as_u64().expect("npc id");
            let already_greeted = record["greeting_string_id"].as_u64().unwrap_or(0) > 0;
            if is_drawable(&record) && !already_greeted && !edited.contains(&npc_id) {
                unbound.insert(npc_id);
            }
        }
    }
    let listed = listed_unbound_ids();
    assert_eq!(
        listed.len(),
        unbound.len(),
        "the note must list every unbound drawable row exactly once"
    );
    assert_eq!(listed, unbound, "the note's unbound list is stale");
}

#[test]
fn seaside_and_briarwood_rows_send_their_greeting_in_26_3() {
    // Service path: the context the router copies into the 26/3 body.
    let mut interaction = service();
    for row in &NEW_GREETINGS {
        assert_eq!(
            service_talk_body(&mut interaction, row.npc_id),
            Some(row.greeting),
            "NPC {}: service Talk body",
            row.npc_id
        );
    }

    // Dispatch path: gateway -> Gameplay -> gameplay/npc.rs -> dialogue_packet_with.
    let mut game = Gameplay::load(&gameplay_dir(), None).expect("gameplay content loads");
    for row in &NEW_GREETINGS {
        assert_eq!(
            gameplay_talk_body(&mut game, row.file, row.npc_id),
            row.greeting,
            "NPC {}: in-world 26/3 body",
            row.npc_id
        );
    }

    // A reconnect rebuilds the same static content; the body is not session state.
    let first = &NEW_GREETINGS[0];
    assert_eq!(
        service_talk_body(&mut service(), first.npc_id),
        Some(first.greeting),
        "NPC {}: a fresh loader sends the same body",
        first.npc_id
    );
}

#[test]
fn a_missing_greeting_falls_back_to_no_body() {
    // `dialogue_context` copies the Option, so a literal 0 would be Some(0) and
    // render blank at the wire; the content never stores 0. The missing case is
    // the one the router can distinguish, so this test is named for it.
    let mut interaction = service();
    let blank = npc_record(SEASIDE, BLANK);
    assert_drawable(&blank, BLANK);
    assert!(
        blank.get("greeting_string_id").is_none(),
        "the fallback control row must stay unbound"
    );
    assert_eq!(
        service_talk_body(&mut interaction, BLANK),
        None,
        "an NPC without a greeting has no body"
    );
    assert!(
        NEW_GREETINGS.iter().all(|row| row.greeting != 0),
        "zero is not a recovered greeting id"
    );

    // A refused choice must change nothing: select Talk directly on the still-open menu
    // (no second `interact`), so this fails if `select` consumes the menu on a refusal.
    let (zone, position) = interaction.location(BLANK).unwrap();
    let menu = interaction.interact(BLANK, zone, position, None).unwrap();
    let talk = menu
        .options
        .iter()
        .find(|option| option.action == NpcAction::Talk)
        .expect("the blank NPC offers Talk");
    assert!(matches!(
        interaction.select(&select(BLANK, i32::MAX as u32), zone, position, None),
        Err(NpcInteractionError::InvalidOption)
    ));
    match interaction.select(&select(BLANK, talk.id as u32), zone, position, None) {
        Ok(NpcSelection::Talk(context)) => assert_eq!(
            context.body_string_id, None,
            "the refused choice must not consume the menu or inject a body"
        ),
        _ => panic!("the refused choice must leave the menu usable"),
    }
    // An unknown NPC id resolves to no row at all; a client cannot choose a body.
    assert!(matches!(
        interaction.interact(123, zone, position, None),
        Err(NpcInteractionError::UnknownNpc)
    ));

    // The real router still encodes 26/3 with a zero body; no bound body is a name id.
    let mut game = Gameplay::load(&gameplay_dir(), None).expect("gameplay content loads");
    assert_eq!(
        gameplay_talk_body(&mut game, SEASIDE, BLANK),
        0,
        "no invented locale id"
    );
    for row in &NEW_GREETINGS {
        assert_ne!(
            u64::from(row.greeting),
            npc_record(row.file, row.npc_id)["name_string_id"]
                .as_u64()
                .expect("name id"),
            "NPC {}: a name id is not a dialogue body",
            row.npc_id
        );
    }
}

#[test]
fn the_existing_greetings_and_other_regions_are_unchanged() {
    for (npc_id, greeting) in PRE_EXISTING {
        assert_eq!(
            npc_record(SEASIDE, npc_id)["greeting_string_id"].as_u64(),
            Some(u64::from(greeting)),
            "existing W126 binding {npc_id} changed"
        );
    }

    let edited: BTreeSet<u64> = NEW_GREETINGS.iter().map(|row| row.npc_id).collect();
    for (file, expected_rows) in ROW_COUNTS {
        let rows = npc_records(file);
        assert_eq!(rows.len(), expected_rows, "{file}: row count");
        for record in &rows {
            let npc_id = record["npc_id"].as_u64().expect("npc id");
            if edited.contains(&npc_id) || APPENDED_BY_OTHER_SLICES.contains(&npc_id) {
                continue;
            }
            match PRE_EXISTING.iter().find(|(id, _)| *id == npc_id) {
                Some((_, greeting)) => assert_eq!(
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
                !record.to_string().contains("T317"),
                "NPC {npc_id}: T317 evidence outside the target rows"
            );
        }
    }

    // Every other region file keeps its top-level identity and carries no T317
    // evidence: the T246/T280/T298/T299/T315/T316 rows live there.
    for (file, region_page, region_zone_id) in OTHER_REGIONS {
        let document = read_json(&content_dir().join("npcs").join(file));
        assert_eq!(
            document["schema_version"].as_u64(),
            Some(1),
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
            !document.to_string().contains("T317"),
            "{file}: T317 edited a region file outside this slice"
        );
    }
}

#[test]
fn another_npcs_valid_locale_line_is_rejected() {
    // The T317 blocker: Andrew was bound to Steelbloom's 95156, a valid line
    // for a different speaker. Briarwood.md:48 names Andrew but does not quote
    // the line, and the text is not Andrew's.
    let wrong = Greeting {
        npc_id: 9000000009,
        file: BRIARWOOD,
        greeting: 95156,
        hash: 3961426188,
        dat_line: 104776,
        guide: BRIARWOOD_GUIDE,
        guide_line: 48,
        quest: None,
        proof: "first-person Brawler quest offer",
    };
    let error = speaker_proof(&wrong, "Andrew", &locale_text(wrong.hash))
        .expect_err("Steelbloom's line is not Andrew's");
    assert!(error.contains("does not quote"), "{error}");

    // The corrected binding is accepted: the Brawler guide quotes hash 1975092436.
    let right = NEW_GREETINGS
        .iter()
        .find(|row| row.npc_id == 9000000009)
        .expect("Andrew is bound");
    assert_eq!(
        speaker_proof(right, "Andrew", &locale_text(right.hash)),
        Ok(())
    );
}

#[test]
fn the_content_stays_within_the_loader_bounds() {
    let data = content_dir().join("npcs");
    let mut total = 0u64;
    for (file, expected_rows) in ROW_COUNTS {
        let path = data.join(file);
        let bytes = fs::read(&path).unwrap();
        let size = bytes.len() as u64;
        assert!(
            size <= 8 * 1024 * 1024,
            "{file}: exceeds the 8 MiB per-file loader bound"
        );
        total += size;
        std::str::from_utf8(&bytes).unwrap_or_else(|error| panic!("{file}: {error}"));
        let rows = read_json(&path)["records"]
            .as_array()
            .unwrap_or_else(|| panic!("{file}: records array"))
            .len();
        assert_eq!(rows, expected_rows, "{file}: row count");
        assert!(
            rows <= 20_000,
            "{file}: exceeds the 20,000-row loader bound"
        );
    }
    assert!(
        total <= 32 * 1024 * 1024,
        "the two edited files exceed the 32 MiB loader bound"
    );
    // The production loader is the shape gate for the added rows and sources.
    let _ = service();

    for row in &NEW_GREETINGS {
        let record = npc_record(row.file, row.npc_id);
        let sources = record["field_sources"]["greeting_string_id"]
            .as_array()
            .unwrap_or_else(|| panic!("NPC {}: greeting sources", row.npc_id));
        for source in sources {
            let kind = source["kind"].as_str().unwrap_or_default();
            assert!(
                matches!(kind, "client" | "reconstructed"),
                "NPC {}: unexpected source kind {kind:?}",
                row.npc_id
            );
            let evidence = source["evidence"].as_str().unwrap_or_default();
            assert!(
                !evidence.trim().is_empty(),
                "NPC {}: empty source evidence",
                row.npc_id
            );
            assert!(
                !evidence.chars().any(char::is_control),
                "NPC {}: control character in source",
                row.npc_id
            );
            assert!(
                evidence.len() <= 4096,
                "NPC {}: source evidence too long",
                row.npc_id
            );
            if kind == "reconstructed" {
                assert!(
                    evidence.starts_with("(R)"),
                    "NPC {}: (R) sources must start with (R)",
                    row.npc_id
                );
            }
        }
    }
}
