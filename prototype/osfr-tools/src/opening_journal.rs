//! Native opening quest journal, derived from the preserved client readers.
//!
//! Full records: FUN_00940c60 / FUN_0093c840 / FUN_008fd770.
//! Incremental opcode 49: FUN_00c7cc80, FUN_00c7aeb0, FUN_00c7af60,
//! FUN_00c7b070 and their dispatch/mutation functions. The native runtime quest
//! constructor (00BAF380 / 00BAB950) rekeys objectives by definition ID, so
//! incremental packet keys differ from the full record's keyed-list ordinal.
//!
//! Unknown presentation flags and reward fields are zero for reconstructed
//! opening entries. Rewards are granted by the authoritative opening engine;
//! this module does not invent native reward previews or completion dialogue.

use crate::opening::OpeningJournalEntry;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

const MAX_BYTES: usize = 2 * 1024 * 1024;
const MAX_QUESTS: usize = 256;
const MAX_OBJECTIVES: usize = 64;
// Pinned community locale tool's proven ID domain.
const MAX_LOCALE_ID: u32 = 5_103_267;

fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

fn bounded_file(path: &Path) -> io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    File::open(path)?
        .take(MAX_BYTES as u64 + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() > MAX_BYTES {
        return Err(invalid("opening journal reference exceeds size limit"));
    }
    Ok(bytes)
}

#[derive(Deserialize)]
struct Reference {
    schema_version: u32,
    journal: JournalReference,
}

#[derive(Deserialize)]
struct JournalReference {
    /// The journal file named by existing content; the release leaves it empty.
    #[serde(default)]
    file: String,
    /// T853: when non-empty this hex span replaces `file`; the same length/hash/offset proof
    /// applies. The release uses it so no binary ships.
    #[serde(default)]
    hex: String,
    sha256: String,
    length: usize,
    offset: usize,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PresentationFile {
    schema_version: u32,
    opening_sha256: String,
    locale: u32,
    notice: String,
    entries: Vec<Presentation>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Presentation {
    quest_id: u32,
    objective_id: u32,
    source_name_id: u32,
    source_description_id: u32,
    name_id: u32,
    description_id: u32,
    name: String,
    description: String,
}

// Only the identity/text fields are relevant here. OpeningEngine validates the
// authoritative gameplay definition separately; this file never mutates it.
#[derive(Deserialize)]
struct PresentationOpening {
    stages: Vec<PresentationStage>,
}
#[derive(Deserialize)]
struct PresentationStage {
    id: u32,
    objectives: Vec<PresentationGoal>,
}
#[derive(Deserialize)]
struct PresentationGoal {
    id: u32,
    name_id: u32,
    description_id: u32,
}

fn load_presentation(directory: &Path) -> io::Result<BTreeMap<(u32, u32), Presentation>> {
    let path = directory.join("journal-text.json");
    if !path.try_exists()? {
        return Ok(BTreeMap::new());
    }
    let presentation: PresentationFile = serde_json::from_slice(&bounded_file(&path)?)
        .map_err(|e| invalid(&format!("invalid journal text presentation: {e}")))?;
    let opening = bounded_file(&directory.join("opening.json"))?;
    if presentation.schema_version != 1
        || presentation.locale != 8
        || presentation.opening_sha256 != format!("{:x}", Sha256::digest(&opening))
        || presentation.notice.is_empty()
        || presentation.notice.len() > 512
        || presentation.entries.is_empty()
        || presentation.entries.len() > MAX_QUESTS * MAX_OBJECTIVES
    {
        return Err(invalid(
            "journal text schema/locale/content binding mismatch",
        ));
    }
    let opening: PresentationOpening = serde_json::from_slice(&opening)
        .map_err(|e| invalid(&format!("invalid journal text opening binding: {e}")))?;
    let source: BTreeMap<_, _> = opening
        .stages
        .iter()
        .flat_map(|stage| {
            stage
                .objectives
                .iter()
                .map(move |goal| ((stage.id, goal.id), goal))
        })
        .collect();
    let mut result = BTreeMap::new();
    let mut strings = BTreeMap::new();
    for entry in presentation.entries {
        if source
            .get(&(entry.quest_id, entry.objective_id))
            .is_none_or(|goal| {
                goal.name_id != entry.source_name_id
                    || goal.description_id != entry.source_description_id
            })
        {
            return Err(invalid("journal text source objective mismatch"));
        }
        for (id, text) in [
            (entry.name_id, &entry.name),
            (entry.description_id, &entry.description),
        ] {
            if id == 0
                || id > MAX_LOCALE_ID
                || text.is_empty()
                || text.len() > 512
                || text.chars().any(char::is_control)
                || strings.get(&id).is_some_and(|prior| prior != text)
            {
                return Err(invalid("invalid/conflicting journal text locale ID/string"));
            }
            strings.insert(id, text.clone());
        }
        if result
            .insert((entry.quest_id, entry.objective_id), entry)
            .is_some()
        {
            return Err(invalid("duplicate journal text objective mapping"));
        }
    }
    if source.iter().any(|(key, goal)| {
        (goal.name_id == 0 || goal.description_id == 0) && !result.contains_key(key)
    }) {
        return Err(invalid(
            "journal text leaves an unlabeled opening objective",
        ));
    }
    Ok(result)
}

#[derive(Clone, Debug)]
pub struct OpeningJournal {
    original: Vec<u8>,
    offset: usize,
    // Last two root i32 fields are not semantically proven. Preserve them.
    root_unknown: [u8; 8],
    presentation: BTreeMap<(u32, u32), Presentation>,
}

impl OpeningJournal {
    fn validate_presentation(&self, entries: &[OpeningJournalEntry]) -> io::Result<()> {
        for entry in entries {
            for goal in &entry.objectives {
                if self
                    .presentation
                    .get(&(entry.quest_id, goal.id))
                    .is_some_and(|text| {
                        goal.name_id != text.source_name_id
                            || goal.description_id != text.source_description_id
                    })
                {
                    return Err(invalid("runtime journal text source objective mismatch"));
                }
            }
        }
        Ok(())
    }
    /// Load and parse the exact hash-proven original journal span.
    pub fn load(directory: &Path) -> io::Result<Self> {
        let reference: Reference =
            serde_json::from_slice(&bounded_file(&directory.join("quests.json"))?)
                .map_err(|e| invalid(&format!("invalid quest reference: {e}")))?;
        let journal = reference.journal;
        let name = Path::new(&journal.file);
        if reference.schema_version != 1
            || journal.offset < 13
            || journal
                .offset
                .checked_add(journal.length)
                .is_none_or(|end| end > MAX_BYTES)
            || (journal.hex.is_empty()
                && (name.is_absolute()
                    || name.components().count() != 1
                    || !matches!(
                        name.components().next(),
                        Some(std::path::Component::Normal(_))
                    )
                    || journal.file.contains(['/', '\\', ':'])))
        {
            return Err(invalid(
                "invalid opening journal reference path/schema/offset",
            ));
        }
        let original = if journal.hex.is_empty() {
            bounded_file(&directory.join(name))?
        } else {
            // T853: the release ships the fresh journal span as typed JSON hex; no
            // capture character journal binary is published.
            if journal.hex.len() > MAX_BYTES * 2 {
                return Err(invalid("opening journal hex too large"));
            }
            crate::startup_snapshot::decode_hex(&journal.hex)
                .map_err(|_| invalid("invalid opening journal hex"))?
        };
        if original.len() != journal.length
            || format!("{:x}", Sha256::digest(&original)) != journal.sha256
        {
            return Err(invalid("opening journal reference hash/length mismatch"));
        }
        let root_unknown = verify_original(&original)?;
        Ok(Self {
            original,
            offset: journal.offset,
            root_unknown,
            presentation: load_presentation(directory)?,
        })
    }

    /// Replace inherited quests with accepted opening quests. Apply before any
    /// variable-size splice that moves this reference span (notably inventory).
    pub fn patch_self(
        &self,
        message: &[u8],
        entries: &[OpeningJournalEntry],
    ) -> io::Result<Vec<u8>> {
        let journal = self.journal(entries)?;
        if message.len() < 13
            || message.len() > MAX_BYTES
            || message[..3] != [5, 0, 1]
            || message[7..9] != [12, 0]
            || u32::from_le_bytes(message[3..7].try_into().unwrap()) as usize != message.len() - 7
            || u32::from_le_bytes(message[9..13].try_into().unwrap()) as usize != message.len() - 13
        {
            return Err(invalid("invalid native self packet for opening journal"));
        }
        let end = self.offset + self.original.len();
        if message.get(self.offset..end) != Some(self.original.as_slice())
            || message
                .windows(self.original.len())
                .filter(|span| *span == self.original)
                .count()
                != 1
        {
            return Err(invalid(
                "native self journal differs from unique verified reference span",
            ));
        }
        let mut result = Vec::with_capacity(message.len() - self.original.len() + journal.len());
        result.extend_from_slice(&message[..self.offset]);
        result.extend(journal);
        result.extend_from_slice(&message[end..]);
        if result.len() > MAX_BYTES {
            return Err(invalid("opening journal self packet exceeds size limit"));
        }
        let tunnel_length = (result.len() - 7) as u32;
        let self_length = (result.len() - 13) as u32;
        result[3..7].copy_from_slice(&tunnel_length.to_le_bytes());
        result[9..13].copy_from_slice(&self_length.to_le_bytes());
        Ok(result)
    }

    /// The self-data journal span (`00940c60`) holding only these entries: the accepted,
    /// unrewarded quests, the rewarded count, the first of them selected (or -1) and the
    /// captured quest limits.
    pub fn journal(&self, entries: &[OpeningJournalEntry]) -> io::Result<Vec<u8>> {
        validate_entries(entries)?;
        self.validate_presentation(entries)?;
        let active: Vec<_> = entries.iter().filter(|entry| visible(entry)).collect();
        let mut journal = Vec::new();
        integer(&mut journal, active.len() as i32);
        for entry in &active {
            quest_record(&mut journal, entry, &self.presentation);
        }
        integer(&mut journal, rewarded_count(entries));
        integer(
            &mut journal,
            active.first().map_or(-1, |entry| entry.quest_id as i32),
        );
        journal.push(0); // Selection-policy flag: no inherited research selection.
        journal.extend(self.root_unknown);
        Ok(journal)
    }

    /// Return INNER native packets; the gateway adds the normal tunnel wrapper.
    /// Only changed objectives are sent. QuestComplete (4) removes a turned-in
    /// quest; objective completion (10) keeps it available for turn-in.
    pub fn updates(
        &self,
        before: &[OpeningJournalEntry],
        after: &[OpeningJournalEntry],
    ) -> io::Result<Vec<Vec<u8>>> {
        validate_entries(before)?;
        validate_entries(after)?;
        self.validate_presentation(before)?;
        self.validate_presentation(after)?;
        let old: BTreeMap<_, _> = before.iter().map(|entry| (entry.quest_id, entry)).collect();
        let new: BTreeMap<_, _> = after.iter().map(|entry| (entry.quest_id, entry)).collect();
        for (id, prior) in &old {
            let current = new
                .get(id)
                .ok_or_else(|| invalid("opening journal lost a campaign entry"))?;
            if (prior.accepted && !current.accepted)
                || (prior.completed && !current.completed)
                || (prior.rewarded && !current.rewarded)
                || !same_definition(prior, current)
                || prior
                    .objectives
                    .iter()
                    .zip(&current.objectives)
                    .any(|(a, b)| a.count > b.count)
            {
                return Err(invalid(
                    "opening journal cannot regress or change quest definitions",
                ));
            }
        }
        let mut packets = Vec::new();
        for entry in after {
            let prior = old.get(&entry.quest_id).copied();
            if entry.rewarded {
                if prior.is_some_and(|old| visible(old)) {
                    let mut packet = packet(4);
                    integer(&mut packet, entry.quest_id as i32);
                    packets.push(packet);
                }
                continue;
            }
            if !entry.accepted {
                continue;
            }
            if !prior.is_some_and(|old| visible(old)) {
                let mut packet = packet(3);
                quest_record(&mut packet, entry, &self.presentation);
                packet.push(1); // Add-packet force-selection flag (client dispatch).
                packets.push(packet);
                continue;
            }
            let prior = prior.unwrap();
            for (old_goal, goal) in prior.objectives.iter().zip(&entry.objectives) {
                if old_goal.count == goal.count {
                    continue;
                }
                // 00BAF380 copies ObjectiveData using its definition ID as
                // the runtime key; 00C7A990 / 00C7AA10 look up that key.
                let key = goal.id as i32;
                let mut update = packet(9);
                integer(&mut update, entry.quest_id as i32);
                integer(&mut update, key);
                integer(&mut update, goal.count as i32);
                update.extend(progress(entry).to_le_bytes());
                update.push(0); // Unknown UI-presentation flag; false is neutral.
                integer(&mut update, 0); // Unknown optional presentation ID.
                packets.push(update);
                if goal.count == goal.required && old_goal.count < goal.required {
                    let mut complete = packet(10);
                    integer(&mut complete, entry.quest_id as i32);
                    integer(&mut complete, key);
                    complete.extend(progress(entry).to_le_bytes());
                    integer(&mut complete, 0); // Optional reward/presentation lookup; absent.
                    complete.push(0); // Unknown UI-presentation flag.
                    packets.push(complete);
                }
            }
        }
        if rewarded_count(before) != rewarded_count(after) {
            let mut packet = packet(12);
            integer(&mut packet, rewarded_count(after));
            packets.push(packet);
        }
        Ok(packets)
    }
}

fn visible(entry: &OpeningJournalEntry) -> bool {
    entry.accepted && !entry.rewarded
}

fn rewarded_count(entries: &[OpeningJournalEntry]) -> i32 {
    entries.iter().filter(|entry| entry.rewarded).count() as i32
}

fn validate_entries(entries: &[OpeningJournalEntry]) -> io::Result<()> {
    let mut ids = BTreeSet::new();
    if entries.len() > MAX_QUESTS {
        return Err(invalid("too many opening journal quests"));
    }
    for entry in entries {
        if entry.quest_id == 0
            || entry.quest_id > i32::MAX as u32
            || entry.title_id > i32::MAX as u32
            || entry.description_id > i32::MAX as u32
            || !ids.insert(entry.quest_id)
            || entry.objectives.is_empty()
            || entry.objectives.len() > MAX_OBJECTIVES
            || (entry.completed && !entry.accepted)
            || (entry.rewarded && !entry.completed)
        {
            return Err(invalid("invalid opening journal quest identity/state"));
        }
        let mut objective_ids = BTreeSet::new();
        for goal in &entry.objectives {
            if goal.id == 0
                || goal.id > i32::MAX as u32
                || !objective_ids.insert(goal.id)
                || goal.name_id > i32::MAX as u32
                || goal.description_id > i32::MAX as u32
                || goal.required == 0
                || goal.required > i32::MAX as u32
                || goal.count > goal.required
                || (!entry.accepted && goal.count != 0)
                || (entry.completed && goal.count != goal.required)
            {
                return Err(invalid("invalid opening journal objective identity/count"));
            }
        }
    }
    Ok(())
}

fn same_definition(a: &OpeningJournalEntry, b: &OpeningJournalEntry) -> bool {
    a.title_id == b.title_id
        && a.description_id == b.description_id
        && a.objectives.len() == b.objectives.len()
        && a.objectives.iter().zip(&b.objectives).all(|(x, y)| {
            x.id == y.id
                && x.name_id == y.name_id
                && x.description_id == y.description_id
                && x.required == y.required
        })
}

fn progress(entry: &OpeningJournalEntry) -> f32 {
    // Reconstructed policy: average normalized objective progress, always [0,1].
    (entry
        .objectives
        .iter()
        .map(|goal| goal.count as f64 / goal.required as f64)
        .sum::<f64>()
        / entry.objectives.len() as f64) as f32
}

fn integer(bytes: &mut Vec<u8>, value: i32) {
    bytes.extend(value.to_le_bytes());
}

fn packet(subtype: i32) -> Vec<u8> {
    let mut bytes = 49u16.to_le_bytes().to_vec();
    integer(&mut bytes, subtype);
    bytes
}

fn empty_reward(bytes: &mut Vec<u8>) {
    // bool, 9*i32, 2*i64, 2*i32, entry count i32, final i32.
    bytes.extend([0; 69]);
}

fn quest_record(
    bytes: &mut Vec<u8>,
    entry: &OpeningJournalEntry,
    presentation: &BTreeMap<(u32, u32), Presentation>,
) {
    integer(bytes, entry.quest_id as i32);
    integer(bytes, entry.title_id as i32);
    integer(bytes, entry.description_id as i32);
    integer(bytes, 0); // Unknown4: ending-description locale ID is not supplied.
    bytes.push(0); // Unknown5.
    bytes.extend(0i64.to_le_bytes()); // Unknown6: no recovered acceptance timestamp.
    integer(bytes, 0); // Unknown7.
    bytes.push(0); // Unknown8.
    bytes.extend(progress(entry).to_le_bytes()); // Unknown9 is f32, proven native.
    empty_reward(bytes);
    integer(bytes, entry.objectives.len() as i32);
    for (index, goal) in entry.objectives.iter().enumerate() {
        integer(bytes, index as i32 + 1); // Wire tuple key; native runtime rekeys by ID.
        integer(bytes, goal.id as i32); // Definition ID, independent from key.
        let labels = presentation.get(&(entry.quest_id, goal.id));
        integer(
            bytes,
            labels.map_or(goal.name_id, |text| text.name_id) as i32,
        );
        integer(
            bytes,
            labels.map_or(goal.description_id, |text| text.description_id) as i32,
        );
        bytes.push(0); // Objective Unknown4.
        empty_reward(bytes);
        integer(bytes, if goal.count == goal.required { 2 } else { 1 }); // State.
        integer(bytes, goal.count as i32); // Unknown6: actual current counter.
        integer(bytes, goal.required as i32);
        integer(bytes, 0); // Unknown8: not the counter.
        bytes.push(0); // Unknown9.
        integer(bytes, 0); // Unknown10.
    }
    integer(bytes, 0); // Quest Unknown10.
    bytes.extend([0, 0]); // Unknown11, Unknown12.
}

struct Reader<'a> {
    bytes: &'a [u8],
    offset: usize,
}
impl<'a> Reader<'a> {
    fn take(&mut self, length: usize) -> io::Result<&'a [u8]> {
        let end = self
            .offset
            .checked_add(length)
            .ok_or_else(|| invalid("journal range overflow"))?;
        let value = self
            .bytes
            .get(self.offset..end)
            .ok_or_else(|| invalid("truncated original journal"))?;
        self.offset = end;
        Ok(value)
    }
    fn integer(&mut self) -> io::Result<i32> {
        Ok(i32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }
    fn count(&mut self, limit: usize) -> io::Result<usize> {
        let value = self.integer()?;
        if value < 0 || value as usize > limit {
            return Err(invalid("invalid original journal array count"));
        }
        Ok(value as usize)
    }
    fn boolean(&mut self) -> io::Result<bool> {
        match self.take(1)?[0] {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(invalid("invalid original journal bool")),
        }
    }
    fn reward(&mut self) -> io::Result<()> {
        let extended_item = self.boolean()?;
        self.take(60)?;
        for _ in 0..self.count(256)? {
            if self.integer()? != 1 {
                return Err(invalid("unsupported original journal reward type"));
            }
            self.boolean()?;
            self.take(24)?;
            let length = self.count(65536)?;
            self.take(length)?;
            self.take(4)?;
            self.boolean()?;
            if extended_item {
                self.take(4)?;
            }
        }
        self.take(4)?;
        Ok(())
    }
}

fn verify_original(bytes: &[u8]) -> io::Result<[u8; 8]> {
    let mut reader = Reader { bytes, offset: 0 };
    for _ in 0..reader.count(MAX_QUESTS)? {
        reader.take(16)?;
        reader.boolean()?;
        reader.take(12)?;
        reader.boolean()?;
        if !f32::from_le_bytes(reader.take(4)?.try_into().unwrap()).is_finite() {
            return Err(invalid("nonfinite original journal quest progress"));
        }
        reader.reward()?;
        for _ in 0..reader.count(MAX_OBJECTIVES)? {
            reader.take(16)?;
            reader.boolean()?;
            reader.reward()?;
            reader.take(16)?;
            reader.boolean()?;
            reader.take(4)?;
        }
        reader.take(4)?;
        reader.boolean()?;
        reader.boolean()?;
    }
    reader.take(8)?;
    reader.boolean()?;
    let unknown = reader.take(8)?.try_into().unwrap();
    if reader.offset != bytes.len() {
        return Err(invalid("original journal has trailing bytes"));
    }
    Ok(unknown)
}
