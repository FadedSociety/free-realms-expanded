//! Builder tests for work item 079, "World terrain readers".
//!
//! ORIGINAL evidence this file exercises (all outside Git, under `../lab`):
//! * `lab/extracted/client-assets/pack_*/**.gcnk` - the 2,344 preserved terrain chunks exported
//!   from the 22 supplied asset packs that carry GCNK entries.
//! * `lab/references/ForgeLightToolkit` revision `9c06714c5d79d9152c322814bc904abad944e06e`
//!   (MIT, `SOURCE.json` hashes every compiled reader file) - the community reader the project
//!   already runs on these chunks.
//!
//! The module is not registered in `src/lib.rs` (a shared file owned by the lead), so this target
//! compiles the exact source with `#[path]`, as the other piece-NNN targets do. Lead integration
//! step: register `pub mod piece_079;` under `src/restoration/` and drop the include.
//!
//! These tests re-derive the chunk inventory from the preserved originals and assert the declared
//! reader-coverage constants. They never claim decoded collision, decoded detail masks or native
//! gameplay behaviour.

#[path = "../src/restoration/piece_079.rs"]
mod piece_079;

use piece_079::{
    COLLISION_DECODED, ChunkHeader, DETAIL_MASK_DECODED, GCNK_MAGIC, PINNED_ABORT_CHUNKS,
    PINNED_READER_MAX_VERSION, PINNED_READER_MIN_VERSION, PINNED_SHORT_PARSE_CHUNKS,
    PINNED_SURVEY_FAILED, PINNED_SURVEY_OK, PLACEHOLDER_CHUNK_COUNT, PLACEHOLDER_TEXT,
    PRESERVED_CHUNK_COUNT, SHORT_PARSE_CHUNKS, TerrainSurvey, VARIANT_RECOVERED_CHUNKS,
    VARIANT_RECOVERED_V3, VARIANT_RECOVERED_V4, VARIANT_RECOVERED_V4_COMPLETE,
    VARIANT_RESIDUAL_CHUNKS, VERSION_HISTOGRAM, classify,
};
use serde_json::Value;
use std::path::{Path, PathBuf};

/// The recorded 079 run under `../lab/reports`.
const REPORT: &str = "piece-079-20261009T0455Z";

/// `../lab/extracted/client-assets` relative to `prototype/osfr-tools`.
fn extracted_assets() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../lab/extracted/client-assets")
}

fn report_file(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../lab/reports")
        .join(REPORT)
        .join(relative)
}

fn survey_rows(relative: &str) -> Vec<Value> {
    let path = report_file(relative);
    let bytes = std::fs::read(&path).expect("read recorded survey");
    let document: Value = serde_json::from_slice(&bytes).expect("parse recorded survey");
    document["files"].as_array().expect("files array").clone()
}

fn row_is_ok(row: &Value) -> bool {
    row["ok"] == Value::Bool(true)
}

fn row_is_aborted_chunk(row: &Value) -> bool {
    !row_is_ok(row) && row["header_magic"] == Value::String("GCNK".to_string())
}

fn gcnk_files(root: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(directory) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&directory) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|ext| ext == "gcnk") {
                found.push(path);
            }
        }
    }
    found.sort();
    found
}

#[test]
fn classifies_the_preserved_header_shapes() {
    let mut header = Vec::from(GCNK_MAGIC);
    header.extend_from_slice(&5_i32.to_le_bytes());
    assert_eq!(classify(&header), ChunkHeader::Gcnk { version: 5 });

    let mut header = Vec::from(GCNK_MAGIC);
    header.extend_from_slice(&6_i32.to_le_bytes());
    assert_eq!(classify(&header), ChunkHeader::Gcnk { version: 6 });

    assert_eq!(classify(&PLACEHOLDER_TEXT), ChunkHeader::PlaceholderText);
    assert_eq!(
        classify(b"NOPE\x02\x00\x00\x00"),
        ChunkHeader::UnknownMagic(*b"NOPE")
    );
    assert_eq!(classify(b"GC"), ChunkHeader::Truncated { bytes: 2 });
    assert_eq!(classify(b""), ChunkHeader::Truncated { bytes: 0 });
}

#[test]
fn survey_accumulates_versions_and_placeholders() {
    let mut survey = TerrainSurvey::default();
    survey.classify(b"GCNK\x05\x00\x00\x00");
    survey.classify(b"GCNK\x05\x00\x00\x00");
    survey.classify(b"GCNK\x06\x00\x00\x00");
    survey.classify(&PLACEHOLDER_TEXT);

    assert_eq!(survey.decoded_chunks(), 3);
    assert_eq!(survey.placeholders(), 1);
    assert_eq!(survey.versions().get(&5), Some(&2));
    assert_eq!(survey.versions().get(&6), Some(&1));
    assert_eq!(survey.total(), 4);
    assert!(survey.unsupported_versions().is_empty());
    assert!(survey.unknown_magic().is_empty());

    // Version 7 is outside the pinned reader's declared window (0..=6).
    survey.classify(b"GCNK\x07\x00\x00\x00");
    assert_eq!(survey.unsupported_versions(), vec![7]);
    assert!(!survey.within_pinned_reader());
    survey.classify(b"NOPE\x01\x00\x00\x00");
    assert_eq!(survey.unknown_magic(), vec![*b"NOPE"]);
}

#[test]
fn declared_coverage_matches_the_pinned_reader() {
    assert_eq!(PINNED_READER_MIN_VERSION, 0);
    assert_eq!(PINNED_READER_MAX_VERSION, 6);
    assert!(
        !COLLISION_DECODED,
        "the pinned ForgeLightToolkit LoadCollision is a TODO placeholder"
    );
    assert!(
        !DETAIL_MASK_DECODED,
        "the pinned ForgeLightToolkit skips the detail mask / height map bytes"
    );

    let decoded: usize = VERSION_HISTOGRAM.iter().map(|(_, count)| count).sum();
    assert_eq!(decoded + PLACEHOLDER_CHUNK_COUNT, PRESERVED_CHUNK_COUNT);
    assert!(
        VERSION_HISTOGRAM
            .iter()
            .all(
                |(version, _)| (PINNED_READER_MIN_VERSION..=PINNED_READER_MAX_VERSION)
                    .contains(version)
            ),
        "every preserved chunk version sits inside the pinned reader's window"
    );
}

#[test]
fn preserved_world_matches_the_recovered_inventory() {
    let root = extracted_assets();
    let files = gcnk_files(&root);
    assert_eq!(
        files.len(),
        PRESERVED_CHUNK_COUNT,
        "preserved GCNK count under {}",
        root.display()
    );

    let mut survey = TerrainSurvey::default();
    let mut placeholders = Vec::new();
    for path in &files {
        let bytes = std::fs::read(path).expect("read preserved chunk");
        match classify(&bytes) {
            ChunkHeader::PlaceholderText => placeholders.push(path.clone()),
            _ => {}
        }
        survey.classify(&bytes);
    }

    assert_eq!(survey.placeholders(), PLACEHOLDER_CHUNK_COUNT);
    assert_eq!(survey.truncated(), 0);
    assert_eq!(
        survey
            .versions()
            .iter()
            .map(|(v, c)| (*v, *c))
            .collect::<Vec<_>>(),
        VERSION_HISTOGRAM.to_vec()
    );
    assert!(survey.unsupported_versions().is_empty());
    assert!(survey.unknown_magic().is_empty());
    assert_eq!(survey.total(), PRESERVED_CHUNK_COUNT);
    assert_eq!(piece_079::validate(), Ok(()));

    // Every shipped placeholder is the same five-byte ASCII stub, not a truncated chunk.
    for path in placeholders {
        assert_eq!(std::fs::read(&path).unwrap(), PLACEHOLDER_TEXT);
    }
}

#[test]
fn pinned_survey_matches_the_declared_gap_numbers() {
    let rows = survey_rows("survey/reader-survey.json");
    assert_eq!(rows.len(), PRESERVED_CHUNK_COUNT);

    let ok = rows.iter().filter(|row| row_is_ok(row)).count();
    assert_eq!(ok, PINNED_SURVEY_OK);
    assert_eq!(rows.len() - ok, PINNED_SURVEY_FAILED);

    let aborts = rows.iter().filter(|row| row_is_aborted_chunk(row)).count();
    assert_eq!(aborts, PINNED_ABORT_CHUNKS);

    let not_implemented = rows
        .iter()
        .filter(|row| {
            row["error"]
                .as_str()
                .is_some_and(|error| error.starts_with("NotImplementedException"))
        })
        .count();
    assert_eq!(not_implemented, PINNED_ABORT_CHUNKS);
}

#[test]
fn variant_survey_recovers_the_documented_share_and_keeps_the_v4_gap_open() {
    let pinned = survey_rows("survey/reader-survey.json");
    let variant = survey_rows("survey-variant/reader-survey-variant.json");
    assert_eq!(variant.len(), PRESERVED_CHUNK_COUNT);

    let aborted: Vec<&Value> = pinned
        .iter()
        .filter(|row| row_is_aborted_chunk(row))
        .collect();
    assert_eq!(aborted.len(), PINNED_ABORT_CHUNKS);

    let variant_by_name = |name: &str| {
        variant
            .iter()
            .find(|row| row["name"] == Value::String(name.to_string()))
            .expect("variant row")
    };

    let recovered: Vec<&Value> = aborted
        .iter()
        .filter(|row| row_is_ok(variant_by_name(row["name"].as_str().unwrap())))
        .copied()
        .collect();
    assert_eq!(recovered.len(), VARIANT_RECOVERED_CHUNKS);
    assert_eq!(aborted.len() - recovered.len(), VARIANT_RESIDUAL_CHUNKS);

    // Every recovered chunk read at least one runtime object where the pinned reader threw; the
    // recovered v3 chunks then consume their whole declared decompressed chunk, which is the
    // structural evidence for the variable-length terrain object name field. The name text itself
    // reads back empty in this corpus, so it is not the evidence.
    let mut named_objects = 0_u64;
    let mut recovered_v3 = 0;
    let mut recovered_v4 = 0;
    let mut recovered_v4_complete = 0;
    for row in &recovered {
        let variant_row = variant_by_name(row["name"].as_str().unwrap());
        assert!(
            variant_row["terrain_object_count"].as_u64().unwrap_or(0) > 0,
            "{} decoded without reading a runtime object",
            row["name"]
        );
        named_objects += variant_row["terrain_object_named"].as_u64().unwrap_or(0);
        match variant_row["version"].as_i64() {
            Some(3) => {
                recovered_v3 += 1;
                assert_eq!(
                    variant_row["consumed_all"],
                    Value::Bool(true),
                    "{} is a recovered v3 chunk that did not consume its whole chunk",
                    row["name"]
                );
            }
            Some(4) => {
                recovered_v4 += 1;
                if variant_row["consumed_all"] == Value::Bool(true) {
                    recovered_v4_complete += 1;
                }
            }
            other => panic!("unexpected recovered version {other:?}"),
        }
    }
    assert_eq!(recovered_v3, VARIANT_RECOVERED_V3);
    assert_eq!(recovered_v4, VARIANT_RECOVERED_V4);
    assert_eq!(recovered_v4_complete, VARIANT_RECOVERED_V4_COMPLETE);
    assert_eq!(
        named_objects, 0,
        "the recovered version <= 4 terrain object names read back empty in this corpus"
    );

    let residual: Vec<&Value> = aborted
        .iter()
        .map(|row| variant_by_name(row["name"].as_str().unwrap()))
        .filter(|row| !row_is_ok(row))
        .collect();
    assert_eq!(residual.len(), VARIANT_RESIDUAL_CHUNKS);
    assert!(residual.iter().all(|row| {
        row["error"]
            .as_str()
            .is_some_and(|error| error.starts_with("EndOfStreamException"))
    }));

    let short_parse: Vec<&Value> = variant
        .iter()
        .filter(|row| row_is_ok(row) && row["consumed_all"] == Value::Bool(false))
        .collect();
    assert_eq!(short_parse.len(), SHORT_PARSE_CHUNKS);
    assert!(
        short_parse
            .iter()
            .all(|row| row["version"] == Value::from(4))
    );
    assert!(
        short_parse
            .iter()
            .all(|row| row["chunk_consumed"].as_u64() < row["chunk_length"].as_u64())
    );

    let pinned_short_parse = short_parse
        .iter()
        .filter(|row| {
            row_is_ok(
                &pinned
                    .iter()
                    .find(|p| p["name"] == row["name"])
                    .expect("pinned row"),
            )
        })
        .count();
    assert_eq!(pinned_short_parse, PINNED_SHORT_PARSE_CHUNKS);
}
