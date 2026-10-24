//! The Jobs window's Become-a list: the 147/1 `ActivityProfileList` the client fills its
//! `ActivityFinderDataSource` from, so `JobBrowser:StartJobQuest` (`ScriptsBase.lua`) finds
//! the job's row and sends 49/15 (`Ui.StartBreadcrumbQuest`, `00c18aa0`).
//!
//! Wire (own 2014 capture `lab/reports/original-minigame-packets/demoderby-23083-flow0-
//! frame5077-op147-sub1.bin`; client packet ctor `00be5490`, reader `00be6f20`, dispatch
//! `00be7360` sub 1): `u16 147`, `u8 1`, `u32 count`, then per row seven `i32`: the row key
//! (the ProfileId), ProfileId, QuestId, IconId, BadgeId, TitleStringId, DescriptionStringId.
//! The getter `00ceab40` reads those as the data-source columns ProfileId, QuestId (col 4),
//! IconId, BadgeId, QuestTitle and QuestDescription; HasQuest (col 1) is a client lookup.
//!
//! The QuestId is the job's ORIGINAL Become-a quest id. The regional catalogue does not use
//! it, so `data/content/job_browser.json` also maps it to a regional quest; the rows with
//! one join the 49/15 crosswalk (`quest_journal`) and the others are refused there (R).
//! The list is static: every player gets the same rows (R).

use crate::content::{Job, SCHEMA_VERSION, SourceTag, VersionedFile};
use crate::quest_journal::Crosswalk;
use serde::Deserialize;
use std::{collections::BTreeSet, fs, io, path::Path};

const FILE: &str = "job_browser.json";
const MAX_BYTES: u64 = 64 << 10;
const MAX_ROWS: usize = 64;

/// One 147/1 row. Every id is in `1..=i32::MAX`, so each is the same bytes as an `i32`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProfileRow {
    pub profile_id: u32,
    pub original_quest_id: u32,
    pub icon_id: u32,
    pub badge_id: u32,
    pub title_string_id: u32,
    pub description_string_id: u32,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Record {
    profile_id: u32,
    original_quest_id: u32,
    icon_id: u32,
    badge_id: u32,
    title_string_id: u32,
    description_string_id: u32,
    /// `None`: no regional quest stands in for the original one.
    regional_quest_id: Option<u32>,
    source: Vec<SourceTag>,
}

/// What a content folder gives the session: the packet (empty with no rows) and the
/// original-to-regional entries to merge into the 49/15 crosswalk.
#[derive(Debug, Default)]
pub struct JobBrowser {
    pub packet: Vec<u8>,
    pub crosswalk: Crosswalk,
}

pub fn encode(rows: &[ProfileRow]) -> Vec<u8> {
    let mut packet = vec![147, 0, 1];
    packet.extend((rows.len() as u32).to_le_bytes());
    for row in rows {
        for id in [
            row.profile_id,
            row.profile_id,
            row.original_quest_id,
            row.icon_id,
            row.badge_id,
            row.title_string_id,
            row.description_string_id,
        ] {
            packet.extend(id.to_le_bytes());
        }
    }
    packet
}

/// Read `job_browser.json` from a content folder. A missing file lists nothing; a malformed
/// one is an error at bind time, never at login. `breadcrumb_originals` are the original
/// ids `breadcrumb_quests.json` already lists: an id in both files is refused.
pub fn load(dir: &Path, breadcrumb_originals: &BTreeSet<u32>) -> io::Result<JobBrowser> {
    let invalid = |message: &'static str| io::Error::new(io::ErrorKind::InvalidData, message);
    let parse_error = |error| io::Error::new(io::ErrorKind::InvalidData, error);
    let path = dir.join(FILE);
    if !path.is_file() {
        return Ok(JobBrowser::default());
    }
    if fs::metadata(&path)?.len() > MAX_BYTES {
        return Err(invalid("job browser file over 64 KiB"));
    }
    let file: VersionedFile<Record> =
        serde_json::from_slice(&fs::read(&path)?).map_err(parse_error)?;
    if file.schema_version != SCHEMA_VERSION {
        return Err(invalid("unsupported job browser schema"));
    }
    if file.records.len() > MAX_ROWS {
        return Err(invalid("too many job browser rows"));
    }
    let jobs: VersionedFile<Job> =
        serde_json::from_slice(&fs::read(dir.join("jobs.json"))?).map_err(parse_error)?;
    let profiles: BTreeSet<i32> = jobs
        .records
        .iter()
        .filter_map(|job| job.profile_id)
        .collect();
    let valid_id = |id: u32| (1..=i32::MAX as u32).contains(&id);
    let (mut rows, mut crosswalk) = (Vec::new(), Crosswalk::new());
    let (mut seen_profiles, mut seen_originals) = (BTreeSet::new(), BTreeSet::new());
    for record in file.records {
        let ids = [
            record.profile_id,
            record.original_quest_id,
            record.icon_id,
            record.badge_id,
            record.title_string_id,
            record.description_string_id,
        ];
        if !ids
            .into_iter()
            .chain(record.regional_quest_id)
            .all(valid_id)
            || record.source.is_empty()
            || record
                .source
                .iter()
                .any(|tag| tag.is_reconstructed() && !tag.evidence().starts_with("(R)"))
        {
            return Err(invalid("invalid job browser record"));
        }
        if !profiles.contains(&(record.profile_id as i32)) {
            return Err(invalid("job browser row names an unknown profile"));
        }
        if !seen_profiles.insert(record.profile_id)
            || !seen_originals.insert(record.original_quest_id)
        {
            return Err(invalid("duplicate job browser profile or quest"));
        }
        if breadcrumb_originals.contains(&record.original_quest_id) {
            return Err(invalid(
                "job browser quest is also in the breadcrumb crosswalk",
            ));
        }
        if let Some(regional) = record.regional_quest_id {
            crosswalk.insert(record.original_quest_id, regional);
        }
        rows.push(ProfileRow {
            profile_id: record.profile_id,
            original_quest_id: record.original_quest_id,
            icon_id: record.icon_id,
            badge_id: record.badge_id,
            title_string_id: record.title_string_id,
            description_string_id: record.description_string_id,
        });
    }
    let packet = if rows.is_empty() {
        Vec::new()
    } else {
        encode(&rows)
    };
    Ok(JobBrowser { packet, crosswalk })
}
