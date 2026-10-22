//! T431: the stress harness (scripts/stress/run.py) runs from a playtest bundle, fights only
//! always-on field enemies, counts knockouts apart from protocol errors, scores single-sample
//! purchases as unmeasured, and the bundle check's `--bin-dir` refuses an incomplete set.
use std::{path::Path, process::Command};

#[test]
fn python_stress_harness_scores_a_bundle_run() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let python = std::env::var_os("PYTHON").unwrap_or_else(|| "python".into());
    let status = Command::new(python)
        .arg("-I")
        .arg("tests/test_stress_bundle.py")
        .current_dir(&root)
        .status()
        .expect("run the Python stress-harness test");
    assert!(status.success(), "the stress harness regressed");
}
