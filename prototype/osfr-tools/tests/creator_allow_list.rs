//! T444 (L024 part 3): character creation refuses the skin tones and faces the
//! pinned creator tables hide. The fixture is the creator tree the native harness
//! hands osfr-login (`--resources client/Resources`): stock client tables, then the
//! client overlay (`OVERLAY` in scripts/bundle/client_overlay.py) installed by its own
//! `install`, which puts the pinned SkinTone and FaceType tables in Resources/, where
//! the client reads them (0x008ec05d). identity.rs pins the same two tables, so a
//! re-pin or a moved destination on either side alone fails `Resources::load` here.
use osfr_tools::{
    identity::{NativeCreation, Resources},
    login::{self, LoginConfig, LoginService},
};
// Not registered in lib.rs yet; included the way tests/piece_049.rs does.
#[path = "../src/restoration/piece_049.rs"]
#[allow(dead_code)]
mod piece_049;
use piece_049::{SUPPORTED_HEAD_IDS, SUPPORTED_SKIN_TONE_IDS};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

struct Temp(PathBuf);
impl Temp {
    fn new(name: &str) -> Self {
        let path = std::env::temp_dir().join(format!("osfr-t444-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn client_resources() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../lab/working/OSFR.Client/OSFR Client/Resources")
}

/// The harness's client tree under `root/client`; returns its Resources/.
fn harness_tree(root: &Path) -> PathBuf {
    let client = root.join("client");
    let tree = client.join("Resources");
    fs::create_dir_all(tree.join("CharacterCreate")).unwrap();
    for name in [
        "Models.txt",
        "CharacterCreateCustomizationSkinTone.txt",
        "CharacterCreateCustomizationFaceType.txt",
        "CharacterCreate/HeadMappings.txt",
        "CharacterCreate/HairMappings.txt",
        "CharacterCreate/ModelCustomizationMappings.txt",
        "CharacterCreate/FacePaintMappings.txt",
        "CharacterCreate/SkinToneMappings.txt",
        "CharacterCreate/ItemDefinitions.txt",
        "CharacterCreate/CharacterCreateItemMappings.txt",
    ] {
        fs::copy(client_resources().join(name), tree.join(name)).unwrap();
    }
    // The harness's own call (scripts/run-local-baseline.py install_client_overlay).
    let python = std::env::var_os("PYTHON").unwrap_or_else(|| "python".into());
    let status = Command::new(python)
        .args(["-I", "-c"])
        .arg("import pathlib, sys; sys.path.insert(0, sys.argv[1]); import client_overlay; client_overlay.install(pathlib.Path(sys.argv[2]))")
        .arg(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/bundle"))
        .arg(&client)
        .status()
        .expect("run client_overlay.install");
    assert!(status.success(), "client_overlay.install failed");
    tree
}

/// Human male (model 1) in supplied feet/legs/chest items 201/189/199.
fn creation(head: i32, skin: i32) -> NativeCreation {
    let mut choices = [0; 21];
    choices[..6].copy_from_slice(&[1, head, 2, 0, 0, skin]);
    choices[7..10].copy_from_slice(&[201, 189, 199]);
    NativeCreation {
        first_name: "Ari".into(),
        last_name: "Vale".into(),
        temporary_first_name: String::new(),
        temporary_last_name: String::new(),
        choices,
    }
}

fn string(out: &mut Vec<u8>, value: &[u8]) {
    out.extend((value.len() as u32).to_le_bytes());
    out.extend(value);
}

fn create_request(creation: &NativeCreation) -> Vec<u8> {
    let mut out = vec![5];
    out.extend(1i32.to_le_bytes());
    string(&mut out, &creation.encode());
    out
}

/// Every file under `root` except the read-only client tree.
fn files(root: &Path, out: &mut BTreeMap<PathBuf, Vec<u8>>) {
    for entry in fs::read_dir(root).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            if !path.ends_with("client") {
                files(&path, out);
            }
        } else {
            out.insert(path.clone(), fs::read(&path).unwrap());
        }
    }
}

fn snapshot(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    let mut out = BTreeMap::new();
    files(root, &mut out);
    out
}

#[test]
fn only_the_looks_the_pinned_tables_offer_resolve() {
    let t = Temp::new("looks");
    let resources = Resources::load(&harness_tree(&t.0)).unwrap();
    // SkinToneMappings and HeadMappings both hold ids 1..8; the pinned tables
    // reach skin 1..6 and the Caucasian/African/Asian heads 1..3 only, the sets
    // work item 049 recovered (head 4 has no creator row, so it is refused too).
    for skin in 1..=8 {
        for head in 1..=8 {
            assert_eq!(
                resources.resolve(creation(head, skin)).is_ok(),
                SUPPORTED_SKIN_TONE_IDS.contains(&skin) && SUPPORTED_HEAD_IDS.contains(&head),
                "head {head} skin {skin}"
            );
        }
    }
}

#[test]
fn hidden_skin_or_face_is_refused_and_writes_nothing() {
    let t = Temp::new("refuse");
    let resources = Resources::load(&harness_tree(&t.0)).unwrap();
    let mut service = LoginService::new(
        resources,
        LoginConfig {
            identity_file: t.0.join("identity.json"),
            legacy_save_file: t.0.join("state.json"),
            account_store_file: t.0.join("accounts.json"),
            gateway_address: "127.0.0.1:20260".into(),
            local_session: "synthetic-session".into(),
            gateway_ticket: "synthetic-ticket".into(),
            handoff_key: b"synthetic-handoff-key".to_vec(),
            max_characters: 6,
            underage: false,
            open_chat: true,
        },
    )
    .unwrap();
    let mut login_request = vec![1];
    string(&mut login_request, b"synthetic-session");
    string(&mut login_request, b"synthetic-machine");
    service.handle(&login_request).unwrap();
    let before = snapshot(&t.0);
    for (head, skin) in [(1, 7), (1, 8), (5, 1), (6, 1), (7, 1), (8, 1)] {
        let refused = service
            .handle(&create_request(&creation(head, skin)))
            .unwrap();
        assert!(!refused.created, "head {head} skin {skin}");
        assert_eq!(refused.replies, [login::create_reply(2, 0)]);
        assert_eq!(snapshot(&t.0), before, "head {head} skin {skin} wrote");
    }
    assert_eq!(
        service.handle(&[11]).unwrap().replies,
        [login::selection_reply(None)]
    );

    let created = service.handle(&create_request(&creation(1, 1))).unwrap();
    assert!(created.created);
    assert_eq!(created.replies, [login::create_reply(1, 1)]);
    assert!(t.0.join("accounts.json").is_file());
}

#[test]
fn a_client_tree_without_its_pinned_tables_does_not_load() {
    let t = Temp::new("unpinned");
    let tree = harness_tree(&t.0);
    let skin = tree.join("CharacterCreateCustomizationSkinTone.txt");
    let pinned = fs::read(&skin).unwrap();
    // The supplied original table offers skins 7/8 again; its hash is not the pin.
    fs::copy(
        client_resources().join("CharacterCreateCustomizationSkinTone.txt"),
        &skin,
    )
    .unwrap();
    assert!(Resources::load(&tree).is_err());
    fs::remove_file(&skin).unwrap();
    assert!(Resources::load(&tree).is_err());
    // Pinned tables without the item mappings they join through refuse to load too.
    fs::write(&skin, pinned).unwrap();
    fs::remove_file(tree.join("CharacterCreate/CharacterCreateItemMappings.txt")).unwrap();
    assert!(Resources::load(&tree).is_err());
}
