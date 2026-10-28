//! T853: the publishable gameplay overlay in `data/release/`.
//!
//! The released gateway runs on `data/content`; the opening, combat, quests, profiles,
//! inventory and the cooking/forging/smelting campaigns only existed in private lab packages.
//! This suite pins the overlay that fills the gap:
//!
//! * `the_release_overlay_reaches_world_entry_with_the_opening_active` merges `data/content`
//!   and `data/release` into a scratch folder and drives the real gateway through login, ready
//!   and world entry with the opening module loaded.
//! * `the_release_overlay_is_publishable` checks the scrub and the per-file provenance rules.
//! * `a_broken_overlay_is_refused_without_touching_the_files` pins that a bad campaign marker or
//!   category proof fails the load and changes nothing on disk.
//! * `export_release_content` (ignored) regenerates the overlay from the private lab package and
//!   the packet capture; it is the only place a lab path is read.
//!
//! The overlay carries no binary: the two capture-derived templates are typed JSON hex fields
//! (`combat.json` `template_hex`, `quests.json` journal `hex`) built by the same loaders, as
//! T807 typed the captured self data.
use osfr_tools::{
    character_store::CharacterState,
    combat::Combat,
    gateway::Gateway,
    inventory::{Inventory, InventoryItem},
    item_categories::ItemCategories,
    opening_journal::OpeningJournal,
    opening_world::OpeningWorld,
    profiles::Profiles,
    quests::QuestEngine,
    startup_selfdata::{self, SelfData},
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
};

#[path = "common/content_copy.rs"]
mod content_copy;
#[path = "common/gateway_packets.rs"]
mod gateway_packets;
#[path = "support/json.rs"]
mod json;
#[path = "support/npc_visibility.rs"]
mod npc_visibility;
use gateway_packets::{login_with_version, wrap};
use json::read as read_json;
use npc_visibility::content_dir as content;

/// The 44/2 reference-data payload captured from the original game server: 71 categories,
/// 74 group associations, 2,675 bytes. `categories.json` rebuilds exactly these bytes and
/// `ItemCategories::load` re-checks the digest.
const CATEGORY_PACKET_SHA256: &str =
    "a75f5e2beb3be02c76434145e0cf62b6dc7f98438e48791bd1f569a531f0c21f";

/// The captured AddNpc skeleton embedded as `combat.json` `template_hex`; 334 bytes with the
/// name, aliases and effects zeroed. The digest pins the exact vetted bytes.
const COMBAT_TEMPLATE_SHA256: &str =
    "602c66b1a4ea9fac97eaec2d64c380bee17c33ff86e0aa9fe0b10c1da3bbfc19";

/// The files the release overlay owns; everything else stays in `data/content`.
const RELEASE_FILES: [&str; 11] = [
    "opening.json",
    "combat.json",
    "profiles.json",
    "quests.json",
    "categories.json",
    "inventory.json",
    "cooking.json",
    "forging.json",
    "smelting.json",
    "cooking-campaign.json",
    "forging-campaign.json",
];

fn crate_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

fn release_dir() -> PathBuf {
    crate_dir().join("data/release")
}

fn sha(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn write_json(path: &Path, value: &Value) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    let mut bytes = serde_json::to_vec_pretty(value).unwrap();
    bytes.push(b'\n');
    fs::write(path, bytes).unwrap();
}

/// The private lab native package; only the ignored exporter reads it.
fn source_dir() -> PathBuf {
    std::env::var_os("OSFR_RELEASE_SOURCE")
        .map(PathBuf::from)
        .unwrap_or_else(|| crate_dir().join("../../../lab/reports/native-20261027T2300Z/content"))
}

/// The pinned packet capture that carries the 44/2 reference data. The default is the lab
/// reference; `OSFR_CATEGORY_CAPTURE` overrides it.
fn capture_path(source: &Path) -> PathBuf {
    std::env::var_os("OSFR_CATEGORY_CAPTURE")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            source
                .parent()
                .unwrap()
                .parent()
                .unwrap()
                .parent()
                .unwrap()
                .join("references/OpenSourceFreeRealms/ofrserver/Packets/p1.pcap")
        })
}

/// A scratch `data/content` + `data/release` merge. The overlay wins.
fn merged_content(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("osfr-release-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    content_copy::copy_tree(&content(), &root);
    content_copy::copy_tree(&release_dir(), &root);
    root
}

fn startup_of(gateway: &mut Gateway, state: &CharacterState, version: &str) -> Vec<Vec<u8>> {
    let mut messages = gateway
        .handle(&login_with_version(state.guid, version))
        .unwrap()
        .replies;
    messages.extend(gateway.handle(&wrap(&[13, 0])).unwrap().replies);
    let world = gateway.handle(&wrap(&[10, 0])).unwrap();
    assert!(world.entered_world);
    assert!(gateway.is_world_loaded());
    messages.extend(world.replies);
    messages
}

fn client_version() -> String {
    read_json(&content().join("startup/manifest.json"))["client_version"]
        .as_str()
        .unwrap()
        .to_owned()
}

/// The bytes of the fresh character's journal span in the typed self data: an empty first
/// list followed by the scrubbed 17-byte tail.
fn fresh_journal(self_data: &SelfData) -> Vec<u8> {
    assert!(self_data.quest_records.first.is_empty());
    let mut bytes = 0u32.to_le_bytes().to_vec();
    bytes.extend(self_data.quest_records.first_tail);
    bytes
}

fn equipment_span(state: &osfr_tools::inventory::InventoryState, profile: i32) -> Vec<u8> {
    let rows: Vec<_> = state
        .equipment
        .iter()
        .filter(|row| row.profile_id == profile)
        .collect();
    let mut bytes = (rows.len() as i32).to_le_bytes().to_vec();
    for row in rows {
        for value in [row.slot, row.item_id, row.category] {
            bytes.extend(value.to_le_bytes());
        }
    }
    bytes
}

fn find_unique(bytes: &[u8], needle: &[u8]) -> usize {
    assert!(!needle.is_empty());
    let mut positions = (0..=bytes.len().saturating_sub(needle.len()))
        .filter(|index| bytes[*index..*index + needle.len()] == *needle);
    let first = positions
        .next()
        .expect("the derived span is in the typed self");
    assert!(positions.next().is_none(), "the derived span is not unique");
    first
}

/// Offsets and proof bytes for the fresh inventory and journal, all taken from the actual
/// serialized self message so the overlay and `data/content/startup/self_data.json` agree.
struct Derived {
    coins_offset: usize,
    items_offset: usize,
    journal_offset: usize,
    journal: Vec<u8>,
    equipment: Vec<(i32, usize, Vec<u8>)>,
}

fn derive(self_data: &SelfData, bytes: &[u8], source: &Path) -> Derived {
    let mut probe = self_data.clone();
    probe.inventory.coins = probe.inventory.coins.wrapping_add(1);
    let probe_bytes = startup_selfdata::build_self(&probe);
    let coins_offset = bytes
        .iter()
        .zip(&probe_bytes)
        .position(|(a, b)| a != b)
        .expect("coins field");
    assert!(coins_offset >= 13);

    let mut probe = self_data.clone();
    probe.inventory.items.push(InventoryItem {
        definition_id: 1,
        tint: 0,
        id: 1,
        count: 1,
        consumed_count: 0,
        stored_count: 0,
        last_cast_time: 0,
        rental: None,
        ability_count: 0,
        activate_enabled: false,
    });
    let probe_bytes = startup_selfdata::build_self(&probe);
    let items_offset = (13..bytes.len())
        .find(|index| bytes[*index] != probe_bytes[*index])
        .expect("items count");
    assert_eq!(&bytes[items_offset..items_offset + 4], &[0; 4]);

    let journal = fresh_journal(self_data);
    let journal_offset = find_unique(bytes, &journal);
    assert_eq!(
        journal_offset,
        items_offset + 8,
        "gender follows the item list"
    );

    // The lab inventory's coins span is the reference offset; the scrubbed name shifts every
    // field after it by a constant. Use that shift for the profile spans, which can be
    // byte-identical across two jobs (same empty slots) and therefore not searchable.
    let lab_inventory = read_json(&source.join("inventory.json"));
    let lab_coins = lab_inventory["spans"]
        .as_array()
        .unwrap()
        .iter()
        .find(|span| span["kind"] == "coins")
        .unwrap()["offset"]
        .as_u64()
        .unwrap() as isize;
    let shift = coins_offset as isize - lab_coins;
    let lab_profiles = read_json(&source.join("profiles.json"));
    let mut equipment = Vec::new();
    for profile in &self_data.jobs.profiles {
        let id = profile.map_key as i32;
        let row = lab_profiles["profiles"]
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row["id"] == id)
            .expect("the captured profile row");
        let span = equipment_span(&self_data.inventory, id);
        let offset = (row["offset"].as_u64().unwrap() as isize
            + shift
            + row["equipment_offset"].as_u64().unwrap() as isize) as usize;
        assert_eq!(
            row["equipment_length"].as_u64().unwrap() as usize,
            span.len(),
            "profile {id} equipment length"
        );
        assert_eq!(
            &bytes[offset..offset + span.len()],
            span.as_slice(),
            "profile {id} equipment span"
        );
        equipment.push((id, offset, span));
    }
    assert_eq!(
        equipment.len(),
        self_data
            .inventory
            .equipment
            .iter()
            .map(|row| row.profile_id)
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        "every equipment profile has one span"
    );
    Derived {
        coins_offset,
        items_offset,
        journal_offset,
        journal,
        equipment,
    }
}

/// Rewrite a lab path into a named lab source: no home or absolute path survives.
fn scrub_path(input: &str) -> String {
    let normalized = input.replace('\\', "/");
    let relative = normalized
        .find("/lab/")
        .map(|index| &normalized[index + 5..])
        .or_else(|| normalized.strip_prefix("lab/"));
    if let Some(rest) = relative {
        return scrub_lab_source(rest);
    }
    if normalized.starts_with("working/")
        || normalized.starts_with("decompiled/")
        || normalized.starts_with("extracted/")
    {
        return scrub_lab_source(&normalized);
    }
    if normalized.starts_with("lab:") {
        return normalized;
    }
    if input.contains('\\')
        || normalized.starts_with("C:")
        || normalized.starts_with('/')
        || normalized.starts_with("Users/")
    {
        return format!("lab:{normalized}");
    }
    input.to_owned()
}

fn scrub_lab_source(rest: &str) -> String {
    if let Some(rest) = rest.strip_prefix("working/OSFR.Client/OSFR Client/") {
        format!("lab:client/{rest}")
    } else if let Some(rest) = rest.strip_prefix("working/OSFR.Server/OSFR Server/") {
        format!("lab:server/{rest}")
    } else if let Some(rest) = rest.strip_prefix("working/") {
        format!("lab:{rest}")
    } else if let Some(rest) = rest.strip_prefix("decompiled/") {
        format!("lab:minigame-scripts/{rest}")
    } else if let Some(rest) = rest.strip_prefix("extracted/client-assets/") {
        format!("lab:client-assets/{rest}")
    } else {
        format!("lab:{rest}")
    }
}

fn looks_like_path(text: &str) -> bool {
    text.contains('\\')
        || text.starts_with("C:")
        || text.starts_with('/')
        || text.starts_with("lab:")
        || text.starts_with("lab/")
        || text.starts_with("working")
        || text.starts_with("decompiled")
        || text.starts_with("extracted")
}

/// Scrub every path-like string and key in a provenance tree. JSON keys are rewritten too,
/// because `provenance.source_sha256` uses the source path as the key.
fn scrub_json(value: &mut Value) {
    match value {
        Value::Object(map) => {
            let old = std::mem::take(map);
            for (key, mut value) in old {
                let key = if looks_like_path(&key) {
                    scrub_path(&key)
                } else {
                    key
                };
                scrub_json(&mut value);
                assert!(
                    map.insert(key.clone(), value).is_none(),
                    "scrubbing collapsed two provenance keys into {key}"
                );
            }
        }
        Value::Array(rows) => {
            for value in rows {
                scrub_json(value);
            }
        }
        Value::String(text) if looks_like_path(text) => *text = scrub_path(text),
        _ => {}
    }
}

fn assert_no_absolute_path(text: &str) {
    for bad in [
        "C:\\",
        "c:\\",
        "C:/",
        "c:/",
        "\\Users\\",
        "/Users/",
        "\\users\\",
        "/users/",
    ] {
        assert!(
            !text.contains(bad),
            "home or absolute path contains {bad:?}"
        );
    }
}

fn inflate_payload(payload: &[u8]) -> Option<Vec<u8>> {
    (0..payload.len().min(8))
        .find_map(|start| miniz_oxide::inflate::decompress_to_vec_zlib(&payload[start..]).ok())
}

/// Reassemble the 44/2 payload from the capture's zlib-compressed reliable fragments and pin
/// its digest. The packet is server-to-client reference data: category ids and icons only.
fn captured_categories(path: &Path) -> (Vec<Value>, Vec<Value>) {
    let data = fs::read(path).unwrap();
    assert_eq!(&data[..4], &[0xd4, 0xc3, 0xb2, 0xa1], "little-endian pcap");
    let mut offset = 24;
    let mut fragments: Vec<(u8, Vec<u8>)> = Vec::new();
    while offset + 16 <= data.len() {
        let included =
            u32::from_le_bytes(data[offset + 8..offset + 12].try_into().unwrap()) as usize;
        offset += 16;
        if offset + included > data.len() {
            break;
        }
        let packet = &data[offset..offset + included];
        offset += included;
        if packet.len() < 42 || u16::from_be_bytes([packet[12], packet[13]]) != 0x0800 {
            continue;
        }
        let ip = &packet[14..];
        let header = ((ip[0] & 0x0f) as usize) * 4;
        if ip[9] != 17 {
            continue;
        }
        let source = u16::from_be_bytes([ip[header], ip[header + 1]]);
        let total = u16::from_be_bytes([ip[2], ip[3]]) as usize;
        if source != 20232 || header + 8 > total || total > ip.len() {
            continue;
        }
        if let Some(decoded) = inflate_payload(&ip[header + 8..total])
            && decoded.first() == Some(&6)
            && decoded.len() > 2
        {
            fragments.push((decoded[1], decoded[2..].to_vec()));
        }
    }
    let start = fragments
        .iter()
        .position(|(_, bytes)| {
            bytes
                .windows(12)
                .any(|window| window == [44, 0, 2, 0, 71, 0, 0, 0, 52, 1, 0, 0])
        })
        .expect("the 44/2 reference-data fragment is in the capture");
    let mut stream = Vec::new();
    for (index, (sequence, bytes)) in fragments.iter().enumerate().skip(start) {
        if index > start {
            assert_eq!(
                *sequence,
                fragments[index - 1].0.wrapping_add(1),
                "reliable fragment order"
            );
        }
        stream.extend_from_slice(bytes);
        if stream.len() >= 11 + 2675 {
            break;
        }
    }
    let header = find_unique(&stream, &[44, 0, 2, 0, 71, 0, 0, 0]);
    let packet = stream[header..header + 2675].to_vec();
    assert_eq!(sha(&packet), CATEGORY_PACKET_SHA256);

    let mut cursor = 4;
    let count = i32::from_le_bytes(packet[cursor..cursor + 4].try_into().unwrap());
    cursor += 4;
    assert_eq!(count, 71);
    let mut categories = Vec::new();
    for _ in 0..count {
        let key = i32::from_le_bytes(packet[cursor..cursor + 4].try_into().unwrap());
        let id = i32::from_le_bytes(packet[cursor + 4..cursor + 8].try_into().unwrap());
        let name_id = i32::from_le_bytes(packet[cursor + 8..cursor + 12].try_into().unwrap());
        let icon_id = i32::from_le_bytes(packet[cursor + 12..cursor + 16].try_into().unwrap());
        let unknown = i32::from_le_bytes(packet[cursor + 16..cursor + 20].try_into().unwrap());
        let unknown2 = packet[cursor + 20] != 0;
        let display = i32::from_le_bytes(packet[cursor + 21..cursor + 25].try_into().unwrap());
        cursor += 25;
        assert_eq!(key, id);
        categories.push(json!({
            "id": id,
            "name_id": name_id,
            "icon_id": icon_id,
            "unknown": unknown,
            "unknown2": unknown2,
            "display": display,
        }));
    }
    let count = i32::from_le_bytes(packet[cursor..cursor + 4].try_into().unwrap());
    cursor += 4;
    assert_eq!(count, 74);
    let mut groups = Vec::new();
    for _ in 0..count {
        let id = i32::from_le_bytes(packet[cursor..cursor + 4].try_into().unwrap());
        let key = i32::from_le_bytes(packet[cursor + 4..cursor + 8].try_into().unwrap());
        let category_id = i32::from_le_bytes(packet[cursor + 8..cursor + 12].try_into().unwrap());
        cursor += 12;
        assert_eq!(id, key);
        groups.push(json!({"id": id, "category_id": category_id}));
    }
    assert_eq!(cursor, packet.len());
    (categories, groups)
}

/// Regenerate the overlay from the private lab package and the capture. This is the only
/// ignored test that reads lab data; run it after a package or capture change.
#[test]
#[ignore = "lab use: needs the private native package and the category packet capture"]
fn export_release_content() {
    let source = source_dir();
    let release = release_dir();
    fs::create_dir_all(&release).unwrap();

    let self_data: SelfData =
        serde_json::from_slice(&fs::read(content().join("startup/self_data.json")).unwrap())
            .unwrap();
    let built = startup_selfdata::build_self(&self_data);
    let derived = derive(&self_data, &built, &source);

    // Our reconstructed campaign content: scrub every lab path.
    for file in [
        "opening.json",
        "cooking.json",
        "forging.json",
        "smelting.json",
    ] {
        let mut value = read_json(&source.join(file));
        scrub_json(&mut value);
        write_json(&release.join(file), &value);
    }
    let opening = fs::read(release.join("opening.json")).unwrap();
    let cooking = fs::read(release.join("cooking.json")).unwrap();

    // The markers are derivable from the two files the loader checks; stale lab hashes for
    // files the release does not ship are not copied.
    let cooking_marker = read_json(&source.join("cooking-campaign.json"));
    write_json(
        &release.join("cooking-campaign.json"),
        &json!({
            "schema_version": 1,
            "opening_sha256": sha(&opening),
            "cooking_sha256": sha(&cooking),
            "goal_shape_changes": cooking_marker["goal_shape_changes"],
            "changes": cooking_marker["changes"],
        }),
    );
    let forging_marker = read_json(&source.join("forging-campaign.json"));
    write_json(
        &release.join("forging-campaign.json"),
        &json!({
            "schema_version": 1,
            "opening_sha256": sha(&opening),
            "disappear_after_interaction": forging_marker["disappear_after_interaction"],
            "changes": forging_marker["changes"],
        }),
    );

    // The capture-derived AddNpc skeleton becomes typed hex; the file stays in existing
    // content, the release ships no binary.
    let template = fs::read(source.join("combat-template.bin")).unwrap();
    let mut combat = read_json(&source.join("combat.json"));
    scrub_json(&mut combat);
    combat.as_object_mut().unwrap().remove("template_file");
    combat["template_hex"] = json!(hex(&template));
    combat["template_sha256"] = json!(sha(&template));
    combat["provenance"]["template"] = json!(
        "T853 overlay: the AddNpc skeleton is embedded as template_hex; the release ships no binary. The bytes are the captured 0010-ready.bin shape with the name, aliases and effects zeroed by the lab exporter."
    );
    write_json(&release.join("combat.json"), &combat);
    assert_eq!(template.len(), 334);

    // Categories are rebuilt from the original server's 44/2 capture, not the community tables.
    let (categories, groups) = captured_categories(&capture_path(&source));
    write_json(
        &release.join("categories.json"),
        &json!({
            "schema_version": 1,
            "categories": categories,
            "groups": groups,
            "packet_sha256": CATEGORY_PACKET_SHA256,
            "provenance": {
                "capture": "Reference-data 44/2 packet observed in a captured game-server session (UDP source port 20232 -> client 59128) in the pinned community packet capture p1.pcap; the payload was reassembled from its zlib-compressed reliable fragments.",
                "packet": "2,675 bytes: i32 category count 71, category rows (dictionary key, id, name id, icon id, unknown, bool, display), i32 group count 74, group rows (group id, key, category id).",
                "scrub": "Category ids and client locale/icon ids only; no account, character, server text or message bytes are copied. The capture stays in the lab.",
                "wire": "ReferenceDataPacket 44/2; client dispatcher 0x00928d00, category parser 0x009215f0 -> 0x008fca00, row readers 0x008e98b0 (categories) and 0x008e9a10 (groups).",
            },
        }),
    );

    // The fresh character's journal: the 21-byte empty span from the typed self data, embedded
    // as hex so no capture character journal (and no binary) ships.
    let mut quests = read_json(&source.join("quests.json"));
    scrub_json(&mut quests);
    quests["journal"] = json!({
        "offset": derived.journal_offset,
        "length": derived.journal.len(),
        "sha256": sha(&derived.journal),
        "hex": hex(&derived.journal),
        "bindings": [],
    });
    for definition in quests["definitions"].as_array_mut().unwrap() {
        definition["initially_accepted"] = json!(false);
    }
    quests["provenance"]["release"] = json!(
        "T853 overlay: the journal is the fresh character's 21-byte empty span from data/content/startup/self_data.json, embedded as hex; no capture character journal is published, and the original quest 1617 starts unaccepted."
    );
    write_json(&release.join("quests.json"), &quests);

    // Profile rows are capture game data. The starter kit map and its community license
    // provenance are removed; T846 moves the kit into start-quest rewards (R).
    let mut profiles = read_json(&source.join("profiles.json"));
    scrub_json(&mut profiles);
    profiles["starter_items"] = json!({});
    profiles["provenance"]
        .as_object_mut()
        .unwrap()
        .remove("starter_definitions");
    profiles["provenance"]["release"] = json!(
        "T853 overlay: the 16 captured job rows, active/default progression and profile bytes stay; starter_items is empty (R) and T846 moves the starter kit to start-quest rewards."
    );
    write_json(&release.join("profiles.json"), &profiles);

    // Item definitions stay (game data). The live state is the fresh character typed in the
    // startup self: no coins, no owned items, the empty equipment slot rows, guid 1.
    let mut inventory = read_json(&source.join("inventory.json"));
    scrub_json(&mut inventory);
    inventory.as_object_mut().unwrap().remove("initial_state");
    inventory["default_state"] = serde_json::to_value(&self_data.inventory).unwrap();
    inventory["provenance"]
        .as_object_mut()
        .unwrap()
        .remove("starter_definitions");
    let mut spans = vec![json!({
        "kind": "coins",
        "offset": derived.coins_offset,
        "length": 4,
        "sha256": sha(&self_data.inventory.coins.to_le_bytes()),
        "profile_id": 0,
    })];
    for (profile_id, offset, span) in &derived.equipment {
        spans.push(json!({
            "kind": "equipment",
            "offset": offset,
            "length": span.len(),
            "sha256": sha(span),
            "profile_id": profile_id,
        }));
    }
    spans.push(json!({
        "kind": "items",
        "offset": derived.items_offset,
        "length": 4,
        "sha256": sha(&[0; 4]),
        "profile_id": 0,
    }));
    spans.sort_by_key(|span| span["offset"].as_u64().unwrap());
    inventory["spans"] = Value::Array(spans);
    inventory["provenance"]["release"] = json!(
        "T853 overlay: the 1,703 capture item definitions and 16 job slot tables stay; default_state is the fresh character typed in data/content/startup/self_data.json (0 coins, no owned items, the empty equipment slot rows), the capture inventory and 7-item starter kit are gone, and the spans were recomputed from that serialized self. T846 moves the kit to start-quest rewards (R)."
    );
    write_json(&release.join("inventory.json"), &inventory);

    println!(
        "wrote {} release files to {}",
        RELEASE_FILES.len(),
        release.display()
    );
}

#[test]
fn the_release_overlay_reaches_world_entry_with_the_opening_active() {
    let merged = merged_content("world");
    let state = Gateway::content_state(&merged).unwrap();
    let mut gateway = Gateway::load_gameplay_from_content(state.clone(), &merged, None).unwrap();
    assert!(gateway.gameplay().unwrap().is_opening());
    let messages = startup_of(&mut gateway, &state, &client_version());
    assert!(!messages.is_empty());
    let summary = gateway.gameplay_summary().expect("gameplay is loaded");
    let opening = &summary["opening"];
    assert_eq!(opening["version"], 1);
    assert!(
        !opening["stages"]
            .as_object()
            .expect("opening stages")
            .is_empty(),
        "the opening module started with no stages"
    );
    assert_eq!(summary["coins"], 0);
    assert_eq!(summary["inventory_instances"], 0);
    assert_eq!(summary["profiles"]["active"], 1);
    assert_eq!(summary["combat"]["enemy_health"], 100);
    for campaign in ["cooking", "forging", "smelting"] {
        assert!(
            !summary[campaign].is_null(),
            "the {campaign} campaign did not load"
        );
    }
    let opening_actor = 5_715_144_128_442_925_057_u64.to_le_bytes();
    assert!(
        messages.iter().any(|message| {
            message.get(7..11) == Some(&[35, 0, 2, 0][..])
                && message.get(11..19) == Some(&opening_actor[..])
        }),
        "the opening actor spawn is missing"
    );
    let categories = messages
        .iter()
        .find(|message| message.get(7..11) == Some(&[44, 0, 2, 0]))
        .expect("the 44/2 category packet is in the world-entry replies");
    assert_eq!(
        categories[7..].to_vec(),
        ItemCategories::load(&release_dir())
            .unwrap()
            .initialization()
    );
    assert_eq!(categories.len(), 7 + 2675);
    let _ = fs::remove_dir_all(merged);
}

#[test]
fn every_release_file_loads_through_its_real_loader() {
    let release = release_dir();
    let categories = ItemCategories::load(&release).unwrap().initialization();
    assert_eq!(&categories[..8], &[44, 0, 2, 0, 71, 0, 0, 0]);
    assert_eq!(categories.len(), 2675);
    let inventory = Inventory::load(&release, None).unwrap();
    assert_eq!(inventory.state().coins, 0);
    assert!(inventory.state().items.is_empty());
    assert!(!inventory.state().equipment.is_empty());
    let profiles = Profiles::load(&release, None).unwrap();
    assert_eq!(profiles.active(), 1);
    assert!(profiles.starter_items(1).is_empty());
    let combat = Combat::load(&release, None).unwrap();
    assert_eq!(combat.state().enemy_health, 100);
    let quests = QuestEngine::load(&release, None).unwrap();
    assert!(quests.state().quests.values().all(|quest| !quest.accepted));
    OpeningJournal::load(&release).unwrap();
}

#[test]
fn the_release_overlay_is_publishable() {
    let release = release_dir();
    let mut names: Vec<String> = fs::read_dir(&release)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    let mut expected: Vec<String> = RELEASE_FILES
        .iter()
        .map(|name| (*name).to_owned())
        .collect();
    expected.sort();
    assert_eq!(names, expected);
    for name in RELEASE_FILES {
        let path = release.join(name);
        let bytes = fs::read(&path).unwrap();
        assert!(
            bytes.len() < 10 * 1024 * 1024,
            "{name} exceeds the upload size"
        );
        let text = String::from_utf8(bytes.clone()).unwrap_or_else(|_| {
            panic!("{name} is not UTF-8 JSON; release content carries no binary")
        });
        assert_no_absolute_path(&text);
        for forbidden in ["osfr", "discord", "open source"] {
            assert!(
                !text.to_ascii_lowercase().contains(forbidden),
                "{name} contains capture server text {forbidden:?}"
            );
        }
        let value: Value = serde_json::from_str(&text).unwrap();
        assert!(value.is_object(), "{name} is not a JSON object");
    }

    let categories = read_json(&release.join("categories.json"));
    assert_eq!(categories["packet_sha256"], CATEGORY_PACKET_SHA256);
    let category_provenance = categories["provenance"].to_string().to_ascii_lowercase();
    for forbidden in [
        "sanctuary",
        "agpl",
        "free realms legacy",
        "freerealms-legacy",
    ] {
        assert!(
            !category_provenance.contains(forbidden),
            "categories.json still cites the AGPL source {forbidden:?}"
        );
    }

    let combat = read_json(&release.join("combat.json"));
    let template = hex_bytes(combat["template_hex"].as_str().unwrap());
    assert_eq!(template.len(), 334);
    assert_eq!(sha(&template), COMBAT_TEMPLATE_SHA256);
    assert_eq!(combat["template_sha256"], COMBAT_TEMPLATE_SHA256);
    assert!(
        template
            .windows(5)
            .all(|window| !window.iter().all(u8::is_ascii_graphic)),
        "the template carries a text run, not just numeric fields"
    );
    assert!(combat.get("template_file").is_none());

    let profiles = read_json(&release.join("profiles.json"));
    assert_eq!(profiles["starter_items"], json!({}));
    assert!(profiles["provenance"].get("starter_definitions").is_none());

    let inventory = read_json(&release.join("inventory.json"));
    assert!(inventory.get("initial_state").is_none());
    assert_eq!(inventory["default_state"]["coins"], 0);
    assert!(
        inventory["default_state"]["items"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    assert!(
        !inventory["default_state"]["equipment"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        inventory["spans"].as_array().unwrap().len(),
        inventory["profiles"].as_array().unwrap().len() + 2
    );
    assert!(inventory["provenance"].get("starter_definitions").is_none());

    let quests = read_json(&release.join("quests.json"));
    let journal = &quests["journal"];
    assert!(journal.get("file").is_none());
    assert!(
        journal
            .get("bindings")
            .unwrap()
            .as_array()
            .unwrap()
            .is_empty()
    );
    assert_eq!(journal["length"], 21);
    let journal_bytes = hex_bytes(journal["hex"].as_str().unwrap());
    assert_eq!(sha(&journal_bytes), journal["sha256"].as_str().unwrap());
    assert!(
        journal_bytes
            .windows(4)
            .all(|window| !window.iter().all(u8::is_ascii_graphic)),
        "the fresh journal carries no account text"
    );
    assert!(
        quests["definitions"]
            .as_array()
            .unwrap()
            .iter()
            .all(|definition| definition["initially_accepted"] == false)
    );

    let opening = fs::read(release.join("opening.json")).unwrap();
    let cooking = fs::read(release.join("cooking.json")).unwrap();
    let cooking_marker = read_json(&release.join("cooking-campaign.json"));
    assert_eq!(cooking_marker["opening_sha256"], sha(&opening));
    assert_eq!(cooking_marker["cooking_sha256"], sha(&cooking));
    let forging_marker = read_json(&release.join("forging-campaign.json"));
    assert_eq!(forging_marker["opening_sha256"], sha(&opening));
    assert!(
        !forging_marker["disappear_after_interaction"]
            .as_array()
            .unwrap()
            .is_empty()
    );
}

fn hex_bytes(text: &str) -> Vec<u8> {
    assert_eq!(text.len() % 2, 0);
    (0..text.len())
        .step_by(2)
        .map(|index| u8::from_str_radix(&text[index..index + 2], 16).unwrap())
        .collect()
}

#[test]
fn every_release_reward_and_recipe_item_has_a_definition() {
    let release = release_dir();
    let inventory = Inventory::load(&release, None).unwrap();
    let mut ids = Vec::new();
    let opening = read_json(&release.join("opening.json"));
    for stage in opening["stages"].as_array().unwrap() {
        for item in stage["reward"]["items"].as_array().unwrap() {
            ids.push(item[0].as_i64().unwrap() as i32);
        }
    }
    for file in ["cooking.json", "forging.json", "smelting.json"] {
        let recipe = &read_json(&release.join(file))["recipe"];
        ids.push(recipe["product_definition_id"].as_i64().unwrap() as i32);
        for row in recipe["consumes"].as_array().unwrap() {
            ids.push(row[0].as_i64().unwrap() as i32);
        }
    }
    ids.sort_unstable();
    ids.dedup();
    assert!(!ids.is_empty());
    for id in ids {
        assert!(
            inventory.has_definition(id),
            "release reward/recipe item {id} is not defined"
        );
    }
}

#[test]
fn a_wrong_embedded_template_or_journal_hex_is_refused() {
    let directory = std::env::temp_dir().join(format!("osfr-release-hex-{}", std::process::id()));
    let _ = fs::remove_dir_all(&directory);
    fs::create_dir_all(&directory).unwrap();

    let mut combat = read_json(&release_dir().join("combat.json"));
    combat["template_hex"] = json!("00".repeat(334));
    fs::write(
        directory.join("combat.json"),
        serde_json::to_vec(&combat).unwrap(),
    )
    .unwrap();
    assert!(Combat::load(&directory, None).is_err());
    assert!(OpeningWorld::load(&directory).is_err());

    let mut quests = read_json(&release_dir().join("quests.json"));
    quests["journal"]["hex"] = json!("00".repeat(21));
    fs::write(
        directory.join("quests.json"),
        serde_json::to_vec(&quests).unwrap(),
    )
    .unwrap();
    assert!(QuestEngine::load(&directory, None).is_err());
    assert!(OpeningJournal::load(&directory).is_err());
    let _ = fs::remove_dir_all(directory);
}

#[test]
fn a_broken_overlay_is_refused_without_touching_the_files() {
    let broken = merged_content("broken");
    let before: Vec<(String, Vec<u8>)> = RELEASE_FILES
        .iter()
        .map(|name| {
            (
                (*name).to_owned(),
                fs::read(release_dir().join(name)).unwrap(),
            )
        })
        .collect();
    let mut marker = read_json(&broken.join("forging-campaign.json"));
    marker["opening_sha256"] = json!("00".repeat(32));
    fs::write(
        broken.join("forging-campaign.json"),
        serde_json::to_vec(&marker).unwrap(),
    )
    .unwrap();
    let state = Gateway::content_state(&broken).unwrap();
    let error = Gateway::load_gameplay_from_content(state, &broken, None)
        .err()
        .expect("the broken marker is refused");
    assert!(
        error.to_string().contains("forging campaign marker"),
        "{error}"
    );
    for (name, bytes) in &before {
        assert_eq!(
            &fs::read(release_dir().join(name)).unwrap(),
            bytes,
            "a refused load changed {name}"
        );
    }
    let _ = fs::remove_dir_all(broken);

    let merged = merged_content("categories");
    let mut categories = read_json(&merged.join("categories.json"));
    categories["packet_sha256"] = json!("00".repeat(32));
    fs::write(
        merged.join("categories.json"),
        serde_json::to_vec(&categories).unwrap(),
    )
    .unwrap();
    let state = Gateway::content_state(&merged).unwrap();
    assert!(Gateway::load_gameplay_from_content(state, &merged, None).is_err());
    for (name, bytes) in &before {
        assert_eq!(
            &fs::read(release_dir().join(name)).unwrap(),
            bytes,
            "a refused load changed {name}"
        );
    }
    let _ = fs::remove_dir_all(merged);
}
