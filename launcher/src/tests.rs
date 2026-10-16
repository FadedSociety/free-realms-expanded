use super::{
    LaunchPlan, LaunchSettings, LocalLoginStub, LoginProvider, SessionToken, validate_endpoint,
};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

static SERIAL: AtomicU64 = AtomicU64::new(0);

struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join(".test-data")
            .join(format!(
                "{}-{}",
                std::process::id(),
                SERIAL.fetch_add(1, Ordering::Relaxed)
            ));
        fs::create_dir_all(root.join("locale")).unwrap();
        fs::write(
            root.join("FreeRealms.exe"),
            b"MZ synthetic fixture; never execute",
        )
        .unwrap();
        fs::write(root.join("FreeRealms.bat"), "start /min FreeRealms.exe inifile=ClientConfig.ini Guid=9 Server=old:20042 Ticket=discard Key=discard CasSessionId=discard Internationalization:Locale=8 Country=US\n").unwrap();
        fs::write(root.join("ClientConfig.ini"), "Server=old:20042\nIndirectServerAddress=http://invalid.example/assets\nAddress=lvsadp-old\n").unwrap();
        fs::write(root.join("UserOptions.ini"), "WindowedWidth=1920\nWindowedHeight=1080\nMode=Fullscreen\nMaximized=1\nMaster=1.000000\n").unwrap();
        for name in ["en_us_data.dat", "en_us_data.dir"] {
            fs::write(root.join("locale").join(name), b"old fixture").unwrap();
        }
        Self(root)
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .canonicalize()
            .unwrap();
        let target = self.0.canonicalize().unwrap();
        assert!(target.starts_with(root.join(".test-data")));
        fs::remove_dir_all(target).unwrap();
    }
}

#[test]
fn endpoint_and_login_boundaries() {
    for value in ["127.0.0.1:20042", "realm.example:20042", "[::1]:20042"] {
        assert!(validate_endpoint(value).is_ok(), "{value}");
    }
    for value in [
        "host:0",
        "host:65536",
        "host:1;whoami",
        "http://host:20042",
        "::1:20042",
        "a..b:1",
        "[host]:1",
        "host:１２",
        "-x:10",
    ] {
        assert!(validate_endpoint(value).is_err(), "{value}");
    }
    let mut login = LocalLoginStub;
    assert!(login.session("127.0.0.1:20042").is_ok());
    assert!(login.session("[::1]:20042").is_ok());
    assert!(login.session("192.0.2.1:20042").is_err());
    assert!(login.session("localhost:20042").is_err()); // No DNS trust in the stub.
}

#[test]
fn preparation_preserves_inputs_backs_up_and_filters_auth() {
    let scratch = Scratch::new();
    let overlay = scratch.0.join("overlay");
    fs::create_dir(&overlay).unwrap();
    for name in ["en_us_data.dat", "en_us_data.dir"] {
        fs::write(overlay.join(name), b"new fixture").unwrap();
    }
    let original = fs::read(scratch.0.join("UserOptions.ini")).unwrap();
    let source = fs::read(scratch.0.join("ClientConfig.ini")).unwrap();
    let plan = LaunchPlan::build(
        &scratch.0,
        "127.0.0.1:20042",
        Some(&overlay),
        LaunchSettings {
            width: 1280,
            height: 720,
        },
        &mut LocalLoginStub,
    )
    .unwrap();
    // Building a plan is the dry run; it must not write.
    assert_eq!(
        original,
        fs::read(scratch.0.join("UserOptions.ini")).unwrap()
    );
    assert!(!scratch.0.join("Baseline.ini").exists());
    let args = &plan.arguments;
    assert!(args.contains(&"inifile=Baseline.ini".to_owned()));
    assert!(args.contains(&"Server=127.0.0.1:20042".to_owned()));
    assert!(args.contains(&"SessionId=local-session".to_owned()));
    assert!(!args.iter().any(|s| {
        ["guid=", "ticket=", "key=", "cassessionid="]
            .iter()
            .any(|p| s.to_lowercase().starts_with(p))
    }));
    let backup = plan.apply().unwrap();
    assert_eq!(original, fs::read(backup.join("UserOptions.ini")).unwrap());
    assert_eq!(
        source,
        fs::read(scratch.0.join("ClientConfig.ini")).unwrap()
    );
    let baseline = fs::read_to_string(scratch.0.join("Baseline.ini")).unwrap();
    assert!(baseline.contains("Server=127.0.0.1:20042"));
    assert!(baseline.contains("IndirectServerAddress=http://127.0.0.1:9/"));
    assert!(baseline.contains("Address=127.0.0.1"));
    let options = fs::read_to_string(scratch.0.join("UserOptions.ini")).unwrap();
    assert!(options.contains("WindowedWidth=1280\nWindowedHeight=720"));
    assert!(options.contains("Mode=Windowed\nMaximized=0\nMaster=0.000000"));
    for name in ["en_us_data.dat", "en_us_data.dir"] {
        assert_eq!(
            fs::read(scratch.0.join("locale").join(name)).unwrap(),
            b"new fixture"
        );
        assert_eq!(
            fs::read(backup.join("locale").join(name)).unwrap(),
            b"old fixture"
        );
    }
    assert!(plan.apply().is_err()); // Exclusive backup directory; no accidental replay.
}

#[test]
fn invalid_config_or_target_never_changes_files() {
    let scratch = Scratch::new();
    fs::write(scratch.0.join("Baseline.ini"), b"preserve").unwrap();
    fs::write(scratch.0.join("ClientConfig.ini"), "Server=a:1\nServer=b:2").unwrap();
    assert!(
        LaunchPlan::build(
            &scratch.0,
            "127.0.0.1:20042",
            None,
            LaunchSettings::default(),
            &mut LocalLoginStub
        )
        .is_err()
    );
    assert_eq!(
        fs::read(scratch.0.join("Baseline.ini")).unwrap(),
        b"preserve"
    );
    assert!(!scratch.0.join(".friend-launcher-backups").exists());
}

#[test]
fn hard_link_target_is_refused() {
    let scratch = Scratch::new();
    let target = scratch.0.join("Baseline.ini");
    fs::hard_link(scratch.0.join("ClientConfig.ini"), &target).unwrap();
    assert!(
        LaunchPlan::build(
            &scratch.0,
            "127.0.0.1:20042",
            None,
            LaunchSettings::default(),
            &mut LocalLoginStub
        )
        .is_err()
    );
}

#[test]
fn arguments_reject_batch_commands_and_missing_ini() {
    let scratch = Scratch::new();
    for line in [
        "FreeRealms.exe inifile=x & calc.exe",
        "FreeRealms.exe Country=US",
        "FreeRealms.exe inifile=x\nFreeRealms.exe inifile=y",
        "FreeRealms.exe inifile=\"unterminated",
    ] {
        fs::write(scratch.0.join("FreeRealms.bat"), line).unwrap();
        assert!(
            LaunchPlan::build(
                &scratch.0,
                "127.0.0.1:20042",
                None,
                LaunchSettings::default(),
                &mut LocalLoginStub
            )
            .is_err(),
            "{line}"
        );
    }
}

#[test]
fn omitted_overlay_and_stale_plan_preserve_local_files() {
    let scratch = Scratch::new();
    let plan = LaunchPlan::build(
        &scratch.0,
        "127.0.0.1:20042",
        None,
        LaunchSettings::default(),
        &mut LocalLoginStub,
    )
    .unwrap();
    assert_eq!(
        plan.changed_files().collect::<Vec<_>>(),
        ["Baseline.ini", "UserOptions.ini"]
    );
    fs::write(scratch.0.join("UserOptions.ini"), b"changed after planning").unwrap();
    assert!(plan.apply().is_err());
    assert!(!scratch.0.join("Baseline.ini").exists());
    assert!(!scratch.0.join(".friend-launcher-backups").exists());
    assert_eq!(
        fs::read(scratch.0.join("locale/en_us_data.dat")).unwrap(),
        b"old fixture"
    );
    for token in ["", "contains spaces", "line\nbreak", "a=b", "unicode-☃"] {
        assert!(SessionToken::new(token.into()).is_err());
    }
    assert!(
        LaunchSettings {
            width: 1,
            height: u32::MAX
        }
        .validate()
        .is_err()
    );
}

#[cfg(windows)]
#[test]
fn failed_replace_rolls_back_completed_changes() {
    let scratch = Scratch::new();
    let options = scratch.0.join("UserOptions.ini");
    let original = fs::read(&options).unwrap();
    let plan = LaunchPlan::build(
        &scratch.0,
        "127.0.0.1:20042",
        None,
        LaunchSettings::default(),
        &mut LocalLoginStub,
    )
    .unwrap();
    let mut permissions = fs::metadata(&options).unwrap().permissions();
    permissions.set_readonly(true);
    fs::set_permissions(&options, permissions.clone()).unwrap();
    let result = plan.apply();
    permissions.set_readonly(false);
    fs::set_permissions(&options, permissions).unwrap();
    assert!(result.is_err());
    assert!(!scratch.0.join("Baseline.ini").exists());
    assert_eq!(original, fs::read(&options).unwrap());
    assert_eq!(
        original,
        fs::read(plan.backup.join("UserOptions.ini")).unwrap()
    );
}

#[cfg(windows)]
#[test]
fn junction_locale_is_refused() {
    use std::process::Command;
    let scratch = Scratch::new();
    let original = scratch.0.join("locale-original");
    fs::rename(scratch.0.join("locale"), &original).unwrap();
    // Fixed command on generated test-owned paths only; never used in launcher.
    let result = Command::new("cmd.exe")
        .args(["/d", "/c", "mklink", "/J"])
        .arg(scratch.0.join("locale"))
        .arg(&original)
        .output()
        .unwrap();
    assert!(result.status.success());
    let result = LaunchPlan::build(
        &scratch.0,
        "127.0.0.1:20042",
        None,
        LaunchSettings::default(),
        &mut LocalLoginStub,
    );
    fs::remove_dir(scratch.0.join("locale")).unwrap();
    assert!(result.is_err());
    assert_eq!(
        fs::read(original.join("en_us_data.dat")).unwrap(),
        b"old fixture"
    );
}
