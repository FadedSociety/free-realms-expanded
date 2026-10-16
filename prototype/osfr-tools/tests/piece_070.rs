//! Work item 070: quest journal root-tail wire, writer-verified serialization.
//!
//! ORIGINAL facts these vectors rely on (private paths under `../lab`):
//! * Supplied compiled writer `Gateway.Login.ClientPcData` (decompiled at
//!   `lab/decompiled/server/Gateway/Gateway.Login/ClientPcData.cs`, quest block ~1492-1529) writes the
//!   root as `quest count, records..., Unknown(i32), Unknown2(i32), Unknown3(bool),
//!   Unknown4(i32), Unknown5(i32)`.
//! * Preserved native reader `FUN_00940c60` stores the same five values at quest-manager
//!   `+0x10fc, +0x10f4, +0x10f8, +0x1100, +0x1104`; `FUN_00940e60` (the shorter variant) also
//!   writes `+0x1100`. No preserved function reads either offset back from the quest manager, so
//!   the two unnamed i32s are write-only presentation-neutral fields.
//! * `lab/references/rust-gameplay/quests.json` + `quest-journal.bin` are the preserved 7,970-byte
//!   reference span (offset 157832) whose tail is `241, 2513, true, 10, 30` — the same values the
//!   recorded `Customize/PacketSendSelfToClient.json` declares for `ClientQuestData`.
//!
//! RECONSTRUCTED: emitting `10`/`30` for reconstructed campaigns (they are constant in every
//! preserved self input) instead of copying those bytes out of the capture.
//!
//! The module is not registered in `src/lib.rs` (lead-owned). Including the exact source file
//! keeps this target compiling against the real artifact until the lead registers it.

#[path = "../src/restoration/piece_070.rs"]
#[allow(dead_code)]
mod piece_070;

use piece_070::{
    JournalRootTail, MAX_QUESTS, RECOVERED_FIELD_0X1100, RECOVERED_FIELD_0X1104, TAIL_BYTES,
};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;

// ---------------------------------------------------------------------------
// Independent synthetic fixtures in the supplied writer's field order.
// ---------------------------------------------------------------------------

fn i32le(out: &mut Vec<u8>, value: i32) {
    out.extend(value.to_le_bytes());
}

fn empty_reward(out: &mut Vec<u8>) {
    out.push(0);
    out.extend([0u8; 68]);
}

/// One full quest record: `(definition, name, description, required, current)` objectives.
fn quest(out: &mut Vec<u8>, id: i32, progress: f32, objectives: &[(i32, i32, i32, i32, i32)]) {
    i32le(out, id);
    i32le(out, 100);
    i32le(out, 101);
    i32le(out, 0);
    out.push(0);
    out.extend(0i64.to_le_bytes());
    i32le(out, 0);
    out.push(0);
    out.extend(progress.to_le_bytes());
    empty_reward(out);
    i32le(out, objectives.len() as i32);
    for (index, (definition, name, description, required, current)) in objectives.iter().enumerate()
    {
        i32le(out, index as i32 + 1);
        i32le(out, *definition);
        i32le(out, *name);
        i32le(out, *description);
        out.push(0);
        empty_reward(out);
        i32le(out, if current == required { 2 } else { 1 });
        i32le(out, *current);
        i32le(out, *required);
        i32le(out, 0);
        out.push(0);
        i32le(out, 0);
    }
    i32le(out, 0);
    out.push(0);
    out.push(0);
}

fn reference_span() -> Vec<u8> {
    let mut span = Vec::new();
    i32le(&mut span, 2);
    quest(&mut span, 600, 0.25, &[(12001, 501, 502, 4, 1)]);
    quest(&mut span, 601, 0.0, &[]);
    i32le(&mut span, 241);
    i32le(&mut span, 2513);
    span.push(1);
    i32le(&mut span, 10);
    i32le(&mut span, 30);
    span
}

fn opening_tail_bytes(completed: i32, selected: i32) -> Vec<u8> {
    let mut bytes = Vec::new();
    i32le(&mut bytes, completed);
    i32le(&mut bytes, selected);
    bytes.push(0);
    i32le(&mut bytes, 10);
    i32le(&mut bytes, 30);
    bytes
}

#[test]
fn recovered_constants_and_opening_tail_serialization() {
    assert_eq!(RECOVERED_FIELD_0X1100, 10);
    assert_eq!(RECOVERED_FIELD_0X1104, 30);
    assert_eq!(TAIL_BYTES, 17);

    let tail = JournalRootTail::opening_campaign(0, -1, false).expect("fresh campaign tail");
    assert_eq!(tail.completed_quests(), 0);
    assert_eq!(tail.selected_quest_id(), -1);
    assert!(!tail.auto_select_selected_quest());
    assert_eq!(
        tail.to_bytes().expect("serialize").to_vec(),
        opening_tail_bytes(0, -1)
    );

    let mut written = Vec::new();
    tail.write(&mut written).expect("write");
    assert_eq!(written, tail.to_bytes().unwrap());
    assert_eq!(JournalRootTail::parse(&written).unwrap(), tail);

    let selected = JournalRootTail::opening_campaign(4, 910004, false).unwrap();
    assert_eq!(
        selected.to_bytes().unwrap().to_vec(),
        opening_tail_bytes(4, 910004)
    );
    assert_eq!(
        JournalRootTail::parse(&selected.to_bytes().unwrap()).unwrap(),
        selected
    );
}

#[test]
fn locate_reads_the_reference_tail_and_replace_changes_only_those_bytes() {
    let original = reference_span();
    let range = JournalRootTail::locate(&original).expect("locate");
    assert_eq!(range, original.len() - TAIL_BYTES..original.len());

    let parsed = JournalRootTail::from_span(&original).expect("parse the span tail");
    assert_eq!(parsed.completed_quests(), 241);
    assert_eq!(parsed.selected_quest_id(), 2513);
    assert!(parsed.auto_select_selected_quest());
    assert_eq!(parsed.unknown_0x1100(), 10);
    assert_eq!(parsed.unknown_0x1104(), 30);

    let replacement = JournalRootTail::opening_campaign(1, -1, false).unwrap();
    let patched = JournalRootTail::replace_in_span(&original, &replacement).expect("replace");
    assert_eq!(patched.len(), original.len());
    assert_eq!(&patched[..range.start], &original[..range.start]);
    assert_eq!(
        &patched[range.start..],
        opening_tail_bytes(1, -1).as_slice()
    );
    assert_eq!(JournalRootTail::from_span(&patched).unwrap(), replacement);

    // The reference bytes stay decodable by the same independent fixture reader.
    assert_eq!(
        &original[range],
        [241, 0, 0, 0, 209, 9, 0, 0, 1, 10, 0, 0, 0, 30, 0, 0, 0]
    );
}

#[test]
fn malformed_spans_and_tails_are_rejected() {
    let original = reference_span();

    assert!(JournalRootTail::locate(&[]).is_err());
    assert!(JournalRootTail::locate(&original[..original.len() - 1]).is_err());
    let mut trailing = original.clone();
    trailing.push(0);
    assert!(JournalRootTail::locate(&trailing).is_err());

    let mut too_many = Vec::new();
    i32le(&mut too_many, MAX_QUESTS as i32 + 1);
    assert!(JournalRootTail::locate(&too_many).is_err());

    let mut negative_count = Vec::new();
    i32le(&mut negative_count, -1);
    assert!(JournalRootTail::locate(&negative_count).is_err());

    // Invalid bool inside the record region (first record starts after the quest count).
    let quest_start = 4;
    let mut bad_bool = original.clone();
    bad_bool[quest_start + 16] = 2;
    assert!(JournalRootTail::locate(&bad_bool).is_err());

    // Unsupported reward entry type inside the record region.
    let mut bad_reward = reference_span_with_reward_entry(2);
    assert!(JournalRootTail::locate(&bad_reward).is_err());
    bad_reward = reference_span_with_reward_entry(1);
    assert!(JournalRootTail::locate(&bad_reward).is_ok());

    // Non-finite progress.
    let mut bad_progress = original.clone();
    bad_progress[quest_start + 30..quest_start + 34].copy_from_slice(&f32::NAN.to_le_bytes());
    assert!(JournalRootTail::locate(&bad_progress).is_err());

    // Tail-domain and length checks.
    assert!(JournalRootTail::parse(&[0u8; TAIL_BYTES - 1]).is_err());
    assert!(JournalRootTail::parse(&[0u8; TAIL_BYTES + 1]).is_err());
    let mut bad_tail_bool = opening_tail_bytes(0, -1);
    bad_tail_bool[8] = 2;
    assert!(JournalRootTail::parse(&bad_tail_bool).is_err());
    assert!(JournalRootTail::parse(&opening_tail_bytes(-1, -1)).is_err());
    assert!(JournalRootTail::parse(&opening_tail_bytes(0, 0)).is_err());
    assert!(JournalRootTail::opening_campaign(-1, -1, false).is_err());
    assert!(JournalRootTail::opening_campaign(0, 0, false).is_err());
    assert!(JournalRootTail::with_recovered_fields(0, -1, false, 10, 30).is_ok());
    assert!(JournalRootTail::with_recovered_fields(0, -1, false, i32::MIN, i32::MAX).is_ok());
}

/// One quest with a single non-empty reward entry of the given type, then a captured-style tail.
fn reference_span_with_reward_entry(entry_type: i32) -> Vec<u8> {
    let mut span = Vec::new();
    i32le(&mut span, 1);
    i32le(&mut span, 700);
    i32le(&mut span, 100);
    i32le(&mut span, 101);
    i32le(&mut span, 0);
    span.push(0);
    span.extend(0i64.to_le_bytes());
    i32le(&mut span, 0);
    span.push(0);
    span.extend(0.0f32.to_le_bytes());
    // Non-empty RewardBundle: bool, 9*i32, 2*i64, 2*i32, entries, final i32.
    span.push(0);
    span.extend([0u8; 36]);
    span.extend([0u8; 16]);
    span.extend([0u8; 8]);
    i32le(&mut span, 1);
    i32le(&mut span, entry_type);
    span.push(0);
    span.extend([0u8; 24]);
    i32le(&mut span, 0); // Empty reward string.
    i32le(&mut span, 0);
    span.push(0);
    i32le(&mut span, 0); // Final i32.
    i32le(&mut span, 0); // No objectives.
    i32le(&mut span, 0);
    span.push(0);
    span.push(0);
    i32le(&mut span, 241);
    i32le(&mut span, 2513);
    span.push(1);
    i32le(&mut span, 10);
    i32le(&mut span, 30);
    span
}

#[test]
#[ignore = "requires the private preserved reference journal content"]
fn private_reference_tail_is_written_not_copied() {
    let content =
        PathBuf::from(std::env::var_os("OSFR_JOURNAL_REFERENCE").expect("OSFR_JOURNAL_REFERENCE"));
    let metadata: Value =
        serde_json::from_slice(&fs::read(content.join("quests.json")).expect("quests.json"))
            .expect("quests.json json");
    let journal = &metadata["journal"];
    let file = journal["file"].as_str().expect("file");
    let length = journal["length"].as_u64().expect("length") as usize;
    // The reference file is the exact journal span; `offset` locates it inside the self payload.
    let span = fs::read(content.join(file)).expect("reference content file");
    assert_eq!(
        journal["sha256"].as_str().expect("sha256"),
        format!("{:x}", Sha256::digest(&span))
    );
    assert_eq!(span.len(), length);
    let span = span.as_slice();

    let range = JournalRootTail::locate(span).expect("locate the preserved reference span");
    let captured = JournalRootTail::parse(&span[range.clone()]).expect("parse captured tail");
    assert_eq!(captured.completed_quests(), 241);
    assert_eq!(captured.selected_quest_id(), 2513);
    assert!(captured.auto_select_selected_quest());
    assert_eq!(captured.unknown_0x1100(), RECOVERED_FIELD_0X1100);
    assert_eq!(captured.unknown_0x1104(), RECOVERED_FIELD_0X1104);

    let replacement = JournalRootTail::opening_campaign(1, -1, false).unwrap();
    let patched = JournalRootTail::replace_in_span(span, &replacement).expect("replace tail");
    assert_eq!(patched.len(), span.len());
    assert_eq!(&patched[..range.start], &span[..range.start]);
    assert_eq!(
        &patched[range.start..],
        replacement.to_bytes().unwrap().as_slice()
    );
}
