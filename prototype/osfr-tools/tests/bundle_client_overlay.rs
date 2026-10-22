//! T575: the playtest bundle serves the pinned creator overlay as its client patch
//! folder (tests/test_bundle_client_overlay.py), and osfr-login loads the bundled
//! login-resources with the creator allow-list on (T444). Both need the lab asset
//! cache; without it they pass without checking, as the Python test skips.
use osfr_tools::identity::Resources;
use std::{ffi::OsString, fs, path::PathBuf, process::Command};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn python() -> OsString {
    std::env::var_os("PYTHON").unwrap_or_else(|| "python".into())
}

#[test]
fn python_bundle_overlay_is_pinned_or_not_built() {
    let status = Command::new(python())
        .arg("-I")
        .arg("tests/test_bundle_client_overlay.py")
        .current_dir(root())
        .status()
        .expect("run the Python bundle-overlay test");
    assert!(status.success(), "the bundle's client overlay regressed");
}

#[test]
fn bundled_login_resources_turn_on_the_creator_allow_list() {
    if !root().join("../lab/references/asset-cache").is_dir() {
        eprintln!("skipped: no lab asset cache");
        return;
    }
    let bundle = std::env::temp_dir().join(format!("osfr-t575-{}", std::process::id()));
    let _ = fs::remove_dir_all(&bundle);
    fs::create_dir_all(&bundle).unwrap();
    // make_bundle.main's two steps, in its order.
    let status = Command::new(python())
        .args(["-I", "-c"])
        .arg(
            "import pathlib, sys; sys.path.insert(0, sys.argv[1]); import make_bundle; \
             t = pathlib.Path(sys.argv[2]); make_bundle.copy_fixed_inputs(t); \
             make_bundle.copy_client_overlay(t)",
        )
        .arg(root().join("scripts/bundle"))
        .arg(&bundle)
        .status();
    let resources = Resources::load(&bundle.join("login-resources"));
    let _ = fs::remove_dir_all(&bundle);
    assert!(
        status.expect("run make_bundle").success(),
        "bundle inputs failed"
    );
    assert!(
        resources
            .expect("bundled login-resources load")
            .filters_looks(),
        "osfr-login would create any look from this bundle"
    );
}
