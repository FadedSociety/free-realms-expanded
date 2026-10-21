//! Shared harness for the NPC greeting suites (`tests/npc_greetings_*.rs`).
//!
//! Include with `#[path = "common/npc_greetings.rs"] mod npc_greetings;`.
//! Test-only; the production path is `NpcInteraction::dialogue_context` ->
//! `gameplay/npc.rs::dialogue_packet_with`.
//!
//! Client shapes: 26/8 `CommandPacketInteractRequest` constructor `008b6bc0`
//! (u64 target + bool) and 26/10 `CommandPacketInteractionSelect` constructor
//! `00a36f80` (u64 target + i32 choice) are the shared `gateway_packets`
//! builders. `sub_A9CB50` reads the 26/3 header as two u16 (family 26, subtype
//! 3, four bytes), then `00a9ef10` (`sub_A9EF10`) stores the first payload dword
//! at object `+16`; display `009f5ae0` passes that dword to the
//! `Global.Text.<id>` lookup. The body id is therefore the first dword after
//! the four-byte family/subtype header, and `patch_npc_actor` writes the runtime
//! actor GUID only into bytes 12..20.
//!
//! Locale line convention: `str::lines()` (T246; every `\n` terminates a line),
//! so the lone-LF records in `en_us_data.dat` count as lines. T298/T299 used a
//! `split("\r\n")` index, which differs after a lone LF; every suite uses this
//! helper's convention.

#[path = "gateway_packets.rs"]
mod gateway_packets;

pub use gateway_packets::{interact, select};

use osfr_tools::{
    character_store::CharacterState,
    gameplay::Gameplay,
    npc_interaction::{NpcAction, NpcInteraction, NpcSelection},
};
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

pub const GREETING_HEADER: [u8; 4] = [26, 0, 3, 0];
pub const MENU_HEADER: [u8; 4] = [26, 0, 9, 0];

pub fn content_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content")
}

/// Full server content used by the existing in-world router tests. It has no
/// `npcs/` directory, so `Gameplay::load` takes the NPC rows from the bundled
/// `data/content` (the same rows the suites edit).
pub fn gameplay_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../lab/references/opening-anvil-placement-v1/content")
}

/// Stock 2014 client locale. `hash_to_ids` in `lab/reports/locale-id-map.json`
/// was built from it; the suites re-derive each key instead of trusting it.
pub fn locale_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../lab/working/OSFR.Client/OSFR Client/locale/en_us_data.dat")
}

/// A generated lab report used by the T246/T280 resolution method (outside the
/// repository, like the client locale read above).
pub fn lab_report(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../lab/reports")
        .join(name)
}

pub fn read_json(path: &Path) -> Value {
    serde_json::from_slice(
        &fs::read(path).unwrap_or_else(|error| panic!("{}: {error}", path.display())),
    )
    .unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

pub fn npc_records(file: &str) -> Vec<Value> {
    let path = content_dir().join("npcs").join(file);
    read_json(&path)["records"]
        .as_array()
        .unwrap_or_else(|| panic!("{}: records array", path.display()))
        .clone()
}

pub fn npc_record(file: &str, npc_id: u64) -> Value {
    npc_records(file)
        .into_iter()
        .find(|record| record["npc_id"].as_u64() == Some(npc_id))
        .unwrap_or_else(|| panic!("{file}: NPC {npc_id}"))
}

pub fn npc_position(record: &Value, npc_id: u64) -> [f32; 3] {
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

pub fn service() -> NpcInteraction {
    let data = content_dir();
    NpcInteraction::load(&data.join("npcs"), &data.join("stores.json"))
        .expect("NPC content loads through the real loader")
}

/// Real Talk selection through the service the router calls.
pub fn service_talk_body(service: &mut NpcInteraction, npc_id: u64) -> Option<u32> {
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
    let choice = u32::try_from(talk.id).expect("menu option id fits u32");
    let selected = service
        .select(&select(npc_id, choice), zone, position, None)
        .unwrap_or_else(|error| panic!("NPC {npc_id} Talk: {error:?}"));
    match selected {
        NpcSelection::Talk(context) => context.body_string_id,
        _ => panic!("NPC {npc_id} Talk must return dialogue"),
    }
}

pub fn character_at(position: [f32; 3]) -> CharacterState {
    CharacterState {
        guid: 101,
        zone_id: 0,
        position,
        rotation: [1.0, 0.0, 0.0],
    }
}

/// Real in-world Talk: 26/8 opens the menu, 26/10 selects Talk, and the 26/3
/// reply is decoded from the production packet.
pub fn gameplay_talk_body(game: &mut Gameplay, file: &str, npc_id: u64) -> u32 {
    let character = character_at(npc_position(&npc_record(file, npc_id), npc_id));
    let menu = game
        .handle_in_world(&interact(npc_id), &character, 1_000)
        .unwrap_or_else(|error| panic!("NPC {npc_id} interact: {error}"));
    assert!(
        menu.replies
            .iter()
            .any(|packet| packet.starts_with(&MENU_HEADER)),
        "NPC {npc_id} must answer 26/8 with a 26/9 menu"
    );
    let outcome = game
        .handle_in_world(&select(npc_id, 1), &character, 1_100)
        .unwrap_or_else(|error| panic!("NPC {npc_id} Talk: {error}"));
    let dialogue = outcome
        .replies
        .into_iter()
        .find(|packet| packet.starts_with(&GREETING_HEADER))
        .unwrap_or_else(|| panic!("NPC {npc_id} Talk must answer 26/3"));
    u32::from_le_bytes(dialogue[4..8].try_into().unwrap())
}

/// Physical stock-locale rows for the wanted key hashes: one-based line and
/// text, under the T246 `str::lines()` convention documented above.
pub fn locale_rows(wanted: &BTreeSet<u32>) -> BTreeMap<u32, (usize, String)> {
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

pub fn locale_id_to_text() -> BTreeMap<u64, String> {
    read_json(&lab_report("locale-id-to-text.json"))
        .as_array()
        .expect("locale-id-to-text rows")
        .iter()
        .filter_map(|row| Some((row["id"].as_u64()?, row["text"].as_str()?.to_owned())))
        .collect()
}

/// Every `(file, npc_id)` -> `greeting_string_id` binding in `npcs/*.json`, so
/// a new slice can prove it does not reuse another row's greeting id.
pub fn greeting_bindings() -> BTreeMap<(String, u64), u32> {
    let dir = content_dir().join("npcs");
    let mut files: Vec<PathBuf> = fs::read_dir(&dir)
        .unwrap_or_else(|error| panic!("{}: {error}", dir.display()))
        .map(|entry| entry.expect("NPC content entry").path())
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "json")
        })
        .collect();
    files.sort();
    let mut bindings = BTreeMap::new();
    for path in files {
        let file = path
            .file_name()
            .and_then(|name| name.to_str())
            .expect("NPC file name")
            .to_owned();
        let document = read_json(&path);
        let records = document["records"]
            .as_array()
            .unwrap_or_else(|| panic!("{}: records array", path.display()));
        for record in records {
            let Some(greeting) = record["greeting_string_id"].as_u64() else {
                continue;
            };
            let npc_id = record["npc_id"]
                .as_u64()
                .unwrap_or_else(|| panic!("{}: greeting row without npc_id", path.display()));
            let greeting = u32::try_from(greeting)
                .unwrap_or_else(|_| panic!("{file} NPC {npc_id}: greeting id fits u32"));
            bindings.insert((file.clone(), npc_id), greeting);
        }
    }
    bindings
}
