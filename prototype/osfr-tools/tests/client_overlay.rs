//! L024: the launcher client overlay (scripts/bundle/client_overlay.py) holds exactly
//! the harness's pinned creator files at the paths the client opens, or nothing.
use std::{path::Path, process::Command};

#[test]
fn python_client_overlay_is_pinned_and_byte_identical() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let python = std::env::var_os("PYTHON").unwrap_or_else(|| "python".into());
    let status = Command::new(python)
        .arg("-I")
        .arg("tests/test_client_overlay.py")
        .current_dir(&root)
        .status()
        .expect("run the Python client-overlay test");
    assert!(status.success(), "the client overlay regressed");
}
