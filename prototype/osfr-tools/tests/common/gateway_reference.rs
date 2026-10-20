//! Synthetic gateway startup reference shared by integration tests.
//!
//! Builds the proven 46-template login/ready sequence, its manifest and a typed empty
//! startup snapshot under one temp root. `content` injects that content's job profiles
//! into the self-data template so `Gateway::build_self_message` can patch the character's
//! profile at login; `None` keeps the plain fixture.
use osfr_tools::{character_store::CharacterState, startup_snapshot::StartupSnapshot};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

#[path = "startup_selfdata_fixture.rs"]
mod startup_selfdata_fixture;

static NEXT_DIRECTORY: AtomicU64 = AtomicU64::new(0);

pub fn initial_state() -> CharacterState {
    CharacterState {
        guid: 1,
        zone_id: 0,
        position: [1.0, 2.0, 3.0],
        rotation: [0.1, 0.2, 0.3],
    }
}

pub struct Reference {
    /// The gateway reference directory (`<root>/references/gateway`).
    pub directory: PathBuf,
    pub manifest: Value,
    /// The temp root that also owns `content/startup-snapshot`; removed on drop.
    root: PathBuf,
}

impl Reference {
    /// `version` must match the client login version. The snapshot path is derived by
    /// `Gateway::load_gameplay` as `<root>/content/startup-snapshot/startup.json`.
    pub fn new(version: &str, content: Option<&Path>) -> Self {
        let serial = NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed);
        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "osfr-gateway-test-{}-{time}-{serial}",
            std::process::id()
        ));
        let directory = root.join("references/gateway");
        fs::create_dir_all(&directory).unwrap();
        let snapshot_dir = root.join("content/startup-snapshot");
        fs::create_dir_all(&snapshot_dir).unwrap();
        fs::write(
            snapshot_dir.join("startup.json"),
            serde_json::to_vec(&StartupSnapshot::empty()).unwrap(),
        )
        .unwrap();
        let state = initial_state();
        let (self_packet, position_offset, rotation_offset) = match content {
            Some(content) => self_packet_with_content_profiles(
                state.guid,
                state.position,
                state.rotation,
                content,
            ),
            None => startup_selfdata_fixture::self_data_packet(
                state.guid,
                state.position,
                state.rotation,
            ),
        };
        let mut zone = Vec::new();
        put_string(&mut zone, "test zone");
        zone.extend_from_slice(&2u32.to_le_bytes());
        zone.extend_from_slice(&[0, 0]);
        put_string(&mut zone, "sky");
        zone.push(0);
        let zone_id_offset = 9 + zone.len();
        zone.extend_from_slice(&state.zone_id.to_le_bytes());
        zone.extend_from_slice(&5u32.to_le_bytes());
        zone.extend_from_slice(&[0, 0]);
        let zone_packet = template(43, &zone);
        let mut templates = Vec::new();
        for index in 0..46 {
            let opcode = if index < 8 {
                [41, 2, 166, 43, 144, 35, 193, 12][index]
            } else if index == 8 {
                67
            } else if index == 45 {
                14
            } else {
                200
            };
            let bytes = match index {
                3 => zone_packet.clone(),
                7 => self_packet.clone(),
                _ => template(opcode, &[]),
            };
            let name = format!("{index:04}.bin");
            fs::write(directory.join(&name), &bytes).unwrap();
            templates.push(json!({
                "kind": if index < 8 { "login" } else { "ready" },
                "file": name,
                "length": bytes.len(),
                "sha256": format!("{:x}", Sha256::digest(&bytes)),
                "opcode": opcode,
            }));
        }
        let manifest = json!({
            "schema_version": 1,
            "client_version": version,
            "initial_state": state,
            "self_layout": {
                "guid_offset": 21,
                "position_offset": position_offset,
                "rotation_offset": rotation_offset,
            },
            "zone_layout": { "zone_id_offset": zone_id_offset },
            "templates": templates,
        });
        let reference = Self {
            directory,
            manifest,
            root,
        };
        reference.write_manifest();
        reference
    }

    pub fn write_manifest(&self) {
        fs::write(
            self.directory.join("manifest.json"),
            serde_json::to_vec(&self.manifest).unwrap(),
        )
        .unwrap();
    }
}

impl Drop for Reference {
    fn drop(&mut self) {
        // This path was created under the process temp directory above.
        if self.root.parent() == Some(std::env::temp_dir().as_path()) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }
}

fn put_string(bytes: &mut Vec<u8>, value: &str) {
    bytes.extend_from_slice(&(value.len() as u32).to_le_bytes());
    bytes.extend_from_slice(value.as_bytes());
}

/// One startup template: the tunnel header plus opcode and fields.
fn template(opcode: u16, fields: &[u8]) -> Vec<u8> {
    let mut bytes = vec![5, 0, 1];
    bytes.extend_from_slice(&(u32::try_from(2 + fields.len()).unwrap()).to_le_bytes());
    bytes.extend_from_slice(&opcode.to_le_bytes());
    bytes.extend_from_slice(fields);
    bytes
}

fn hex_bytes(text: &str) -> Vec<u8> {
    text.as_bytes()
        .chunks(2)
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect()
}

/// The typed self-data template must carry a row for every profile the gameplay content
/// configures, or `Gateway::build_self_message` refuses the login. Copy each content
/// profile's opaque prefix (the part before its equipment span) out of `profiles.json`;
/// the gateway only pokes the rank fields inside that prefix.
fn self_packet_with_content_profiles(
    guid: u64,
    position: [f32; 3],
    rotation: [f32; 3],
    content: &Path,
) -> (Vec<u8>, usize, usize) {
    let (packet, position_offset, rotation_offset) =
        startup_selfdata_fixture::self_data_packet(guid, position, rotation);
    let mut data = osfr_tools::startup_selfdata::parse_self(&packet);
    let profiles: Value =
        serde_json::from_slice(&fs::read(content.join("profiles.json")).unwrap()).unwrap();
    data.jobs.profiles = profiles["profiles"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| {
            let record = hex_bytes(row["serialized_hex"].as_str().unwrap());
            let equipment_offset =
                usize::try_from(row["equipment_offset"].as_u64().unwrap()).unwrap();
            osfr_tools::startup_selfdata::JobProfile {
                map_key: u32::from_le_bytes(record[..4].try_into().unwrap()),
                opaque_before_equipment_without_key: record[4..equipment_offset].to_vec(),
                after_equipment_value: 0,
                abilities: Vec::new(),
                trailing_records: Vec::new(),
            }
        })
        .collect();
    (
        osfr_tools::startup_selfdata::build_self(&data),
        position_offset,
        rotation_offset,
    )
}
