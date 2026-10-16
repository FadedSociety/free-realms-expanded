use osfr_tools::opening::{OpeningJournalEntry, OpeningJournalObjective};
use osfr_tools::opening_journal::OpeningJournal;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::fs;
use std::io::{Cursor, Read};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT: AtomicU64 = AtomicU64::new(0);

// Independent fixture follows supplied ClientPcData.Serialize field order.
fn old_journal() -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend(1i32.to_le_bytes());
    for value in [600i32, 601, 602, 603] {
        bytes.extend(value.to_le_bytes());
    }
    bytes.push(0);
    bytes.extend(123i64.to_le_bytes());
    bytes.extend(0i32.to_le_bytes());
    bytes.push(0);
    bytes.extend(0.0f32.to_le_bytes());
    bytes.extend([0; 69]); // Empty RewardBundleBase.
    bytes.extend(0i32.to_le_bytes()); // No objectives in inherited fixture.
    bytes.extend(0i32.to_le_bytes());
    bytes.extend([0, 0]);
    bytes.extend(241i32.to_le_bytes()); // Inherited completed count.
    bytes.extend(600i32.to_le_bytes()); // Inherited selected quest.
    bytes.push(1);
    bytes.extend(10i32.to_le_bytes());
    bytes.extend(30i32.to_le_bytes());
    bytes
}

struct Reference {
    directory: PathBuf,
    journal: Vec<u8>,
    message: Vec<u8>,
}
impl Reference {
    fn new() -> Self {
        let directory = std::env::temp_dir().join(format!(
            "osfr-opening-journal-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&directory).unwrap();
        let journal = old_journal();
        let mut message = vec![5, 0, 1, 0, 0, 0, 0, 12, 0, 0, 0, 0, 0];
        message.extend([0x91; 16]);
        message.extend(&journal);
        message.extend([0x92; 11]);
        let length = message.len();
        message[3..7].copy_from_slice(&((length - 7) as u32).to_le_bytes());
        message[9..13].copy_from_slice(&((length - 13) as u32).to_le_bytes());
        let reference = Self {
            directory,
            journal,
            message,
        };
        reference.write();
        reference
    }
    fn write(&self) {
        fs::write(self.directory.join("quest-journal.bin"), &self.journal).unwrap();
        fs::write(self.directory.join("quests.json"), serde_json::to_vec(&json!({
            "schema_version":1,
            "journal":{"file":"quest-journal.bin","offset":29,"length":self.journal.len(),"sha256":format!("{:x}",Sha256::digest(&self.journal))}
        })).unwrap()).unwrap();
    }
    fn load(&self) -> OpeningJournal {
        OpeningJournal::load(&self.directory).unwrap()
    }
}
impl Drop for Reference {
    fn drop(&mut self) {
        if self.directory.parent() == Some(std::env::temp_dir().as_path()) {
            let _ = fs::remove_dir_all(&self.directory);
        }
    }
}

fn entry() -> OpeningJournalEntry {
    OpeningJournalEntry {
        quest_id: 900001,
        title_id: 501,
        description_id: 502,
        title: "Synthetic opening".into(),
        description: "Synthetic fixture".into(),
        accepted: true,
        completed: false,
        rewarded: false,
        objectives: vec![
            OpeningJournalObjective {
                id: 12001,
                name_id: 503,
                description_id: 504,
                description: "First".into(),
                required: 4,
                count: 1,
            },
            OpeningJournalObjective {
                id: 12002,
                name_id: 505,
                description_id: 506,
                description: "Second".into(),
                required: 1,
                count: 0,
            },
        ],
    }
}

// Deliberately independent cursor decoder from the native reader field order.
fn i32le(reader: &mut Cursor<&[u8]>) -> i32 {
    let mut b = [0; 4];
    reader.read_exact(&mut b).unwrap();
    i32::from_le_bytes(b)
}
fn f32le(reader: &mut Cursor<&[u8]>) -> f32 {
    f32::from_bits(i32le(reader) as u32)
}
fn byte(reader: &mut Cursor<&[u8]>) -> u8 {
    let mut b = [0];
    reader.read_exact(&mut b).unwrap();
    b[0]
}
fn skip(reader: &mut Cursor<&[u8]>, count: usize) {
    let mut b = vec![0; count];
    reader.read_exact(&mut b).unwrap();
}
fn reward(reader: &mut Cursor<&[u8]>) {
    assert_eq!(byte(reader), 0);
    for _ in 0..9 {
        assert_eq!(i32le(reader), 0);
    }
    skip(reader, 16);
    assert_eq!(i32le(reader), 0);
    assert_eq!(i32le(reader), 0);
    assert_eq!(i32le(reader), 0); // Array count.
    assert_eq!(i32le(reader), 0);
}

#[derive(Debug, PartialEq)]
struct DecodedQuest {
    id: i32,
    title: i32,
    description: i32,
    progress: f32,
    objectives: Vec<(i32, i32, i32, i32, i32, i32)>,
    objective_texts: Vec<(i32, i32)>,
}
fn decode_quest(reader: &mut Cursor<&[u8]>) -> DecodedQuest {
    let id = i32le(reader);
    let title = i32le(reader);
    let description = i32le(reader);
    assert_eq!(i32le(reader), 0);
    assert_eq!(byte(reader), 0);
    skip(reader, 8);
    assert_eq!(i32le(reader), 0);
    assert_eq!(byte(reader), 0);
    let progress = f32le(reader);
    reward(reader);
    let count = i32le(reader);
    let mut objectives = Vec::new();
    let mut objective_texts = Vec::new();
    for _ in 0..count {
        let key = i32le(reader);
        let definition = i32le(reader);
        objective_texts.push((i32le(reader), i32le(reader)));
        assert_eq!(byte(reader), 0);
        reward(reader);
        let state = i32le(reader);
        let current = i32le(reader);
        let required = i32le(reader);
        let unknown8 = i32le(reader);
        assert_eq!(byte(reader), 0);
        assert_eq!(i32le(reader), 0);
        objectives.push((key, definition, state, current, required, unknown8));
    }
    assert_eq!(i32le(reader), 0);
    assert_eq!(byte(reader), 0);
    assert_eq!(byte(reader), 0);
    DecodedQuest {
        id,
        title,
        description,
        progress,
        objectives,
        objective_texts,
    }
}

fn write_presentation(reference: &Reference) -> serde_json::Value {
    let opening = json!({"stages":[{"id":900001,"objectives":[
        {"id":12001,"name_id":503,"description_id":504},
        {"id":12002,"name_id":505,"description_id":506}
    ]}]});
    let bytes = serde_json::to_vec(&opening).unwrap();
    fs::write(reference.directory.join("opening.json"), &bytes).unwrap();
    let value = json!({"schema_version":1,"locale":8,"opening_sha256":format!("{:x}",Sha256::digest(bytes)),
    "notice":"Reconstructed local objective labels.","entries":[{
        "quest_id":900001,"objective_id":12001,"source_name_id":503,"source_description_id":504,
        "name_id":700001,"description_id":700002,"name":"Switch to Chef","description":"Choose Chef in the Jobs menu."
    }]});
    fs::write(
        reference.directory.join("journal-text.json"),
        serde_json::to_vec(&value).unwrap(),
    )
    .unwrap();
    value
}

#[test]
fn optional_text_changes_only_label_fields_in_full_and_add_records() {
    let reference = Reference::new();
    let original = reference.load();
    let active = entry();
    let baseline = original
        .patch_self(&reference.message, std::slice::from_ref(&active))
        .unwrap();
    write_presentation(&reference);
    let journal = reference.load();
    let result = journal
        .patch_self(&reference.message, std::slice::from_ref(&active))
        .unwrap();
    assert_eq!(result.len(), baseline.len());
    let mut decoded = Cursor::new(&result[29..result.len() - 11]);
    assert_eq!(i32le(&mut decoded), 1);
    let mapped = decode_quest(&mut decoded);
    assert_eq!(mapped.objective_texts, vec![(700001, 700002), (505, 506)]);
    assert_eq!(
        mapped.objectives,
        vec![(1, 12001, 1, 1, 4, 0), (2, 12002, 1, 0, 1, 0)]
    );
    let mut expected = baseline.clone();
    // Independent full record offset: root count4 + quest fixed prefix107 +
    // objective ordinal/definition8. Only the two text IDs may differ.
    let text_offset = 29 + 4 + 107 + 8;
    expected[text_offset..text_offset + 4].copy_from_slice(&700001i32.to_le_bytes());
    expected[text_offset + 4..text_offset + 8].copy_from_slice(&700002i32.to_le_bytes());
    assert_eq!(result, expected);
    let mut inactive = active.clone();
    inactive.accepted = false;
    inactive.objectives[0].count = 0;
    let add = journal
        .updates(&[inactive], std::slice::from_ref(&active))
        .unwrap();
    assert_eq!(subtype(&add[0]), 3);
    assert_eq!(
        decode_quest(&mut Cursor::new(&add[0][6..])).objective_texts,
        mapped.objective_texts
    );
    let mut counted = active.clone();
    counted.objectives[0].count += 1;
    assert_eq!(
        journal
            .updates(
                std::slice::from_ref(&active),
                std::slice::from_ref(&counted)
            )
            .unwrap(),
        original.updates(&[active], &[counted]).unwrap()
    );
    let mut mismatch = entry();
    mismatch.objectives[0].name_id += 1;
    assert!(journal.patch_self(&reference.message, &[mismatch]).is_err());
}

#[test]
fn optional_text_rejects_wrong_content_objective_ids_locale_and_conflicting_strings() {
    let reference = Reference::new();
    let valid = write_presentation(&reference);
    for case in 0..10 {
        let mut bad = valid.clone();
        match case {
            0 => bad["opening_sha256"] = json!("00"),
            1 => bad["locale"] = json!(4),
            2 => bad["schema_version"] = json!(2),
            3 => bad["entries"][0]["source_name_id"] = json!(0),
            4 => bad["entries"][0]["objective_id"] = json!(999),
            5 => bad["entries"][0]["name_id"] = json!(0),
            6 => bad["entries"][0]["name_id"] = json!(5_103_268),
            7 => bad["entries"][0]["description_id"] = bad["entries"][0]["name_id"].clone(),
            8 => bad["entries"][0]["name"] = json!("bad\nlabel"),
            9 => {
                let duplicate = bad["entries"][0].clone();
                bad["entries"].as_array_mut().unwrap().push(duplicate);
            }
            _ => unreachable!(),
        }
        fs::write(
            reference.directory.join("journal-text.json"),
            serde_json::to_vec(&bad).unwrap(),
        )
        .unwrap();
        assert!(
            OpeningJournal::load(&reference.directory).is_err(),
            "case {case}"
        );
    }
    let mut opening: serde_json::Value =
        serde_json::from_slice(&fs::read(reference.directory.join("opening.json")).unwrap())
            .unwrap();
    opening["stages"][0]["objectives"][1]["name_id"] = json!(0);
    let bytes = serde_json::to_vec(&opening).unwrap();
    let mut incomplete = valid;
    incomplete["opening_sha256"] = json!(format!("{:x}", Sha256::digest(&bytes)));
    fs::write(reference.directory.join("opening.json"), bytes).unwrap();
    fs::write(
        reference.directory.join("journal-text.json"),
        serde_json::to_vec(&incomplete).unwrap(),
    )
    .unwrap();
    assert!(OpeningJournal::load(&reference.directory).is_err());
}

fn subtype(packet: &[u8]) -> i32 {
    assert_eq!(&packet[..2], &[49, 0]);
    i32::from_le_bytes(packet[2..6].try_into().unwrap())
}

#[test]
fn fresh_self_removes_inherited_quests_preserves_other_fields_and_native_current_counter() {
    let reference = Reference::new();
    let journal = reference.load();
    let active = entry();
    let mut future = entry();
    future.quest_id += 1;
    future.accepted = false;
    future.objectives[0].count = 0;
    let patched = journal
        .patch_self(&reference.message, &[active.clone(), future])
        .unwrap();
    assert_eq!(&patched[13..29], &reference.message[13..29]);
    assert_eq!(
        &patched[patched.len() - 11..],
        &reference.message[reference.message.len() - 11..]
    );
    assert_eq!(
        u32::from_le_bytes(patched[3..7].try_into().unwrap()) as usize,
        patched.len() - 7
    );
    assert_eq!(
        u32::from_le_bytes(patched[9..13].try_into().unwrap()) as usize,
        patched.len() - 13
    );
    let mut reader = Cursor::new(&patched[29..patched.len() - 11]);
    assert_eq!(i32le(&mut reader), 1);
    let decoded = decode_quest(&mut reader);
    assert_eq!(
        (decoded.id, decoded.title, decoded.description),
        (900001, 501, 502)
    );
    assert_eq!(decoded.progress, 0.125);
    assert_eq!(
        decoded.objectives,
        vec![(1, 12001, 1, 1, 4, 0), (2, 12002, 1, 0, 1, 0)]
    );
    assert_eq!(i32le(&mut reader), 0); // Completed count reset for fresh campaign.
    assert_eq!(i32le(&mut reader), 900001);
    assert_eq!(byte(&mut reader), 0);
    assert_eq!(i32le(&mut reader), 10);
    assert_eq!(i32le(&mut reader), 30);
    assert_eq!(reader.position() as usize, reader.get_ref().len());
    assert!(journal.patch_self(&patched, &[active]).is_err()); // Cannot patch twice.
}

#[test]
fn empty_campaign_has_no_quests_or_inherited_selection() {
    let reference = Reference::new();
    let patched = reference
        .load()
        .patch_self(&reference.message, &[])
        .unwrap();
    let mut reader = Cursor::new(&patched[29..patched.len() - 11]);
    assert_eq!(i32le(&mut reader), 0);
    assert_eq!(i32le(&mut reader), 0);
    assert_eq!(i32le(&mut reader), -1);
    assert_eq!(byte(&mut reader), 0);
    assert_eq!(i32le(&mut reader), 10);
    assert_eq!(i32le(&mut reader), 30);
    assert_eq!(reader.position() as usize, reader.get_ref().len());
}

#[test]
fn acceptance_counter_completion_and_turnin_use_native_runtime_objective_keys() {
    let reference = Reference::new();
    let journal = reference.load();
    let active = entry();
    let mut pending = active.clone();
    pending.accepted = false;
    pending.objectives[0].count = 0;
    let added = journal
        .updates(&[pending], std::slice::from_ref(&active))
        .unwrap();
    assert_eq!(added.len(), 1);
    assert_eq!(subtype(&added[0]), 3);
    let mut reader = Cursor::new(&added[0][6..]);
    assert_eq!(
        decode_quest(&mut reader).objectives[0],
        (1, 12001, 1, 1, 4, 0)
    );
    assert_eq!(byte(&mut reader), 1);
    assert_eq!(reader.position() as usize, reader.get_ref().len());
    let mut progressed = active.clone();
    progressed.objectives[0].count = 4;
    let updates = journal
        .updates(
            std::slice::from_ref(&active),
            std::slice::from_ref(&progressed),
        )
        .unwrap();
    assert_eq!(
        updates.iter().map(|p| subtype(p)).collect::<Vec<_>>(),
        vec![9, 10]
    );
    assert_eq!(updates[0].len(), 27);
    assert_eq!(updates[1].len(), 23);
    let mut reader = Cursor::new(&updates[0][6..]);
    assert_eq!(i32le(&mut reader), 900001);
    // The full QuestAdd tuple above has key1, definition12001. Native
    // 00BAF380 / 00BAB950 normalize the live hash key to definition12001.
    assert_eq!(i32le(&mut reader), 12001);
    assert_eq!(i32le(&mut reader), 4);
    assert_eq!(f32le(&mut reader), 0.5);
    assert_eq!(byte(&mut reader), 0);
    assert_eq!(i32le(&mut reader), 0);
    assert_eq!(reader.position() as usize, reader.get_ref().len());
    let mut reader = Cursor::new(&updates[1][6..]);
    assert_eq!(i32le(&mut reader), 900001);
    assert_eq!(i32le(&mut reader), 12001);
    assert_eq!(f32le(&mut reader), 0.5);
    assert_eq!(i32le(&mut reader), 0);
    assert_eq!(byte(&mut reader), 0);
    assert_eq!(reader.position() as usize, reader.get_ref().len());
    let mut complete = progressed.clone();
    complete.objectives[1].count = 1;
    complete.completed = true;
    let second = journal
        .updates(&[progressed], std::slice::from_ref(&complete))
        .unwrap();
    assert_eq!(
        second.iter().map(|p| subtype(p)).collect::<Vec<_>>(),
        vec![9, 10]
    );
    for packet in &second {
        assert_eq!(&packet[10..14], &12002i32.to_le_bytes());
    }
    let mut turned_in = complete.clone();
    turned_in.rewarded = true;
    let updates = journal
        .updates(&[complete], std::slice::from_ref(&turned_in))
        .unwrap();
    assert_eq!(
        updates.iter().map(|p| subtype(p)).collect::<Vec<_>>(),
        vec![4, 12]
    );
    assert_eq!(&updates[0][6..], &900001i32.to_le_bytes());
    assert_eq!(&updates[1][6..], &1i32.to_le_bytes());
    assert!(
        journal
            .updates(
                std::slice::from_ref(&turned_in),
                std::slice::from_ref(&turned_in)
            )
            .unwrap()
            .is_empty()
    );
    let patched = journal
        .patch_self(&reference.message, &[turned_in])
        .unwrap();
    let mut reader = Cursor::new(&patched[29..patched.len() - 11]);
    assert_eq!(i32le(&mut reader), 0);
    assert_eq!(i32le(&mut reader), 1);
    assert_eq!(i32le(&mut reader), -1);
}

#[test]
fn invalid_reference_packet_and_regressive_state_are_rejected() {
    let mut reference = Reference::new();
    fs::write(reference.directory.join("quest-journal.bin"), b"corrupt").unwrap();
    assert!(OpeningJournal::load(&reference.directory).is_err());
    reference.write();
    let mut trailing = reference.journal.clone();
    trailing.push(0);
    reference.journal = trailing;
    reference.write();
    assert!(OpeningJournal::load(&reference.directory).is_err());
    reference.journal = old_journal();
    reference.write();
    let journal = reference.load();
    let mut bad = reference.message.clone();
    bad[30] ^= 1;
    assert!(journal.patch_self(&bad, &[entry()]).is_err());
    bad = reference.message.clone();
    bad[3] ^= 1;
    assert!(journal.patch_self(&bad, &[entry()]).is_err());
    let mut regression = entry();
    regression.objectives[0].count = 0;
    assert!(journal.updates(&[entry()], &[regression]).is_err());
    let mut changed = entry();
    changed.objectives[0].id += 1;
    assert!(journal.updates(&[entry()], &[changed]).is_err());
    assert!(
        journal
            .patch_self(&reference.message, &[entry(), entry()])
            .is_err()
    );
    let mut bad_state = entry();
    bad_state.rewarded = true;
    assert!(
        journal
            .patch_self(&reference.message, &[bad_state])
            .is_err()
    );
}

#[test]
#[ignore = "requires private user-supplied captured reference paths"]
fn private_captured_journal_loads_and_only_the_proven_span_changes() {
    let content =
        PathBuf::from(std::env::var_os("OSFR_JOURNAL_CONTENT").expect("OSFR_JOURNAL_CONTENT"));
    let template =
        fs::read(std::env::var_os("OSFR_JOURNAL_SELF").expect("OSFR_JOURNAL_SELF")).unwrap();
    let metadata: serde_json::Value =
        serde_json::from_slice(&fs::read(content.join("quests.json")).unwrap()).unwrap();
    let offset = metadata["journal"]["offset"].as_u64().unwrap() as usize;
    let length = metadata["journal"]["length"].as_u64().unwrap() as usize;
    let patched = OpeningJournal::load(&content)
        .unwrap()
        .patch_self(&template, &[entry()])
        .unwrap();
    if let Some(oracle) = std::env::var_os("OSFR_JOURNAL_ORACLE") {
        // Entire payload equality with supplied compiled ReadFromJSON, not just
        // a second implementation of this module's field writer.
        assert_eq!(&patched[13..], fs::read(oracle).unwrap().as_slice());
    }
    assert_eq!(&patched[13..offset], &template[13..offset]);
    let suffix = &template[offset + length..];
    assert_eq!(&patched[patched.len() - suffix.len()..], suffix);
    let mut reader = Cursor::new(&patched[offset..patched.len() - suffix.len()]);
    assert_eq!(i32le(&mut reader), 1);
    assert_eq!(
        decode_quest(&mut reader).objectives[0],
        (1, 12001, 1, 1, 4, 0)
    );
}

#[test]
#[ignore = "requires private original and mapped opening content plus native self capture"]
fn private_all_51_goal_labels_map_without_changing_definitions_progress_or_live_keys() {
    use osfr_tools::opening::OpeningEngine;
    let original_path = PathBuf::from(
        std::env::var_os("OSFR_JOURNAL_SOURCE_CONTENT").expect("OSFR_JOURNAL_SOURCE_CONTENT"),
    );
    let mapped_path = PathBuf::from(
        std::env::var_os("OSFR_JOURNAL_MAPPED_CONTENT").expect("OSFR_JOURNAL_MAPPED_CONTENT"),
    );
    let template =
        fs::read(std::env::var_os("OSFR_JOURNAL_SELF").expect("OSFR_JOURNAL_SELF")).unwrap();
    assert_eq!(
        fs::read(original_path.join("opening.json")).unwrap(),
        fs::read(mapped_path.join("opening.json")).unwrap()
    );
    let original_engine = OpeningEngine::load(&original_path, None).unwrap();
    let mut checkpoint = original_engine.state().clone();
    let mut definitions = original_engine.journal_entries();
    for quest in &definitions {
        if quest.quest_id < 910004 {
            let progress = checkpoint.stages.get_mut(&quest.quest_id).unwrap();
            progress.accepted = true;
            progress.completed = true;
            progress.rewarded = true;
            progress.counts = quest.objectives.iter().map(|goal| goal.required).collect();
        }
    }
    let cooking = checkpoint.stages.get_mut(&910004).unwrap();
    cooking.accepted = true;
    cooking.counts = vec![1, 1, 1, 0, 0, 0];
    let original_checkpoint =
        OpeningEngine::load(&original_path, Some(checkpoint.clone())).unwrap();
    let mapped_checkpoint = OpeningEngine::load(&mapped_path, Some(checkpoint.clone())).unwrap();
    assert_eq!(original_checkpoint.state(), mapped_checkpoint.state());
    assert_eq!(mapped_checkpoint.state(), &checkpoint);
    let mapping: serde_json::Value =
        serde_json::from_slice(&fs::read(mapped_path.join("journal-text.json")).unwrap()).unwrap();
    let labels: std::collections::BTreeMap<_, _> = mapping["entries"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| {
            (
                (
                    row["quest_id"].as_u64().unwrap() as i32,
                    row["objective_id"].as_u64().unwrap() as i32,
                ),
                (
                    row["name_id"].as_u64().unwrap() as i32,
                    row["description_id"].as_u64().unwrap() as i32,
                ),
            )
        })
        .collect();
    assert_eq!(labels.len(), 24);
    // Synthetic accepted presentation copies exercise all17 full/Add records;
    // authoritative engine state and the real native checkpoint stay untouched.
    for quest in &mut definitions {
        quest.accepted = true;
    }
    assert_eq!(
        definitions
            .iter()
            .map(|q| q.objectives.len())
            .sum::<usize>(),
        51
    );
    let original = OpeningJournal::load(&original_path).unwrap();
    let mapped = OpeningJournal::load(&mapped_path).unwrap();
    let before = original.patch_self(&template, &definitions).unwrap();
    let after = mapped.patch_self(&template, &definitions).unwrap();
    assert_eq!(before.len(), after.len());
    let metadata: serde_json::Value =
        serde_json::from_slice(&fs::read(mapped_path.join("quests.json")).unwrap()).unwrap();
    let offset = metadata["journal"]["offset"].as_u64().unwrap() as usize;
    let mut old = Cursor::new(&before[offset..]);
    let mut new = Cursor::new(&after[offset..]);
    assert_eq!(i32le(&mut old), 17);
    assert_eq!(i32le(&mut new), 17);
    for _ in 0..17 {
        let prior = decode_quest(&mut old);
        let mut current = decode_quest(&mut new);
        for (index, goal) in current.objectives.iter().enumerate() {
            let expected = labels
                .get(&(current.id, goal.1))
                .copied()
                .unwrap_or(prior.objective_texts[index]);
            assert_eq!(current.objective_texts[index], expected);
            assert!(expected.0 > 0 && expected.1 > 0);
        }
        current.objective_texts = prior.objective_texts.clone();
        assert_eq!(current, prior);
    }
    assert_eq!(
        old.get_ref()[old.position() as usize..],
        new.get_ref()[new.position() as usize..]
    );
    let inactive = original_engine.journal_entries();
    let added = mapped.updates(&inactive, &definitions).unwrap();
    assert_eq!(added.len(), 17);
    for packet in added {
        assert_eq!(subtype(&packet), 3);
    }
    let first = definitions[0].clone();
    let mut advanced = first.clone();
    advanced.objectives[0].count = 1;
    assert_eq!(
        mapped
            .updates(
                std::slice::from_ref(&first),
                std::slice::from_ref(&advanced)
            )
            .unwrap(),
        original.updates(&[first], &[advanced]).unwrap()
    );
    assert_eq!(mapped_checkpoint.state(), &checkpoint);
}
