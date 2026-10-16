use osfr_tools::{gateway::Gateway, startup_snapshot::StartupSnapshot};
use std::{fs, path::PathBuf};

#[test]
fn empty_snapshot_serializes_as_typed_json_and_loads() {
    let path =
        std::env::temp_dir().join(format!("osfr-startup-snapshot-{}.json", std::process::id()));
    let snapshot = StartupSnapshot::empty();
    fs::write(&path, serde_json::to_vec(&snapshot).unwrap()).unwrap();
    let loaded = StartupSnapshot::load(&path).unwrap();
    assert_eq!(loaded.schema_version, 1);
    assert_eq!(loaded.world.npc_spawns.len(), 23);
    assert_eq!(loaded.catalogues.effect_tags.len(), 2);
    fs::remove_file(path).unwrap();
}

#[test]
#[ignore = "requires private startup reference via OSFR_GATEWAY_REFERENCE"]
fn exports_typed_snapshot_to_lab_and_matches_all_45_builders() {
    let reference = PathBuf::from(
        std::env::var("OSFR_GATEWAY_REFERENCE").expect("private reference path required"),
    );
    let lab_root = reference
        .parent()
        .and_then(std::path::Path::parent)
        .unwrap();
    let output = lab_root.join("content/startup-snapshot/startup.json");
    Gateway::reference_state(&reference).expect("capture manifest and hashes must validate");
    let snapshot = StartupSnapshot::export_from_reference(&reference, &output).unwrap();
    assert_eq!(snapshot.world.quick_chat.len(), 2470);
    assert_eq!(snapshot.world.points_of_interest.len(), 149);
    assert_eq!(snapshot.world.npc_spawns.len(), 23);
    let loaded = StartupSnapshot::load(&output).unwrap();
    assert_eq!(loaded.schema_version, 1);
    assert!(output.starts_with(lab_root));
    println!(
        "wrote {} bytes to {}",
        fs::metadata(&output).unwrap().len(),
        output.display()
    );
}
