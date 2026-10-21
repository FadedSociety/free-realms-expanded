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
//! No production file changes: this is content plus this test.

use osfr_tools::{
    character_store::CharacterState,
    gameplay::Gameplay,
    npc_interaction::{NpcAction, NpcInteraction, NpcInteractionError, NpcSelection},
    world::NpcVisibility,
};
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    process::Command,
};

const GREETING_HEADER: [u8; 4] = [26, 0, 3, 0];
const MENU_HEADER: [u8; 4] = [26, 0, 9, 0];

const SEASIDE: &str = "seaside.json";
const BRIARWOOD: &str = "briarwood.json";
const SEASIDE_GUIDE: &str = "notes/Build Guides/Content/NPCs/NPCs - Seaside.md";
const BRIARWOOD_GUIDE: &str = "notes/Build Guides/Content/NPCs/NPCs - Briarwood.md";

/// One edited row: the content key, bound `Global.Text` id, its JenkinsLookup2 key hash,
/// the physical stock-locale line, the guide row that proves the speaker, and the proof
/// phrase the `client` source must carry.
struct Greeting {
    npc_id: u64,
    file: &'static str,
    greeting: u32,
    hash: u32,
    dat_line: u32,
    guide: &'static str,
    guide_line: u32,
    proof: &'static str,
}

const NEW_GREETINGS: [Greeting; 24] = [
    Greeting {
        npc_id: 9000000072,
        file: SEASIDE,
        greeting: 387576,
        hash: 4293886124,
        dat_line: 113302,
        guide: SEASIDE_GUIDE,
        guide_line: 51,
        proof: "first-person Medic trainer quest offer",
    },
    Greeting {
        npc_id: 9000000080,
        file: SEASIDE,
        greeting: 387934,
        hash: 1553080327,
        dat_line: 41118,
        guide: SEASIDE_GUIDE,
        guide_line: 53,
        proof: "first-person Postman trainer quest offer",
    },
    Greeting {
        npc_id: 9000000112,
        file: SEASIDE,
        greeting: 104270,
        hash: 3098965505,
        dat_line: 81814,
        guide: SEASIDE_GUIDE,
        guide_line: 55,
        proof: "first-person Card Duelist quest offer",
    },
    Greeting {
        npc_id: 9500000598,
        file: SEASIDE,
        greeting: 5101019,
        hash: 250427778,
        dat_line: 6572,
        guide: SEASIDE_GUIDE,
        guide_line: 31,
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
        proof: "first-person Seaside Summer Splash secret-deck challenge",
    },
    Greeting {
        npc_id: 9500000559,
        file: SEASIDE,
        greeting: 406431,
        hash: 108726065,
        dat_line: 2823,
        guide: SEASIDE_GUIDE,
        guide_line: 113,
        proof: "first-person Sandstorm Speedway welcome",
    },
    Greeting {
        npc_id: 9000000009,
        file: BRIARWOOD,
        greeting: 95156,
        hash: 3961426188,
        dat_line: 104776,
        guide: BRIARWOOD_GUIDE,
        guide_line: 48,
        proof: "first-person Brawler quest offer",
    },
    Greeting {
        npc_id: 9000000071,
        file: BRIARWOOD,
        greeting: 74072,
        hash: 1961466830,
        dat_line: 51685,
        guide: BRIARWOOD_GUIDE,
        guide_line: 70,
        proof: "first-person Medic trainer quest offer",
    },
    Greeting {
        npc_id: 9000000082,
        file: BRIARWOOD,
        greeting: 407023,
        hash: 2823732375,
        dat_line: 74435,
        guide: BRIARWOOD_GUIDE,
        guide_line: 117,
        proof: "first-person Card Duelist quest offer",
    },
    Greeting {
        npc_id: 9000000308,
        file: BRIARWOOD,
        greeting: 412249,
        hash: 1500179458,
        dat_line: 39754,
        guide: BRIARWOOD_GUIDE,
        guide_line: 162,
        proof: "first-person Fisherman quest offer",
    },
    Greeting {
        npc_id: 9000000268,
        file: BRIARWOOD,
        greeting: 93894,
        hash: 2733008456,
        dat_line: 71999,
        guide: BRIARWOOD_GUIDE,
        guide_line: 189,
        proof: "first-person Adventurer quest offer",
    },
    Greeting {
        npc_id: 9000000470,
        file: BRIARWOOD,
        greeting: 78312,
        hash: 2183725585,
        dat_line: 57531,
        guide: BRIARWOOD_GUIDE,
        guide_line: 59,
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
        proof: "self-identifying first-person line",
    },
];

/// The bindings already in the two files before T317 (W126): Burglz and Troy Ridewrangler.
const PRE_EXISTING: [(u64, u32); 2] = [(9500000563, 441432), (9500000639, 441037)];

/// Row counts: this slice edits rows, it never adds or removes one.
const ROW_COUNTS: [(&str, usize); 2] = [(SEASIDE, 108), (BRIARWOOD, 181)];

/// Alonzo: a drawable Seaside quest giver left blank because the guide only carries
/// third-person narration about him. A blank row must not inherit another row's body.
const BLANK: u64 = 9500000548;

fn content_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content")
}

/// Full server content used by the existing in-world router tests. It has no `npcs/`
/// directory, so `Gameplay::load` takes the NPC rows from the bundled `data/content`
/// (the same rows this test edits).
fn gameplay_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../lab/references/opening-anvil-placement-v1/content")
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Generated reports used by the T246/T280/T298 resolution method (outside the
/// repository, like the stock client locale the test also reads).
fn lab_report(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../lab/reports")
        .join(name)
}

/// The stock 2014 client locale. `hash_to_ids` was built from it; this test re-derives
/// each key instead of trusting an export.
fn locale_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../lab/working/OSFR.Client/OSFR Client/locale/en_us_data.dat")
}

fn read_json(path: &Path) -> Value {
    serde_json::from_slice(
        &fs::read(path).unwrap_or_else(|error| panic!("{}: {error}", path.display())),
    )
    .unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

fn records(file: &str) -> Vec<Value> {
    let path = content_dir().join("npcs").join(file);
    read_json(&path)["records"]
        .as_array()
        .unwrap_or_else(|| panic!("{}: records array", path.display()))
        .clone()
}

fn npc_record(file: &str, npc_id: u64) -> Value {
    let path = content_dir().join("npcs").join(file);
    read_json(&path)["records"]
        .as_array()
        .unwrap_or_else(|| panic!("{}: records array", path.display()))
        .iter()
        .find(|record| record["npc_id"].as_u64() == Some(npc_id))
        .unwrap_or_else(|| panic!("{}: NPC {npc_id}", path.display()))
        .clone()
}

fn position(record: &Value, npc_id: u64) -> [f32; 3] {
    let position = record["position"]
        .as_array()
        .unwrap_or_else(|| panic!("NPC {npc_id}: position"));
    assert_eq!(position.len(), 3, "NPC {npc_id}: position shape");
    assert!(
        position
            .iter()
            .all(|value| value.as_f64().is_some_and(f64::is_finite)),
        "NPC {npc_id}: finite position"
    );
    [
        position[0].as_f64().expect("x") as f32,
        position[1].as_f64().expect("y") as f32,
        position[2].as_f64().expect("z") as f32,
    ]
}

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
    let _ = position(record, npc_id);
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

/// Real in-world Talk: 26/8 opens the menu, 26/10 selects Talk, and the 26/3 reply is
/// decoded from the production packet.
fn gameplay_talk_body(game: &mut Gameplay, file: &str, npc_id: u64) -> u32 {
    let record = npc_record(file, npc_id);
    let character = character_at(position(&record, npc_id));
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
    // Body dword follows the four-byte family/subtype header; patch_npc_actor writes
    // the runtime actor GUID only into bytes 12..20.
    u32::from_le_bytes(dialogue[4..8].try_into().unwrap())
}

/// Physical stock-locale rows for the wanted key hashes: line number (1-based) and text.
fn locale_rows(wanted: &BTreeSet<u32>) -> BTreeMap<u32, (usize, String)> {
    let path = locale_path();
    let text =
        fs::read_to_string(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    let mut rows = BTreeMap::new();
    for (index, line) in text.lines().enumerate() {
        let mut parts = line.splitn(3, '\t');
        let (Some(key), Some(_kind), Some(body)) = (parts.next(), parts.next(), parts.next())
        else {
            continue;
        };
        let Ok(hash) = key.parse::<u32>() else {
            continue;
        };
        if wanted.contains(&hash) {
            rows.insert(hash, (index + 1, body.to_string()));
        }
    }
    rows
}

/// HEAD bytes for a repository-relative path. The autopilot workspace is a git worktree,
/// so this proves the diff did not touch a file the slice does not own.
fn head_bytes(relative: &str) -> Vec<u8> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_root())
        .args(["show", &format!("HEAD:{relative}")])
        .output()
        .unwrap_or_else(|error| panic!("git is required for the unchanged-files gate: {error}"));
    assert!(
        output.status.success(),
        "git show HEAD:{relative} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    output.stdout
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
    let id_to_text: BTreeMap<u64, String> = read_json(&lab_report("locale-id-to-text.json"))
        .as_array()
        .expect("locale-id-to-text rows")
        .iter()
        .filter_map(|row| Some((row["id"].as_u64()?, row["text"].as_str()?.to_string())))
        .collect();
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
            Some(&position(&record, row.npc_id)),
            "NPC {}: NpcVisibility must spawn the row at its content position",
            row.npc_id
        );
    }

    // Every unbound drawable row is listed with a reason, and no bound row is.
    let mut unbound = BTreeSet::new();
    for (file, _) in ROW_COUNTS {
        for record in records(file) {
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
fn a_zero_or_missing_greeting_falls_back_to_no_body() {
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
        interaction.select(&select_request(BLANK, i32::MAX), zone, position, None),
        Err(NpcInteractionError::InvalidOption)
    ));
    match interaction.select(&select_request(BLANK, talk.id), zone, position, None) {
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
    let name_ids: BTreeSet<u64> = NEW_GREETINGS
        .iter()
        .map(|row| {
            npc_record(row.file, row.npc_id)["name_string_id"]
                .as_u64()
                .expect("name id")
        })
        .collect();
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
    assert_eq!(name_ids.len(), NEW_GREETINGS.len());
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
        let rows = records(file);
        assert_eq!(rows.len(), expected_rows, "{file}: row count");
        for record in &rows {
            let npc_id = record["npc_id"].as_u64().expect("npc id");
            if edited.contains(&npc_id) {
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

    // The two edited files differ from HEAD only on the target rows, so every
    // non-target row keeps its top-level values. (No-op once the diff is committed.)
    for (file, _) in ROW_COUNTS {
        let relative = format!("prototype/osfr-tools/data/content/npcs/{file}");
        let base: Value = serde_json::from_slice(&head_bytes(&relative))
            .unwrap_or_else(|error| panic!("{relative}: HEAD is not JSON: {error}"));
        let base_records = base["records"].as_array().expect("HEAD records");
        let current = records(file);
        assert_eq!(base_records.len(), current.len(), "{file}: HEAD row count");
        for (old, new) in base_records.iter().zip(&current) {
            let npc_id = new["npc_id"].as_u64().expect("npc id");
            if !edited.contains(&npc_id) {
                assert_eq!(old, new, "NPC {npc_id}: non-target row changed");
            }
        }
    }

    // Every other region file is byte-identical to HEAD: the T246/T280/T298/T299 rows
    // live there, so a change to any of them fails here.
    let npc_dir = content_dir().join("npcs");
    let mut others = 0usize;
    for entry in fs::read_dir(&npc_dir).expect("npcs directory") {
        let path = entry.expect("npc entry").path();
        if path.extension().and_then(|extension| extension.to_str()) != Some("json") {
            continue;
        }
        let file = path.file_name().unwrap().to_string_lossy();
        if file == SEASIDE || file == BRIARWOOD {
            continue;
        }
        let relative = format!("prototype/osfr-tools/data/content/npcs/{file}");
        assert_eq!(
            fs::read(&path).expect("region file"),
            head_bytes(&relative),
            "{file}: another region's NPC file changed"
        );
        others += 1;
    }
    assert_eq!(others, 15, "expected fifteen untouched region files");
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
