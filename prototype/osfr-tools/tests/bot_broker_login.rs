//! W130: real broker, native login, character handoff, gateway and wrong-key canary.
//! Synthetic fixtures reused from bot_smoke.rs; no captured assets or real names.
#[path = "common/startup_selfdata_fixture.rs"]
mod startup_selfdata_fixture;
use ed25519_dalek::SigningKey;
use osfr_tools::{
    account::{AccountFlags, SlotPolicy, create_character},
    account_store::AccountStore,
    character_store::CharacterState,
    identity::{NativeCreation, Resources},
    startup_snapshot::StartupSnapshot,
};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::{BufRead, BufReader},
    net::UdpSocket,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::mpsc,
    time::Duration,
};
const VERSION: &str = "synthetic.1";
struct SecretCleanup(PathBuf);
impl Drop for SecretCleanup {
    fn drop(&mut self) {
        for name in ["key.ed25519", "login.key", "handoff.key"] {
            let _ = fs::remove_file(self.0.join(name));
        }
    }
}
struct Service(Child);
impl Drop for Service {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}
fn start(mut command: Command, event: &str) -> (Service, String) {
    let mut child = Service(
        command
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .unwrap(),
    );
    let stdout = child.0.stdout.take().unwrap();
    let (tx, rx) = mpsc::channel();
    let wanted = event.to_owned();
    std::thread::spawn(move || {
        for line in BufReader::new(stdout).lines().map_while(Result::ok) {
            if let Ok(value) = serde_json::from_str::<serde_json::Value>(&line) {
                if value["event"] == wanted {
                    let _ = tx.send(value["address"].as_str().unwrap().to_owned());
                }
            }
        }
    });
    let address = rx
        .recv_timeout(Duration::from_secs(60))
        .expect("service did not listen");
    (child, address)
}
fn put_string(out: &mut Vec<u8>, value: &str) {
    out.extend_from_slice(&(value.len() as u32).to_le_bytes());
    out.extend_from_slice(value.as_bytes());
}

fn wrapped(outer: u16, inner: &[u8]) -> Vec<u8> {
    let mut out = outer.to_le_bytes().to_vec();
    out.push(1);
    out.extend_from_slice(&(inner.len() as u32).to_le_bytes());
    out.extend_from_slice(inner);
    out
}

fn packet(opcode: u16, fields: &[u8]) -> Vec<u8> {
    let mut inner = opcode.to_le_bytes().to_vec();
    inner.extend_from_slice(fields);
    wrapped(5, &inner)
}

fn create_reference(root: &Path) -> PathBuf {
    let reference = root.join("lab/references/rust-gateway-reference");
    fs::create_dir_all(&reference).unwrap();
    let state = CharacterState {
        guid: 1,
        zone_id: 0,
        position: [1.0, 2.0, 3.0],
        rotation: [0.0, 0.0, 1.0],
    };
    let (self_packet, position_offset, rotation_offset) =
        startup_selfdata_fixture::self_data_packet(state.guid, state.position, state.rotation);

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
    let zone_packet = wrapped(5, &{
        let mut inner = 43u16.to_le_bytes().to_vec();
        inner.extend_from_slice(&zone);
        inner
    });

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
            _ => packet(opcode, &[]),
        };
        let name = format!("{index:04}.bin");
        fs::write(reference.join(&name), &bytes).unwrap();
        templates.push(json!({
            "kind": if index < 8 { "login" } else { "ready" },
            "file": name,
            "length": bytes.len(),
            "sha256": format!("{:x}", Sha256::digest(&bytes)),
            "opcode": opcode
        }));
    }
    let manifest = json!({
        "schema_version": 1,
        "client_version": VERSION,
        "initial_state": state,
        "self_layout": {
            "guid_offset": 21,
            "position_offset": position_offset,
            "rotation_offset": rotation_offset
        },
        "zone_layout": {"zone_id_offset": zone_id_offset},
        "templates": templates
    });
    fs::write(
        reference.join("manifest.json"),
        serde_json::to_vec(&manifest).unwrap(),
    )
    .unwrap();
    let startup = root.join("lab/content/startup-snapshot/startup.json");
    fs::create_dir_all(startup.parent().unwrap()).unwrap();
    fs::write(
        startup,
        serde_json::to_vec(&StartupSnapshot::empty()).unwrap(),
    )
    .unwrap();
    reference
}

fn create_account_store(root: &Path) -> PathBuf {
    let resources_dir = root.join("creator");
    let character_create = resources_dir.join("CharacterCreate");
    fs::create_dir_all(&character_create).unwrap();
    let mut model = vec!["0"; 17];
    model[0] = "1";
    model[5] = "1";
    model[13] = "1";
    fs::write(resources_dir.join("Models.txt"), model.join("^")).unwrap();
    for (file, row) in [
        ("HeadMappings.txt", "1^head.adr^"),
        ("HairMappings.txt", "2^hair.adr^"),
        ("ModelCustomizationMappings.txt", "1^beard.adr^"),
        ("FacePaintMappings.txt", "0^none^"),
        ("SkinToneMappings.txt", "1^skin.adr^"),
    ] {
        fs::write(character_create.join(file), row).unwrap();
    }
    let mut definitions = String::new();
    for (id, slot) in [(201, 5), (189, 4), (199, 3)] {
        let mut row = vec!["0".to_owned(); 42];
        row[0] = id.to_string();
        row[12] = slot.to_string();
        row[16] = "item.adr".into();
        row[17] = "1".into();
        row[18] = "cloth".into();
        row[19] = "0".into();
        row[41] = "tint".into();
        definitions.push_str(&row.join("^"));
        definitions.push('\n');
    }
    fs::write(character_create.join("ItemDefinitions.txt"), definitions).unwrap();
    let resources = Resources::load(&resources_dir).unwrap();
    let mut store = AccountStore::new();
    let accounts = [
        store
            .create_account(
                osfr_tools::login_auth::login_handle_hash("acct_001").unwrap(),
                AccountFlags::default(),
                SlotPolicy { max_characters: 1 },
            )
            .unwrap(),
        store
            .create_account(
                osfr_tools::login_auth::login_handle_hash("acct_002").unwrap(),
                AccountFlags::default(),
                SlotPolicy { max_characters: 1 },
            )
            .unwrap(),
        store
            .create_account(
                [8; 32],
                AccountFlags::default(),
                SlotPolicy { max_characters: 1 },
            )
            .unwrap(),
    ];
    for (index, (first, last, temporary)) in [
        ("FixtureA", "Record", "Tmp"),
        ("FixtureB", "Record", "TempA"),
        ("FixtureC", "Record", "TempB"),
    ]
    .into_iter()
    .enumerate()
    {
        let mut choices = [0; 21];
        choices[0] = 1;
        choices[1] = 1;
        choices[2] = 2;
        choices[5] = 1;
        choices[7] = 201;
        choices[8] = 189;
        choices[9] = 199;
        choices[20] = 1;
        let reply = create_character(
            &mut store,
            accounts[index],
            NativeCreation {
                first_name: first.into(),
                last_name: last.into(),
                temporary_first_name: temporary.into(),
                temporary_last_name: "TempName".into(),
                choices,
            },
            &resources,
        );
        assert_eq!(
            reply.status,
            osfr_tools::account::CharacterCreateStatus::Created
        );
    }
    let path = root.join("accounts.json");
    store.save(&path).unwrap();
    path
}

/// L053: a broker profile on a private-network or VPN (100.64/10) address is refused unless
/// `--allow-lan` opts in, the launcher's rule; with the flag the address gate passes and the run
/// fails later for its own reasons (this profile has no login key), never at the gate.
#[test]
fn private_broker_profile_needs_allow_lan() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../lab/reports/broker-login-w130")
        .join(format!("allow-lan-{}", std::process::id()));
    fs::create_dir_all(&root).unwrap();
    let _secrets = SecretCleanup(root.clone());
    fs::write(root.join("key.ed25519"), [7; 32]).unwrap();
    fs::write(root.join("server.id"), [9; 32]).unwrap();
    let profile_path = root.join("profile.json");
    let run = |broker: &str, lan: bool| {
        let profile = json!({"handle":"acct_002", "key_file":"key.ed25519", "server_id_file":"server.id",
            "broker":broker, "login":"10.255.255.1:20042", "login_key_file":"login.key"});
        fs::write(&profile_path, serde_json::to_vec(&profile).unwrap()).unwrap();
        let mut command = Command::new(env!("CARGO_BIN_EXE_osfr-bot"));
        command.arg("--broker-profile").arg(&profile_path).args(["--guid", "2", "--version", VERSION]);
        if lan {
            command.arg("--allow-lan");
        }
        let output = command.output().unwrap();
        assert!(!output.status.success());
        String::from_utf8_lossy(&output.stderr).into_owned()
    };
    let refused = run("10.255.255.1:20043", false);
    assert!(refused.contains("private-network address with --allow-lan"), "{refused}");
    let opted_in = run("100.64.8.1:20043", true);
    assert!(!opted_in.contains("private-network address"), "{opted_in}");
    // A public address stays refused even with the flag.
    let public = run("8.8.8.8:20043", true);
    assert!(public.contains("private-network address with --allow-lan"), "{public}");
}

#[test]
fn broker_bot_reaches_world_and_wrong_key_sends_no_udp() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../lab/reports/broker-login-w130")
        .join(format!("integration-{}", std::process::id()));
    fs::create_dir_all(&root).unwrap();
    let _secrets = SecretCleanup(root.clone());
    let reference = create_reference(&root);
    let store_path = create_account_store(&root);
    let mut seed = [0; 32];
    getrandom::getrandom(&mut seed).unwrap();
    let signing = SigningKey::from_bytes(&seed);
    fs::write(root.join("key.ed25519"), seed).unwrap();
    let mut store = AccountStore::load(&store_path).unwrap();
    store
        .add_public_key(2, &signing.verifying_key().to_bytes())
        .unwrap();
    store.save(&store_path).unwrap();
    let mut key = [0; 32];
    getrandom::getrandom(&mut key).unwrap();
    fs::write(root.join("handoff.key"), key).unwrap();
    getrandom::getrandom(&mut key).unwrap();
    fs::write(root.join("login.key"), key).unwrap();
    getrandom::getrandom(&mut key).unwrap();
    fs::write(root.join("server.id"), key).unwrap();
    let mut gateway = Command::new(env!("CARGO_BIN_EXE_osfr-gateway"));
    gateway
        .arg("127.0.0.1:0")
        .arg("--reference")
        .arg(reference)
        .arg("--state")
        .arg(root.join("state/gateway.json"))
        .arg("--account-store")
        .arg(&store_path)
        .arg("--handoff-key-file")
        .arg(root.join("handoff.key"))
        .args(["--studio-port", "0"]);
    let (_gateway, gateway_addr) = start(gateway, "listening");
    let udp = UdpSocket::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap();
    let mut login = Command::new(env!("CARGO_BIN_EXE_osfr-login"));
    login
        .args([
            "--bind",
            &udp.to_string(),
            "--auth-bind",
            "127.0.0.1:0",
            "--gateway",
            &gateway_addr,
        ])
        .arg("--resources")
        .arg(root.join("creator"))
        .arg("--identity")
        .arg(root.join("identity.json"))
        .arg("--account-store")
        .arg(&store_path)
        .arg("--server-id-file")
        .arg(root.join("server.id"))
        .arg("--key-file")
        .arg(root.join("login.key"))
        .arg("--handoff-key-file")
        .arg(root.join("handoff.key"));
    let (_login, broker) = start(login, "auth_listening");
    let trap = UdpSocket::bind("127.0.0.1:0").unwrap();
    trap.set_read_timeout(Some(Duration::from_millis(200)))
        .unwrap();
    let profile_path = root.join("profile.json");
    let mut profile = json!({"handle":"acct_002", "key_file":"key.ed25519",
        "server_id_file":"server.id", "broker":broker, "login":trap.local_addr().unwrap().to_string(),
        "login_key_file":"login.key"});
    let invoke = || {
        Command::new(env!("CARGO_BIN_EXE_osfr-bot"))
            .arg("--broker-profile")
            .arg(&profile_path)
            .args([
                "--guid",
                "2",
                "--version",
                VERSION,
                "--cmd",
                "raw 340000000000000000000000000001",
                "--cmd",
                "expect 52",
                "--cmd",
                "quit",
                "--expect-timeout",
                "30000",
            ])
            .output()
            .unwrap()
    };
    let mut wrong = [0; 32];
    getrandom::getrandom(&mut wrong).unwrap();
    fs::write(root.join("key.ed25519"), wrong).unwrap();
    fs::write(&profile_path, serde_json::to_vec(&profile).unwrap()).unwrap();
    let refused = invoke();
    let refused_text = String::from_utf8_lossy(&refused.stderr);
    assert!(!refused.status.success());
    assert!(
        refused_text.contains("broker refused"),
        "expected generic broker refusal"
    );
    assert!(
        trap.recv(&mut [0; 512]).is_err(),
        "wrong key reached UDP login"
    );
    // Pin-before-sign canary uses a live broker and observes zero UDP traffic.
    fs::write(root.join("key.ed25519"), seed).unwrap();
    fs::write(root.join("wrong.id"), [0; 32]).unwrap();
    profile["server_id_file"] = json!("wrong.id");
    fs::write(&profile_path, serde_json::to_vec(&profile).unwrap()).unwrap();
    let bad_pin = invoke();
    assert!(!bad_pin.status.success());
    assert!(String::from_utf8_lossy(&bad_pin.stderr).contains("server identity mismatch"));
    assert!(trap.recv(&mut [0; 512]).is_err());
    profile["server_id_file"] = json!("server.id");
    profile["login"] = json!(udp.to_string());
    fs::write(&profile_path, serde_json::to_vec(&profile).unwrap()).unwrap();
    let accepted = invoke();
    let output = String::from_utf8_lossy(&accepted.stdout);
    fs::write(root.join("bot.stdout.log"), &accepted.stdout).unwrap();
    fs::write(root.join("bot.stderr.log"), &accepted.stderr).unwrap();
    assert!(
        accepted.status.success(),
        "bot failed; see retained integration logs"
    );
    assert!(
        output.contains("in world") && output.contains("ok expect 52") && output.contains("PASS")
    );
    // Account 2 / character 2 exercises the HMAC handoff, beyond the fixed 1/1 path.
    for secret in [seed, wrong] {
        let hex: String = secret.iter().map(|b| format!("{b:02x}")).collect();
        assert!(!output.contains(&hex));
        assert!(!String::from_utf8_lossy(&accepted.stderr).contains(&hex));
    }
}
