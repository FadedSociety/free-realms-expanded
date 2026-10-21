//! T316: Merry Vale and Stillwater Crossing drawable NPC rows carry recovered
//! locale greetings.
//!
//! Content only: `NpcInteraction::dialogue_context` already copies
//! `greeting_string_id` into `DialogueContext.body_string_id`, and
//! `gameplay/npc.rs::dialogue_packet_with` writes that body into 26/3. The
//! clicked row's own id is the body, never the nearest locale line.
//!
//! Client evidence re-checked with IDA for T316:
//! - 26/3 handler `0x00aa12e0` calls payload reader `0x00a9ef10` and then
//!   display routine `0x009f5ae0` (ShowDialogFromPacket).
//! - Reader `0x00a9ef10` stores the first payload dword at object `+16`
//!   (`*(_DWORD *)(this + 16) = *v3`) and advances the cursor by 4.
//! - Display `0x009f5ae0` passes `*(_DWORD *)(a2 + 16)` to the `Global.Text`
//!   lookup `0x0097c1e0` immediately before `:SetDialogText`.
//! The body is therefore the first dword after the four-byte family/subtype
//! header, matching `encode_dialogue_26_3`'s `initial_dwords[0]`. The byte
//! layout is pinned by
//! `tests/npc_interaction.rs::dialogue_26_3_synthetic_fixture_round_trips_exact_native_field_order`.
//! Locale ids, hashes, dat lines and speaker proofs are in
//! `notes/Autopilot/T316.md`.
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
    collections::{BTreeSet, HashMap},
    fs,
    path::{Path, PathBuf},
    process::Command,
};

const GREETING_HEADER: [u8; 4] = [26, 0, 3, 0];
const MENU_HEADER: [u8; 4] = [26, 0, 9, 0];
const MAX_GREETINGS: usize = 24;

const WARRIOR_GUIDE: &str = "notes/Build Guides/Content/Jobs/Warrior.md";
const POSTMAN_GUIDE: &str = "notes/Build Guides/Content/Jobs/Postman.md";
const CHEF_GUIDE: &str = "notes/Build Guides/Content/Jobs/Chef.md";
const WIZARD_GUIDE: &str = "notes/Build Guides/Content/Jobs/Wizard.md";
const BRAWLER_GUIDE: &str = "notes/Build Guides/Content/Jobs/Brawler.md";
const FISHERMAN_GUIDE: &str = "notes/Build Guides/Content/Jobs/Fisherman.md";
const MERRY_VALE_GUIDE: &str = "notes/Build Guides/Content/NPCs/NPCs - Merry Vale.md";
const STILLWATER_GUIDE: &str = "notes/Build Guides/Content/NPCs/NPCs - Stillwater Crossing.md";

/// One edited row: content key, the `Global.Text` id the client looks up, its
/// JenkinsLookup2 hash, the physical stock-locale line, the speaker proof and
/// the 49/1 quest intro when the line is that quest's offer speech.
struct Greeting {
    npc_id: u64,
    file: &'static str,
    greeting: u32,
    hash: u32,
    dat_line: u32,
    proof_guide: &'static str,
    proof_line: u32,
    region_line: u32,
    quest: Option<(&'static str, u32)>,
}

const NEW_GREETINGS: [Greeting; 17] = [
    Greeting {
        npc_id: 9000000222,
        file: "merry_vale.json",
        greeting: 71035,
        hash: 2377109724,
        dat_line: 62495,
        proof_guide: WARRIOR_GUIDE,
        proof_line: 94,
        region_line: 45,
        quest: Some(("warrior.json", 3150008)),
    },
    Greeting {
        npc_id: 9000000030,
        file: "merry_vale.json",
        greeting: 71067,
        hash: 2242122283,
        dat_line: 59014,
        proof_guide: WARRIOR_GUIDE,
        proof_line: 94,
        region_line: 46,
        quest: None,
    },
    Greeting {
        npc_id: 9000000174,
        file: "merry_vale.json",
        greeting: 117035,
        hash: 1942512514,
        dat_line: 51179,
        proof_guide: POSTMAN_GUIDE,
        proof_line: 88,
        region_line: 43,
        quest: Some(("postman.json", 3130012)),
    },
    Greeting {
        npc_id: 9500000371,
        file: "merry_vale.json",
        greeting: 427481,
        hash: 13348600,
        dat_line: 337,
        proof_guide: MERRY_VALE_GUIDE,
        proof_line: 115,
        region_line: 115,
        quest: None,
    },
    Greeting {
        npc_id: 9000000044,
        file: "merry_vale.json",
        greeting: 71783,
        hash: 217741357,
        dat_line: 5717,
        proof_guide: CHEF_GUIDE,
        proof_line: 118,
        region_line: 41,
        quest: Some(("chef.json", 3050022)),
    },
    Greeting {
        npc_id: 9500000417,
        file: "merry_vale.json",
        greeting: 387081,
        hash: 2145785937,
        dat_line: 56498,
        proof_guide: MERRY_VALE_GUIDE,
        proof_line: 90,
        region_line: 90,
        quest: None,
    },
    Greeting {
        npc_id: 9500000420,
        file: "merry_vale.json",
        greeting: 41743,
        hash: 3010630538,
        dat_line: 79507,
        proof_guide: MERRY_VALE_GUIDE,
        proof_line: 72,
        region_line: 72,
        quest: None,
    },
    Greeting {
        npc_id: 9500000424,
        file: "merry_vale.json",
        greeting: 441732,
        hash: 3419032289,
        dat_line: 90415,
        proof_guide: MERRY_VALE_GUIDE,
        proof_line: 196,
        region_line: 100,
        quest: None,
    },
    Greeting {
        npc_id: 9500000427,
        file: "merry_vale.json",
        greeting: 141684,
        hash: 1290681492,
        dat_line: 34188,
        proof_guide: WIZARD_GUIDE,
        proof_line: 92,
        region_line: 44,
        quest: Some(("wizard.json", 3160014)),
    },
    Greeting {
        npc_id: 9500000400,
        file: "merry_vale.json",
        greeting: 73922,
        hash: 1124293448,
        dat_line: 29803,
        proof_guide: MERRY_VALE_GUIDE,
        proof_line: 32,
        region_line: 32,
        quest: None,
    },
    Greeting {
        npc_id: 9500000381,
        file: "merry_vale.json",
        greeting: 387043,
        hash: 2082414646,
        dat_line: 54875,
        proof_guide: MERRY_VALE_GUIDE,
        proof_line: 76,
        region_line: 76,
        quest: None,
    },
    Greeting {
        npc_id: 9000000038,
        file: "stillwater_crossing.json",
        greeting: 116793,
        hash: 1220502472,
        dat_line: 32325,
        proof_guide: STILLWATER_GUIDE,
        proof_line: 44,
        region_line: 44,
        quest: None,
    },
    Greeting {
        npc_id: 9500000785,
        file: "stillwater_crossing.json",
        greeting: 388502,
        hash: 3566246703,
        dat_line: 94453,
        proof_guide: BRAWLER_GUIDE,
        proof_line: 125,
        region_line: 49,
        quest: Some(("brawler.json", 3030022)),
    },
    Greeting {
        npc_id: 9000000196,
        file: "stillwater_crossing.json",
        greeting: 116508,
        hash: 1942927184,
        dat_line: 51192,
        proof_guide: BRAWLER_GUIDE,
        proof_line: 113,
        region_line: 50,
        quest: Some(("brawler.json", 3030017)),
    },
    Greeting {
        npc_id: 9000000103,
        file: "stillwater_crossing.json",
        greeting: 116777,
        hash: 3160199473,
        dat_line: 83418,
        proof_guide: STILLWATER_GUIDE,
        proof_line: 45,
        region_line: 45,
        quest: None,
    },
    Greeting {
        npc_id: 9000000096,
        file: "stillwater_crossing.json",
        greeting: 116496,
        hash: 724072415,
        dat_line: 19001,
        proof_guide: CHEF_GUIDE,
        proof_line: 103,
        region_line: 46,
        quest: Some(("chef.json", 3050017)),
    },
    Greeting {
        npc_id: 9000000181,
        file: "stillwater_crossing.json",
        greeting: 408272,
        hash: 2663118808,
        dat_line: 70112,
        proof_guide: FISHERMAN_GUIDE,
        proof_line: 32,
        region_line: 47,
        quest: None,
    },
];

/// The 94 bindings this slice must not disturb (W126 plus T246, T280, T298 and
/// T299). Greetings that a later slice added before this one integrates are
/// covered by the per-row HEAD comparison in the same test.
const EXISTING_GREETINGS: [(&str, u64, u32); 94] = [
    ("cobblestone_village.json", 9000000021, 4420092),
    ("cobblestone_village.json", 9000000120, 94293),
    ("cobblestone_village.json", 9000000121, 94297),
    ("cobblestone_village.json", 9000000189, 24940),
    ("cobblestone_village.json", 9500000245, 388245),
    ("cobblestone_village.json", 9500000254, 94388),
    ("farnum_farmstead.json", 9000000050, 5100125),
    ("farnum_farmstead.json", 9000000104, 391041),
    ("farnum_farmstead.json", 9000000165, 4420036),
    ("farnum_farmstead.json", 9000000396, 5100509),
    ("farnum_farmstead.json", 9500000266, 101014),
    ("farnum_farmstead.json", 9500000271, 391391),
    ("greenwood_forest.json", 9000000005, 413790),
    ("greenwood_forest.json", 9000000106, 383044),
    ("greenwood_forest.json", 9000000334, 47871),
    ("greenwood_forest.json", 9000000337, 77548),
    ("greenwood_forest.json", 9000000387, 22074),
    ("greenwood_forest.json", 9000000405, 94431),
    ("greenwood_forest.json", 9000000444, 389186),
    ("greenwood_forest.json", 9000000479, 94417),
    ("greenwood_forest.json", 9500000291, 24816),
    ("greenwood_forest.json", 9500000297, 429061),
    ("greenwood_forest.json", 9500000298, 429370),
    ("greenwood_forest.json", 9500000301, 429430),
    ("highroad_junction.json", 9000000015, 436533),
    ("highroad_junction.json", 9000000019, 431140),
    ("highroad_junction.json", 9000000102, 140509),
    ("highroad_junction.json", 9000000177, 104100),
    ("highroad_junction.json", 9000000402, 80),
    ("lakeshore.json", 9000000223, 24829),
    ("lakeshore.json", 9000000233, 409412),
    ("lakeshore.json", 9500000363, 41207),
    ("merry_vale.json", 9500000430, 41268),
    ("sanctuary.json", 9000000028, 387246),
    ("sanctuary.json", 9000000052, 73782),
    ("sanctuary.json", 9000000073, 76747),
    ("sanctuary.json", 9000000091, 24842),
    ("sanctuary.json", 9000000097, 91895),
    ("sanctuary.json", 9000000126, 76438),
    ("sanctuary.json", 9000000160, 101215),
    ("sanctuary.json", 9000000161, 73360),
    ("sanctuary.json", 9000000205, 71435),
    ("sanctuary.json", 9000000207, 46166),
    ("sanctuary.json", 9000000467, 94274),
    ("sanctuary.json", 9500000438, 37452),
    ("sanctuary.json", 9500000440, 22622),
    ("sanctuary.json", 9500000450, 441420),
    ("sanctuary.json", 9500000465, 74526),
    ("sanctuary.json", 9500000474, 21785),
    ("sanctuary.json", 9500000476, 390786),
    ("sanctuary.json", 9500000477, 21557),
    ("sanctuary.json", 9500000487, 37872),
    ("sanctuary.json", 9500000493, 415302),
    ("sanctuary.json", 9500000497, 15500),
    ("sanctuary.json", 9500000500, 103560),
    ("sanctuary.json", 9500000505, 70827),
    ("sanctuary.json", 9500000509, 46279),
    ("sanctuary.json", 9500000515, 415266),
    ("sanctuary.json", 9500000520, 428076),
    ("sanctuary.json", 9500000522, 37041),
    ("sanctuary.json", 9500000528, 21534),
    ("sanctuary.json", 9500000529, 74933),
    ("sanctuary.json", 9500000541, 21602),
    ("sanctuary.json", 9500000545, 76061),
    ("seaside.json", 9500000563, 441432),
    ("seaside.json", 9500000639, 441037),
    ("snowhill.json", 9000000033, 91267),
    ("snowhill.json", 9000000076, 46059),
    ("snowhill.json", 9000000093, 430860),
    ("snowhill.json", 9000000151, 78904),
    ("snowhill.json", 9000000155, 70892),
    ("snowhill.json", 9000000158, 5696),
    ("snowhill.json", 9000000206, 78478),
    ("snowhill.json", 9000000208, 387432),
    ("snowhill.json", 9000000220, 76208),
    ("snowhill.json", 9000000234, 74937),
    ("stillwater_crossing.json", 9500000786, 116463),
    ("sunstone_valley.json", 9500000824, 413082),
    ("wilds.json", 9000000243, 387488),
    ("wilds.json", 9000000273, 394709),
    ("wilds.json", 9000000317, 130251),
    ("wilds.json", 9000000324, 24947),
    ("wilds.json", 9000000332, 73077),
    ("wilds.json", 9000000335, 78924),
    ("wilds.json", 9000000347, 104327),
    ("wilds.json", 9000000415, 141009),
    ("wilds.json", 9000000477, 391043),
    ("wilds.json", 9000000488, 73492),
    ("wilds.json", 9500000869, 115443),
    ("wilds.json", 9500000876, 73378),
    ("wilds.json", 9500000893, 25001),
    ("wilds.json", 9500000907, 9599),
    ("wilds.json", 9500000954, 20984),
    ("wilds.json", 9500000958, 21014),
];

/// Drawable rows with no recovered first-person speaker line, one per owned
/// file. They stay blank and must not inherit another row's body.
const BLANK_MERRY_VALE: u64 = 9500000405;
const BLANK_STILLWATER: u64 = 9000000227;

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

/// The read-only stock client locale the server content cites.
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

fn assert_drawable(record: &Value, npc_id: u64) {
    assert!(
        record["position"].as_array().is_some_and(|position| {
            position.len() == 3 && position.iter().all(Value::is_number)
        }),
        "NPC {npc_id}: finite 3D position"
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

/// The stock client locale rows for the 17 target hashes, keyed by hash with
/// the one-based physical line and the client text.
fn locale_rows() -> HashMap<u32, (u32, String)> {
    let path = locale_path();
    let text =
        fs::read_to_string(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    let mut rows = HashMap::new();
    for (index, line) in text.lines().enumerate() {
        let mut fields = line.splitn(3, '\t');
        let (Some(key), Some(_table), Some(body)) = (fields.next(), fields.next(), fields.next())
        else {
            continue;
        };
        let Ok(hash) = key.trim_start_matches('\u{feff}').parse::<u32>() else {
            continue;
        };
        if NEW_GREETINGS.iter().any(|target| target.hash == hash) {
            rows.insert(hash, (index as u32 + 1, body.to_owned()));
        }
    }
    rows
}

fn region_guide(file: &str) -> &'static str {
    match file {
        "merry_vale.json" => MERRY_VALE_GUIDE,
        "stillwater_crossing.json" => STILLWATER_GUIDE,
        other => panic!("not an owned region file: {other}"),
    }
}

/// Repository root: the manifest directory is `prototype/osfr-tools`.
fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// HEAD bytes for a repository-relative path. The workspace is a git worktree,
/// so comparing against HEAD proves this diff did not touch a file the slice
/// does not own without pinning a hash that another slice's landing would stale.
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

/// Every edited row: one client source and one `(R)` policy source, a drawable
/// spawned row, and a locale id that resolves on its cited physical line.
/// `edited.len() <= 24` is the cap; the note lists the 37 drawable rows left
/// blank.
#[test]
fn every_edited_row_has_resolved_client_provenance() {
    assert!(
        NEW_GREETINGS.len() <= MAX_GREETINGS,
        "the slice binds at most {MAX_GREETINGS} rows"
    );
    let locale = locale_rows();
    let visibility =
        NpcVisibility::load(&content_dir()).expect("the production visibility loader accepts it");
    let mut greeting_ids = BTreeSet::new();
    for target in &NEW_GREETINGS {
        assert!(
            target.greeting > 0,
            "NPC {}: greeting must be positive",
            target.npc_id
        );
        assert!(
            greeting_ids.insert(target.greeting),
            "NPC {}: greeting {} reused",
            target.npc_id,
            target.greeting
        );

        let record = npc_record(target.file, target.npc_id);
        assert_eq!(
            record["greeting_string_id"].as_u64(),
            Some(u64::from(target.greeting)),
            "NPC {}: greeting binding",
            target.npc_id
        );
        assert_drawable(&record, target.npc_id);
        assert_eq!(
            visibility.content_key(target.npc_id),
            Some(target.npc_id),
            "NPC {} is not spawned by NpcVisibility",
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
        let reconstructed = sources
            .iter()
            .find(|source| source["kind"] == "reconstructed")
            .unwrap_or_else(|| panic!("NPC {}: reconstructed source", target.npc_id));
        assert!(
            reconstructed["evidence"].as_str().is_some_and(|evidence| {
                evidence.starts_with("(R)") && evidence.contains("fixed default Talk greeting")
            }),
            "NPC {}: the reconstruction must be labelled (R) fixed-default policy",
            target.npc_id
        );
        let evidence = client["evidence"].as_str().unwrap_or_default();
        for expected in [
            format!("Global.Text.{}", target.greeting),
            format!("JenkinsLookup2 hash {}", target.hash),
            format!("line {}", target.dat_line),
            format!("{} line {}", target.proof_guide, target.proof_line),
            format!("{} line {}", region_guide(target.file), target.region_line),
        ] {
            assert!(
                evidence.contains(&expected),
                "NPC {}: source must name {expected:?}",
                target.npc_id
            );
        }
        if let Some((quest_file, quest_id)) = target.quest {
            assert!(
                evidence.contains(&format!(
                    "data/content/quests/{quest_file} quest {quest_id}"
                )),
                "NPC {}: source must name its quest record",
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

    // The owned files add exactly these rows to the two W126 bindings.
    let expected = NEW_GREETINGS
        .iter()
        .map(|target| (target.file, target.npc_id, target.greeting))
        .chain(
            EXISTING_GREETINGS
                .iter()
                .filter(|(file, _, _)| {
                    *file == "merry_vale.json" || *file == "stillwater_crossing.json"
                })
                .map(|(file, npc_id, greeting)| (*file, *npc_id, *greeting)),
        )
        .collect::<BTreeSet<_>>();
    let mut found = BTreeSet::new();
    for file in ["merry_vale.json", "stillwater_crossing.json"] {
        for record in read_json(&content_dir().join("npcs").join(file))["records"]
            .as_array()
            .unwrap()
        {
            if let Some(greeting) = record["greeting_string_id"].as_u64().filter(|id| *id > 0) {
                found.insert((file, record["npc_id"].as_u64().unwrap(), greeting as u32));
            }
        }
    }
    assert_eq!(
        found, expected,
        "greeted rows changed outside the target set"
    );
}

#[test]
fn merry_vale_and_stillwater_rows_send_their_greeting_in_26_3() {
    // Service path: the context the router copies into the 26/3 body.
    let mut interaction = service();
    for target in &NEW_GREETINGS {
        assert_eq!(
            service_talk_body(&mut interaction, target.npc_id),
            Some(target.greeting),
            "NPC {}: service Talk body",
            target.npc_id
        );
    }

    // Dispatch path: gateway -> Gameplay -> gameplay/npc.rs -> dialogue_packet_with.
    let mut game = Gameplay::load(&gameplay_dir(), None).expect("gameplay content loads");
    for target in &NEW_GREETINGS {
        assert_eq!(
            gameplay_talk_body(&mut game, target.file, target.npc_id),
            target.greeting,
            "NPC {}: in-world 26/3 body",
            target.npc_id
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
    let blanks = [
        ("merry_vale.json", BLANK_MERRY_VALE),
        ("stillwater_crossing.json", BLANK_STILLWATER),
    ];
    for (file, npc_id) in blanks {
        let record = npc_record(file, npc_id);
        assert!(
            record.get("greeting_string_id").is_none()
                || record["greeting_string_id"].as_u64() == Some(0),
            "NPC {npc_id}: the blank control must stay blank"
        );
        assert_drawable(&record, npc_id);
        assert_eq!(
            service_talk_body(&mut interaction, npc_id),
            None,
            "NPC {npc_id}: an NPC without a greeting has no body"
        );

        // A refused choice must leave the menu usable (F271): select Talk
        // directly on the still-open menu, with no second `interact`, so this
        // fails if `select` consumes the open menu on `InvalidOption`.
        let (zone, position) = interaction.location(npc_id).unwrap();
        let menu = interaction.interact(npc_id, zone, position, None).unwrap();
        let talk = menu
            .options
            .iter()
            .find(|option| option.action == NpcAction::Talk)
            .expect("the blank NPC offers Talk")
            .id;
        assert!(matches!(
            interaction.select(&select_request(npc_id, i32::MAX), zone, position, None),
            Err(NpcInteractionError::InvalidOption)
        ));
        match interaction.select(&select_request(npc_id, talk), zone, position, None) {
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
    }
    assert!(
        NEW_GREETINGS.iter().all(|target| target.greeting != 0),
        "zero is not a recovered greeting id"
    );

    // The real router still encodes 26/3 with a zero body and no borrowed id.
    let mut game = Gameplay::load(&gameplay_dir(), None).expect("gameplay content loads");
    for (file, npc_id) in blanks {
        assert_eq!(
            gameplay_talk_body(&mut game, file, npc_id),
            0,
            "NPC {npc_id}: no invented locale id"
        );
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

    // In the two files this slice owns, every non-target record keeps its
    // top-level values: only the 17 edited rows may differ from HEAD.
    let edited: BTreeSet<u64> = NEW_GREETINGS.iter().map(|target| target.npc_id).collect();
    for file in ["merry_vale.json", "stillwater_crossing.json"] {
        let relative = format!("prototype/osfr-tools/data/content/npcs/{file}");
        let base: Value = serde_json::from_slice(&head_bytes(&relative))
            .unwrap_or_else(|error| panic!("{relative}: HEAD is not JSON: {error}"));
        let current = read_json(&content_dir().join("npcs").join(file));
        let base_rows = base["records"].as_array().expect("HEAD records");
        let current_rows = current["records"].as_array().expect("current records");
        assert_eq!(base_rows.len(), current_rows.len(), "{file}: row count");
        for (old, new) in base_rows.iter().zip(current_rows) {
            let npc_id = new["npc_id"].as_u64().expect("npc id");
            if !edited.contains(&npc_id) {
                assert_eq!(old, new, "{file}: NPC {npc_id} non-target row changed");
            }
        }
    }

    // Every region file this slice does not own is byte-identical to HEAD, so
    // the T246/T280/T298/T299 rows there cannot change. Comparing against HEAD
    // (not a hash pinned at authoring time) keeps the gate true when another
    // slice's work lands before this one integrates.
    let data = content_dir().join("npcs");
    let mut others = 0usize;
    for entry in fs::read_dir(&data).expect("npcs directory") {
        let path = entry.expect("npc entry").path();
        if path.extension().and_then(|extension| extension.to_str()) != Some("json") {
            continue;
        }
        let file = path.file_name().unwrap().to_string_lossy().into_owned();
        if file == "merry_vale.json" || file == "stillwater_crossing.json" {
            continue;
        }
        let relative = format!("prototype/osfr-tools/data/content/npcs/{file}");
        assert_eq!(
            fs::read(&path).expect("region file"),
            head_bytes(&relative),
            "{file}: an out-of-scope region file changed"
        );
        others += 1;
    }
    assert!(
        others >= 15,
        "expected at least fifteen untouched region files"
    );
}

#[test]
fn the_content_stays_within_the_loader_bounds() {
    let data = content_dir().join("npcs");
    let mut total = 0u64;
    for file in ["merry_vale.json", "stillwater_crossing.json"] {
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
        assert!(
            document["records"].as_array().unwrap().len() <= 20_000,
            "{file}: exceeds the 20,000-row loader bound"
        );
    }
    assert!(
        total <= 32 * 1024 * 1024,
        "the two edited files exceed the 32 MiB loader bound"
    );
    // The loader itself is the shape/source gate.
    let _ = service();

    for target in &NEW_GREETINGS {
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
