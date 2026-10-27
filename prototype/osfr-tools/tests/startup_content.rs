//! T807: the gateway runs from repo data alone. `data/content/startup/` replaces the captured
//! reference folder (46 startup templates from a community server): `manifest.json` (client
//! version, initial character), `self_data.json` (typed defaults of the self-data message) and
//! `snapshot.json` (the typed startup snapshot). The files are scrubbed of the capturing server's
//! own text and of the capture character's names, fresh-character state and journal.
//!
//! Lab use, with the private capture folder (the typed lab snapshot beside it is read too):
//!   OSFR_GATEWAY_REFERENCE=<capture folder> cargo test --test startup_content -- --ignored
//! `export_scrubbed_startup_content` rewrites the three files and `tests/fixtures/startup_golden.json`;
//! `content_startup_equals_the_captured_startup_except_the_scrub` compares both gateways.
use osfr_tools::{
    character_store::CharacterState,
    gateway::Gateway,
    startup_selfdata::{self, SelfData},
    startup_snapshot::StartupSnapshot,
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
};

#[path = "common/gateway_packets.rs"]
mod gateway_packets;
#[path = "common/gateway_reference.rs"]
mod gateway_reference;
#[path = "support/json.rs"]
mod json;
#[path = "support/npc_visibility.rs"]
mod npc_visibility;
use gateway_packets::{login_with_version, wrap};
use json::read as read_json;
use npc_visibility::content_dir as content;

fn golden_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/startup_golden.json")
}

/// What the capturing server wrote into its own startup text; none of it may survive in content
/// or in a built message. The sample character's own name and pets are gone structurally
/// (`self_data_content_is_typed_json_that_round_trips`), so their names are not listed here.
const FORBIDDEN: [&str; 3] = ["osfr", "discord", "open source"];

/// (R) The self data's neutral birthday; the capture account's own numbers are never kept,
/// not even here as a forbidden list.
const NEUTRAL_BIRTHDAY: u64 = 1_240_963_200;

/// The fresh-character, scrubbed form of the captured self-data template. Gameplay overwrites
/// the inventory, equipment, quests and collected flags of a live login anyway
/// (`Gateway::build_self_message`); a gateway without gameplay sends this fresh character.
fn scrub_self(data: &mut SelfData) {
    // (R) Neutral default name; an account-store login replaces it with the character's own.
    data.name.first = "New".into();
    data.name.last = "Player".into();
    // (R) Stand-ins that carry nothing of the capture account: 2009-04-29 00:00 UTC, and 0 for the
    // unknown identity (the client reader `00944fe0` stores it unchecked).
    data.account.birthday = NEUTRAL_BIRTHDAY;
    data.identity.unknown_identity_u64 = 0;
    // The capturing server's tag.
    for row in &mut data.post_quest_records.keyed_strings {
        row.value_bytes.clear();
    }
    // (R) None of the sample character's progress: its second quest list (101 records), known
    // recipes, pet rows and active effect. Gameplay writes none of these at login.
    data.quest_records.second.clear();
    data.post_quest_records.recipes.clear();
    data.post_quest_records.variable_records.clear();
    data.effects.clear();
    // Nothing owned or worn (the slot rows stay), nothing collected, an empty quest journal.
    data.inventory.items.clear();
    for worn in &mut data.inventory.equipment {
        worn.item_id = 0;
    }
    for row in data.collections.iter_mut().flat_map(|c| &mut c.keyed_rows) {
        row.payload[28] = 0;
    }
    data.quests = osfr_tools::quests::QuestState {
        version: 1,
        quests: Default::default(),
        trial_best_ms: None,
        trial_completions: 0,
    };
    data.quest_bindings.clear();
    data.quest_records.first.clear();
    data.quest_records.first_tail[..9].copy_from_slice(&[0, 0, 0, 0, 255, 255, 255, 255, 0]);
}

/// (R) Our own welcome text replaces the capturing server's; its Halloween greeter loses the
/// server name. Every other value is the capture's, unchanged.
fn scrub_snapshot(snapshot: &mut StartupSnapshot) {
    snapshot.world.motd.first_string = "Welcome to Free Realms Expanded!".into();
    snapshot.world.motd.second_string =
        "This is a community-run server. Things may change or break while it is being built."
            .into();
    for npc in &mut snapshot.world.npc_spawns {
        npc.npc_name = npc.npc_name.replace(" OSFR", "");
    }
}

fn write_json(path: &Path, value: &impl serde::Serialize) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    let mut text = serde_json::to_vec_pretty(value).unwrap();
    text.push(b'\n');
    fs::write(path, text).unwrap();
}

/// A session up to the client's ready and loaded signals: the 46 startup messages, in order.
fn startup_of(gateway: &mut Gateway, state: &CharacterState, version: &str) -> Vec<Vec<u8>> {
    let mut messages = gateway
        .handle(&login_with_version(state.guid, version))
        .unwrap()
        .replies;
    messages.extend(gateway.handle(&wrap(&[13, 0])).unwrap().replies);
    assert!(gateway.handle(&wrap(&[10, 0])).unwrap().entered_world);
    assert!(gateway.is_world_loaded());
    messages
}

fn content_version() -> String {
    read_json(&content().join("startup/manifest.json"))["client_version"]
        .as_str()
        .unwrap()
        .to_owned()
}

fn describe(messages: &[Vec<u8>]) -> Value {
    let rows: Vec<Value> = messages
        .iter()
        .enumerate()
        .map(|(index, message)| {
            json!({
                "index": index,
                "opcode": u16::from_le_bytes(message[7..9].try_into().unwrap()),
                "length": message.len(),
                "sha256": format!("{:x}", Sha256::digest(message)),
            })
        })
        .collect();
    json!({
        "schema_version": 1,
        "note": "Digests of the 46 startup messages a gateway built from data/content/startup sends \
                 to the initial character. Where the content is unscrubbed they equal the capture's \
                 template digests (see the ignored comparison test); refresh with the exporter.",
        "messages": rows,
    })
}

#[test]
fn a_gateway_without_a_reference_folder_reaches_world_ready_with_the_golden_startup() {
    let content = content();
    let state = Gateway::content_state(&content).unwrap();
    let mut gateway = Gateway::load_from_content(&content, state.clone()).unwrap();
    let messages = startup_of(&mut gateway, &state, &content_version());
    assert_eq!(messages.len(), 46);
    assert_eq!(describe(&messages), read_json(&golden_path()));
    let opcodes: Vec<u64> = describe(&messages)["messages"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| row["opcode"].as_u64().unwrap())
        .collect();
    assert_eq!(opcodes[..8], [41, 2, 166, 43, 144, 35, 193, 12]);
    assert_eq!((opcodes[8], opcodes[45]), (67, 87));
}

#[test]
fn content_startup_is_the_same_as_a_reference_run_with_that_snapshot() {
    // The synthetic reference stands in for the capture folder; only message 7 (its template is
    // synthetic) differs. Every other message comes from the typed snapshot in both modes.
    let content = content();
    let state = Gateway::content_state(&content).unwrap();
    let version = content_version();
    let mut from_content = Gateway::load_from_content(&content, state.clone()).unwrap();
    let reference = gateway_reference::Reference::new(&version, None);
    let snapshot = StartupSnapshot::load(&content.join("startup/snapshot.json")).unwrap();
    let mut from_reference =
        Gateway::load_with_startup_snapshot(&reference.directory, state.clone(), snapshot).unwrap();
    let want = startup_of(&mut from_content, &state, &version);
    let got = startup_of(&mut from_reference, &state, &version);
    for (index, (want, got)) in want.iter().zip(&got).enumerate() {
        if index != 7 {
            assert_eq!(want, got, "startup message {index:02}");
        }
    }
}

#[test]
fn startup_content_has_no_capture_server_text_or_names() {
    let directory = content().join("startup");
    for file in ["self_data.json", "snapshot.json"] {
        let text = fs::read_to_string(directory.join(file))
            .unwrap()
            .to_lowercase();
        for word in FORBIDDEN {
            assert!(!text.contains(word), "{file} contains {word:?}");
        }
    }
    // The account numbers are the neutral stand-ins; the golden digest of message 7 pins the wire.
    let data: SelfData =
        serde_json::from_str(&fs::read_to_string(directory.join("self_data.json")).unwrap())
            .unwrap();
    assert_eq!(data.account.birthday, NEUTRAL_BIRTHDAY);
    assert_eq!(data.identity.unknown_identity_u64, 0);
    // Text inside hex spans only shows once built.
    let state = Gateway::content_state(&content()).unwrap();
    let mut gateway = Gateway::load_from_content(&content(), state.clone()).unwrap();
    for (index, message) in startup_of(&mut gateway, &state, &content_version())
        .iter()
        .enumerate()
    {
        let lower = message.to_ascii_lowercase();
        for word in FORBIDDEN {
            assert!(
                !lower.windows(word.len()).any(|w| w == word.as_bytes()),
                "startup message {index:02} contains {word:?}"
            );
        }
    }
    let manifest = fs::read_to_string(directory.join("manifest.json")).unwrap();
    assert!(
        !manifest.contains(":\\") && !manifest.contains("/Users/"),
        "home path in manifest"
    );
}

#[test]
fn unusable_startup_content_is_refused_not_panicked() {
    let root = std::env::temp_dir().join(format!("osfr-startup-content-{}", std::process::id()));
    let directory = root.join("startup");
    fs::create_dir_all(&directory).unwrap();
    let state = Gateway::content_state(&content()).unwrap();
    // Nothing there yet.
    assert!(Gateway::content_state(&root).is_err());
    assert!(Gateway::load_from_content(&root, state.clone()).is_err());
    let source = content().join("startup");
    for file in ["manifest.json", "self_data.json", "snapshot.json"] {
        fs::copy(source.join(file), directory.join(file)).unwrap();
    }
    assert!(Gateway::load_from_content(&root, state.clone()).is_ok());
    // A truncated self-data file, a wrong schema, a hex span of the wrong length.
    let self_data = fs::read_to_string(directory.join("self_data.json")).unwrap();
    fs::write(
        directory.join("self_data.json"),
        &self_data[..self_data.len() / 2],
    )
    .unwrap();
    assert!(Gateway::load_from_content(&root, state.clone()).is_err());
    let mut data = read_json(&source.join("self_data.json"));
    data["post_effect_tail"] = json!("0");
    write_json(&directory.join("self_data.json"), &data);
    assert!(Gateway::load_from_content(&root, state.clone()).is_err());
    fs::copy(
        source.join("self_data.json"),
        directory.join("self_data.json"),
    )
    .unwrap();
    let mut manifest = read_json(&source.join("manifest.json"));
    manifest["schema_version"] = json!(2);
    write_json(&directory.join("manifest.json"), &manifest);
    assert!(Gateway::load_from_content(&root, state.clone()).is_err());
    // An initial zone the snapshot does not belong to, and no snapshot at all.
    manifest["schema_version"] = json!(1);
    manifest["initial_state"]["zone_id"] = json!(5);
    write_json(&directory.join("manifest.json"), &manifest);
    let mut elsewhere = state.clone();
    elsewhere.zone_id = 5;
    let error = Gateway::load_from_content(&root, elsewhere).err().unwrap();
    assert!(
        error.to_string().contains("another initial zone"),
        "{error}"
    );
    fs::copy(
        source.join("manifest.json"),
        directory.join("manifest.json"),
    )
    .unwrap();
    fs::remove_file(directory.join("snapshot.json")).unwrap();
    assert!(Gateway::load_from_content(&root, state).is_err());
    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn self_data_content_is_typed_json_that_round_trips() {
    let text = fs::read_to_string(content().join("startup/self_data.json")).unwrap();
    let data: SelfData = serde_json::from_str(&text).unwrap();
    assert_eq!(
        serde_json::to_value(&data).unwrap(),
        json!(serde_json::from_str::<Value>(&text).unwrap())
    );
    assert_eq!(data.name.first, "New");
    // A fresh character: nothing owned, no journal entries or other quest records, no recipes,
    // pets or effects, no collected flag.
    assert!(data.inventory.items.is_empty() && data.quest_records.first.is_empty());
    assert!(data.quest_records.second.is_empty() && data.effects.is_empty());
    let post = &data.post_quest_records;
    assert!(post.recipes.is_empty() && post.variable_records.is_empty());
    assert!(
        data.collections
            .iter()
            .flat_map(|c| &c.keyed_rows)
            .all(|row| row.payload[28] == 0)
    );
    // The wire form keeps its tunnel header and opcode.
    let wire = startup_selfdata::build_self(&data);
    assert_eq!(&wire[..3], [5, 0, 1]);
    assert_eq!(u16::from_le_bytes(wire[7..9].try_into().unwrap()), 12);
}

#[test]
#[ignore = "lab use: needs OSFR_GATEWAY_REFERENCE (the private capture) and the typed lab snapshot"]
fn export_scrubbed_startup_content() {
    let (reference, lab_snapshot) = lab_inputs();
    let state = Gateway::reference_state(&reference).unwrap();
    let version = read_json(&reference.join("manifest.json"))["client_version"].clone();
    let directory = content().join("startup");
    write_json(
        &directory.join("manifest.json"),
        &json!({
            "schema_version": 1,
            "client_version": version,
            "initial_state": state,
            "provenance": [
                "Typed fields of the 46 startup messages an original 1.910.1.530630 client received in a session against a community server on 2026-10-08. The capture stays in the lab and is not shipped.",
                "self_data.json: message 7 (opcode 12) parsed by startup_selfdata::parse_self, then reduced to a fresh character by scrub_self in tests/startup_content.rs.",
                "snapshot.json: the typed startup snapshot of the other 45 messages, with the two texts of scrub_snapshot replaced.",
                "(R) The motd text, the default name 'New Player' and the 'Happy Halloween!' nameplate are ours, not the capture's.",
                "(R) account.birthday (1240963200, 2009-04-29 00:00 UTC) and identity.unknown_identity_u64 (0) are neutral stand-ins; the capture account's own numbers are not kept. The client reader (00944fe0) stores the identity u64 without checking it.",
                "(R) Unnamed spans stay as hex, as startup_selfdata and startup_snapshot read them; nothing here is replayed bytes.",
                "Refresh: OSFR_GATEWAY_REFERENCE=<capture folder> cargo test --test startup_content -- --ignored export_scrubbed_startup_content"
            ],
        }),
    );
    let template = fs::read(reference.join("0007-login.bin")).unwrap();
    let mut data = startup_selfdata::parse_self(&template);
    scrub_self(&mut data);
    write_json(&directory.join("self_data.json"), &data);
    let mut snapshot = StartupSnapshot::load(&lab_snapshot).unwrap();
    scrub_snapshot(&mut snapshot);
    write_json(&directory.join("snapshot.json"), &snapshot);

    let mut gateway = Gateway::load_from_content(&content(), state.clone()).unwrap();
    let messages = startup_of(&mut gateway, &state, version.as_str().unwrap());
    write_json(&golden_path(), &describe(&messages));
}

/// The capture folder and the typed lab snapshot beside it.
fn lab_inputs() -> (PathBuf, PathBuf) {
    let reference = PathBuf::from(
        std::env::var("OSFR_GATEWAY_REFERENCE").expect("OSFR_GATEWAY_REFERENCE is required"),
    );
    let lab = reference
        .parent()
        .and_then(Path::parent)
        .unwrap()
        .to_owned();
    (reference, lab.join("content/startup-snapshot/startup.json"))
}

#[test]
#[ignore = "lab use: needs OSFR_GATEWAY_REFERENCE (the private capture) and the typed lab snapshot"]
fn content_startup_equals_the_captured_startup_except_the_scrub() {
    let (reference, lab_snapshot) = lab_inputs();
    let state = Gateway::reference_state(&reference).unwrap();
    let version = content_version();
    let mut scrubbed = StartupSnapshot::load(&lab_snapshot).unwrap();
    scrub_snapshot(&mut scrubbed);

    let mut captured =
        Gateway::load_with_startup_snapshot(&reference, state.clone(), scrubbed).unwrap();
    let mut from_content = Gateway::load_from_content(&content(), state.clone()).unwrap();
    let want = startup_of(&mut captured, &state, &version);
    let got = startup_of(&mut from_content, &state, &version);
    assert_eq!((want.len(), got.len()), (46, 46));
    let normalized = |message: &[u8]| {
        let mut data = startup_selfdata::parse_self(message);
        scrub_self(&mut data);
        data
    };
    for (index, (want, got)) in want.iter().zip(&got).enumerate() {
        if index == 7 {
            assert_eq!(
                normalized(want),
                normalized(got),
                "self-data differs beyond the scrub"
            );
            assert_eq!(*got, startup_selfdata::build_self(&normalized(want)));
        } else {
            assert_eq!(want, got, "startup message {index:02}");
        }
    }

    // Unscrubbed messages carry the capture's own digests. Messages 6 and 7 are built
    // differently from the capture on purpose (T437 closeWindow, fresh character), and
    // 12 and 45 hold the two scrubbed texts.
    let manifest = read_json(&reference.join("manifest.json"));
    let templates = manifest["templates"].as_array().unwrap();
    for (index, message) in got.iter().enumerate() {
        if ![6, 7, 12, 45].contains(&index) {
            assert_eq!(
                format!("{:x}", Sha256::digest(message)),
                templates[index]["sha256"].as_str().unwrap(),
                "startup message {index:02} differs from the capture"
            );
        }
    }
}
